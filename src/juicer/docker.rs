use anyhow::{Result, Error, anyhow};
use bollard::{container, Docker};
use bollard::container::{CreateContainerOptions, LogsOptions, StartContainerOptions, WaitContainerOptions};
use futures::stream::StreamExt;

use async_trait::async_trait;

use crate::config::DockerJuicerConfig;
use crate::repo::BundleStaging;
use crate::model::Kind;

use futures::{TryStreamExt, AsyncWriteExt, SinkExt};
use std::collections::HashMap;

pub struct Juicer {
    docker: Docker,

    image: String,
}

impl Juicer {
    const DOCKER_IMAGE: &'static str = "adacta/juicer";

    pub async fn from_config(config: DockerJuicerConfig) -> Result<Self> {
        let docker = Docker::connect_with_unix_defaults()?;
        docker.ping().await?;

        let image = config.image.unwrap_or_else(|| Self::DOCKER_IMAGE.to_string());

        return Ok(Self {
            docker,
            image,
        });
    }
}

#[async_trait]
impl super::Juicer for Juicer {
    async fn extract(&self, bundle: &BundleStaging) -> Result<()> {
        let name = format!("juicer-{}", bundle.id());

        let volumes = HashMap::new();

        self.docker.create_container(
            Some(CreateContainerOptions { name: name.clone() }),
            container::Config {
                image: Some(self.image.clone()),
                env: Some(vec![ format!("DID={}", bundle.id()) ]),
                volumes: Some(volumes),
                network_disabled: Some(true),
                ..Default::default()
            },
        ).await?;

        self.docker.start_container(&name,
                                    None::<StartContainerOptions<String>>).await?;

        let logs = bundle.write(Kind::other("juicer.log")).await?
            .into_sink()
            .sink_map_err(Error::from);
        let logs = self.docker.logs(
            &name,
            Some(LogsOptions {
                stdout: true,
                stderr: true,
                tail: String::from("all"),
                follow: true,
                ..Default::default()
            }))
            .map_ok(|out| format!("{}", out).into_bytes())
            .map_err(Error::from)
            .forward(logs);

        let result = self.docker.wait_container(&name,
                                                Some(WaitContainerOptions {
                                                    condition: "not-running",
                                                })).fuse().select_next_some().await?;


        logs.await?;

        if result.status_code != 0 {
            return Err(anyhow!("Error while juicing: {}", result.error.map(|err| err.message).unwrap_or_else(|| String::from("unknown"))));
        }

        return Ok(());
    }
}