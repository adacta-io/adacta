#![feature(proc_macro_hygiene)]
#![feature(decl_macro)]

use anyhow::Result;

use crate::config::{Config, IndexConfig, JuicerConfig};
use crate::index::Index;
use crate::juicer::Juicer;
use crate::repo::Repository;

pub mod meta;
pub mod repo;
pub mod index;
pub mod api;
pub mod model;
pub mod config;
pub mod juicer;


#[async_std::main]
async fn main() -> Result<()> {
    let config = Config::load("adacta.yaml").await?;

    // Open repository
    let repo = Repository::open(config.repository).await?;

    // Create indexier instance
    let index: Box<dyn Index + Send + Sync> = match config.index {
        IndexConfig::Elasticsearch(config) => Box::new(crate::index::elasticsearch::Index::from_config(config).await?),
    };

    // Create juicer instance
    let juicer: Box<dyn Juicer + Send + Sync> = match config.juicer {
        JuicerConfig::Docker(config) => Box::new(crate::juicer::docker::Juicer::from_config(config).await?),
    };

    rocket::ignite()
        .manage(repo)
        .manage(index)
        .manage(juicer)
//        .mount("/", routes![hello])
        .mount("/api", api::routes())
        .launch()?;

    return Ok(());
}
