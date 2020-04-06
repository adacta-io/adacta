use rocket::{get, State};
use rocket_contrib::json::Json;
use serde::Serialize;

use crate::model::DocId;
use crate::repo::Repository;

#[derive(Debug, Clone, Serialize)]
pub struct BundleResponse {
    id: String,

//    created: DateTime<Utc>,
//    modified: DateTime<Utc>,

    // Other metadata...
}

//#[get("/repo")]
//pub fn list() -> &'static str {
//    return "Hello, world!";
//}

#[get("/repo/<id>")]
pub async fn bundle(id: DocId,
                        repo: State<'_, Repository>) -> Option<Json<BundleResponse>> {
    let bundle = repo.get(id).await?;

    return Some(Json(BundleResponse {
        id: id.to_string(),
    }));
}

#[get("/repo/<id>/<fragment>")]
pub async fn fragment(id: DocId,
                          fragment: String,
                          repo: State<'_, Repository>) -> &'static str {
//    let kind = Kind::
//
//    NamedFile::open();

    return "Hello, world!";
}

