use anyhow::{Context, Result};
use async_trait::async_trait;
use futures::StreamExt;
use log::{debug, error, trace};
use shiplift::{ContainerOptions, Docker, LogsOptions};
use tokio::io::AsyncWriteExt;

use crate::config::DockerJuicer as Config;
use crate::proto::model::Kind;
use crate::repository::{Bundle, Staging};

#[cfg(test)]
mod test;

pub struct Juicer {
    docker: Docker,

    image: String,
}

impl Juicer {
    const DOCKER_IMAGE: &'static str = "adacta10/juicer:develop";

    pub async fn from_config(config: Config) -> Result<Self> {
        let docker = Docker::new();
        // docker.ping().await?; // TODO: Implement?

        let image = config.image
            .unwrap_or_else(|| Self::DOCKER_IMAGE.to_string());

        Ok(Self { docker, image })
    }
}

#[async_trait]
impl super::Juicer for Juicer {
    async fn extract<'r>(&self, bundle: &Bundle<'r, Staging>) -> Result<()> {
        // Open the log file
        let mut logfile = bundle.write(Kind::other("juicer.log")).await
            .with_context(|| "Failed to open juicer.log")?;

        let containers = self.docker.containers();

        debug!("Creating container");
        let create = ContainerOptions::builder(&self.image)
            .name(&format!("juicer-{}", bundle.id()))
            .network_mode("none")
            .volumes(vec![
                &format!("{}:/juicer", bundle.path().display())
            ])
            .build();
        let container = containers.create(&create).await
            .with_context(|| format!("Error creating container (image={})", self.image))?;
        let container = containers.get(&container.id);

        debug!("Starting container (id={})", container.id());
        container.start().await
            .with_context(|| format!("Error starting container (id={})", container.id()))?;

        // Read the output from container and write to log file
        let mut output = container.logs(&LogsOptions::builder()
            .follow(true)
            .stdout(true)
            .stderr(true)
            .build());
        while let Some(chunk) = output.next().await {
            let chunk = chunk.with_context(|| format!("Error running container (id={})", container.id()))?;

            trace!("{}: {}", container.id(), String::from_utf8_lossy(&chunk));

            logfile.write_all(&chunk).await
                .with_context(|| "Failed to write log")?;
        }

        debug!("Waiting for container to finish (id={})", container.id());
        let result = container.wait().await
            .with_context(|| format!("Error waiting for container (id={})", container.id()))?;

        debug!("Deleting container (id={})", container.id());
        container.delete().await
            .with_context(|| format!("Error deleting container (id={})", container.id()))?;

        // Fail with error depending on status-code
        if result.status_code != 0 {
            error!("Container failed (id={}): {}", container.id(), result.status_code);
            anyhow::bail!("Juicing failed (id={}): {}", container.id(), result.status_code);
        }

        return Ok(());
    }
}
