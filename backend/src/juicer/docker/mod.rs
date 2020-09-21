use anyhow::{anyhow, Result};
use async_trait::async_trait;
use bollard::{container, Docker};
use bollard::container::CreateContainerOptions;
use bollard::models::HostConfig;
use bytes::Bytes;
use log::{error, info};
use tokio::stream::StreamExt;

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
        let docker = Docker::connect_with_local_defaults()?;
        // docker.ping().await?; // TODO: Implement?

        let image = config.image
            .unwrap_or_else(|| Self::DOCKER_IMAGE.to_string());

        Ok(Self { docker, image })
    }
}

#[async_trait]
impl super::Juicer for Juicer {
    async fn extract<'r>(&self, bundle: &Bundle<'r, Staging>) -> Result<()> {
        let name = format!("juicer-{}", bundle.id());

        info!("Creating container {}", name);
        self.docker.create_container(Some(CreateContainerOptions { name: name.clone() }),
                                     container::Config {
                                         image: Some(self.image.clone()),
                                         env: Some(vec![format!("DID={}", bundle.id())]),
                                         network_disabled: Some(true),
                                         host_config: Some(HostConfig {
                                             binds: Some(vec![format!("{}:/juicer", bundle.path().display())]),
                                             ..HostConfig::default()
                                         }),
                                         ..container::Config::default()
                                     },
        ).await?;

        info!("Starting container {}", name);
        self.docker.start_container(&name, None::<container::StartContainerOptions<String>>).await?;

        let mut log_writer = bundle.write(Kind::other("juicer.log")).await?;
        let mut log_reader = tokio::io::stream_reader(self.docker.logs(&name,
                                                                       Some(container::LogsOptions {
                                                                           stdout: true,
                                                                           stderr: true,
                                                                           tail: String::from("all"),
                                                                           follow: true,
                                                                           ..container::LogsOptions::default()
                                                                       }))
            .map(|v| {
                info!(">> {:?}", v);
                match v {
                    Ok(out) => Ok(Bytes::from(format!("{}\n", out))),
                    Err(err) => Err(std::io::Error::new(std::io::ErrorKind::Other, err)),
                }
            }));
        let logs = tokio::io::copy(&mut log_reader, &mut log_writer);

        info!("Waiting for container to finish");
        let result = self.docker.wait_container(&name,
                                                Some(container::WaitContainerOptions {
                                                    condition: "next-exit",
                                                }))
            .next().await
            .unwrap()?; // TODO: ist this the way to use this?

        logs.await?;

        if result.status_code != 0 {
            error!("Container failed: {:?}", result);

            Err(anyhow!("Error while juicing: {}",
                        result.error.and_then(|err| err.message).unwrap_or_else(|| String::from("unknown"))))
        } else {
            Ok(())
        }
    }
}
