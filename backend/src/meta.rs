use std::collections::{HashMap, HashSet};

use anyhow::Result;
use chrono::{DateTime, Utc};
use proto::model::Label;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Metadata {
    pub uploaded: DateTime<Utc>,
    pub archived: Option<DateTime<Utc>>,

    pub title: Option<String>,
    pub pages: u32,

    pub labels: HashSet<Label>,

    pub properties: HashMap<String, String>,
}

impl Metadata {
    pub fn new() -> Self {
        Self {
            uploaded: Utc::now(),
            archived: None,
            title: None,
            pages: 0,
            labels: HashSet::new(),
            properties: HashMap::new(),
        }
    }

    pub async fn load(mut r: impl AsyncRead + Unpin) -> Result<Self> {
        let mut buffer = Vec::new();
        r.read_to_end(&mut buffer).await?;

        Ok(serde_json::from_slice(&buffer)?)
    }

    pub async fn save(&self, mut w: impl AsyncWrite + Unpin) -> Result<()> {
        return Ok(w.write_all(&self.to_vec()?).await?);
    }

    pub fn to_vec(&self) -> Result<Vec<u8>> {
        return Ok(serde_json::to_vec_pretty(self)?);
    }
}

impl Default for Metadata {
    fn default() -> Self { Self::new() }
}

impl Into<proto::model::Metadata> for Metadata {
    fn into(self) -> proto::model::Metadata {
        return proto::model::Metadata {
            uploaded: self.uploaded,
            archived: self.archived,
            title: self.title,
            pages: self.pages,
            labels: self.labels,
            properties: self.properties,
        };
    }
}
