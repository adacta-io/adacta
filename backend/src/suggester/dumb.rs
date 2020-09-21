use std::collections::HashSet;
use std::path::{Path, PathBuf};

use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::config::DumbSuggester as Config;
use crate::proto::model::Label;

pub struct Suggester {
    path: PathBuf,
    labels: RwLock<HashSet<Label>>,
}

impl Suggester {
    pub async fn from_config(config: Config) -> Result<Self> {
        let path = PathBuf::from(config.path);

        let labels = RwLock::new(Self::load(&path).await?);

        Ok(Self { path, labels })
    }

    async fn load(path: impl AsRef<Path>) -> Result<HashSet<Label>> {
        match tokio::fs::read(path).await {
            Ok(data) => Ok(bincode::deserialize(&data)?),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(HashSet::new()),
            Err(err) => Err(err.into()),
        }
    }

    async fn save(path: impl AsRef<Path>, classifiers: &HashSet<Label>) -> Result<()> {
        let data = bincode::serialize(classifiers)?;
        tokio::fs::write(path, &data).await?;

        Ok(())
    }
}

#[async_trait]
impl super::Suggester for Suggester {
    async fn labels(&self) -> HashSet<Label> {
        let labels = self.labels.read().await;
        return labels.clone();
    }

    async fn guess(&self, _text: &str) -> Result<HashSet<Label>> { Ok(HashSet::new()) }

    async fn train(&self, _text: &str, expected_labels: &HashSet<Label>) -> Result<()> {
        let mut labels = self.labels.write().await;

        labels.extend(expected_labels.clone());

        Self::save(&self.path, &labels).await?;

        return Ok(());
    }
}
