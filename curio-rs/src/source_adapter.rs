//! Provider-neutral source collection boundary. Adapters never route or publish.
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, Default)]
pub struct AdapterCapabilities {
    pub incremental_sync: bool,
    pub recursive_traversal: bool,
    pub acl_support: bool,
    pub comments: bool,
    pub attachments: bool,
}
#[derive(Debug, Clone)]
pub struct SourceAdapterInfo {
    pub identity: String,
    pub version: String,
    pub capabilities: AdapterCapabilities,
}
#[derive(Debug, Clone)]
pub struct SourceItem {
    pub stable_id: String,
    pub canonical_url: Option<String>,
    pub title: String,
    pub markdown: String,
    pub mime_type: String,
    pub source_updated_at: Option<DateTime<Utc>>,
    pub owner_metadata: Option<String>,
    pub parent_id: Option<String>,
    pub raw_acl_principals: Vec<String>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceEvent {
    Create,
    Update,
    Delete,
    Move,
    AclChange,
}
#[derive(Debug, Clone, Default)]
pub struct SyncCursor(pub String);

pub trait SourceAdapter {
    fn info(&self) -> SourceAdapterInfo;
    fn enumerate(&self, cursor: Option<&SyncCursor>) -> Result<Vec<SourceItem>>;
    fn fetch(&self, stable_id: &str) -> Result<SourceItem>;
    fn sync(
        &self,
        cursor: Option<&SyncCursor>,
    ) -> Result<(Vec<(SourceEvent, SourceItem)>, SyncCursor)> {
        Ok((
            self.enumerate(cursor)?
                .into_iter()
                .map(|item| (SourceEvent::Update, item))
                .collect(),
            SyncCursor::default(),
        ))
    }
}

/// Reference adapter for a local Markdown/Git tree. It is read-only and
/// intentionally emits stable path identities for unchanged re-syncs.
pub struct LocalMarkdownAdapter {
    root: PathBuf,
}
impl LocalMarkdownAdapter {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }
}
impl SourceAdapter for LocalMarkdownAdapter {
    fn info(&self) -> SourceAdapterInfo {
        SourceAdapterInfo {
            identity: "local-markdown".into(),
            version: "1".into(),
            capabilities: AdapterCapabilities {
                recursive_traversal: true,
                ..Default::default()
            },
        }
    }
    fn enumerate(&self, _cursor: Option<&SyncCursor>) -> Result<Vec<SourceItem>> {
        let mut out = Vec::new();
        for entry in walkdir::WalkDir::new(&self.root).follow_links(false) {
            let entry = entry?;
            let path = entry.path();
            if !entry.file_type().is_file()
                || path.extension().and_then(|x| x.to_str()) != Some("md")
            {
                continue;
            }
            let markdown = std::fs::read_to_string(path)
                .with_context(|| format!("Failed to read {}", path.display()))?;
            let rel = path
                .strip_prefix(&self.root)
                .unwrap_or(path)
                .to_string_lossy()
                .replace('\\', "/");
            let title = markdown
                .lines()
                .find_map(|line| line.strip_prefix("# "))
                .unwrap_or_else(|| {
                    path.file_stem()
                        .and_then(|x| x.to_str())
                        .unwrap_or("untitled")
                })
                .trim()
                .to_string();
            out.push(SourceItem {
                stable_id: format!("file:{rel}"),
                canonical_url: None,
                title,
                markdown,
                mime_type: "text/markdown".into(),
                source_updated_at: None,
                owner_metadata: None,
                parent_id: Path::new(&rel).parent().map(|p| p.to_string_lossy().into()),
                raw_acl_principals: Vec::new(),
            });
        }
        out.sort_by(|a, b| a.stable_id.cmp(&b.stable_id));
        Ok(out)
    }
    fn fetch(&self, stable_id: &str) -> Result<SourceItem> {
        self.enumerate(None)?
            .into_iter()
            .find(|x| x.stable_id == stable_id)
            .ok_or_else(|| anyhow::anyhow!("source item not found: {stable_id}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn local_adapter_enumerates_sorted_markdown_and_fetches_by_stable_id() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("nested")).unwrap();
        fs::write(dir.path().join("z.md"), "# Zed\n\nbody").unwrap();
        fs::write(dir.path().join("nested/a.md"), "# Alpha\n\nbody").unwrap();
        fs::write(dir.path().join("ignored.txt"), "not markdown").unwrap();

        let adapter = LocalMarkdownAdapter::new(dir.path());
        let items = adapter.enumerate(None).unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].stable_id, "file:nested/a.md");
        assert_eq!(items[0].title, "Alpha");
        assert_eq!(items[1].stable_id, "file:z.md");
        assert_eq!(items[1].parent_id.as_deref(), Some(""));
        assert_eq!(adapter.fetch("file:z.md").unwrap().title, "Zed");
    }

    #[test]
    fn local_adapter_reports_missing_items_and_capabilities() {
        let dir = tempdir().unwrap();
        let adapter = LocalMarkdownAdapter::new(dir.path());
        let info = adapter.info();
        assert_eq!(info.identity, "local-markdown");
        assert!(info.capabilities.recursive_traversal);
        assert!(!info.capabilities.acl_support);
        assert!(adapter.fetch("file:missing.md").is_err());
    }
}
