use std::collections::HashSet;

use anyhow::Result;
use async_trait::async_trait;

#[cfg(test)]
use mockall::automock;

use crate::model::Label;

pub mod bayesian;
pub mod dumb;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait Suggester {
    async fn labels(&self) -> HashSet<Label>;

    async fn guess(&self, text: &str) -> Result<HashSet<Label>>; // TODO: Can this be a stream of tokens or a reader?
    async fn train(&self, text: &str, expected_labels: &HashSet<Label>) -> Result<()>;
}
