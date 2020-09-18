use std::collections::{HashMap, HashSet};
use std::io::Write;

use anyhow::Result;
use colored::Colorize;
use proto::api::inbox::{ArchiveRequest, GetResponse, ListResponse};

use crate::client::Client;
use crate::output::{Output, SimpleOutput};

pub async fn exec(matches: &clap::ArgMatches<'_>, client: &mut Client) -> Result<Box<dyn Output>> {
    return match matches.subcommand() {
        ("list", Some(matches)) => list(matches, client).await,
        ("show", Some(matches)) => show(matches, client).await,
        ("get", Some(matches)) => get(matches, client).await,
        ("delete", Some(matches)) => delete(matches, client).await,
        ("archive", Some(matches)) => archive(matches, client).await,

        _ => unreachable!()
    };
}


pub async fn list(_: &clap::ArgMatches<'_>, client: &mut Client) -> Result<Box<dyn Output>> {
    let response = client.inbox_list().await?;

    return Ok(Box::new(response));
}

pub async fn show(matches: &clap::ArgMatches<'_>, client: &mut Client) -> Result<Box<dyn Output>> {
    let id = matches.value_of("id").expect("Required ID missing");

    let response = client.inbox_bundle(id).await?;

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
            client.inbox_fragment(id, kind, tokio::io::stdout()).await?;
        }

        _ => {
            let target = tokio::fs::File::create(target).await?;
            client.inbox_fragment(id, kind, target).await?;
        }
    };

    return Ok(Box::new(()));
}

pub async fn delete(matches: &clap::ArgMatches<'_>, client: &mut Client) -> Result<Box<dyn Output>> {
    let id = matches.value_of("id").expect("Required ID missing");

    client.inbox_delete(id).await?;

    return Ok(Box::new(()));
}

pub async fn archive(matches: &clap::ArgMatches<'_>, client: &mut Client) -> Result<Box<dyn Output>> {
    let id = matches.value_of("id").expect("Required ID missing");

    let labels = matches.values_of("labels")
        .map(|labels| labels.map(|v| v.into()).collect())
        .unwrap_or_else(HashSet::default);

    // TODO: Custom typed parsers in clap?
    let properties = matches.values_of("labels")
        .map(|properties| properties.map(|property| property.split_once('=').unwrap_or((property, ""))).map(|(k, v)| (k.to_string(), v.to_string())).collect())
        .unwrap_or_else(HashMap::default);

    let data = ArchiveRequest {
        labels,
        properties,
    };

    client.inbox_archive(id, &data).await?;

    return Ok(Box::new(()));
}

impl SimpleOutput for ListResponse {
    fn to_text(&self, w: &mut dyn Write) -> Result<()> {
        if self.count == 0 {
            writeln!(w, "{} {}", "✓".bright_green(), "No documents pending in inbox".green())?;
        } else {
            writeln!(w, "{} {}", "📥".bright_yellow(), format!("{} documents pending in inbox", self.count).yellow())?;

            for doc in &self.docs {
                writeln!(w, "    {} {} {}", "·".white(), "📄".bright_cyan(), doc.id.to_string().cyan())?;
            }
        }

        return Ok(());
    }
}

impl SimpleOutput for GetResponse {
    fn to_text(&self, w: &mut dyn Write) -> Result<()> {
        SimpleOutput::to_text(&self.doc, w)?;

        writeln!(w, "    {}:", "Suggested Labels".bold())?;
        for suggestion in self.suggestions.iter() {
            writeln!(w, "        {} {}", "-".white(), suggestion)?;
        }

        return Ok(());
    }
}
