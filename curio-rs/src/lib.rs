pub mod cli;
pub mod commands;
pub mod config;
pub mod confluence;
pub mod curio_docs;
pub mod error;
pub mod harness;
pub mod northstar;
pub mod output;

use anyhow::Result as AnyhowResult;
pub use error::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

// --- Shared Placeholder for External Agent Integration ---
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AgentAnalysis {
    pub summary: String,
    pub keywords: Vec<String>,
    pub confidence_score: f32, // 0.0 to 1.0
                               // potentially embedding vectors for semantic search
}

pub async fn analyze_content_with_agent(_content: &str) -> AnyhowResult<AgentAnalysis> {
    // This is a placeholder. In a real scenario, this would make an HTTP call
    // to an external AI agent service.

    // Simulate some analysis results
    Ok(AgentAnalysis {
        summary: "This is a simulated summary of the content.".to_string(),
        keywords: vec![
            "simulated".to_string(),
            "analysis".to_string(),
            "curio".to_string(),
        ],
        confidence_score: 0.85, // Simulate high confidence
    })
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Change {
    pub target_page_id: String,
    pub target_page_title: String,
    pub change_type: String, // e.g., "update_section", "add_note"
    pub summary_of_change: String,
    pub proposed_content_diff: String, // A diff or the full new section content
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChangeProposal {
    pub summary: String,
    #[serde(default)]
    pub target_path: Vec<String>,
    #[serde(default)]
    pub registry_path: Vec<String>,
    #[serde(default)]
    pub source_refs: Vec<String>,
    #[serde(default)]
    pub validation_requirements: Vec<String>,
    #[serde(default)]
    pub sibling_context: Vec<String>,
    pub rationale: Option<String>,
    pub rollback_plan: Option<String>,
    pub model_used: Option<String>,
    pub pre_change_snapshot_hash: Option<String>,
    pub proposed_changes: Vec<Change>,
}

pub async fn generate_change_proposal_with_agent(
    title: Option<&str>,
    content: &str,
    source_refs: &[String],
    hints: Option<&Value>,
) -> AnyhowResult<ChangeProposal> {
    // This is a placeholder. In a real scenario, this would make an HTTP call
    // to an external AI agent service to get change proposals.

    let route_plan = crate::curio_docs::infer_route_plan(title, content, hints);
    let snapshot_hash = format!("{:x}", Sha256::digest(content.as_bytes()));
    let summary = title
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("Curio change proposal")
        .to_string();

    Ok(ChangeProposal {
        summary,
        target_path: route_plan.target_path,
        registry_path: route_plan.registry_path,
        source_refs: source_refs.to_vec(),
        validation_requirements: route_plan.validation_requirements,
        sibling_context: route_plan.sibling_context,
        rationale: Some(route_plan.rationale),
        rollback_plan: Some(
            "Return the page to Staged and preserve the proposal metadata.".to_string(),
        ),
        model_used: Some("curio-route-plan-v1".to_string()),
        pre_change_snapshot_hash: Some(snapshot_hash),
        proposed_changes: vec![Change {
            target_page_id: source_refs.first().cloned().unwrap_or_default(),
            target_page_title: title.unwrap_or("Curio Page").to_string(),
            change_type: "update_section".to_string(),
            summary_of_change: "Creates or updates the staged/publish route proposal.".to_string(),
            proposed_content_diff: content.to_string(),
        }],
    })
}

pub fn compact_change_proposal(proposal: &ChangeProposal) -> Value {
    serde_json::json!({
        "summary": proposal.summary,
        "target_path": proposal.target_path,
        "registry_path": proposal.registry_path,
        "source_refs": proposal.source_refs,
        "validation_requirements": proposal.validation_requirements,
        "sibling_context": proposal.sibling_context,
        "rationale": proposal.rationale,
        "rollback_plan": proposal.rollback_plan,
        "model_used": proposal.model_used,
        "pre_change_snapshot_hash": proposal.pre_change_snapshot_hash,
        "proposed_change_count": proposal.proposed_changes.len(),
    })
}

// --- Branch index data structures for agent-efficient navigation ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchChildEntry {
    pub page_id: String,
    pub title: String,
    /// "branch" | "leaf" | "record"
    pub child_type: String,
    /// 1-2 sentence routing hint for agents, max ~200 chars
    pub summary: String,
    /// 0 for leaves, N for branches; tells agent whether to drill down
    pub child_count: u32,
    pub status: String,
    pub labels: Vec<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchIndex {
    pub branch_page_id: String,
    pub branch_title: String,
    pub branch_path: Vec<String>,
    pub parent_page_id: Option<String>,
    pub children: Vec<BranchChildEntry>,
    pub total_descendants: u32,
    pub index_updated_at: String,
    pub index_version: u32,
}

// Helper to get page ID (moved from commands)
pub async fn get_page_id_by_title(
    client: &crate::confluence::ConfluenceClient,
    space_key: &str,
    parent_id: Option<&str>,
    title: &str,
    page_type: &str,
) -> AnyhowResult<String> {
    client
        .get_page_by_title(space_key, parent_id, title)
        .await?
        .and_then(|page| page["id"].as_str().map(|s| s.to_string()))
        .ok_or_else(|| {
            anyhow::anyhow!(format!(
                "Could not find {} page '{}' in space '{}'",
                page_type, title, space_key
            ))
        })
}

/// Discriminates the three source types that drive pipeline branching.
/// Confluence pages and URLs are reference sources — Curio stores a reference card, not a copy.
/// Files are capture sources — Curio becomes the canonical home for the content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SourceKind {
    ConfluencePage {
        page_id: String,
        /// The `/wiki/spaces/...` path from `page["_links"]["webui"]`, combined with
        /// `confluence_base_url` to produce the clickable link.
        webui_path: Option<String>,
    },
    Url {
        url: String,
    },
    File {
        path: String,
        mime: String,
    },
}

impl SourceKind {
    /// True for sources that have a durable origin URL — Curio writes a reference card, not a copy.
    pub fn is_reference(&self) -> bool {
        matches!(self, SourceKind::ConfluencePage { .. } | SourceKind::Url { .. })
    }

    /// The label-safe content_type string stored in curio_metadata and Confluence labels.
    pub fn content_type(&self) -> &str {
        match self {
            SourceKind::ConfluencePage { .. } => "confluence_page",
            SourceKind::Url { .. } => "web_page",
            SourceKind::File { mime, .. } => mime.as_str(),
        }
    }

    /// Infer SourceKind from a source_id string.
    ///
    /// source_id formats:
    ///   `confluence-page:{root_id}:page:{page_id}`
    ///   `confluence-folder:{folder_id}:page:{page_id}`
    ///   `confluence-space:{space_key}:page:{page_id}`
    ///   `url:{url}`
    ///   `file:{path}`
    pub fn from_source_id(source_id: &str) -> Self {
        let is_confluence = source_id.starts_with("confluence-page:")
            || source_id.starts_with("confluence-folder:")
            || source_id.starts_with("confluence-space:");

        if is_confluence {
            let page_id = source_id
                .split(":page:")
                .last()
                .unwrap_or(source_id)
                .to_string();
            return SourceKind::ConfluencePage { page_id, webui_path: None };
        }
        if let Some(url) = source_id.strip_prefix("url:") {
            return SourceKind::Url { url: url.to_string() };
        }
        if let Some(path) = source_id.strip_prefix("file:") {
            return SourceKind::File {
                path: path.to_string(),
                mime: "application/octet-stream".to_string(),
            };
        }
        // Unknown format: treat as file
        SourceKind::File {
            path: source_id.to_string(),
            mime: "application/octet-stream".to_string(),
        }
    }

    /// The full clickable URL for the source origin, or None for files.
    pub fn origin_url(&self, confluence_base_url: &str) -> Option<String> {
        match self {
            SourceKind::ConfluencePage { webui_path: Some(path), .. } => {
                Some(format!("{}{}", confluence_base_url.trim_end_matches('/'), path))
            }
            SourceKind::ConfluencePage { page_id, webui_path: None } => {
                Some(format!(
                    "{}/wiki/pages/viewpage.action?pageId={}",
                    confluence_base_url.trim_end_matches('/'),
                    page_id
                ))
            }
            SourceKind::Url { url } => Some(url.clone()),
            SourceKind::File { .. } => None,
        }
    }
}

/// A single piece of content to ingest, collected from any source.
/// Replaces the 4-tuple `(text, source_id, content_type, subject_hint)` used in the old pipeline.
#[derive(Debug, Clone)]
pub struct ContentItem {
    /// Extracted plain text, used for routing, deduplication hash, and analysis.
    pub text: String,
    /// Stable source identifier, e.g. `confluence-page:X:page:Y` or `url:https://...` or `file:/path`.
    pub source_id: String,
    /// Optional human hint for the subject key, overrides title heuristic.
    pub subject_hint: Option<String>,
    /// Source type discriminant — drives reference vs capture pipeline branching.
    pub kind: SourceKind,
    /// Short summary extracted at collection time before the raw body is discarded.
    /// Max ~300 chars. Used in reference card bodies and curio_metadata.
    pub summary: Option<String>,
}

pub async fn resolve_or_create_scoped_child_page_id(
    client: &crate::confluence::ConfluenceClient,
    space_key: &str,
    parent_id: &str,
    title: &str,
    body_content: &str,
) -> AnyhowResult<String> {
    if let Some(page) = client
        .get_page_by_title(space_key, Some(parent_id), title)
        .await?
    {
        if let Some(page_id) = page["id"].as_str() {
            if client.page_is_descendant_of(page_id, parent_id).await? {
                return Ok(page_id.to_string());
            }
            client.migrate_page_to_parent(page_id, parent_id).await?;
            return Ok(page_id.to_string());
        }
    }

    client
        .create_or_update_page(space_key, Some(parent_id), title, "storage", body_content)
        .await
}

#[cfg(test)]
mod source_kind_tests {
    use super::*;

    #[test]
    fn confluence_page_is_reference() {
        let kind = SourceKind::ConfluencePage { page_id: "123".into(), webui_path: None };
        assert!(kind.is_reference());
    }

    #[test]
    fn url_is_reference() {
        let kind = SourceKind::Url { url: "https://example.com".into() };
        assert!(kind.is_reference());
    }

    #[test]
    fn file_is_not_reference() {
        let kind = SourceKind::File { path: "/tmp/notes.txt".into(), mime: "text/plain".into() };
        assert!(!kind.is_reference());
    }

    #[test]
    fn from_source_id_confluence_page() {
        let kind = SourceKind::from_source_id("confluence-page:111:page:999");
        assert!(matches!(kind, SourceKind::ConfluencePage { ref page_id, .. } if page_id == "999"));
    }

    #[test]
    fn from_source_id_confluence_folder() {
        let kind = SourceKind::from_source_id("confluence-folder:111:page:888");
        assert!(matches!(kind, SourceKind::ConfluencePage { ref page_id, .. } if page_id == "888"));
    }

    #[test]
    fn from_source_id_confluence_space() {
        let kind = SourceKind::from_source_id("confluence-space:CURIO:page:777");
        assert!(matches!(kind, SourceKind::ConfluencePage { ref page_id, .. } if page_id == "777"));
    }

    #[test]
    fn from_source_id_url() {
        let kind = SourceKind::from_source_id("url:https://example.com/docs");
        assert!(matches!(kind, SourceKind::Url { ref url } if url == "https://example.com/docs"));
    }

    #[test]
    fn from_source_id_file() {
        let kind = SourceKind::from_source_id("file:/home/user/notes.txt");
        assert!(matches!(kind, SourceKind::File { ref path, .. } if path == "/home/user/notes.txt"));
    }

    #[test]
    fn content_type_values() {
        assert_eq!(SourceKind::ConfluencePage { page_id: "1".into(), webui_path: None }.content_type(), "confluence_page");
        assert_eq!(SourceKind::Url { url: "https://x.com".into() }.content_type(), "web_page");
        assert_eq!(SourceKind::File { path: "f".into(), mime: "text/plain".into() }.content_type(), "text/plain");
    }

    #[test]
    fn origin_url_confluence_with_webui_path() {
        let kind = SourceKind::ConfluencePage {
            page_id: "123".into(),
            webui_path: Some("/wiki/spaces/TEST/pages/123/Title".into()),
        };
        assert_eq!(
            kind.origin_url("https://company.atlassian.net"),
            Some("https://company.atlassian.net/wiki/spaces/TEST/pages/123/Title".into())
        );
    }

    #[test]
    fn origin_url_url_source() {
        let kind = SourceKind::Url { url: "https://example.com".into() };
        assert_eq!(kind.origin_url("https://ignored"), Some("https://example.com".into()));
    }

    #[test]
    fn origin_url_file_is_none() {
        let kind = SourceKind::File { path: "/tmp/f".into(), mime: "text/plain".into() };
        assert_eq!(kind.origin_url("https://ignored"), None);
    }

    #[test]
    fn origin_url_confluence_without_webui_path() {
        let kind = SourceKind::ConfluencePage {
            page_id: "456".into(),
            webui_path: None,
        };
        assert_eq!(
            kind.origin_url("https://company.atlassian.net"),
            Some("https://company.atlassian.net/wiki/pages/viewpage.action?pageId=456".into())
        );
    }
}
