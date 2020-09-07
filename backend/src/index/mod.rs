use anyhow::Result;
use async_trait::async_trait;

#[cfg(test)]
use mockall::automock;

use crate::model::DocId;
use crate::repository::{Archived, Bundle};

pub mod elasticsearch;

#[derive(Debug, Clone)]
pub struct SearchResponse {
    pub count: u64,
    pub docs: Vec<DocId>,
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait Index {
    async fn index<'r>(&self, bundle: &Bundle<'r, Archived>) -> Result<()>;

    async fn search(&self, query: &str) -> Result<SearchResponse>;
    async fn inbox(&self) -> Result<SearchResponse>;
}
