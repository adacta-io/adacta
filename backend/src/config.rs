use std::collections::HashMap;
use std::path::Path;

use anyhow::Result;
use serde::Deserialize;
use tokio::fs::OpenOptions;
use tokio::io::AsyncReadExt;

#[derive(Debug, Clone, Deserialize)]
pub struct Auth {
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
pub struct DumbSuggester {
    pub path: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BayesicSuggester {
    pub path: String,
    pub certainty: f64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum Suggester {
    Dumb(DumbSuggester),
    Bayesic(BayesicSuggester),
}

#[derive(Debug, Clone, Deserialize)]
pub struct Web {
    pub address: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub auth: Auth,

    pub repository: Repository,

    pub index: Index,
    pub juicer: Juicer,
    pub suggester: Suggester,

    pub web: Web,
}

impl Config {
    pub async fn load(path: impl AsRef<Path>) -> Result<Self> {
        let mut file = OpenOptions::new().read(true).open(path).await?;

        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).await?;

        Ok(serde_yaml::from_slice(&buffer)?)
    }
}
