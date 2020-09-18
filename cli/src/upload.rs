use std::io::Write;
use std::path::Path;

use anyhow::Result;
use colored::Colorize;
use proto::api::upload::UploadResponse;

use crate::client::Client;
use crate::output::{Output, SimpleOutput};

pub async fn exec(matches: &clap::ArgMatches<'_>, client: &mut Client) -> Result<Box<dyn Output>> {
    let pdf = Path::new(matches.value_of_os("pdf").expect("Document missing"));
    let pdf = tokio::fs::File::open(pdf).await?;

    let response = client.upload(pdf).await?;

    return Ok(Box::new(response));
}

impl SimpleOutput for UploadResponse {
    fn to_text(&self, w: &mut dyn Write) -> Result<()> {
        writeln!(w, "{} {}", "✓".bright_green(), "Upload successful".green())?;

        SimpleOutput::to_text(&self.doc, w)?;

        return Ok(());
    }
}