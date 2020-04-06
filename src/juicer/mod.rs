use async_trait::async_trait;
use anyhow::Result;

use crate::repo::BundleStaging;

pub mod docker;

#[async_trait]
pub trait Juicer {
    async fn extract(&self, bundle: &BundleStaging) -> Result<()>;
}