use proto::api::archive::{BundleResponse, SearchResponse};
use rocket::{get, http::ContentType, State};
use rocket::http::RawStr;
use rocket::response::{Content, Stream};
use rocket_contrib::json::Json;
use tokio::io::AsyncRead;

use crate::index::Index;
use crate::model::{DocId, Kind};
use crate::repository::Repository;

use super::{ApiError, InternalError, Token};

#[get("/archive/<id>")]
pub(super) async fn bundle(id: DocId,
                           repository: State<'_, Repository>,
                           _token: &'_ Token) -> Result<Json<BundleResponse>, ApiError> {
    let bundle = repository.archive().get(id).await
        .ok_or_else(|| ApiError::not_found(format!("Bundle not found: {}", id)))?;

    Ok(Json(BundleResponse { id: bundle.id().to_string() }))
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
        .map_err(|err| InternalError(err))?
        .ok_or_else(|| ApiError::not_found(format!("Fragment not found: {}/{}", id, fragment)));
}

#[get("/archive?<query>")]
pub(super) async fn search(query: &RawStr,
                           repository: State<'_, Repository>,
                           index: State<'_, Box<dyn Index + Send + Sync>>,
                           _token: &'_ Token) -> Result<Json<SearchResponse>, ApiError> {
    let response = index.search(query).await?;

    Ok(Json(SearchResponse {
        count: response.count,
        docs: response.docs.iter().map(DocId::to_string).collect(),
    }))
}
