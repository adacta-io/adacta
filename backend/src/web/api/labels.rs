use std::collections::HashSet;

use rocket::{get, State};
use rocket_contrib::json::Json;

use crate::proto::model::Label;
use crate::suggester::Suggester;

use super::{ApiError, Token};

#[get("/labels")]
pub(super) async fn list(suggester: State<'_, Box<dyn Suggester + Send + Sync>>,
                         _token: &'_ Token) -> Result<Json<HashSet<Label>>, ApiError> {
    let labels = suggester.labels().await;

    Ok(Json(labels))
}

// #[get("/labels/guess/<id>")]
// pub(super) async fn guess(id: DocId,
//                           repo: State<'_, Repository>,
//                           pigeonhole: State<'_, Box<dyn Pigeonhole + Send + Sync>>,
//                           _token: &'_ Token) -> Result<Json<HashSet<Label>>, ApiError> {
//     let bundle = repo.get(id).await
//         .ok_or_else(|| ApiError::not_found(format!("Bundle not found: {}", id)))?;
//     let plaintext = bundle.plaintext().await
//         .ok_or_else(|| ApiError::not_found(format!("Fragment not found: {}/plaintext", id)))??;
//
//     let labels = pigeonhole.guess(&plaintext)
//         .await?;
//
//     Ok(Json(labels))
// }
