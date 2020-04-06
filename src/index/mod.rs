use anyhow::Result;
use async_trait::async_trait;

use crate::model::DocId;
use crate::repo::Bundle;

pub mod elasticsearch;

#[async_trait]
pub trait Index {
    async fn index(&self, bundle: &Bundle) -> Result<()>;

    async fn search(&self, query: &str) -> Result<Vec<DocId>>;
    async fn inbox(&self) -> Result<Vec<DocId>>;
}

