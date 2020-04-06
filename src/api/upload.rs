use rocket::{Data, http::Status, post, response::status::Custom, State};
use rocket_contrib::json::Json;
use serde::Serialize;
use tokio_util::compat::FuturesAsyncWriteCompatExt;

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
pub async fn upload_pdf(data: Data,
                        repo: State<'_, Repository>,
                        index: State<'_, Box<dyn Index + Send + Sync>>,
                        juicer: State<'_, Box<dyn Juicer + Send + Sync>>) -> Result<Json<UploadResponse>, Custom<String>> {
    // Create a new staging area
    let staging = repo.stage().await
        .map_err(|err| Custom(Status::InternalServerError, err.to_string()))?;

    // Write the uploaded file to the staging area
    let original_fragment = staging.write(Kind::other("original.pdf")).await
        .map_err(|err| Custom(Status::InternalServerError, err.to_string()))?;
    data.stream_to(original_fragment.compat_write()).await
        .map_err(|err| Custom(Status::InternalServerError, err.to_string()))?;

    // Create initial metadata file for the uploaded bundle
    let metadata = Metadata::new();
    let metadata_fragment = staging.write(Kind::Metadata).await
        .map_err(|err| Custom(Status::InternalServerError, err.to_string()))?;
    metadata.save(metadata_fragment).await
        .map_err(|err| Custom(Status::InternalServerError, err.to_string()))?;

    // Run the juicer over this upload
    juicer.extract(&staging).await
        .map_err(|err| Custom(Status::InternalServerError, err.to_string()))?;

    // Make a bundle from the staging
    let bundle = staging.create(&repo).await
        .map_err(|err| Custom(Status::InternalServerError, err.to_string()))?;

    // Index the bundle
    index.index(&bundle).await
        .map_err(|err| Custom(Status::InternalServerError, err.to_string()))?;

    return Ok(Json(UploadResponse {
        id: bundle.id().to_string(),
    }));
}
