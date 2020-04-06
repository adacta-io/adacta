use anyhow::Result;
use async_std::fs::OpenOptions;
use async_std::path::Path;
use futures::AsyncReadExt;
use serde::Deserialize;

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
    pub repository: String,

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
