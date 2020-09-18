use anyhow::Result;
use serde::Serialize;
use serde_json::Value as JsonValue;
use std::io::Write;
use proto::model::DocInfo;
use colored::Colorize;

pub trait Output {
    fn to_json(&self) -> Result<JsonValue>;
    fn to_text(&self, w: &mut dyn Write) -> Result<()>;
}

impl Output for () {
    fn to_json(&self) -> Result<JsonValue> {
        return Ok(JsonValue::Null);
    }

    fn to_text(&self, _: &mut dyn Write) -> Result<()> { return Ok(()); }
}

pub trait SimpleOutput {
    fn to_text(&self, w: &mut dyn Write) -> Result<()>;
}

impl<T> Output for T
    where T: Serialize + SimpleOutput
{
    fn to_json(&self) -> Result<JsonValue> {
        return Ok(serde_json::to_value(self)?);
    }

    fn to_text(&self, w: &mut dyn Write) -> Result<()> {
        return <Self as SimpleOutput>::to_text(self, w);
    }
}

impl SimpleOutput for DocInfo {
    fn to_text(&self, w: &mut dyn Write) -> Result<()> {
        writeln!(w, "{} {}", "📄".bright_cyan(), self.id.to_string().cyan().bold())?;

        writeln!(w, "    {}: {}", "Uploaded".bold(), self.metadata.uploaded.to_string())?;

        if let Some(archived) = self.metadata.archived {
            writeln!(w, "    {}: {}", "Archived".bold(), archived.to_string())?;
        }

        writeln!(w, "    {}: {}", "Title".bold(), self.metadata.title.as_ref().map(|title| title.to_string()).unwrap_or_else(String::new))?;
        writeln!(w, "    {}: {}", "Pages".bold(), self.metadata.pages)?;

        writeln!(w, "    {}:", "Labels".bold())?;
        for label in self.metadata.labels.iter() {
            writeln!(w, "        {} {}", "-".white(), label)?;
        }

        writeln!(w, "    {}:", "Properties".bold())?;
        for (key, value) in self.metadata.properties.iter() {
            writeln!(w, "        {} {} {} {}", "-".white(), key.bold(), "🢒".white(), value)?;
        }

        return Ok(());
    }
}
