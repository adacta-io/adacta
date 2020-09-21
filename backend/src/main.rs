#![feature(bool_to_option)]

pub use adacta_proto as proto;
use anyhow::Result;
use clap::{App, Arg};

use crate::auth::Authenticator;
use crate::config::{Config, Index as IndexConfig, Juicer as JuicerConfig, Suggester as SuggesterConfig};
use crate::index::Index;
use crate::juicer::Juicer;
use crate::repository::Repository;
use crate::suggester::Suggester;

pub mod auth;
pub mod config;
pub mod index;
pub mod juicer;
pub mod meta;
pub mod suggester;
pub mod repository;
pub mod utils;
pub mod web;

#[tokio::main]
async fn main() -> Result<()> {
    let matches = App::new("adacta")
        .version(env!("CARGO_PKG_VERSION"))
        .name(env!("CARGO_PKG_NAME"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .arg(Arg::with_name("debug")
            .long("debug")
            .help("Enable debug messages")
            .takes_value(false))
        .arg(Arg::with_name("config")
            .short("c")
            .long("config")
            .value_name("FILE")
            .help("Sets a custom config file")
            .takes_value(true)
            .default_value("adacta.yaml"))
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

    // Load suggester
    let suggester: Box<dyn Suggester + Send + Sync> = match config.suggester {
        SuggesterConfig::Dumb(config) => {
            Box::new(crate::suggester::dumb::Suggester::from_config(config).await?)
        }
        SuggesterConfig::Bayesic(config) => {
            Box::new(crate::suggester::bayesian::Suggester::from_config(config).await?)
        }
    };

    // Serve the HTTP Interface
    web::server(config.web, auth, repo, index, juicer, suggester)?.launch().await?;

    return Ok(());
}
