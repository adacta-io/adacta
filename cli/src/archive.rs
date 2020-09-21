use std::io::Write;

use anyhow::Result;
use colored::Colorize;

use crate::client::Client;
use crate::output::{Output, SimpleOutput};
use crate::proto::api::archive::{BundleResponse, SearchResponse};

pub async fn exec(matches: &clap::ArgMatches<'_>, client: &mut Client) -> Result<Box<dyn Output>> {
    return match matches.subcommand() {
        ("show", Some(matches)) => show(matches, client).await,
        ("get", Some(matches)) => get(matches, client).await,
        ("search", Some(matches)) => search(matches, client).await,

        _ => unreachable!()
    };
}

pub async fn show(matches: &clap::ArgMatches<'_>, client: &mut Client) -> Result<Box<dyn Output>> {
    let id = matches.value_of("id").expect("Required ID missing");

    let response = client.archive_bundle(id).await?;
    return Ok(Box::new(response));
}

pub async fn get(matches: &clap::ArgMatches<'_>, client: &mut Client) -> Result<Box<dyn Output>> {
    let id = matches.value_of("id").expect("Required ID missing");
    let kind = matches.value_of("kind").expect("Required kind missing");

    let target = matches.value_of("target").map(str::to_string)
        .unwrap_or_else(|| match kind {
            "document" => format!("{}.pdf", id),
            _ => format!("{}.{}", id, kind),
        });

    match target.as_str() {
        "-" => {
            client.archive_fragment(id, kind, tokio::io::stdout()).await?;
        }

        _ => {
            let target = tokio::fs::File::create(target).await?;
            client.archive_fragment(id, kind, target).await?;
        }
    };

    return Ok(Box::new(()));
}

pub async fn search(matches: &clap::ArgMatches<'_>, client: &mut Client) -> Result<Box<dyn Output>> {
    let query = matches.value_of("query").expect("Required query missing");

    let response = client.archive_search(query).await?;
    return Ok(Box::new(response));
}

impl SimpleOutput for BundleResponse {
    fn to_text(&self, w: &mut dyn Write) -> Result<()> {
        SimpleOutput::to_text(&self.doc, w)?;

        return Ok(());
    }
}

impl SimpleOutput for SearchResponse {
    fn to_text(&self, w: &mut dyn Write) -> Result<()> {
        if self.count == 0 {
            writeln!(w, "{} {}", "❌".bright_red(), "No Documents found".red())?;
        } else {
            writeln!(w, "{} {}", "🔎".bright_green(), format!("{} Documents found", self.count).green())?;

            for doc in &self.docs {
                writeln!(w, "    {} {} {}", "-".white(), "📄".bright_cyan(), doc.id.to_string().cyan())?;
            }
        }

        return Ok(());
    }
}