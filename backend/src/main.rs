#![feature(proc_macro_hygiene)]
#![feature(decl_macro)]
#![feature(bool_to_option)]

use anyhow::Result;
use clap::{App, Arg};

use crate::auth::Authenticator;
use crate::config::{
    Config, Index as IndexConfig, Juicer as JuicerConfig, Pigeonhole as PigeonholeConfig
};
use crate::index::Index;
use crate::juicer::Juicer;
use crate::pigeonhole::Pigeonhole;
use crate::repo::Repository;

pub mod auth;
pub mod config;
pub mod index;
pub mod juicer;
pub mod meta;
pub mod model;
pub mod pigeonhole;
pub mod repo;
pub mod utils;
pub mod web;

#[tokio::main]
async fn main() -> Result<()> {
    let matches = App::new("Adacta")
        .version(env!("CARGO_PKG_VERSION"))
        .name(env!("CARGO_PKG_NAME"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Sets a custom config file")
                .takes_value(true)
                .default_value("config.yaml"),
        )
        .get_matches();

    let config = Config::load(matches.value_of("config").expect("No config arg")).await?;

    // Create auth instance
    let auth = Authenticator::from_config(config.auth).await?;

    // Open repository
    let repo = Repository::from_config(config.repository).await?;

    // Connect to index
    let index: Box<dyn Index + Send + Sync> = match config.index {
        IndexConfig::Elasticsearch(config) => {
            Box::new(crate::index::elasticsearch::Index::from_config(config).await?)
        }
    };

    // Create juicer instance
    let juicer: Box<dyn Juicer + Send + Sync> = match config.juicer {
        JuicerConfig::Docker(config) => {
            Box::new(crate::juicer::docker::Juicer::from_config(config).await?)
        }
    };

    // Load Pigeonhole
    let pigeonhole: Box<dyn Pigeonhole + Send + Sync> = match config.pigeonhole {
        PigeonholeConfig::Dumb(config) => {
            Box::new(crate::pigeonhole::dumb::Pigeonhole::from_config(config).await?)
        }
        PigeonholeConfig::Bayesic(config) => {
            Box::new(crate::pigeonhole::bayesian::Pigeonhole::from_config(config).await?)
        }
    };

    // Serve the HTTP Interface
    web::serve(auth, repo, index, juicer, pigeonhole).await?;

    return Ok(());
}
