use anyhow::Result;
use async_trait::async_trait;

#[cfg(test)]
use mockall::automock;

use crate::repository::{Bundle, Staging};

pub mod docker;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait Juicer {
    async fn extract<'r>(&self, bundle: &Bundle<'r, Staging>) -> Result<()>;
}
