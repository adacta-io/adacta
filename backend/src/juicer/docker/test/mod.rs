use rust_embed::RustEmbed;
use shiplift::BuildOptions;
use tokio::io::AsyncWriteExt;

use crate::meta::Metadata;
use crate::repository::Repository;

use super::*;

mod extract;

#[derive(RustEmbed, Debug, Clone)]
#[folder = "src/juicer/docker/test/resources"]
pub struct Resources;

pub async fn juicer() -> Result<Juicer> {
    let docker = Docker::new();
    let images = docker.images();

    let build = BuildOptions::builder("../juicer")
        .network_mode("host")
        .rm(true)
        .forcerm(true)
        .build();
    let mut image = images.build(&build);

    let id = loop {
        let result = image.next().await
            .with_context(|| "Error building docker image")?;
        let result = result
            .with_context(|| "Finished without ID")?;

        if let Some(stream) = result.get("stream").and_then(|o| o.as_str()) {
            print!("{}", stream);
        }

        if let Some(id) = result.get("aux").and_then(|o| o.get("ID")).and_then(|o| o.as_str()) {
            break id.to_string();
        }
    };

    let juicer = Juicer::from_config(Config { image: Some(id) }).await?;

    return Ok(juicer);
}

pub async fn upload<'r>(repository: &'r Repository, metadata: Metadata, original: &str) -> Result<Bundle<'r, Staging>> {
    let bundle = repository.stage().await?;

    let metadata_fragment = bundle.write(Kind::Metadata).await?;
    metadata.save(metadata_fragment).await?;

    bundle.write(Kind::other("original.pdf")).await?
        .write_all(&Resources::get(original).unwrap()).await?;

    return Ok(bundle);
}