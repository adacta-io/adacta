use rocket::{get, http::ContentType, State};
use rocket::http::RawStr;
use rocket::response::{Content, Stream};
use rocket_contrib::json::Json;
use serde::Serialize;
use tokio::io::AsyncRead;

use crate::index::Index;
use crate::model::{DocId, Kind};
use crate::repository::{BundleContainer, Repository};

use super::{ApiError, InternalError, Token};

#[derive(Debug, Clone, Serialize)]
pub struct BundleResponse {
    id: String,
    //    created: DateTime<Utc>,
    //    modified: DateTime<Utc>,

    // Other metadata...
}

#[get("/archive/<id>")]
pub(super) async fn bundle(id: DocId,
                           repository: State<'_, Repository>,
                           _token: &'_ Token) -> Result<Json<BundleResponse>, ApiError> {
    let _bundle = repository.archive().get(id).await
        .ok_or_else(|| ApiError::not_found(format!("Bundle not found: {}", id)))?;

    Ok(Json(BundleResponse { id: id.to_string() }))
}

#[get("/archive/<id>/<fragment>")]
pub(super) async fn fragment(id: DocId,
                             fragment: String,
                             repository: State<'_, Repository>,
                             _token: &'_ Token) -> Result<Content<Stream<impl AsyncRead>>, ApiError> {
    let kind = match fragment.as_str() {
        "document" => Kind::Document,
        "preview" => Kind::Preview,
        "plaintext" => Kind::Plaintext,
        "metadata" => Kind::Metadata,
        "process_log" => Kind::ProcessLog,
        s => Kind::other(s),
    };

    let bundle = repository.archive().get(id).await
        .ok_or_else(|| ApiError::not_found(format!("Bundle not found: {}", id)))?;

    let fragment = bundle.fragment(kind).await
        .ok_or_else(|| ApiError::not_found(format!("Fragment not found: {}/{}", id, fragment)))?;

    let file = fragment.read().await.map_err(InternalError)?;

    let content_type = match fragment.kind() {
        Kind::Document => ContentType::PDF,
        Kind::Preview => ContentType::PNG,
        Kind::Plaintext => ContentType::Plain,
        Kind::Metadata => ContentType::JSON,
        Kind::ProcessLog => ContentType::Plain,
        Kind::Other { .. } => ContentType::Any,
    };

    Ok(Content(content_type, file.into()))
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchResponse {
    pub count: u64,
    pub docs: Vec<DocId>,
}

#[get("/archive?<query>")]
pub(super) async fn search(query: &RawStr,
                           repository: State<'_, Repository>,
                           index: State<'_, Box<dyn Index + Send + Sync>>,
                           _token: &'_ Token) -> Result<Json<SearchResponse>, ApiError> {
    let response = index.search(query).await?;

    Ok(Json(SearchResponse {
        count: response.count,
        docs: response.docs,
    }))
}
