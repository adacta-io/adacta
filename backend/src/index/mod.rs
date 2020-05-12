use anyhow::Result;
use async_trait::async_trait;

use crate::model::DocId;
use crate::repo::Bundle;

pub mod elasticsearch;

#[derive(Debug, Clone)]
pub struct SearchResponse {
    pub count: u64,
    pub docs: Vec<DocId>,
}

#[async_trait]
pub trait Index {
    async fn index(&self, bundle: &Bundle) -> Result<()>;

    async fn search(&self, query: &str) -> Result<SearchResponse>;
    async fn inbox(&self) -> Result<SearchResponse>;
}
