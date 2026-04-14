use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProposalLane {
    Staged,
    Review,
}

impl ProposalLane {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProposalLane::Staged => "staged",
            ProposalLane::Review => "review",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProposalKind {
    NewPage,
    UpdatePage,
    Merge,
    Split,
    TaxonomyChange,
    Consolidation,
    Rejection,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProposalScores {
    pub route_confidence: f32,
    pub quality_confidence: f32,
    pub hierarchy_fit_confidence: f32,
    pub overlap_risk: f32,
    pub evidence_completeness: f32,
    pub usability: f32,
    pub freshness_confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProposalTaxonomyMutation {
    pub proposed_parent_path: Vec<String>,
    pub proposed_node_title: String,
    pub proposed_node_slug: String,
    pub node_description: String,
    pub rationale: String,
    #[serde(default)]
    pub rejected_nearby_nodes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProposalDossier {
    #[serde(default)]
    pub source_ids: Vec<String>,
    #[serde(default)]
    pub source_locations: Vec<String>,
    #[serde(default)]
    pub fetched_artifacts: Vec<String>,
    #[serde(default)]
    pub compared_pages: Vec<String>,
    #[serde(default)]
    pub alternatives_considered: Vec<String>,
    #[serde(default)]
    pub unresolved_questions: Vec<String>,
    #[serde(default)]
    pub overlap_candidates: Vec<String>,
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposalRecord {
    pub schema_version: u32,
    pub proposal_id: String,
    pub generated_at: String,
    pub lane: ProposalLane,
    pub kind: ProposalKind,
    pub subject_slug: String,
    pub title: String,
    #[serde(default)]
    pub target_path: Vec<String>,
    pub summary: String,
    pub body_markdown: String,
    pub recommended_action: String,
    pub scores: ProposalScores,
    pub review_reason: Option<String>,
    pub merge_target: Option<String>,
    pub taxonomy_mutation: Option<ProposalTaxonomyMutation>,
    pub dossier: ProposalDossier,
}

impl ProposalRecord {
    pub fn is_publish_ready(&self) -> bool {
        self.lane == ProposalLane::Staged
            && self.kind != ProposalKind::Rejection
            && self.scores.quality_confidence >= 0.6
            && self.scores.usability >= 0.6
            && self.scores.hierarchy_fit_confidence >= 0.7
            && self.scores.overlap_risk < 0.7
    }
}

pub fn required_lane(
    route_confidence: f32,
    quality_confidence: f32,
    hierarchy_fit_confidence: f32,
    overlap_risk: f32,
    has_taxonomy_mutation: bool,
    explicit_review_reason: bool,
) -> ProposalLane {
    if explicit_review_reason
        || has_taxonomy_mutation
        || route_confidence < 0.75
        || quality_confidence < 0.6
        || hierarchy_fit_confidence < 0.7
        || overlap_risk >= 0.7
    {
        ProposalLane::Review
    } else {
        ProposalLane::Staged
    }
}

pub fn proposal_sidecar_path(content_path: &Path) -> PathBuf {
    let file_name = content_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("proposal.md");
    content_path.with_file_name(format!("{}.proposal.json", file_name))
}

pub fn save_proposal_record(content_path: &Path, proposal: &ProposalRecord) -> Result<()> {
    let sidecar_path = proposal_sidecar_path(content_path);
    std::fs::write(&sidecar_path, serde_json::to_string_pretty(proposal)?)
        .with_context(|| format!("Failed to write {}", sidecar_path.display()))
}

pub fn load_proposal_record(content_path: &Path) -> Result<Option<ProposalRecord>> {
    let sidecar_path = proposal_sidecar_path(content_path);
    if !sidecar_path.exists() {
        return Ok(None);
    }
    let raw = std::fs::read_to_string(&sidecar_path)
        .with_context(|| format!("Failed to read {}", sidecar_path.display()))?;
    let proposal = serde_json::from_str(&raw)
        .with_context(|| format!("Failed to parse {}", sidecar_path.display()))?;
    Ok(Some(proposal))
}
