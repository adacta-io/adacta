use rocket::{get, State, http::RawStr, http::Status};
use rocket_contrib::json::Json;
use serde::Serialize;

use crate::model::DocId;
use crate::repo::Repository;
use crate::index::Index;
use crate::api::{InternalError, ApiError};

#[derive(Debug, Clone, Serialize)]
pub struct InboxResponse {
    count: u64,
    docs: Vec<DocId>,
}

#[get("/inbox")]
pub(super) async fn inbox(repo: State<'_, Repository>,
                          index: State<'_, Box<dyn Index + Send + Sync>>) -> Result<Json<InboxResponse>, ApiError> {
    let response = index.inbox().await?;

    return Ok(Json(InboxResponse {
        count: response.count,
        docs: response.docs,
    }));
}
