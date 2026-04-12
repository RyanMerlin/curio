use crate::{Frontmatter, PageStatus, WikiIndex, WikiIndexEntry, WikiPage};
use anyhow::{Context, Result};
use chrono::Utc;
use std::path::Path;
use walkdir::WalkDir;

const REGISTRY_FILE: &str = "_index/registry.json";
const INDEX_MD_FILE: &str = "_index/index.md";
const LOG_MD_FILE: &str = "_index/log.md";

// ─── Registry (registry.json) ────────────────────────────────────────────

/// Load the wiki registry from disk. Returns an empty index if the file doesn't exist.
pub fn load_registry(wiki_dir: &Path) -> Result<WikiIndex> {
    let path = wiki_dir.join(REGISTRY_FILE);
    if !path.exists() {
        return Ok(WikiIndex::default());
    }
    let raw = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    let pages: Vec<WikiIndexEntry> = serde_json::from_str(&raw)
        .with_context(|| format!("Failed to parse {}", path.display()))?;
    Ok(WikiIndex { pages })
}

/// Save the wiki registry to disk.
pub fn save_registry(wiki_dir: &Path, index: &WikiIndex) -> Result<()> {
    let path = wiki_dir.join(REGISTRY_FILE);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(&index.pages)?;
    std::fs::write(&path, json)
        .with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(())
}

/// Upsert a single entry in the registry (matched by `id`).
pub fn upsert_registry_entry(wiki_dir: &Path, entry: WikiIndexEntry) -> Result<()> {
    let mut index = load_registry(wiki_dir)?;
    if let Some(existing) = index.pages.iter_mut().find(|e| e.id == entry.id) {
        *existing = entry;
    } else {
        index.pages.push(entry);
    }
    save_registry(wiki_dir, &index)
}

/// Remove a registry entry by id.
pub fn remove_registry_entry(wiki_dir: &Path, id: &str) -> Result<()> {
    let mut index = load_registry(wiki_dir)?;
    index.pages.retain(|e| e.id != id);
    save_registry(wiki_dir, &index)
}

// ─── Reindex from filesystem ─────────────────────────────────────────────

/// Walk `wiki_dir/**/*.md`, parse frontmatter, and rebuild `WikiIndex`.
/// Skips files in `_index/`, `_audit/`, and `_schema/`.
pub fn reindex_from_filesystem(wiki_dir: &Path) -> Result<WikiIndex> {
    let mut pages = Vec::new();

    for entry in WalkDir::new(wiki_dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_file()
                && e.path().extension().map_or(false, |ext| ext == "md")
        })
    {
        let abs = entry.path();
        let rel = abs
            .strip_prefix(wiki_dir)
            .expect("walkdir entry is under wiki_dir");

        // Skip system directories
        let first_component = rel.components().next().map(|c| c.as_os_str().to_string_lossy().into_owned()).unwrap_or_default();
        if matches!(first_component.as_str(), "_index" | "_audit" | "_schema") {
            continue;
        }

        match crate::wiki_fs::parse_wiki_page(abs) {
            Ok(page) => {
                let entry = entry_from_page(&page, rel);
                pages.push(entry);
            }
            Err(e) => {
                eprintln!("Warning: skipping {} — {}", rel.display(), e);
            }
        }
    }

    Ok(WikiIndex { pages })
}

/// Build a `WikiIndexEntry` from a parsed wiki page.
pub fn entry_from_page(page: &WikiPage, rel_path: &Path) -> WikiIndexEntry {
    let summary = crate::wiki_fs::first_line_summary(&page.body, 200);
    WikiIndexEntry {
        path: rel_path.to_string_lossy().replace('\\', "/"),
        title: page.frontmatter.title.clone(),
        category: page.frontmatter.category.clone(),
        keywords: page.frontmatter.keywords.clone(),
        status: page.frontmatter.status.to_string(),
        summary,
        confidence: page.frontmatter.confidence,
        updated_at: page.frontmatter.updated_at.clone(),
        id: page.frontmatter.id.clone(),
    }
}

// ─── index.md ────────────────────────────────────────────────────────────

