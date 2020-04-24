use anyhow::Context;
use rocket::{Data, http::Status, post, response::status::Custom, State};
use rocket_contrib::json::Json;
use serde::Serialize;

use crate::api::{ApiError, InternalError};
use crate::auth::Token;
use crate::index::Index;
use crate::juicer::Juicer;
use crate::meta::Metadata;
use crate::model::Kind;
use crate::repo::Repository;

#[derive(Debug, Clone, Serialize)]
pub struct UploadResponse {
    id: String,
}

#[post("/upload", format = "application/pdf", data = "<data>")]
pub(super) async fn upload_pdf(data: Data,
                               repo: State<'_, Repository>,
                               index: State<'_, Box<dyn Index + Send + Sync>>,
                               juicer: State<'_, Box<dyn Juicer + Send + Sync>>,
                               token: &'_ Token) -> Result<Json<UploadResponse>, ApiError> {
    // Create a new staging area
    let staging = repo.stage().await?;

    // Write the uploaded file to the staging area
    let original_fragment = staging.write(Kind::other("original.pdf")).await?;
    data.stream_to(original_fragment).await
        .context("Writing original.pdf to staging")?;

    // Create initial metadata file for the uploaded bundle
    let metadata = Metadata::new();

    let metadata_fragment = staging.write(Kind::Metadata).await?;
    metadata.save(metadata_fragment).await?;

    // Run the juicer over this upload
    juicer.extract(&staging).await?;

    // Detect and assign tags
    // TODO: Implement

    // Make a bundle from the staging
    let bundle = staging.create(&repo).await?;

    // Index the bundle
    index.index(&bundle).await?;

    return Ok(Json(UploadResponse {
        id: bundle.id().to_string(),
    }));
}
