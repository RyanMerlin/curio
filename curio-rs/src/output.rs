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

#[derive(Debug, Serialize)]
pub struct JsonErrorBody<'a> {
    pub code: &'a str,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<&'a str>,
}

#[derive(Debug, Serialize)]
struct JsonErrorEnvelope<'a> {
    command: &'a str,
    ok: bool,
    error: JsonErrorBody<'a>,
}

/// Emit a structured error envelope on stdout. Use this on all error paths
/// when the user passed `--json` so the response is machine-parseable.
/// `code` is a short, stable identifier (snake_case). `hint` is optional
/// remediation text shown to humans.
pub fn emit_json_error(command: &str, code: &str, message: impl ToString, hint: Option<&str>) {
    let env = JsonErrorEnvelope {
        command,
        ok: false,
        error: JsonErrorBody {
            code,
            message: message.to_string(),
            hint,
        },
    };
    if let Ok(s) = serde_json::to_string_pretty(&env) {
        println!("{}", s);
    } else {
        // Pathological serialization fallback — should never happen with
        // statically-typed envelope, but don't panic in user-facing code.
        eprintln!("{{\"command\":\"{command}\",\"ok\":false,\"error\":{{\"code\":\"{code}\"}}}}");
    }
}
