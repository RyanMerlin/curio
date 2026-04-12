/// Reconciliation types and heuristics for routing intake pages.
///
/// The actual LLM reasoning lives in the agent harness (Claude / Gemini / Codex).
/// This module defines the data contract the agent works with, and provides a
/// basic heuristic fallback for `curio process auto`.
use serde::{Deserialize, Serialize};

/// Routing decision produced by the agent (or the heuristic fallback).
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
    /// The id of a published page this content should be merged into, if any.
    pub merge_target: Option<String>,
    /// The LLM model that produced this decision (or `"heuristic"`).
    pub model_used: String,
}

impl ReconcileDecision {
    /// Simple keyword-based heuristic fallback. Routes to `staged` with medium confidence.
    /// Category slugs match the NORTHSTAR tree structure (e.g. `product-tree/alteryx-server`).
    pub fn heuristic(title: &str, body: &str) -> Self {
        let text = format!("{} {}", title, body).to_lowercase();

        // Check for specific Alteryx product subtrees first (more specific wins).
        // Order matters: server signals are checked before designer to avoid "workflow" false positives.
        let category = if text.contains("intelligence suite")
            || text.contains("automl")
            || text.contains("machine learning")
            || text.contains("ai/ml")
        {
            vec!["product-tree".to_string(), "intelligence-suite".to_string()]
        } else if text.contains("server")
            || text.contains("upgrade")
            || text.contains("install")
            || text.contains("mongodb")
            || text.contains("cryptomigration")
            || text.contains("servicedata")
            || text.contains("rollback")
            || text.contains("downgrade")
            || text.contains("deployment")
            || text.contains("alteryxservice")
            || text.contains("host recovery")
            || text.contains("pre-upgrade")
            || text.contains("server health")
        {
            vec!["product-tree".to_string(), "alteryx-server".to_string()]
        } else if text.contains("designer")
            || text.contains("canvas")
            || text.contains("tool palette")
            || (text.contains("workflow") && !text.contains("server"))
        {
            vec!["product-tree".to_string(), "alteryx-designer".to_string()]
        } else if text.contains("product") || text.contains("feature") || text.contains("release") {
            vec!["product-tree".to_string()]
        } else if text.contains("account") || text.contains("customer") {
            vec!["account-tree".to_string()]
        } else if text.contains("executive") || text.contains("roi") || text.contains("business value") {
            vec!["audience-tree".to_string(), "executive-business".to_string()]
        } else if text.contains("cse") || text.contains("escalation") || text.contains("technical support") {
            vec!["audience-tree".to_string(), "technical-cse".to_string()]
        } else if text.contains("audience") || text.contains("persona") {
            vec!["audience-tree".to_string()]
        } else if text.contains("playbook") || text.contains("use case") || text.contains("runbook") {
            vec!["use-case-tree".to_string()]
        } else {
            vec!["topic-tree".to_string()]
        };

        // Extract naive keywords: most frequent non-stopwords
        let keywords = extract_keywords(&text, 5);
        let summary = title.chars().take(200).collect();

        ReconcileDecision {
            category,
            keywords,
            confidence: 0.55,
            status: "staged".to_string(),
            summary,
            cross_refs: vec![],
            review_reason: None,
            merge_target: None,
            model_used: "heuristic".to_string(),
        }
    }

    /// Whether this decision routes to staged (vs review).
    pub fn is_staged(&self) -> bool {
        self.status == "staged"
    }
}

/// Extract up to `n` naive keywords from lowercased text.
fn extract_keywords(text: &str, n: usize) -> Vec<String> {
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
