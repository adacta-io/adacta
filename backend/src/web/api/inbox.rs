use chrono::Utc;
use proto::api::inbox::{ArchiveRequest, GetResponse, ListResponse};
use rocket::{delete, get, post, State};
use rocket_contrib::json::Json;

use crate::index::Index;
use crate::model::{DocId, Label};
use crate::repository::Repository;
use crate::suggester::Suggester;

use super::{ApiError, Token};

#[get("/inbox")]
pub(super) async fn list(repository: State<'_, Repository>,
                         _token: &'_ Token) -> Result<Json<ListResponse>, ApiError> {
    let docs = repository.inbox().list().await?;

    Ok(Json(ListResponse {
        count: docs.len() as u64,
        docs: docs.iter().take(10).map(DocId::to_string).collect(),
    }))
}

#[get("/inbox/<id>")]
pub(super) async fn get(id: DocId,
                        repository: State<'_, Repository>,
                        suggester: State<'_, Box<dyn Suggester + Send + Sync>>,
                        _token: &'_ Token) -> Result<Json<GetResponse>, ApiError> {
    let bundle = repository.inbox().get(id).await
        .ok_or_else(|| ApiError::not_found(format!("Bundle not found: {}", id)))?;

    let metadata = bundle.metadata().await?
        .ok_or_else(|| ApiError::not_found(format!("Metadata not found: {}", id)))?;

    let plaintext = bundle.plaintext().await?
        .ok_or_else(|| ApiError::not_found(format!("Plaintext not found: {}", id)))?;

    return Ok(Json(GetResponse {
        id: id.to_string(),
        uploaded: metadata.uploaded,
        labels: suggester.guess(&plaintext).await?.iter().map(Label::to_string).collect(),
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
    let mut metadata = bundle.metadata().await?
        .ok_or_else(|| ApiError::not_found(format!("Plaintext not found: {}", id)))?;

    metadata.archived = Some(Utc::now());
    metadata.labels = data.labels.iter().map(Label::from).collect();
    metadata.properties = data.properties.clone();

    bundle.write_metadata(&metadata).await?;

    // Archive the bundle
    let archived = bundle.archive().await?;

    // Add the archived bundle to the index
    index.index(&archived).await?;

    // Train the suggester with the final labels
    let plaintext = archived.plaintext().await?
        .ok_or_else(|| ApiError::not_found(format!("Plaintext not found: {}", id)))?;

    suggester.train(&plaintext, &metadata.labels).await?;

    return Ok(());
}
