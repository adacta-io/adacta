use std::str::FromStr;

use anyhow::anyhow;
use proto::api::archive::{BundleResponse, SearchResponse};
use proto::model::{DocId, Kind};
use rocket::{get, http::ContentType, State};
use rocket::http::RawStr;
use rocket::response::{Content, Stream};
use rocket_contrib::json::Json;
use tokio::io::AsyncRead;

use crate::index::Index;
use crate::repository::Repository;

use super::{ApiError, InternalError, Token};

#[get("/archive/<id>")]
pub(super) async fn bundle(id: &RawStr,
                           repository: State<'_, Repository>,
                           _token: &'_ Token) -> Result<Json<BundleResponse>, ApiError> {
    let id = DocId::from_str(id.as_str())?;

    let bundle = repository.archive().get(id).await
        .ok_or_else(|| ApiError::not_found(format!("Bundle not found: {}", id)))?;

    let metadata = bundle.metadata().await?;

    Ok(Json(BundleResponse {
        doc: (id, metadata).into(),
    }))
}

#[get("/archive/<id>/<fragment>")]
pub(super) async fn fragment(id: &RawStr,
                             fragment: String,
                             repository: State<'_, Repository>,
                             _token: &'_ Token) -> Result<Content<Stream<impl AsyncRead>>, ApiError> {
    let id = DocId::from_str(id.as_str())?;
    let kind = Kind::from(fragment.as_str());

    let bundle = repository.archive().get(id).await
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

#[get("/archive?<query>")]
pub(super) async fn search(query: &RawStr,
                           index: State<'_, Box<dyn Index + Send + Sync>>,
                           repository: State<'_, Repository>,
                           _token: &'_ Token) -> Result<Json<SearchResponse>, ApiError> {
    let response = index.search(query).await?;

    // TODO: Can this be a done as stream?
    let mut docs = Vec::new();
    for id in response.docs {
        let bundle = repository.archive().get(id).await
            .ok_or_else(|| anyhow!("Bundle missing: {}", id))?;

        let metadata = bundle.metadata().await?;

        docs.push((*bundle.id(), metadata).into());
    }

    Ok(Json(SearchResponse {
        count: response.count,
        docs,
    }))
}
