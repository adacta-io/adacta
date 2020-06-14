use std::collections::HashMap;
use std::path::Path;

use anyhow::Result;
use serde::Deserialize;
use tokio::fs::OpenOptions;
use tokio::io::AsyncReadExt;

#[derive(Debug, Clone, Deserialize)]
pub struct DumbPigeonhole {
    pub path: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BayesicPigeonhole {
    pub path: String,
    pub certainty: f64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum Pigeonhole {
    Dumb(DumbPigeonhole),
    Bayesic(BayesicPigeonhole),
}

#[derive(Debug, Clone, Deserialize)]
pub struct Auth {
    pub username: String,
    pub passhash: String,

    pub secret: String,

    pub api_keys: HashMap<String, String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Repository {
    pub path: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ElasticsearchIndex {
    pub url: String,
    pub index: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum Index {
    Elasticsearch(ElasticsearchIndex),
}

#[derive(Debug, Clone, Deserialize)]
pub struct DockerJuicer {
    pub image: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum Juicer {
    Docker(DockerJuicer),
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub auth: Auth,

    pub repository: Repository,

    pub index: Index,
    pub juicer: Juicer,
    pub pigeonhole: Pigeonhole,
}

impl Config {
    pub async fn load(path: impl AsRef<Path>) -> Result<Self> {
        let mut file = OpenOptions::new().read(true).open(path).await?;

        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).await?;

        Ok(serde_yaml::from_slice(&buffer)?)
    }
}
