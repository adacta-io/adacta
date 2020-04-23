use std::path::Path;

use anyhow::Result;
use serde::Deserialize;
use tokio::fs::OpenOptions;
use tokio::io::AsyncReadExt;

#[derive(Debug, Clone, Deserialize)]
pub struct AuthConfig {
    pub username: String,
    pub passhash: String,

    pub secret: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RepositoryConfig {
    pub path: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ElasticsearchIndexConfig {
    pub url: String,
    pub index: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum IndexConfig {
    Elasticsearch(ElasticsearchIndexConfig),
}

#[derive(Debug, Clone, Deserialize)]
pub struct DockerJuicerConfig {
    pub image: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum JuicerConfig {
    Docker(DockerJuicerConfig),
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub auth: AuthConfig,

    pub repository: RepositoryConfig,

    pub index: IndexConfig,
    pub juicer: JuicerConfig,
}

impl Config {
    pub async fn load(path: impl AsRef<Path>) -> Result<Config> {
        let mut file = OpenOptions::new()
            .read(true)
            .open(path)
            .await?;

        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).await?;

        return Ok(serde_yaml::from_slice(&buffer)?);
    }
}
