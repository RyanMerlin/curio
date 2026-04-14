/// LLM-primary reconciliation for routing intake pages into the wiki.
///
/// Flow per page:
///   1. Heuristic pre-signal  (title token scan → hint label, not a decision)
///   2. Build routing prompt  (NORTHSTAR subtrees + leaf index route hints + page content)
///   3. LLM call              (Anthropic Messages API, structured JSON response)
///   4. Validate + parse      (category exists, confidence in range, status field)
///   5. Return ReconcileDecision + RoutingAnalysis for sidecar persistence
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

use crate::commands::sync::TreeNode;

// ─── Decision output ─────────────────────────────────────────────────────

/// Routing decision produced by the LLM (or manual override).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconcileDecision {
    /// Relative path segments for the target category, e.g. `["product-tree", "alteryx-server"]`.
    pub category: Vec<String>,
    pub keywords: Vec<String>,
    pub confidence: f32,
    /// `"staged"` or `"review"`.
    pub status: String,
    /// Short summary of the content (≤200 chars).
    pub summary: String,
    /// Relative paths to related pages in the wiki.
    #[serde(default)]
    pub cross_refs: Vec<String>,
    /// Reason for routing to `review` (if applicable).
    pub review_reason: Option<String>,
    /// Proposed new subtree slug when no existing subtree fits confidently.
    #[serde(default)]
    pub proposed_new_subtree: Option<String>,
    /// Rationale for creating the proposed subtree.
    #[serde(default)]
    pub proposal_rationale: Option<String>,
    /// The id of a published page this content should be merged into, if any.
    pub merge_target: Option<String>,
    /// The LLM model that produced this decision (or `"manual"`).
    pub model_used: String,
}

impl ReconcileDecision {
    pub fn is_staged(&self) -> bool {
        self.status == "staged"
    }

    /// Manual routing decision — bypasses LLM entirely.
    pub fn manual(
        category: Vec<String>,
        keywords: Vec<String>,
        confidence: f32,
        status: String,
        summary: String,
    ) -> Self {
        ReconcileDecision {
            category,
            keywords,
            confidence,
            status,
            summary,
            cross_refs: vec![],
            review_reason: None,
            proposed_new_subtree: None,
            proposal_rationale: None,
            merge_target: None,
            model_used: "manual".to_string(),
        }
    }
}

// ─── Analysis sidecar ────────────────────────────────────────────────────

