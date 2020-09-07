use anyhow::Context;
use rocket::{post, Data, State};
use rocket_contrib::json::Json;
use serde::Serialize;

use crate::juicer::Juicer;
use crate::meta::Metadata;
use crate::model::Kind;
use crate::repository::Repository;

use super::{ApiError, Token};
use rocket::data::ToByteUnit;

#[derive(Debug, Clone, Serialize)]
pub struct UploadResponse {
    id: String,
}

#[post("/upload", format = "application/pdf", data = "<data>")]
pub(super) async fn upload_pdf(data: Data,
                               repository: State<'_, Repository>,
                               juicer: State<'_, Box<dyn Juicer + Send + Sync>>,
                               _token: &'_ Token) -> Result<Json<UploadResponse>, ApiError> {
    // Create a new staging area
    let staging = repository.stage().await?;

    match (|| async {
        // Write the uploaded file to the staging area
        let original_fragment = staging.write(Kind::other("original.pdf")).await?;
        data.open(512.mebibytes()) // TODO: Make this limit configurable
            .stream_to(original_fragment).await
            .context("Writing original.pdf to staging")?;

        // Create initial metadata file for the uploaded bundle
        let metadata = Metadata::new();
        metadata.save(staging.write(Kind::Metadata).await?).await?;

        // Run the juicer over this upload
        juicer.extract(&staging).await?;

        return Result::<_, ApiError>::Ok(());
    })().await {
        Ok(()) => {
            // Make a inboxed bundle from the staging
            let bundle = staging.create().await?;

            return Ok(Json(UploadResponse {
                id: bundle.id().to_string(),
            }));
        }
        Err(err) => {
            // Delete the staging bundle
            staging.delete().await?;

            return Err(err);
        }
    }
}
