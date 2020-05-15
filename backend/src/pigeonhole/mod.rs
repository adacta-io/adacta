use anyhow::Result;

use async_trait::async_trait;

use crate::model::Label;
use std::collections::HashSet;

pub mod bayesian;
pub mod dumb;

#[async_trait]
pub trait Pigeonhole {
    fn labels(&self) -> HashSet<Label>;

    async fn guess(&self, text: &str) -> Result<HashSet<Label>>;
    async fn train(&mut self, text: &str, labels: HashSet<Label>) -> Result<()>;
}
