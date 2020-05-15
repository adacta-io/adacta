use std::collections::HashSet;
use std::path::{Path, PathBuf};

use anyhow::Result;

use async_trait::async_trait;

use crate::config::DumbPigeonhole as Config;
use crate::model::Label;

pub struct Pigeonhole {
    path: PathBuf,
    labels: HashSet<Label>,
}

impl Pigeonhole {
    pub async fn from_config(config: Config) -> Result<Self> {
        let path = PathBuf::from(config.path);

        let labels = Self::load(&path).await?;

        return Ok(Self { path, labels });
    }

    async fn load(path: impl AsRef<Path>) -> Result<HashSet<Label>> {
        match tokio::fs::read(path).await {
            Ok(data) => {
                return Ok(bincode::deserialize(&data)?);
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                return Ok(HashSet::new());
            }
            Err(err) => {
                return Err(err.into());
            }
        };
    }

    async fn save(path: impl AsRef<Path>, classifiers: &HashSet<Label>) -> Result<()> {
        let data = bincode::serialize(classifiers)?;
        tokio::fs::write(path, &data).await?;

        return Ok(());
    }
}

#[async_trait]
impl super::Pigeonhole for Pigeonhole {
    fn labels(&self) -> HashSet<Label> { return self.labels.clone(); }

    async fn guess(&self, _text: &str) -> Result<HashSet<Label>> { return Ok(HashSet::new()); }

    async fn train(&mut self, _text: &str, labels: HashSet<Label>) -> Result<()> {
        self.labels.extend(labels);

        Self::save(&self.path, &self.labels).await?;

        return Ok(());
    }
}
