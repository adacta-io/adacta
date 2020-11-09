use anyhow::Context;
use log::{info, trace};
use rocket::{Data, post, State};
use rocket::data::ToByteUnit;
use rocket_contrib::json::Json;

use crate::juicer::Juicer;
use crate::meta::Metadata;
use crate::proto::api::upload::UploadResponse;
use crate::proto::model::{DocInfo, Kind};
use crate::repository::Repository;

use super::{ApiError, Token};

#[post("/upload", format = "application/pdf", data = "<data>")]
pub(super) async fn upload_pdf(data: Data,
                               repository: State<'_, Repository>,
                               juicer: State<'_, Box<dyn Juicer + Send + Sync>>,
                               _token: &'_ Token) -> Result<Json<UploadResponse>, ApiError> {
    // Create a new staging area
    let staging = repository.stage().await?;

    info!("Uploading to staging bundle {}", staging.id());

    match (|| async {
        // Write the uploaded file to the staging area
        let original_fragment = staging.write(Kind::other("original.pdf")).await?;
        data.open(512.mebibytes()) // TODO: Make this limit configurable
            .stream_to(original_fragment).await
            .context("Writing original.pdf to staging")?;

        trace!("Original fragment written");

        // Create initial metadata file for the uploaded bundle
        let metadata = Metadata::new();
        metadata.save(staging.write(Kind::Metadata).await?).await?;

        trace!("Metadata fragment written");

        // Run the juicer over this upload
        juicer.extract(&staging).await?;

        trace!("Juicer finished");

        return Result::<_, ApiError>::Ok(());
    })().await {
        Ok(()) => {
            // Make a inboxed bundle from the staging
            let bundle = staging.create().await?;
            let metadata = bundle.read_metadata().await?;

            return Ok(Json(UploadResponse {
                doc: DocInfo {
                    id: *bundle.id(),
                    metadata: metadata.into(),
                }
            }));
        }
        Err(err) => {
            // Delete the staging bundle
            staging.delete().await?;

            return Err(err);
        }
    }
}
