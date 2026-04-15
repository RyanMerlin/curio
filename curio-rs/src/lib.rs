pub mod cli;
pub mod commands;
pub mod config;
pub mod workspace;
pub mod confluence;
pub mod error;
pub mod freshness;
pub mod git_ops;
pub mod harness;
pub mod heal_types;
pub mod llm;
pub mod northstar;
pub mod overlap;
pub mod output;
pub mod proposal;
pub mod quality;
pub mod reconcile;
pub mod audit_store;
pub mod wiki_fs;
pub mod wiki_index;

pub use error::Result;
use serde::{Deserialize, Serialize};

// ─── Wiki core types ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PageStatus {
    Intake,
    Staged,
    Review,
    Published,
}

impl PageStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            PageStatus::Intake => "intake",
            PageStatus::Staged => "staged",
            PageStatus::Review => "review",
            PageStatus::Published => "published",
        }
    }
}

impl std::fmt::Display for PageStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceRef {
    pub kind: String, // "url" | "file" | "confluence_page"
    pub id: String,
    pub origin_url: Option<String>,
    pub summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Frontmatter {
    pub id: String,
    pub title: String,
    pub status: PageStatus,
    pub source: SourceRef,
    #[serde(default)]
    pub category: Vec<String>,
    #[serde(default)]
    pub keywords: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    pub confidence: Option<f32>,
    #[serde(default)]
    pub cross_refs: Vec<String>,
    pub content_hash: String,
    pub confluence_page_id: Option<String>,
    pub model_used: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auto_healed_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auto_healed_confidence: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct WikiPage {
    /// Absolute path on disk.
    pub path: std::path::PathBuf,
    pub frontmatter: Frontmatter,
    /// Markdown body (without frontmatter).
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WikiIndex {
    pub pages: Vec<WikiIndexEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiIndexEntry {
    pub path: String,
    pub title: String,
    pub category: Vec<String>,
    pub keywords: Vec<String>,
    pub status: String,
    pub summary: String,
    pub confidence: Option<f32>,
    pub updated_at: String,
    pub id: String,
}

// ─── Source kind (retained for intake pipeline) ───────────────────────────

/// Discriminates the three source types that drive pipeline branching.
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
pub mod md_to_confluence;
