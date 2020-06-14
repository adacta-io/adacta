use rocket::{get, State};
use rocket_contrib::json::Json;

use crate::model::{DocId, Label};
use crate::pigeonhole::Pigeonhole;
use crate::repo::{Repository, FragmentContainer};

use super::{ApiError, Token};
use std::collections::HashSet;

#[get("/labels")]
pub(super) async fn labels(pigeonhole: State<'_, Box<dyn Pigeonhole + Send + Sync>>,
                           _token: &'_ Token) -> Result<Json<HashSet<Label>>, ApiError> {
    let labels = pigeonhole.labels();

    Ok(Json(labels))
}

#[get("/labels/guess/<id>")]
pub(super) async fn guess(id: DocId,
                          repo: State<'_, Repository>,
                          pigeonhole: State<'_, Box<dyn Pigeonhole + Send + Sync>>,
                          _token: &'_ Token) -> Result<Json<HashSet<Label>>, ApiError> {
    let bundle = repo.get(id).await
        .ok_or_else(|| ApiError::not_found(format!("Bundle not found: {}", id)))?;
    let plaintext = bundle.plaintext().await
        .ok_or_else(|| ApiError::not_found(format!("Fragment not found: {}/plaintext", id)))??;

    let labels = pigeonhole.guess(&plaintext)
        .await?;

    Ok(Json(labels))
}
