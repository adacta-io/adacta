use rocket::{get, State};
use rocket_contrib::json::Json;
use serde::Serialize;

use crate::auth::Token;
use crate::index::Index;
use crate::model::DocId;
use crate::repo::Repository;

use super::ApiError;

#[derive(Debug, Clone, Serialize)]
pub struct InboxResponse {
    count: u64,
    docs: Vec<DocId>,
}

#[get("/inbox")]
pub(super) async fn inbox(
    _repo: State<'_, Repository>,
    index: State<'_, Box<dyn Index + Send + Sync>>,
    _token: &'_ Token,
) -> Result<Json<InboxResponse>, ApiError>
{
    let response = index.inbox().await?;

    return Ok(Json(InboxResponse {
        count: response.count,
        docs: response.docs,
    }));
}