/// Rebuild `wiki/_index/index.md` from the registry.
pub fn rebuild_index_md(wiki_dir: &Path, index: &WikiIndex) -> Result<()> {
    let path = wiki_dir.join(INDEX_MD_FILE);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let now = Utc::now().format("%Y-%m-%d %H:%M UTC").to_string();
    let total = index.pages.len();

    // Group by first category segment
    let mut by_category: std::collections::BTreeMap<String, Vec<&WikiIndexEntry>> =
        std::collections::BTreeMap::new();
    for entry in &index.pages {
        let cat = entry.category.first().cloned().unwrap_or_else(|| "uncategorized".to_string());
        by_category.entry(cat).or_default().push(entry);
    }

    let mut out = format!(
        "# Curio Wiki Index\n> Last updated: {} | Pages: {}\n\n",
        now, total
    );

    if index.pages.is_empty() {
        out.push_str("_No pages yet. Run `curio intake` to add content._\n");
    } else {
        for (cat, entries) in &by_category {
            out.push_str(&format!("## {} ({} pages)\n", cat, entries.len()));
            for e in entries {
                let conf = e
                    .confidence
                    .map(|c| format!(" [confidence:{:.0}%]", c * 100.0))
                    .unwrap_or_default();
                let kw = if e.keywords.is_empty() {
                    String::new()
                } else {
                    format!(" | keywords: {}", e.keywords.join(", "))
                };
                out.push_str(&format!(
                    "- **{}** — {}{}{}\n",
                    e.path, e.summary, conf, kw
                ));
            }
            out.push('\n');
        }
    }

    std::fs::write(&path, out)
        .with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(())
}

// ─── log.md ──────────────────────────────────────────────────────────────

/// Append an entry to `wiki/_index/log.md`.
pub fn append_log(wiki_dir: &Path, entry: &str) -> Result<()> {
    let path = wiki_dir.join(LOG_MD_FILE);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let line = format!("- **{}** {}\n", now, entry);

    let mut content = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        "# Curio Operation Log\n\n".to_string()
    };

    content.push_str(&line);
    std::fs::write(&path, content)?;
    Ok(())
}

// ─── Convenience: load index.md text ────────────────────────────────────

/// Read index.md as a string (for passing to LLM context).
pub fn read_index_md(wiki_dir: &Path) -> Result<String> {
    let path = wiki_dir.join(INDEX_MD_FILE);
    if !path.exists() {
        return Ok(String::new());
    }
    std::fs::read_to_string(&path).with_context(|| format!("Failed to read {}", path.display()))
}

// ─── Lookup helpers ──────────────────────────────────────────────────────

/// Find entries by status.
pub fn entries_by_status<'a>(index: &'a WikiIndex, status: &str) -> Vec<&'a WikiIndexEntry> {
    index.pages.iter().filter(|e| e.status == status).collect()
}

/// Check whether a content_hash already exists in the registry (dedup).
pub fn is_duplicate_hash(_index: &WikiIndex, content_hash: &str) -> bool {
    // The hash is stored in frontmatter, not the index entry, so we
    // can only check via the index if we add it. For now, check by path.
    // A more thorough check reads each file's frontmatter.
    let _ = content_hash;
    false // Will be enriched by intake command directly comparing hashes
}

/// Build a `WikiIndexEntry` directly from `Frontmatter` + relative path + summary.
pub fn entry_from_frontmatter(fm: &Frontmatter, rel_path: &str, summary: &str) -> WikiIndexEntry {
    WikiIndexEntry {
        path: rel_path.replace('\\', "/"),
        title: fm.title.clone(),
        category: fm.category.clone(),
        keywords: fm.keywords.clone(),
        status: fm.status.to_string(),
        summary: summary.to_string(),
        confidence: fm.confidence,
        updated_at: fm.updated_at.clone(),
        id: fm.id.clone(),
    }
}

/// Update an entry's path in the registry (after a git mv).
pub fn update_entry_path(wiki_dir: &Path, id: &str, new_path: &str) -> Result<()> {
    let mut index = load_registry(wiki_dir)?;
    if let Some(e) = index.pages.iter_mut().find(|e| e.id == id) {
        e.path = new_path.replace('\\', "/");
    }
    save_registry(wiki_dir, &index)
}

/// Update an entry's status in the registry.
pub fn update_entry_status(wiki_dir: &Path, id: &str, status: &PageStatus) -> Result<()> {
    let mut index = load_registry(wiki_dir)?;
    if let Some(e) = index.pages.iter_mut().find(|e| e.id == id) {
        e.status = status.to_string();
    }
    save_registry(wiki_dir, &index)
}
