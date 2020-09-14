#![feature(str_split_once)]

use anyhow::Result;
use clap::{AppSettings, Arg, SubCommand};

use crate::config::Config;
use colored::Colorize;
use crate::output::Output;
use std::io::stdout;

mod config;
mod output;
mod client;
mod upload;
mod inbox;
mod archive;

#[tokio::main]
async fn main() {
    let matches = SubCommand::with_name("adacta-cli")
        .version(env!("CARGO_PKG_VERSION"))
        .name(env!("CARGO_PKG_NAME"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .setting(AppSettings::ColoredHelp)
        .setting(AppSettings::GlobalVersion)
        .setting(AppSettings::InferSubcommands)
        // .arg(Arg::with_name("verbose")
        //     .long("verbose")
        //     .short("v")
        //     .help("Enable verbose output"))
        // .arg(Arg::with_name("dry")
        //     .long("dry-run")
        //     .short("n")
        //     .help("Do not actually execute actions, just show them"))
        // .arg(Arg::with_name("yes")
        //     .long("yes")
        //     .short("y")
        //     .help("Do not ask for confirmation"))
        .arg(Arg::with_name("config")
            .long("config")
            .short("c")
            .help("The config file to use")
            .takes_value(true))
        .subcommand(SubCommand::with_name("config")
            .about("Configure the client")
            .arg(Arg::with_name("target")
                .long("target")
                .short("t")
                .help("The server URL")
                .takes_value(true)
                .required(true))
            .arg(Arg::with_name("username")
                .help("The application key username")
                .long("username")
                .short("u")
                .takes_value(true))
            .arg(Arg::with_name("password")
                .long("password")
                .short("p")
                .help("The login or application key password")
                .takes_value(true)
                .required(true)))
        .subcommand(SubCommand::with_name("upload")
            .about("Uploads a document")
            .arg(Arg::with_name("pdf")
                .help("The PDF document to upload")
                .takes_value(true)
                .required(true)))
        .subcommand(SubCommand::with_name("inbox")
            .about("Manage your inbox")
            .setting(AppSettings::SubcommandRequiredElseHelp)
            .setting(AppSettings::GlobalVersion)
            .setting(AppSettings::InferSubcommands)
            .subcommand(SubCommand::with_name("list")
                .about("List documents in your inbox"))
            .subcommand(SubCommand::with_name("show")
                .about("Show a document in your inbox")
                .arg(Arg::with_name("id")
                    .help("Document ID or Index to show (default shows oldest document)")))
            .subcommand(SubCommand::with_name("delete")
                .about("Delete a document from your inbox")
                .arg(Arg::with_name("id")
                    .help("Document ID or Index to delete")
                    .takes_value(true)
                    .required(true)))
            .subcommand(SubCommand::with_name("archive")
                .about("Archive a document from your inbox")
                .arg(Arg::with_name("id")
                    .help("Document ID or Index to archive")
                    .takes_value(true)
                    .required(true))
                .arg(Arg::with_name("labels")
                    .help("The labels to put on the document")
                    .takes_value(true)
                    .multiple(true))
                .arg(Arg::with_name("properties")
                    .help("The labels to put on the document")
                    .takes_value(true)
                    .multiple(true))))
        .subcommand(SubCommand::with_name("archive")
            .about("Access your document archive")
            .setting(AppSettings::SubcommandRequiredElseHelp)
            .setting(AppSettings::GlobalVersion)
            .setting(AppSettings::InferSubcommands)
            .subcommand(SubCommand::with_name("show")
                .about("Shows details about a archived document")
                .arg(Arg::with_name("id")
                    .help("The document ID")
                    .takes_value(true)
                    .required(true)))
            .subcommand(SubCommand::with_name("get")
                .about("Downloads a fragment from an archived document")
                .arg(Arg::with_name("id")
                    .help("The document ID")
                    .takes_value(true)
                    .required(true))
                .arg(Arg::with_name("kind")
                    .help("The fragment kind")
                    .takes_value(true)
                    .default_value("document"))
                .arg(Arg::with_name("target")
                    .short("t")
                    .long("target")
                    .help("The target filename or - for standard output")
                    .takes_value(true)))
            .subcommand(SubCommand::with_name("search")
                .about("Search for documents in the archive")
                .arg(Arg::with_name("query")
                    .help("The search query")
                    .takes_value(true)
                    .required(true))))
        .get_matches();

    match exec(&matches).await {
        Ok(output) => {
            let stdout = stdout();
            output.to_text(&mut stdout.lock()).expect("Formatting failed");
        }

        Err(error) => {
            eprintln!("{}: {:#}", "Error".bright_red(), error);
            std::process::exit(1);
        }
    }
}

async fn exec(matches: &clap::ArgMatches<'_>) -> Result<Box<dyn Output>> {
    match matches.subcommand() {
        ("config", Some(matches)) => config::exec(matches),

        command => {
            let config = Config::load(matches.value_of("config"))?;

            let mut client = client::Client::new(&config.url, config.auth.into()).await?;

            match command {
                ("upload", Some(matches)) => upload::exec(matches, &mut client).await,
                ("inbox", Some(matches)) => inbox::exec(matches, &mut client).await,
                ("archive", Some(matches)) => archive::exec(matches, &mut client).await,

                _ => unreachable!()
            }
        }
    }
}