/// Full provenance record for a routing decision. Written as `{slug}.analysis.json`
/// alongside every staged/published page. Never synced to Confluence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingAnalysis {
    pub schema_version: u32,
    pub analyzed_at: String,
    pub model: String,
    pub inputs: AnalysisInputs,
    pub routing: AnalysisRouting,
    pub signals: AnalysisSignals,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisInputs {
    pub title: String,
    pub source_url: Option<String>,
    pub content_hash: String,
    pub content_preview: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisRouting {
    pub decision: Vec<String>,
    pub confidence: f32,
    pub rationale: String,
    pub alternatives_considered: Vec<RoutingAlternative>,
    pub flags: Vec<String>,
    #[serde(default)]
    pub information_quality: Option<f32>,
    #[serde(default)]
    pub usability: Option<f32>,
    pub review_reason: Option<String>,
    #[serde(default)]
    pub proposed_new_subtree: Option<String>,
    #[serde(default)]
    pub proposal_rationale: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingAlternative {
    pub path: Vec<String>,
    pub score: f32,
    pub ruled_out_because: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisSignals {
    pub heuristic_pre_signal: Option<String>,
    pub title_tokens: Vec<String>,
    pub keywords_extracted: Vec<String>,
}

// ─── Heuristic pre-signal ────────────────────────────────────────────────

/// Returns a suggested subtree slug based on title/body keyword scan.
/// This is a HINT fed into the LLM prompt — not a routing decision.
pub fn heuristic_pre_signal(title: &str, body: &str) -> Option<String> {
    let title_text = title.to_lowercase();
    let body_text = body.to_lowercase();

    // Title and the dominant content topic outrank incidental mentions in the body.
    if title_text.contains("server")
        || title_text.contains("mongodb")
        || title_text.contains("cryptomigration")
        || title_text.contains("host recovery")
        || title_text.contains("pre-upgrade")
        || title_text.contains("alteryxservice")
    {
        Some("alteryx-server".to_string())
    } else if title_text.contains("designer")
        || title_text.contains("canvas")
        || title_text.contains("workflow")
        || title_text.contains("tool palette")
    {
        Some("alteryx-designer".to_string())
    } else if title_text.contains("intelligence suite")
        || title_text.contains("automl")
        || title_text.contains("machine learning")
        || title_text.contains("ai/ml")
    {
        Some("intelligence-suite".to_string())
    } else if body_text.contains("server")
        || body_text.contains("mongodb")
        || body_text.contains("cryptomigration")
        || body_text.contains("servicedata")
        || body_text.contains("host recovery")
        || body_text.contains("pre-upgrade")
        || body_text.contains("alteryxservice")
    {
        Some("alteryx-server".to_string())
    } else if body_text.contains("designer")
        || body_text.contains("canvas")
        || body_text.contains("tool palette")
        || (body_text.contains("workflow") && !body_text.contains("server"))
    {
        Some("alteryx-designer".to_string())
    } else if body_text.contains("intelligence suite")
        || body_text.contains("automl")
        || body_text.contains("machine learning")
        || body_text.contains("ai/ml")
    {
        Some("intelligence-suite".to_string())
    } else if title_text.contains("playbook")
        || title_text.contains("use case")
        || title_text.contains("runbook")
        || body_text.contains("playbook")
        || body_text.contains("use case")
        || body_text.contains("runbook")
    {
        Some("use-case-tree".to_string())
    } else if title_text.contains("account")
        || title_text.contains("customer")
        || body_text.contains("account")
        || body_text.contains("customer")
    {
        Some("account-tree".to_string())
    } else {
        None
    }
}

/// Extract up to `n` naive keywords from lowercased text.
pub fn extract_keywords(text: &str, n: usize) -> Vec<String> {
    const STOP: &[&str] = &[
        "the", "a", "an", "is", "in", "of", "and", "or", "to", "for", "with", "this", "that",
        "be", "are", "was", "were", "it", "as", "at", "by", "on", "from", "has", "have",
    ];
    let mut freq: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for word in text.split_whitespace() {
        let w: String = word.chars().filter(|c| c.is_alphabetic()).collect();
        if w.len() >= 4 && !STOP.contains(&w.as_str()) {
            *freq.entry(w).or_insert(0) += 1;
        }
    }
    let mut pairs: Vec<(usize, String)> = freq.into_iter().map(|(k, v)| (v, k)).collect();
    pairs.sort_by(|a, b| b.0.cmp(&a.0));
    pairs.into_iter().take(n).map(|(_, w)| w).collect()
}

// ─── LLM routing ────────────────────────────────────────────────────────

/// Route a page using the Anthropic API. Returns both the decision and the full analysis
/// record for sidecar persistence.
///
/// # Arguments
/// - `api_key` / `model` — LLM credentials
/// - `title`, `body` — page content (body truncated to ~3K tokens in prompt)
/// - `source_url`, `content_hash` — for the analysis sidecar
/// - `trees` — parsed NORTHSTAR blueprint (used to build routing context)
pub async fn route_with_llm(
    api_key: &str,
    model: &str,
    title: &str,
    body: &str,
    source_url: Option<&str>,
    content_hash: &str,
    trees: &[TreeNode],
) -> Result<(ReconcileDecision, RoutingAnalysis)> {
    let pre_signal = heuristic_pre_signal(title, body);
    let title_tokens = title.split_whitespace().map(|w| w.to_lowercase()).collect::<Vec<_>>();

    let system_prompt = build_system_prompt(trees);
    let user_prompt = build_user_prompt(title, body, pre_signal.as_deref());

    let raw = crate::llm::call(api_key, model, &system_prompt, &user_prompt)
        .await
        .context("LLM call failed during routing")?;

    // Parse and validate — retry once on JSON failure
    let json = match crate::llm::extract_json(&raw) {
        Ok(j) => j,
        Err(_) => {
            // One retry with explicit JSON reminder
            let retry_prompt = format!(
                "{}\n\nIMPORTANT: Your previous response could not be parsed as JSON. \
                Return ONLY a valid JSON object, no prose, no markdown fences.",
                user_prompt
            );
            let raw2 = crate::llm::call(api_key, model, &system_prompt, &retry_prompt)
                .await
                .context("LLM retry call failed")?;
            crate::llm::extract_json(&raw2)
                .context("LLM response still not valid JSON after retry — routing to review")?
        }
    };

    parse_llm_response(json, model, title, body, source_url, content_hash, pre_signal, title_tokens)
}

fn build_system_prompt(trees: &[TreeNode]) -> String {
    let mut routes = String::new();
    for tree in trees {
        let desc = strip_html(&tree.description_html);
        if tree.subtrees.is_empty() {
            routes.push_str(&format!(
                "### {}\nSlug: {}\nDescription: {}\n\n",
                tree.title, tree.slug, desc
            ));
        } else {
            for sub in &tree.subtrees {
                let sub_desc = strip_html(&sub.description_html);
                routes.push_str(&format!(
                    "### {}/{}\nDescription: {}\n\n",
                    tree.slug, sub.slug, sub_desc
                ));
            }
            // Also include the parent tree as a catch-all if it has its own content
            if !desc.is_empty() {
                routes.push_str(&format!(
                    "### {} (top-level, no subtree match)\nSlug: {}\nDescription: {}\n\n",
                    tree.title, tree.slug, desc
                ));
            }
        }
    }

    format!(
        r#"You are a routing agent for an enterprise knowledge wiki called Curio.
Your job: classify an article into exactly one route from the Available Routes below.

## Available Routes

{routes}

## Routing Rules
- Choose the MOST SPECIFIC matching route. Prefer a subtree (e.g. product-tree/alteryx-server) over a top-level tree.
- Hierarchy is the main design goal. Do not stop at the first acceptable shallow route if the content clearly implies a deeper intermediate structure.
- Use branch indexes and the existing hierarchy aggressively to understand where this content should live as part of the whole knowledge base.
- Keep searching likely neighboring structure until you believe you have found the relevant peers, branch nodes, and overlap candidates.
- If confidence < 0.75 OR the article genuinely matches multiple routes equally, set status to "review".
- If no existing subtree fits confidently, set status to "review", propose a new subtree slug, and explain why existing routes are insufficient.
- Route names are workspace-specific examples from the current NORTHSTAR, not permanent universal labels. Use the CURRENT route descriptions, not stale assumptions from another project.
- Prioritize the dominant content topic and title wording over incidental mentions in the body.
- A passing mention of another product, dependency, or uninstall step does NOT make that other product the route.
- If a page is primarily about one system's install, upgrade, rollback, recovery, architecture, usage, migration, policy, or operations, keep it with that dominant system even if the body mentions related products.
- Default toward deeper hierarchy for technical-detail documents. If the content is operational, version-specific, scenario-specific, troubleshooting-specific, implementation-specific, or otherwise narrow and detailed, prefer a deeper path or a new intermediate node rather than placing it flat under a shallow branch. Only keep it at a higher level when the content is clearly broad, cross-cutting, or intentionally acts as a branch-level overview.
- When in doubt, route to review rather than guess.

## Output Format
Return ONLY a valid JSON object with these exact fields:
{{
  "category": ["tree-slug", "optional-subtree-slug"],
  "confidence": 0.0-1.0,
  "rationale": "one or two sentences explaining the decision",
  "alternatives_considered": [
    {{"path": ["tree", "subtree"], "score": 0.0, "ruled_out_because": "reason"}}
  ],
  "keywords": ["up to 8 domain keywords"],
  "summary": "max 200 chars describing page content",
  "status": "staged or review",
  "review_reason": null,
  "proposed_new_subtree": null,
  "proposal_rationale": null
}}"#
    )
}

fn build_user_prompt(title: &str, body: &str, pre_signal: Option<&str>) -> String {
    // Truncate body to ~3000 chars to keep token cost bounded
    let body_preview: String = body.chars().take(3000).collect();

    let hint = match pre_signal {
        Some(s) => format!("\n## Heuristic pre-signal (hint — override freely if wrong)\nTitle keyword match suggests: {}\n", s),
        None => String::new(),
    };

    format!(
            "## Article to Route\nTitle: {title}\n{hint}\nFocus on the dominant topic and the main content subject. Treat secondary product mentions in the body as weak evidence.\nBody:\n{body_preview}"
    )
}

/// Parse the LLM JSON response into a ReconcileDecision + RoutingAnalysis.
fn parse_llm_response(
    json: serde_json::Value,
    model: &str,
    title: &str,
    body: &str,
    source_url: Option<&str>,
    content_hash: &str,
    pre_signal: Option<String>,
    title_tokens: Vec<String>,
) -> Result<(ReconcileDecision, RoutingAnalysis)> {
    let category: Vec<String> = json["category"]
        .as_array()
        .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default();

    if category.is_empty() {
        bail!("LLM returned empty category");
    }

    let confidence = json["confidence"].as_f64().unwrap_or(0.5) as f32;
    let rationale = json["rationale"].as_str().unwrap_or("").to_string();
    let status_raw = json["status"].as_str().unwrap_or("review");
    let status = if status_raw == "staged" && confidence >= 0.75 {
        "staged".to_string()
    } else if status_raw == "staged" {
        // LLM said staged but confidence is low — force review
        "review".to_string()
    } else {
        "review".to_string()
    };

    let review_reason = if status == "review" {
        json["review_reason"]
            .as_str()
            .map(|s| s.to_string())
            .or_else(|| {
                if confidence < 0.75 {
                    Some(format!("Low confidence ({:.0}%)", confidence * 100.0))
                } else {
                    None
                }
            })
    } else {
        None
    };

    let keywords: Vec<String> = json["keywords"]
        .as_array()
        .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_else(|| extract_keywords(&format!("{} {}", title, body), 8));

    let summary = json["summary"]
        .as_str()
        .map(|s| s.chars().take(200).collect::<String>())
        .unwrap_or_else(|| title.chars().take(200).collect());
    let proposed_new_subtree = json["proposed_new_subtree"]
        .as_str()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let proposal_rationale = json["proposal_rationale"]
        .as_str()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    let alternatives: Vec<RoutingAlternative> = json["alternatives_considered"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| {
                    let path: Vec<String> = v["path"]
                        .as_array()
                        .map(|a| a.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect())
                        .unwrap_or_default();
                    let score = v["score"].as_f64().unwrap_or(0.0) as f32;
                    let reason = v["ruled_out_because"].as_str().unwrap_or("").to_string();
                    if path.is_empty() { None } else { Some(RoutingAlternative { path, score, ruled_out_because: reason }) }
                })
                .collect()
        })
        .unwrap_or_default();

    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let content_preview: String = body.chars().take(500).collect();
    let kw_for_signals = extract_keywords(&format!("{} {}", title, body), 8);

    let decision = ReconcileDecision {
        category: category.clone(),
        keywords: keywords.clone(),
        confidence,
        status: status.clone(),
        summary: summary.clone(),
        cross_refs: vec![],
        review_reason: review_reason.clone(),
        proposed_new_subtree: proposed_new_subtree.clone(),
        proposal_rationale: proposal_rationale.clone(),
        merge_target: None,
        model_used: model.to_string(),
    };

    let analysis = RoutingAnalysis {
        schema_version: 1,
        analyzed_at: now,
        model: model.to_string(),
        inputs: AnalysisInputs {
            title: title.to_string(),
            source_url: source_url.map(|s| s.to_string()),
            content_hash: content_hash.to_string(),
            content_preview,
        },
        routing: AnalysisRouting {
            decision: category,
            confidence,
            rationale,
            alternatives_considered: alternatives,
            flags: vec![],
            information_quality: None,
            usability: None,
            review_reason,
            proposed_new_subtree,
            proposal_rationale,
        },
        signals: AnalysisSignals {
            heuristic_pre_signal: pre_signal,
            title_tokens,
            keywords_extracted: kw_for_signals,
        },
    };

    Ok((decision, analysis))
}

/// Strip HTML tags for use in plain-text prompt context.
fn strip_html(html: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    // Collapse whitespace
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}
