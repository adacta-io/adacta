use anyhow::Result;
use async_trait::async_trait;

use crate::repo::BundleStaging;

pub mod docker;

#[async_trait]
pub trait Juicer {
    async fn extract(&self, bundle: &BundleStaging) -> Result<()>;
}
