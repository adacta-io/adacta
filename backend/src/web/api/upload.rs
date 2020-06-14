use anyhow::Context;
use rocket::{post, Data, State};
use rocket_contrib::json::Json;
use serde::Serialize;

use crate::index::Index;
use crate::juicer::Juicer;
use crate::meta::Metadata;
use crate::model::Kind;
use crate::repo::{FragmentContainer, Repository};

use super::{ApiError, Token};
use crate::pigeonhole::Pigeonhole;

#[derive(Debug, Clone, Serialize)]
pub struct UploadResponse {
    id: String,
}

#[post("/upload", format = "application/pdf", data = "<data>")]
pub(super) async fn upload_pdf(data: Data,
                               repo: State<'_, Repository>,
                               index: State<'_, Box<dyn Index + Send + Sync>>,
                               juicer: State<'_, Box<dyn Juicer + Send + Sync>>,
                               pigeonhole: State<'_, Box<dyn Pigeonhole + Send + Sync>>,
                               _token: &'_ Token) -> Result<Json<UploadResponse>, ApiError> {
    // Create a new staging area
    let staging = repo.stage().await?;

    // Write the uploaded file to the staging area
    let original_fragment = staging.write(Kind::other("original.pdf")).await?;
    data.stream_to(original_fragment)
        .await
        .context("Writing original.pdf to staging")?;

    // Create initial metadata file for the uploaded bundle
    let metadata = Metadata::new();
    metadata.save(staging.write(Kind::Metadata).await?).await?;

    // Run the juicer over this upload
    juicer.extract(&staging).await?;

    // Detect and assign labels
    if let Some(plaintext) = staging.plaintext().await.transpose()? {
        let labels = pigeonhole.guess(&plaintext).await?;

        // Load, update and save metadata
        let mut metadata = staging.metadata().await.expect("No metadata")?;
        metadata.labels.extend(labels);
        metadata.save(staging.write(Kind::Metadata).await?).await?;
    }

    // Make a bundle from the staging
    let bundle = staging.create(&repo).await?;

    // Index the bundle
    index.index(&bundle).await?;

    Ok(Json(UploadResponse {
        id: bundle.id().to_string(),
    }))
}
