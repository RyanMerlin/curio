//! Shared types for the curio heal pipeline.
//!
//! `HealManifest` is what `curio heal --prepare` emits (Rust → Claude).
//! `HealRoutesFile` is what Claude writes back (Claude → Rust).

use serde::{Deserialize, Serialize};

// ── Manifest (Rust → Claude) ─────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct HealManifest {
    pub scope: String,
    pub confidence_threshold: f64,
    pub pages: Vec<ManifestPage>,
    pub structural_issues: Vec<StructuralIssue>,
    pub external_context: ExternalContext,
    /// Paste this command to apply Claude's route file.
    pub apply_command: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ManifestPage {
    pub slug: String,
    pub path: String,
    pub title: String,
    pub body: String,
    pub category: Vec<String>,
    pub keywords: Vec<String>,
    pub source_url: Option<String>,
    pub updated_at: String,
    pub freshness_score: f64,
    pub quality: ManifestQuality,
    pub overlap_candidates: Vec<OverlapCandidate>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ManifestQuality {
    pub information_quality: f32,
    pub usability: f32,
    pub publishable: bool,
    pub flags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OverlapCandidate {
    pub slug: String,
    pub title: String,
    pub score: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StructuralIssue {
    pub kind: String,
    pub slug: String,
    pub path: String,
    pub detail: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExternalContext {
    pub confluence_space_key: String,
    /// Space key of the original source material (for searching).
    pub source_space_key: Option<String>,
}

// ── Routes file (Claude → Rust) ──────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct HealRoutesFile {
    pub actions: Vec<HealAction>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum HealKind {
    /// Rewrite the page body (new_content required).
    Rewrite,
    /// Merge `merge_sources` into `slug` (new_content required — the merged body).
    Merge,
    /// Archive the page (move to wiki/review/ with status=archive).
    Archive,
    /// Update only frontmatter (keywords, category, title) — no body change.
    UpdateMetadata,
    /// Fix structural issues: repair broken xrefs, populate missing keywords.
    FixStructure,
    /// No action — page is healthy.
    NoAction,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealAction {
    pub kind: HealKind,
    /// Primary slug this action targets.
    pub slug: String,
    /// Confidence score 0.0–1.0 assigned by the Claude agent.
    pub confidence: f64,
    /// Human-readable rationale.
    pub rationale: String,
    /// New page body (Markdown, frontmatter included).
    /// Required for Rewrite, Merge.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_content: Option<String>,
    /// For Merge: the additional slugs being merged in.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub merge_sources: Vec<String>,
    /// For Merge: the canonical slug to keep (defaults to `slug`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub into_slug: Option<String>,
    /// URLs or Confluence page IDs consulted during healing.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sources_consulted: Vec<String>,
}
