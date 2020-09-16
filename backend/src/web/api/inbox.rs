use std::str::FromStr;

use chrono::Utc;
use proto::api::inbox::{ArchiveRequest, GetResponse, ListResponse};
use proto::model::{DocId, Kind};
use rocket::{delete, get, post, State};
use rocket::http::{ContentType, RawStr};
use rocket::response::{Content, Stream};
use rocket_contrib::json::Json;
use tokio::io::AsyncRead;

use crate::index::Index;
use crate::repository::Repository;
use crate::suggester::Suggester;
use crate::web::api::InternalError;

use super::{ApiError, Token};

#[get("/inbox")]
pub(super) async fn list(repository: State<'_, Repository>,
                         _token: &'_ Token) -> Result<Json<ListResponse>, ApiError> {
    let docs = repository.inbox().list().await?;

    Ok(Json(ListResponse {
        count: docs.len() as u64,
        docs: docs.iter().take(10).cloned().collect(),
    }))
}

#[get("/inbox/<id>")]
pub(super) async fn bundle(id: &RawStr,
                           repository: State<'_, Repository>,
                           suggester: State<'_, Box<dyn Suggester + Send + Sync>>,
                           _token: &'_ Token) -> Result<Json<GetResponse>, ApiError> {
    let id = DocId::from_str(id.as_str())?;

    let bundle = repository.inbox().get(id).await
        .ok_or_else(|| ApiError::not_found(format!("Bundle not found: {}", id)))?;

    let metadata = bundle.metadata().await?
        .ok_or_else(|| ApiError::not_found(format!("Metadata not found: {}", id)))?;

    let plaintext = bundle.plaintext().await?
        .ok_or_else(|| ApiError::not_found(format!("Plaintext not found: {}", id)))?;

    return Ok(Json(GetResponse {
        id,
        uploaded: metadata.uploaded,
        labels: suggester.guess(&plaintext).await?,
        properties: metadata.properties,
    }));
}

#[get("/inbox/<id>/<fragment>")]
pub(super) async fn fragment(id: &RawStr,
                             fragment: &RawStr,
                             repository: State<'_, Repository>,
                             _token: &'_ Token) -> Result<Content<Stream<impl AsyncRead>>, ApiError> {
    let id = DocId::from_str(id.as_str())?;
    let kind = Kind::from(fragment.as_str());

    let bundle = repository.inbox().get(id).await
        .ok_or_else(|| ApiError::not_found(format!("Bundle not found: {}", id)))?;

    return bundle.with_fragment(kind, |file, kind| async move {
        let content_type = match kind {
            Kind::Document => ContentType::PDF,
            Kind::Preview => ContentType::PNG,
            Kind::Plaintext => ContentType::Plain,
            Kind::Metadata => ContentType::JSON,
            Kind::ProcessLog => ContentType::Plain,
            Kind::Other { .. } => ContentType::Any,
        };

        return Ok(Content(content_type, file.into()));
    }).await
        .map_err(InternalError)?
        .ok_or_else(|| ApiError::not_found(format!("Fragment not found: {}/{}", id, fragment)));
}

#[delete("/inbox/<id>")]
pub(super) async fn delete(id: &RawStr,
                           repository: State<'_, Repository>,
                           _token: &'_ Token) -> Result<(), ApiError> {
    let id = DocId::from_str(id.as_str())?;

    let bundle = repository.inbox().get(id).await
        .ok_or_else(|| ApiError::not_found(format!("Bundle not found: {}", id)))?;
    bundle.delete().await?;

    return Ok(());
}

#[post("/inbox/<id>", data = "<data>")]
pub(super) async fn archive(id: &RawStr,
                            data: Json<ArchiveRequest>,
                            repository: State<'_, Repository>,
                            index: State<'_, Box<dyn Index + Send + Sync>>,
                            suggester: State<'_, Box<dyn Suggester + Send + Sync>>,
                            _token: &'_ Token) -> Result<(), ApiError> {
    let id = DocId::from_str(id.as_str())?;

    let bundle = repository.inbox().get(id).await
        .ok_or_else(|| ApiError::not_found(format!("Bundle not found: {}", id)))?;

    // Update the metadata
    let mut metadata = bundle.metadata().await?
        .ok_or_else(|| ApiError::not_found(format!("Plaintext not found: {}", id)))?;

    metadata.archived = Some(Utc::now());
    metadata.labels = data.labels.clone();
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
