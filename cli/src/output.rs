use anyhow::Result;
use serde::Serialize;
use serde_json::Value as JsonValue;
use std::io::Write;

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
