use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};
use rocket::{delete, get, post, State};
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};

use crate::index::Index;
use crate::model::{DocId, Label};
use crate::repository::{BundleContainer, Repository};
use crate::suggester::Suggester;

use super::{ApiError, Token};

#[derive(Debug, Clone, Serialize)]
pub struct ListResponse {
    pub count: u64,
    pub docs: Vec<DocId>,
}

#[get("/inbox")]
pub(super) async fn list(index: State<'_, Box<dyn Index + Send + Sync>>,
                         _token: &'_ Token) -> Result<Json<ListResponse>, ApiError> {
    let response = index.inbox().await?;

    Ok(Json(ListResponse {
        count: response.count,
        docs: response.docs,
    }))
}

#[derive(Debug, Clone, Serialize)]
pub struct GetResponse {
    pub id: DocId,
    pub uploaded: DateTime<Utc>,
    pub labels: HashSet<Label>,
    pub properties: HashMap<String, String>,
}

#[get("/inbox/<id>")]
pub(super) async fn get(id: DocId,
                        repository: State<'_, Repository>,
                        suggester: State<'_, Box<dyn Suggester + Send + Sync>>,
                        _token: &'_ Token) -> Result<Json<GetResponse>, ApiError> {
    let bundle = repository.inbox().get(id).await
        .ok_or_else(|| ApiError::not_found(format!("Bundle not found: {}", id)))?;

    let metadata = bundle.metadata().await
        .ok_or_else(|| ApiError::not_found(format!("Metadata not found: {}", id)))??;

    let plaintext = bundle.plaintext().await
        .ok_or_else(|| ApiError::not_found(format!("Plaintext not found: {}", id)))??;

    return Ok(Json(GetResponse {
        id,
        uploaded: metadata.uploaded,
        labels: suggester.guess(&plaintext).await?,
        properties: metadata.properties,
    }));
}

#[delete("/inbox/<id>")]
pub(super) async fn delete(id: DocId,
                           repository: State<'_, Repository>,
                           _token: &'_ Token) -> Result<(), ApiError> {
    let bundle = repository.inbox().get(id).await
        .ok_or_else(|| ApiError::not_found(format!("Bundle not found: {}", id)))?;
    bundle.delete().await?;

    return Ok(());
}

#[derive(Debug, Clone, Deserialize)]
pub struct ArchiveRequest {
    pub labels: HashSet<Label>,
    pub properties: HashMap<String, String>,
}

#[post("/inbox/<id>", data = "<data>")]
pub(super) async fn archive(id: DocId,
                            data: Json<ArchiveRequest>,
                            repository: State<'_, Repository>,
                            index: State<'_, Box<dyn Index + Send + Sync>>,
                            suggester: State<'_, Box<dyn Suggester + Send + Sync>>,
                            _token: &'_ Token) -> Result<(), ApiError> {
    let bundle = repository.inbox().get(id).await
        .ok_or_else(|| ApiError::not_found(format!("Bundle not found: {}", id)))?;

    // Update the metadata
    let mut metadata = bundle.metadata().await.ok_or_else(|| ApiError::not_found(format!("Plaintext not found: {}", id)))??;
    metadata.archived = Some(Utc::now());
    metadata.labels = data.labels.clone();
    metadata.properties = data.properties.clone();
    metadata.save(bundle.write_manifest().await?).await?;

    let archived = bundle.archive().await?;

    // Add the archived bundle to the index
    index.index(&archived).await?;

    // Train the suggester with the final labels
    let plaintext = archived.plaintext().await.ok_or_else(|| ApiError::not_found(format!("Plaintext not found: {}", id)))??;

    suggester.train(&plaintext, &data.labels).await?;

    return Ok(());
}
