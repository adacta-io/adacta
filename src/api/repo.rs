use std::borrow::Cow;
use std::ffi::OsStr;
use std::path::Path;

use rocket::{get, http::ContentType, response::{Content, Debug, Responder, status::NotFound, Stream}, State};
use rocket_contrib::json::Json;
use serde::Serialize;
use tokio::io::AsyncRead;

use crate::api::{ApiError, InternalError};
use crate::model::{DocId, Kind};
use crate::repo::Repository;

#[derive(Debug, Clone, Serialize)]
pub struct BundleResponse {
    id: String,

//    created: DateTime<Utc>,
//    modified: DateTime<Utc>,

    // Other metadata...
}

#[get("/repo/<id>")]
pub(super) async fn bundle(id: DocId,
                           repo: State<'_, Repository>) -> Result<Json<BundleResponse>, ApiError> {
    let bundle = repo.get(id).await
        .ok_or_else(|| NotFound(format!("Bundle not found: {}", id)))?;

    return Ok(Json(BundleResponse {
        id: id.to_string(),
    }));
}

#[get("/repo/<id>/<fragment>")]
pub(super) async fn fragment(id: DocId,
                             fragment: String,
                             repo: State<'_, Repository>) -> Result<Content<Stream<impl AsyncRead>>, ApiError> {
    let kind = match fragment.as_str() {
        "document" => Kind::Document,
        "preview" => Kind::Preview,
        "plaintext" => Kind::Plaintext,
        "metadata" => Kind::Metadata,
        "process_log" => Kind::ProcessLog,
        s @ _ => Kind::other(s),
    };

    let bundle = repo.get(id).await
        .ok_or_else(|| NotFound(format!("Bundle not found: {}", id)))?;

    let fragment = bundle.fragment(kind).await
        .ok_or_else(|| NotFound(format!("Fragment not found: {}/{}", id, fragment)))?;

    let file = fragment.read().await
        .map_err(InternalError)?;

    let content_type = match fragment.kind() {
        Kind::Document => ContentType::PDF,
        Kind::Preview => ContentType::PNG,
        Kind::Plaintext => ContentType::Plain,
        Kind::Metadata => ContentType::JSON,
        Kind::ProcessLog => ContentType::Plain,
        Kind::Other { ref name } => ContentType::Any,
    };

    return Ok(Content(content_type, file.into()));
}

