use bollard::{container, Docker};
use bytes::Bytes;
use tokio::stream::StreamExt;

use anyhow::{anyhow, Result};
use async_trait::async_trait;

use crate::config::DockerJuicer;
use crate::model::Kind;
use crate::repo::{BundleStaging, FragmentContainer};

pub struct Juicer {
    docker: Docker,

    image: String,
}

impl Juicer {
    const DOCKER_IMAGE: &'static str = "adacta10/juicer";

    pub async fn from_config(config: DockerJuicer) -> Result<Self> {
        let docker = Docker::connect_with_local_defaults()?;
        // docker.ping().await?; // TODO: Implement?

        let image = config
            .image
            .unwrap_or_else(|| Self::DOCKER_IMAGE.to_string());

        return Ok(Self { docker, image });
    }
}

#[async_trait]
impl super::Juicer for Juicer {
    async fn extract(&self, bundle: &BundleStaging) -> Result<()> {
        let name = format!("juicer-{}", bundle.id());

        self.docker
            .create_container(
                Some(container::CreateContainerOptions { name: name.clone() }),
                container::Config {
                    image: Some(self.image.clone()),
                    env: Some(vec![format!("DID={}", bundle.id())]),
                    network_disabled: Some(true),
                    host_config: Some(container::HostConfig {
                        binds: Some(vec![format!("{}:/juicer", bundle.path().display())]),
                        ..container::HostConfig::default()
                    }),
                    ..container::Config::default()
                },
            )
            .await?;

        self.docker
            .start_container(&name, None::<container::StartContainerOptions<String>>)
            .await?;

        let mut log_writer = bundle.write(Kind::other("juicer.log")).await?;
        let mut log_reader = tokio::io::stream_reader(
            self.docker
                .logs(
                    &name,
                    Some(container::LogsOptions {
                        stdout: true,
                        stderr: true,
                        tail: String::from("all"),
                        follow: true,
                        ..container::LogsOptions::default()
                    }),
                )
                .map(|v| match v {
                    Ok(out) => Ok(Bytes::from(format!("{}", out))),
                    Err(err) => Err(std::io::Error::new(std::io::ErrorKind::Other, err)),
                }),
        );
        let logs = tokio::io::copy(&mut log_reader, &mut log_writer);

        let result = self
            .docker
            .wait_container(
                &name,
                Some(container::WaitContainerOptions {
                    condition: "not-running",
                }),
            )
            .next()
            .await
            .unwrap()?; // TODO: ist this the way to use this?

        logs.await?;

        if result.status_code != 0 {
            return Err(anyhow!(
                "Error while juicing: {}",
                result
                    .error
                    .map_or_else(|| String::from("unknown"), |err| err.message)
            ));
        }

        return Ok(());
    }
}
