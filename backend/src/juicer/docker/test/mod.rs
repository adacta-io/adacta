use bollard::image::BuildImageOptions;
use rust_embed::RustEmbed;
use tokio::io::AsyncWriteExt;

use crate::meta::Metadata;
use crate::repository::Repository;

use super::*;

mod extract;

#[derive(RustEmbed, Debug, Clone)]
#[folder = "src/juicer/docker/test/resources"]
pub struct Resources;

pub async fn juicer() -> Result<Juicer> {
    let mut body = Vec::<u8>::new();

    {
        let mut tar = tar::Builder::new(&mut body);
        tar.append_dir_all("./", "../juicer")?;
        tar.finish()?;
    }

    let docker = Docker::connect_with_local_defaults()?;
    let mut image = docker.build_image(BuildImageOptions {
        dockerfile: "Dockerfile",
        ..Default::default()
    }, None, Some(body.into()));

    let id = loop {
        match image.next().await {
            Some(Ok(info)) => {
                if let Some(aux) = info.aux {
                    if let Some(id) = aux.id {
                        break id;
                    }
                }
            }
            Some(Err(err)) => {
                return Err(err.into());
            }
            None => {
                return Err(anyhow!("Finished without ID"));
            }
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