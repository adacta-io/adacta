use rocket::{get, State};
use rocket_contrib::json::Json;
use serde::Serialize;

use crate::index::Index;
use crate::model::DocId;

use super::{ApiError, Token};

#[derive(Debug, Clone, Serialize)]
pub struct InboxResponse {
    count: u64,
    docs: Vec<DocId>,
}

#[get("/inbox")]
pub(super) async fn inbox(index: State<'_, Box<dyn Index + Send + Sync>>,
                          _token: &'_ Token) -> Result<Json<InboxResponse>, ApiError> {
    let response = index.inbox().await?;

    Ok(Json(InboxResponse {
        count: response.count,
        docs: response.docs,
    }))
}
