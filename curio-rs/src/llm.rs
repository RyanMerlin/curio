/// OpenAI Chat Completions API client for LLM-driven wiki operations.
///
/// Used by reconcile (routing), query, and lint commands.
/// Auth: OPENAI_API_KEY environment variable or llm.api_key in .curio.yaml.
use anyhow::{Context, Result, bail};
use serde_json::Value;

const OPENAI_API_URL: &str = "https://api.openai.com/v1/chat/completions";
const DEFAULT_MAX_TOKENS: u32 = 2048;

/// Call the OpenAI Chat Completions API with a system + user message.
/// Returns the text content of the first choice.
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
        bail!(
            "OpenAI API key is empty — set OPENAI_API_KEY or configure llm.api_key in .curio.yaml"
        );
    }

    let body = serde_json::json!({
        "model": model,
        "max_tokens": max_tokens,
        "messages": [
            {"role": "system", "content": system},
            {"role": "user", "content": user}
        ]
    });

    let client = reqwest::Client::new();
    let response = client
        .post(OPENAI_API_URL)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .context("Failed to reach OpenAI API")?;

    let status = response.status();
    let text = response
        .text()
        .await
        .context("Failed to read OpenAI response")?;

    if !status.is_success() {
        bail!("OpenAI API error {}: {}", status, text);
    }

    let json: Value = serde_json::from_str(&text)
        .with_context(|| format!("Failed to parse OpenAI response: {}", text))?;

    // Extract text from choices[0].message.content
    json["choices"]
        .as_array()
        .and_then(|arr| arr.first())
        .and_then(|choice| choice["message"]["content"].as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow::anyhow!("Unexpected OpenAI response shape: {}", text))
}

/// Extract a JSON object from an LLM response that may include prose before/after the JSON.
/// Looks for the first `{` ... last `}` span.
pub fn extract_json(text: &str) -> Result<Value> {
    let start = text
        .find('{')
        .ok_or_else(|| anyhow::anyhow!("No JSON object found in response"))?;
    let end = text
        .rfind('}')
        .ok_or_else(|| anyhow::anyhow!("No closing brace found in response"))?;
    let json_str = &text[start..=end];
    serde_json::from_str(json_str)
        .with_context(|| format!("Failed to parse extracted JSON: {}", json_str))
}
