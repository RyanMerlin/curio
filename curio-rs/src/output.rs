use anyhow::Result;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct JsonEnvelope<T> {
    pub command: &'static str,
    pub ok: bool,
    pub data: T,
}

pub fn emit_json<T: Serialize>(command: &'static str, ok: bool, data: T) -> Result<()> {
    println!(
        "{}",
        serde_json::to_string_pretty(&JsonEnvelope { command, ok, data })?
    );
    Ok(())
}
