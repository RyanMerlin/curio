/// Anthropic Messages API client for LLM-driven wiki operations.
///
/// Provides a thin, focused wrapper around the Anthropic REST API.
/// Used by reconcile (routing), query, and lint commands.
use anyhow::{Context, Result, bail};
use serde_json::Value;

const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";
const ANTHROPIC_VERSION: &str = "2023-06-01";
const DEFAULT_MAX_TOKENS: u32 = 2048;

/// Call the Anthropic Messages API with a single user message.
/// Returns the text content of the first response content block.
pub async fn call(api_key: &str, model: &str, system: &str, user: &str) -> Result<String> {
    call_with_max_tokens(api_key, model, system, user, DEFAULT_MAX_TOKENS).await
}

pub async fn call_with_max_tokens(
    api_key: &str,
    model: &str,
    system: &str,
    user: &str,
    max_tokens: u32,
) -> Result<String> {
    if api_key.is_empty() {
        bail!("Anthropic API key is empty — set ANTHROPIC_API_KEY or configure llm.api_key in .curio.yaml");
    }

    let body = serde_json::json!({
        "model": model,
        "max_tokens": max_tokens,
        "system": system,
        "messages": [{"role": "user", "content": user}]
    });

    let client = reqwest::Client::new();
    let response = client
        .post(ANTHROPIC_API_URL)
        .header("x-api-key", api_key)
        .header("anthropic-version", ANTHROPIC_VERSION)
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await
        .context("Failed to reach Anthropic API")?;

    let status = response.status();
    let text = response.text().await.context("Failed to read Anthropic response")?;

    if !status.is_success() {
        bail!("Anthropic API error {}: {}", status, text);
    }

    let json: Value = serde_json::from_str(&text)
        .with_context(|| format!("Failed to parse Anthropic response: {}", text))?;

    // Extract text from content[0].text
    json["content"]
        .as_array()
        .and_then(|arr| arr.first())
        .and_then(|block| block["text"].as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow::anyhow!("Unexpected Anthropic response shape: {}", text))
}

/// Extract a JSON object from an LLM response that may include prose before/after the JSON.
/// Looks for the first `{` ... last `}` span.
pub fn extract_json(text: &str) -> Result<Value> {
    let start = text.find('{').ok_or_else(|| anyhow::anyhow!("No JSON object found in response"))?;
    let end = text.rfind('}').ok_or_else(|| anyhow::anyhow!("No closing brace found in response"))?;
    let json_str = &text[start..=end];
    serde_json::from_str(json_str).with_context(|| format!("Failed to parse extracted JSON: {}", json_str))
}
