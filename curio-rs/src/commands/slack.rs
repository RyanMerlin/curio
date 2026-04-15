use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::{SourceRef, config::Config, output::emit_json};

#[derive(Debug, Deserialize)]
pub struct SlackPayload {
    pub workspace_id: Option<String>,
    pub channel_id: String,
    pub channel_name: Option<String>,
    pub message_ts: String,
    pub thread_ts: Option<String>,
    pub user_id: Option<String>,
    pub user_name: Option<String>,
    pub text: Option<String>,
    pub permalink: Option<String>,
    pub action: String,
    pub request_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SlackProcessOutput {
    pub action: String,
    pub ok: bool,
    pub job_type: String,
    pub source_ref: SourceRef,
    pub title: String,
    pub summary: String,
    pub requested_by: Option<String>,
    pub channel_id: String,
    pub channel_name: Option<String>,
    pub thread_ts: Option<String>,
    pub queued_at: String,
}

#[derive(Debug, Serialize)]
pub struct SlackAuthorizeOutput {
    pub allowed: bool,
    pub user_id: String,
    pub channel_id: Option<String>,
    pub reason: String,
}

#[derive(Debug, Serialize)]
pub struct SlackJobContract {
    pub action_types: Vec<String>,
    pub default_provider: String,
    pub required_fields: Vec<String>,
    pub async_only: bool,
    pub acknowledgement_sla_seconds: u32,
}

pub async fn run_slack_process(config: &Config, payload_file: PathBuf, json: bool) -> Result<()> {
    let raw = std::fs::read_to_string(&payload_file).with_context(|| {
        format!(
            "Failed to read Slack payload file {}",
            payload_file.display()
        )
    })?;
    let payload: SlackPayload =
        serde_json::from_str(&raw).context("Failed to parse Slack payload file as JSON")?;

    let title = payload
        .text
        .as_deref()
        .unwrap_or("Slack intake")
        .lines()
        .next()
        .unwrap_or("Slack intake")
        .trim()
        .to_string();
    let summary = payload
        .text
        .clone()
        .unwrap_or_default()
        .chars()
        .take(280)
        .collect::<String>();

    let source_ref = SourceRef {
        kind: "slack_message".to_string(),
        id: format!("slack:{}:{}", payload.channel_id, payload.message_ts),
        origin_url: payload.permalink.clone(),
        summary: payload.text.clone(),
    };

    let contract = SlackProcessOutput {
        action: payload.action.clone(),
        ok: true,
        job_type: slack_job_type(&payload.action).to_string(),
        source_ref,
        title,
        summary,
        requested_by: payload.user_id.clone(),
        channel_id: payload.channel_id.clone(),
        channel_name: payload.channel_name.clone(),
        thread_ts: payload.thread_ts.clone(),
        queued_at: Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
    };

    if json {
        emit_json("slack.process", true, contract)?;
    } else {
        println!("queued slack action: {}", payload.action);
        println!("default provider: {}", config.slack.job_provider_default());
    }

    Ok(())
}

pub fn run_slack_authorize(
    config: &Config,
    user_id: String,
    channel_id: Option<String>,
    json: bool,
) -> Result<()> {
    let allowed = config.slack.admin_user_ids.iter().any(|id| id == &user_id);
    let reason = if allowed {
        "allowed by admin allowlist".to_string()
    } else {
        "user not in admin allowlist".to_string()
    };
    let out = SlackAuthorizeOutput {
        allowed,
        user_id,
        channel_id,
        reason,
    };

    if json {
        emit_json("slack.authorize", allowed, out)?;
    } else {
        println!("{}", if allowed { "allowed" } else { "denied" });
    }
    Ok(())
}

pub fn run_slack_contract(config: &Config, json: bool) -> Result<()> {
    let contract = SlackJobContract {
        action_types: vec![
            "intake".to_string(),
            "ask".to_string(),
            "heal".to_string(),
            "merge".to_string(),
            "publish".to_string(),
            "sync".to_string(),
        ],
        default_provider: config.slack.job_provider_default().to_string(),
        required_fields: vec![
            "workspace_id".to_string(),
            "channel_id".to_string(),
            "message_ts".to_string(),
            "action".to_string(),
        ],
        async_only: true,
        acknowledgement_sla_seconds: 3,
    };

    if json {
        emit_json("slack.contract", true, contract)?;
    } else {
        println!("async-only: true");
        println!("default provider: {}", config.slack.job_provider_default());
    }
    Ok(())
}

fn slack_job_type(action: &str) -> &str {
    match action {
        "intake" => "intake",
        "ask" => "question",
        "heal" => "heal",
        "merge" => "merge",
        "publish" => "publish",
        "sync" => "sync",
        _ => "unknown",
    }
}
