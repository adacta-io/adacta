use rocket::{get, http::RawStr, State};
use rocket_contrib::json::Json;
use serde::Serialize;

use crate::auth::Token;
use crate::index::Index;
use crate::model::DocId;
use crate::repo::Repository;

use super::ApiError;

#[derive(Debug, Clone, Serialize)]
pub struct SearchResponse {
    count: u64,
    docs: Vec<DocId>,
}

#[get("/search?<query>")]
pub(super) async fn search(query: &RawStr,
                           repo: State<'_, Repository>,
                           index: State<'_, Box<dyn Index + Send + Sync>>,
                           _token: &'_ Token) -> Result<Json<SearchResponse>, ApiError> {
    let response = index.search(query).await?;

    return Ok(Json(SearchResponse {
        count: response.count,
        docs: response.docs,
    }));
}
