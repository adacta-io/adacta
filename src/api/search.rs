use rocket::{get, State, http::RawStr, http::Status};
use rocket_contrib::json::Json;
use serde::Serialize;

use crate::model::DocId;
use crate::repo::Repository;
use crate::index::Index;

#[derive(Debug, Clone, Serialize)]
pub struct SearchResponse {
    count: usize,
    docs: Vec<DocId>,
}

#[get("/search?<query>")]
pub async fn search(query: &RawStr,
                    repo: State<'_, Repository>,
                    index: State<'_, Box<dyn Index + Send + Sync>>) -> Result<Json<SearchResponse>, Status> {
    let docs = index.search(query).await
        .map_err(|e| Status::BadRequest)?; // TODO: Forward error message

    return Ok(Json(SearchResponse {
        count: docs.len(),
        docs,
    }));
}

#[get("/search/inbox")]
pub async fn inbox(repo: State<'_, Repository>,
                   index: State<'_, Box<dyn Index + Send + Sync>>) -> Result<Json<SearchResponse>, Status> {
    let docs = index.inbox().await
        .map_err(|e| Status::BadRequest)?; // TODO: Forward error message

    return Ok(Json(SearchResponse {
        count: docs.len(),
        docs,
    }));
}
