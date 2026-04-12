# Reference Pipeline Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the copy-and-dump intake pipeline with a reference card pipeline: Confluence and URL sources produce living reference cards (link + summary + routing metadata); only file sources capture content.

**Architecture:** A `SourceKind` enum discriminates the three source types throughout the pipeline. `ContentItem` replaces the 4-tuple in `all_content`. Body builders in `curio_docs.rs` are shared across intake, process_intake, and gold_publish. Source kind is inferred from `source_id` in `curio_metadata` for downstream stages.

**Tech Stack:** Rust, Serde JSON (ADF bodies), scraper (HTML → summary), existing Confluence REST client.

---

## File Map

| File | What changes |
|---|---|
| `curio-rs/src/lib.rs` | Add `SourceKind` enum + `ContentItem` struct |
| `curio-rs/src/curio_docs.rs` | Add `extract_summary_from_html`, `extract_summary_from_text`, `build_reference_card_adf`, `build_capture_intake_adf` |
| `curio-rs/src/commands/intake.rs` | Replace tuple with `ContentItem`; branch body on kind; extract summary at collection |
| `curio-rs/src/commands/process_intake.rs` | Infer kind; update staged body builder |
| `curio-rs/src/commands/gold_publish.rs` | Infer kind; reference stub vs content page for published output |

---

## Task 1: Add SourceKind and ContentItem to lib.rs

**Files:**
- Modify: `curio-rs/src/lib.rs`

- [ ] **Step 1: Write the failing tests**

Add at the bottom of `curio-rs/src/lib.rs`:

```rust
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
}
```

- [ ] **Step 2: Run tests — expect compile failure**

```bash
cd curio-rs && cargo test source_kind_tests 2>&1 | head -30
```

Expected: `error[E0422]: cannot find type SourceKind in this scope`

- [ ] **Step 3: Add SourceKind enum and ContentItem struct**

Insert after the `BranchIndex` struct definition (around line 155) in `curio-rs/src/lib.rs`:

```rust
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
                // Fallback: construct a direct link via page ID (works on Confluence Cloud)
                Some(format!(
                    "{}/wiki/pages/{}",
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
```

- [ ] **Step 4: Run tests — expect pass**

```bash
cd curio-rs && cargo test source_kind_tests 2>&1
```

Expected: `test result: ok. 12 passed`

- [ ] **Step 5: Verify full build still clean**

```bash
cd curio-rs && cargo build 2>&1 | tail -5
```

Expected: `Finished` with no errors.

- [ ] **Step 6: Commit**

```bash
git add curio-rs/src/lib.rs
git commit -m "feat: add SourceKind and ContentItem for reference pipeline"
```

---

## Task 2: Add body builders and summary extraction to curio_docs.rs

**Files:**
- Modify: `curio-rs/src/curio_docs.rs`

These functions are shared across intake, process_intake, and gold_publish, so they live in curio_docs.

- [ ] **Step 1: Write failing tests**

Add at the bottom of `curio-rs/src/curio_docs.rs`:

```rust
#[cfg(test)]
mod body_builder_tests {
    use super::*;

    #[test]
    fn extract_summary_from_html_takes_first_paragraph() {
        let html = "<p>Hello world this is a long enough paragraph.</p><p>Second paragraph.</p>";
        let result = extract_summary_from_html(html, 300);
        assert!(result.is_some());
        let s = result.unwrap();
        assert!(s.contains("Hello world"));
    }

    #[test]
    fn extract_summary_from_html_ignores_short_fragments() {
        let html = "<p>Hi</p><p>This one is long enough to be a summary candidate yes.</p>";
        let result = extract_summary_from_html(html, 300);
        assert!(result.is_some());
        assert!(result.unwrap().contains("This one is long enough"));
    }

    #[test]
    fn extract_summary_from_html_returns_none_when_empty() {
        let result = extract_summary_from_html("<p></p><p>   </p>", 300);
        assert!(result.is_none());
    }

    #[test]
    fn extract_summary_from_html_truncates_at_max_chars() {
        let long = "a".repeat(500);
        let html = format!("<p>{}</p>", long);
        let result = extract_summary_from_html(&html, 100);
        assert!(result.is_some());
        let s = result.unwrap();
        assert!(s.len() <= 105); // max_chars + "…"
    }

    #[test]
    fn extract_summary_from_text_takes_first_long_line() {
        let text = "short\n\nThis line is definitely long enough to qualify as a summary candidate.\n\nAnother line.";
        let result = extract_summary_from_text(text, 300);
        assert!(result.is_some());
        assert!(result.unwrap().contains("This line is definitely"));
    }

    #[test]
    fn extract_summary_from_text_returns_none_when_no_long_lines() {
        let text = "hi\nok\nyes";
        assert!(extract_summary_from_text(text, 300).is_none());
    }

    #[test]
    fn reference_card_adf_is_valid_json() {
        let body = build_reference_card_adf(
            "Test Page",
            "confluence-page:1:page:2",
            Some("https://example.com/wiki/pages/2"),
            Some("A useful summary of the page."),
            "Intake",
            "2026-04-11T10:00:00Z",
            &[("subject_key", "test"), ("content_type", "confluence_page")],
            &[],
        );
        let parsed: serde_json::Value = serde_json::from_str(&body).expect("must be valid JSON");
        assert_eq!(parsed["type"], "doc");
        assert_eq!(parsed["version"], 1);
    }

    #[test]
    fn reference_card_adf_includes_source_link_and_summary() {
        let body = build_reference_card_adf(
            "My Page",
            "url:https://example.com",
            Some("https://example.com"),
            Some("Great summary here."),
            "Staged",
            "2026-04-11T10:00:00Z",
            &[],
            &["curio review approve <id>"],
        );
        // The ADF JSON must contain the URL and summary text somewhere
        assert!(body.contains("https://example.com"));
        assert!(body.contains("Great summary here."));
        assert!(body.contains("curio review approve"));
    }

    #[test]
    fn capture_intake_adf_is_valid_json() {
        let body = build_capture_intake_adf(
            "My Notes",
            "file:/tmp/notes.txt",
            "Intake",
            "2026-04-11T10:00:00Z",
            "Line one\nLine two\nLine three",
            "text/plain",
        );
        let parsed: serde_json::Value = serde_json::from_str(&body).expect("must be valid JSON");
        assert_eq!(parsed["type"], "doc");
    }

    #[test]
    fn capture_intake_adf_truncates_long_content() {
        let long_content = (0..600).map(|i| format!("line {}", i)).collect::<Vec<_>>().join("\n");
        let body = build_capture_intake_adf(
            "Big File",
            "file:/tmp/big.txt",
            "Intake",
            "2026-04-11",
            &long_content,
            "text/plain",
        );
        assert!(body.contains("truncated"));
    }
}
```

- [ ] **Step 2: Run tests — expect compile failure**

```bash
cd curio-rs && cargo test body_builder_tests 2>&1 | head -30
```

Expected: errors for missing functions `extract_summary_from_html`, `extract_summary_from_text`, `build_reference_card_adf`, `build_capture_intake_adf`.

- [ ] **Step 3: Add summary extraction helpers**

Add before the `fn append_section` helper at the bottom of `curio-rs/src/curio_docs.rs`:

```rust
/// Extracts a short summary from Confluence storage HTML.
/// Finds the first text span of >= 20 chars, strips tags, and truncates to `max_chars`.
pub fn extract_summary_from_html(html: &str, max_chars: usize) -> Option<String> {
    let fragment = Html::parse_fragment(html);
    let raw = fragment
        .root_element()
        .text()
        .collect::<Vec<_>>()
        .join(" ");
    let cleaned = raw.split_whitespace().collect::<Vec<_>>().join(" ");
    if cleaned.len() < 20 {
        return None;
    }
    Some(truncate_summary(&cleaned, max_chars))
}

/// Extracts a short summary from plain text.
/// Finds the first line of >= 20 non-whitespace chars and truncates to `max_chars`.
pub fn extract_summary_from_text(text: &str, max_chars: usize) -> Option<String> {
    let line = text
        .lines()
        .map(|l| l.trim())
        .find(|l| l.len() >= 20)?;
    Some(truncate_summary(line, max_chars))
}

fn truncate_summary(s: &str, max_chars: usize) -> String {
    if s.len() <= max_chars {
        s.to_string()
    } else {
        // Truncate at a word boundary near max_chars
        let cut = &s[..max_chars];
        let trimmed = cut.rfind(' ').map(|i| &cut[..i]).unwrap_or(cut);
        format!("{}…", trimmed)
    }
}
```

- [ ] **Step 4: Add ADF helpers for the body builders**

Add right below the summary helpers:

```rust
fn rc_heading(level: u8, text: &str) -> Value {
    json!({
        "type": "heading",
        "attrs": { "level": level },
        "content": [{ "type": "text", "text": text }]
    })
}

fn rc_paragraph(text: &str) -> Value {
    json!({
        "type": "paragraph",
        "content": [{ "type": "text", "text": text }]
    })
}

fn rc_paragraph_with_link(label: &str, link_text: &str, href: &str) -> Value {
    json!({
        "type": "paragraph",
        "content": [
            { "type": "text", "text": label },
            {
                "type": "text",
                "text": link_text,
                "marks": [{ "type": "link", "attrs": { "href": href } }]
            }
        ]
    })
}

fn rc_table(rows: &[(&str, &str)]) -> Value {
    let table_rows: Vec<Value> = rows
        .iter()
        .map(|(k, v)| {
            json!({
                "type": "tableRow",
                "content": [
                    { "type": "tableCell", "content": [rc_paragraph(k)] },
                    { "type": "tableCell", "content": [rc_paragraph(v)] }
                ]
            })
        })
        .collect();
    json!({ "type": "table", "content": table_rows })
}

fn rc_bullet_list(items: &[&str]) -> Value {
    let list_items: Vec<Value> = items
        .iter()
        .map(|item| {
            json!({
                "type": "listItem",
                "content": [rc_paragraph(item)]
            })
        })
        .collect();
    json!({ "type": "bulletList", "content": list_items })
}
```

- [ ] **Step 5: Add build_reference_card_adf**

Add right below the rc_ helpers:

```rust
/// Builds an ADF reference card body for Confluence page and URL sources.
/// Produces: title heading, status line, source link, summary, metadata table, optional commands.
pub fn build_reference_card_adf(
    title: &str,
    source_id: &str,
    origin_url: Option<&str>,
    summary: Option<&str>,
    lane: &str,
    ingested_at: &str,
    metadata_rows: &[(&str, &str)],
    commands: &[&str],
) -> String {
    let mut content: Vec<Value> = vec![
        rc_heading(1, title),
        rc_paragraph(&format!("Curio lane: {} | Source: {} | {}", lane, source_id, ingested_at)),
        rc_heading(2, "Source"),
    ];

    match origin_url {
        Some(url) => content.push(rc_paragraph_with_link("Origin: ", title, url)),
        None => content.push(rc_paragraph(&format!("Origin: {}", source_id))),
    }

    content.push(rc_heading(2, "Summary"));
    content.push(rc_paragraph(summary.unwrap_or("No summary available.")));

    if !metadata_rows.is_empty() {
        content.push(rc_heading(2, "Metadata"));
        content.push(rc_table(metadata_rows));
    }

    if !commands.is_empty() {
        content.push(rc_heading(2, "Commands"));
        content.push(rc_bullet_list(commands));
    }

    let adf = json!({ "type": "doc", "version": 1, "content": content });
    serde_json::to_string(&adf).unwrap_or_else(|_| {
        json!({
            "type": "doc", "version": 1,
            "content": [rc_heading(1, title), rc_paragraph(source_id)]
        })
        .to_string()
    })
}
```

- [ ] **Step 6: Add build_capture_intake_adf**

Add right below `build_reference_card_adf`:

```rust
/// Builds an ADF body for file capture sources.
/// Produces: title heading, status line, file metadata, content body (truncated at 500 lines).
pub fn build_capture_intake_adf(
    title: &str,
    source_id: &str,
    lane: &str,
    ingested_at: &str,
    content: &str,
    mime: &str,
) -> String {
    const MAX_LINES: usize = 500;
    let lines: Vec<&str> = content.lines().collect();
    let truncated = lines.len() > MAX_LINES;
    let display_lines = &lines[..lines.len().min(MAX_LINES)];

    let mut body_nodes: Vec<Value> = display_lines
        .iter()
        .filter(|l| !l.trim().is_empty())
        .map(|l| rc_paragraph(l))
        .collect();

    if truncated {
        body_nodes.push(rc_paragraph(&format!(
            "— Content truncated at {} lines ({} total). Open the source file for the full content.",
            MAX_LINES,
            lines.len()
        )));
    }

    let metadata = rc_table(&[
        ("Source", source_id),
        ("Type", mime),
        ("Ingested", ingested_at),
        ("Lines", &lines.len().to_string()),
    ]);

    let mut content_nodes: Vec<Value> = vec![
        rc_heading(1, title),
        rc_paragraph(&format!("Curio lane: {} | Captured file content.", lane)),
        rc_heading(2, "File Metadata"),
        metadata,
        rc_heading(2, "Content"),
    ];
    content_nodes.extend(body_nodes);

    let adf = json!({ "type": "doc", "version": 1, "content": content_nodes });
    serde_json::to_string(&adf).unwrap_or_else(|_| {
        json!({
            "type": "doc", "version": 1,
            "content": [rc_heading(1, title), rc_paragraph(source_id)]
        })
        .to_string()
    })
}
```

- [ ] **Step 7: Run tests — expect pass**

```bash
cd curio-rs && cargo test body_builder_tests 2>&1
```

Expected: `test result: ok. 10 passed`

- [ ] **Step 8: Full build clean**

```bash
cd curio-rs && cargo build 2>&1 | tail -5
```

Expected: `Finished` with no errors.

- [ ] **Step 9: Run all tests**

```bash
cd curio-rs && cargo test 2>&1 | tail -5
```

Expected: all tests pass.

- [ ] **Step 10: Commit**

```bash
git add curio-rs/src/curio_docs.rs
git commit -m "feat: add reference card and capture ADF body builders"
```

---

## Task 3: Refactor intake.rs to use ContentItem

**Files:**
- Modify: `curio-rs/src/commands/intake.rs`

The key changes:
1. Replace the 4-tuple with `ContentItem` at every `all_content.push(...)` call
2. Extract `summary` at collection time from the raw source (before `extract_page_text` discards structure)
3. Update `process_single_content` signature
4. Branch `create_scoped_intake_page` on `item.kind`
5. Delete `build_safe_intake_body` and the `adf_*` helpers

- [ ] **Step 1: Update imports**

Replace the imports block at the top of `curio-rs/src/commands/intake.rs`:

```rust
use crate::confluence::http_timeout_duration;
use crate::curio_docs::{
    ADMIN_TITLE, AUDIT_TITLE, AuditEntry, RegistryRecord, append_audit_entry, audit_bucket_path,
    build_admin_root_body, build_audit_root_body, build_capture_intake_adf,
    build_reference_card_adf, build_registry_root_body, ensure_registry_record, ensure_scoped_page,
    ensure_scoped_structure_page, extract_page_text, extract_summary_from_html,
    extract_summary_from_text, infer_route_plan, update_branch_index,
};
use crate::output::emit_json;
use crate::{
    ChangeProposal, ContentItem, Result, SourceKind, compact_change_proposal, config::Config,
    confluence::ConfluenceClient, generate_change_proposal_with_agent,
};
use anyhow::Context;
use chrono::Utc;
use scraper::{Html, Selector};
use serde::Serialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
```

- [ ] **Step 2: Replace all_content type**

Find the line (around line 91):
```rust
let mut all_content: Vec<(String, String, String, Option<String>)> = Vec::new();
```

Replace with:
```rust
let mut all_content: Vec<ContentItem> = Vec::new();
```

- [ ] **Step 3: Update the Confluence folder push**

Find (around line 162):
```rust
all_content.push((
    content,
    format!("confluence-folder:{}:page:{}", folder_id, page_id),
    "confluence_page".to_string(),
    Some(page_title),
));
```

Replace with:
```rust
let webui_path = page["_links"]["webui"].as_str().map(|s| s.to_string());
let summary = page["body"]["storage"]["value"]
    .as_str()
    .and_then(|html| extract_summary_from_html(html, 300))
    .or_else(|| extract_summary_from_text(&content, 300));
all_content.push(ContentItem {
    text: content,
    source_id: format!("confluence-folder:{}:page:{}", folder_id, page_id),
    subject_hint: Some(page_title),
    kind: SourceKind::ConfluencePage { page_id: page_id.to_string(), webui_path },
    summary,
});
```

- [ ] **Step 4: Update the Confluence page-tree root push**

Find (around line 185):
```rust
all_content.push((
    root_content,
    format!("confluence-page:{}:page:{}", page_id, page_id),
    "confluence_page".to_string(),
    Some(root_title.clone()),
));
```

Replace with:
```rust
let root_webui_path = root_page["_links"]["webui"].as_str().map(|s| s.to_string());
let root_summary = root_page["body"]["storage"]["value"]
    .as_str()
    .and_then(|html| extract_summary_from_html(html, 300))
    .or_else(|| extract_summary_from_text(&root_content, 300));
all_content.push(ContentItem {
    text: root_content,
    source_id: format!("confluence-page:{}:page:{}", page_id, page_id),
    subject_hint: Some(root_title.clone()),
    kind: SourceKind::ConfluencePage { page_id: page_id.to_string(), webui_path: root_webui_path },
    summary: root_summary,
});
```

- [ ] **Step 5: Update the Confluence page-tree descendant push**

Find (around line 249):
```rust
all_content.push((
    content,
    format!("confluence-page:{}:page:{}", page_id, child_page_id),
    "confluence_page".to_string(),
    Some(page_title),
));
```

Replace with:
```rust
let webui_path = page["_links"]["webui"].as_str().map(|s| s.to_string());
let summary = page["body"]["storage"]["value"]
    .as_str()
    .and_then(|html| extract_summary_from_html(html, 300))
    .or_else(|| extract_summary_from_text(&content, 300));
all_content.push(ContentItem {
    text: content,
    source_id: format!("confluence-page:{}:page:{}", page_id, child_page_id),
    subject_hint: Some(page_title),
    kind: SourceKind::ConfluencePage { page_id: child_page_id.to_string(), webui_path },
    summary,
});
```

- [ ] **Step 6: Update the Confluence space push**

Find (around line 304):
```rust
all_content.push((
    content,
    format!("confluence-space:{}:page:{}", space_key, page_id),
    "confluence_page".to_string(),
    Some(page_title),
));
```

Replace with:
```rust
let webui_path = loaded["_links"]["webui"].as_str().map(|s| s.to_string());
let summary = loaded["body"]["storage"]["value"]
    .as_str()
    .and_then(|html| extract_summary_from_html(html, 300))
    .or_else(|| extract_summary_from_text(&content, 300));
all_content.push(ContentItem {
    text: content,
    source_id: format!("confluence-space:{}:page:{}", space_key, page_id),
    subject_hint: Some(page_title),
    kind: SourceKind::ConfluencePage { page_id: page_id.to_string(), webui_path },
    summary,
});
```

- [ ] **Step 7: Update the URL push**

Find (around line 317):
```rust
all_content.push((
    content,
    format!("url:{}", u),
    "web_page".to_string(),
    subject_hint.clone(),
));
```

Replace with:
```rust
let summary = extract_summary_from_text(&content, 300);
all_content.push(ContentItem {
    text: content,
    source_id: format!("url:{}", u),
    subject_hint: subject_hint.clone(),
    kind: SourceKind::Url { url: u.clone() },
    summary,
});
```

- [ ] **Step 8: Update the single-file push**

Find (around line 339):
```rust
all_content.push((
    content,
    format!("file:{}", f.display()),
    get_mime_type(filename),
    derived_subject_hint.or_else(|| subject_hint.clone()),
));
```

Replace with:
```rust
let mime = get_mime_type(filename);
let summary = extract_summary_from_text(&content, 300);
all_content.push(ContentItem {
    text: content,
    source_id: format!("file:{}", f.display()),
    subject_hint: derived_subject_hint.or_else(|| subject_hint.clone()),
    kind: SourceKind::File { path: f.display().to_string(), mime },
    summary,
});
```

- [ ] **Step 9: Update the folder walk push**

Find (around line 365):
```rust
all_content.push((
    content,
    format!("file:{}", path.display()),
    get_mime_type(filename),
    derived_subject_hint.or_else(|| subject_hint.clone()),
));
```

Replace with:
```rust
let mime = get_mime_type(filename);
let summary = extract_summary_from_text(&content, 300);
all_content.push(ContentItem {
    text: content,
    source_id: format!("file:{}", path.display()),
    subject_hint: derived_subject_hint.or_else(|| subject_hint.clone()),
    kind: SourceKind::File { path: path.display().to_string(), mime },
    summary,
});
```

- [ ] **Step 10: Update the for-loop and process_single_content call**

Find (around line 381):
```rust
for (content, source_id, content_type, item_subject_hint) in all_content {
    let effective_subject_hint = item_subject_hint.or_else(|| subject_hint.clone());
    match process_single_content(
        &client,
        config,
        dry_run,
        json_output,
        &intake_page_id,
        &registry_root_id,
        &audit_root_id,
        &content,
        &source_id,
        &content_type,
        &effective_subject_hint,
        metadata_str,
    )
```

Replace with:
```rust
for mut item in all_content {
    // subject_hint from CLI overrides per-item hint only if item has none
    if item.subject_hint.is_none() {
        item.subject_hint = subject_hint.clone();
    }
    match process_single_content(
        &client,
        config,
        dry_run,
        json_output,
        &intake_page_id,
        &registry_root_id,
        &audit_root_id,
        &item,
        metadata_str,
    )
```

- [ ] **Step 11: Update process_single_content signature and body**

Find the function signature:
```rust
async fn process_single_content(
    client: &ConfluenceClient,
    config: &Config,
    dry_run: bool,
    json_output: bool,
    intake_page_id: &str,
    registry_root_id: &str,
    audit_root_id: &str,
    content: &str,
    source_id: &str,
    content_type: &str,
    subject_hint: &Option<String>,
    metadata_str: &Option<String>,
) -> Result<bool> {
```

Replace the signature line and the opening variable bindings with:
```rust
async fn process_single_content(
    client: &ConfluenceClient,
    config: &Config,
    dry_run: bool,
    json_output: bool,
    intake_page_id: &str,
    registry_root_id: &str,
    audit_root_id: &str,
    item: &ContentItem,
    metadata_str: &Option<String>,
) -> Result<bool> {
    let content = &item.text;
    let source_id = &item.source_id;
    let content_type = item.kind.content_type();
    let subject_hint = &item.subject_hint;
```

The rest of the function body uses `content`, `source_id`, `content_type`, `subject_hint` — those locals are now set above. No further changes needed in the function body EXCEPT in the `create_scoped_intake_page` call.

Find the call to `create_scoped_intake_page` (around line 576):
```rust
create_scoped_intake_page(
    client,
    &config.content_model.space_key,
    intake_page_id,
    &page_title,
    content,
    source_id,
    json_output,
)
.await?
```

Replace with:
```rust
create_scoped_intake_page(
    client,
    &config.content_model.space_key,
    intake_page_id,
    &page_title,
    item,
    &config.connection.confluence_url,
    json_output,
)
.await?
```

- [ ] **Step 12: Rewrite create_scoped_intake_page**

Find and replace the entire `create_scoped_intake_page` function:

```rust
async fn create_scoped_intake_page(
    client: &ConfluenceClient,
    space_key: &str,
    intake_page_id: &str,
    page_title: &str,
    item: &ContentItem,
    confluence_base_url: &str,
    json_output: bool,
) -> Result<String> {
    let now = Utc::now().to_rfc3339();
    let body = if item.kind.is_reference() {
        let origin_url = item.kind.origin_url(confluence_base_url);
        build_reference_card_adf(
            page_title,
            &item.source_id,
            origin_url.as_deref(),
            item.summary.as_deref(),
            "Intake",
            &now,
            &[
                ("source_id", &item.source_id),
                ("content_type", item.kind.content_type()),
            ],
            &[],
        )
    } else {
        let mime = match &item.kind {
            SourceKind::File { mime, .. } => mime.as_str(),
            _ => "application/octet-stream",
        };
        build_capture_intake_adf(
            page_title,
            &item.source_id,
            "Intake",
            &now,
            &item.text,
            mime,
        )
    };

    let unique_title_on_collision = |original: &str| -> String {
        build_unique_intake_title(original, &item.source_id)
    };

    match client
        .create_or_update_page(space_key, Some(intake_page_id), page_title, "atlas_doc_format", &body)
        .await
    {
        Ok(page_id) => Ok(page_id),
        Err(err) if err.to_string().contains("same TITLE in this space") => {
            let unique_title = unique_title_on_collision(page_title);
            if !json_output {
                println!(
                    "Title collision for '{}'; retrying with unique title '{}'",
                    page_title, unique_title
                );
            }
            client
                .create_or_update_page(space_key, Some(intake_page_id), &unique_title, "atlas_doc_format", &body)
                .await
                .with_context(|| {
                    format!(
                        "Failed to create intake page '{}' from source {} after title collision retry",
                        page_title, &item.source_id
                    )
                })
        }
        Err(err) => Err(err).with_context(|| {
            format!(
                "Failed to create intake page '{}' from source {}",
                page_title, &item.source_id
            )
        }),
    }
}
```

- [ ] **Step 13: Delete the old body builders and ADF helpers**

Delete these functions entirely from `intake.rs`:
- `fn build_safe_intake_body(...)` 
- `fn adf_heading(...)`
- `fn adf_paragraph_text(...)`
- `fn adf_expand(...)`
- `fn adf_table(...)`
- `fn adf_text(...)`

- [ ] **Step 14: Build to fix any remaining compile errors**

```bash
cd curio-rs && cargo build 2>&1
```

Fix any remaining compile errors (likely minor: unused imports, missing `&item.source_id` vs `source_id` binding). The error messages will point you to the exact lines.

- [ ] **Step 15: Run all tests**

```bash
cd curio-rs && cargo test 2>&1 | tail -5
```

Expected: all tests pass.

- [ ] **Step 16: Commit**

```bash
git add curio-rs/src/commands/intake.rs
git commit -m "feat: refactor intake pipeline to use ContentItem and reference cards"
```

---

## Task 4: Update process_intake.rs staged body for source kind

**Files:**
- Modify: `curio-rs/src/commands/process_intake.rs`

The staged body for reference sources should be a reference card updated with routing analysis. For file captures, keep content body but clean up the layout.

The source kind is inferred from `source_id` in `curio_metadata` — no pipeline struct changes needed here.

- [ ] **Step 1: Update imports**

Replace the first import block:
```rust
use crate::curio_docs::{
    ADMIN_TITLE, AUDIT_TITLE, AuditEntry, RegistryRecord, append_audit_entry, audit_bucket_path,
    build_admin_root_body, build_audit_root_body, build_registry_root_body, ensure_registry_record,
    ensure_scoped_page, ensure_scoped_structure_page, extract_page_text, infer_route_plan,
    update_branch_index,
};
```

With:
```rust
use crate::curio_docs::{
    ADMIN_TITLE, AUDIT_TITLE, AuditEntry, RegistryRecord, append_audit_entry, audit_bucket_path,
    build_admin_root_body, build_audit_root_body, build_capture_intake_adf,
    build_reference_card_adf, build_registry_root_body, ensure_registry_record, ensure_scoped_page,
    ensure_scoped_structure_page, extract_page_text, infer_route_plan, update_branch_index,
};
use crate::SourceKind;
```

- [ ] **Step 2: Replace build_stage_artifact_body with a kind-aware version**

Find and replace the entire `build_stage_artifact_body` function with:

```rust
fn build_stage_artifact_body(
    title: &str,
    source_page: &serde_json::Value,
    source_body: &str,
    analysis_result: &AgentAnalysis,
    proposal: &ChangeProposal,
    lane_name: &str,
    conflict_details: Option<&serde_json::Value>,
    confluence_url: &str,
) -> String {
    let source_id = proposal
        .source_refs
        .first()
        .map(|s| s.as_str())
        .unwrap_or_default();
    let kind = SourceKind::from_source_id(source_id);

    let target_path = if proposal.target_path.is_empty() {
        "Unresolved".to_string()
    } else {
        proposal.target_path.join(" / ")
    };
    let conflict_text = conflict_details
        .and_then(|v| v["message"].as_str())
        .unwrap_or("No conflict detected");
    let now = chrono::Utc::now().to_rfc3339();

    let commands = [
        "curio review approve <page-id>",
        "curio review reject <page-id>",
        "curio gold-resolve <page-id>",
        "curio gold-publish <page-id>",
    ];

    if kind.is_reference() {
        // Reference pipeline: reference card + routing panel
        let origin_url = {
            // For Confluence pages, webui_path isn't available here — construct from page JSON
            let webui = source_page["_links"]["webui"]
                .as_str()
                .map(|p| format!("{}{}", confluence_url.trim_end_matches('/'), p));
            webui.or_else(|| kind.origin_url(confluence_url))
        };
        let source_page_title = source_page["title"].as_str().unwrap_or(title);

        let metadata_rows: Vec<(String, String)> = vec![
            ("Source ID".to_string(), source_id.to_string()),
            ("Target path".to_string(), target_path.clone()),
            ("Confidence".to_string(), format!("{:.2}", analysis_result.confidence_score)),
            ("Keywords".to_string(), analysis_result.keywords.join(", ")),
            ("Review note".to_string(), conflict_text.to_string()),
        ];
        let metadata_ref: Vec<(&str, &str)> = metadata_rows
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();

        build_reference_card_adf(
            source_page_title,
            source_id,
            origin_url.as_deref(),
            Some(&analysis_result.summary),
            lane_name,
            &now,
            &metadata_ref,
            &commands.iter().map(|c| *c).collect::<Vec<_>>(),
        )
    } else {
        // Capture pipeline: Curio analysis panel + content body
        let mime = match &kind {
            SourceKind::File { mime, .. } => mime.clone(),
            _ => "application/octet-stream".to_string(),
        };
        // Re-use capture layout but prepend analysis summary as first content block
        let content_with_header = format!(
            "Curio analysis: {} | Confidence: {:.2} | Target: {} | {}\n\n{}",
            analysis_result.summary,
            analysis_result.confidence_score,
            target_path,
            conflict_text,
            source_body
        );
        build_capture_intake_adf(
            title,
            source_id,
            lane_name,
            &now,
            &content_with_header,
            &mime,
        )
    }
}
```

- [ ] **Step 3: Remove now-unused ADF helpers from process_intake.rs**

Delete these functions from `process_intake.rs` (they are now replaced by the shared `rc_*` helpers in `curio_docs.rs`):
- `fn adf_heading(...)`
- `fn adf_paragraph_text(...)`
- `fn adf_paragraph_from_lines(...)`
- `fn adf_paragraph_mixed(...)`
- `fn adf_text(...)`
- `fn adf_link_text(...)`
- `fn adf_nodes(...)`
- `fn adf_bullet_list(...)`
- `fn adf_table(...)`
- `fn build_source_excerpt_nodes(...)`

- [ ] **Step 4: Build clean**

```bash
cd curio-rs && cargo build 2>&1
```

Fix any compile errors from removed helpers (the `build_stage_artifact_body` call site passes identical arguments — confirm signature matches).

- [ ] **Step 5: Run all tests**

```bash
cd curio-rs && cargo test 2>&1 | tail -5
```

Expected: all pass.

- [ ] **Step 6: Commit**

```bash
git add curio-rs/src/commands/process_intake.rs
git commit -m "feat: update staged body builder for reference vs capture pipeline"
```

---

## Task 5: Update gold_publish.rs published page for source kind

**Files:**
- Modify: `curio-rs/src/commands/gold_publish.rs`

For reference sources, the published output is a reference stub (link + summary + metadata in the right place in the Published tree). For file captures, keep the existing content write.

- [ ] **Step 1: Update imports**

```rust
use crate::curio_docs::{
    ADMIN_TITLE, AUDIT_TITLE, AuditEntry, RegistryRecord, append_audit_entry, audit_bucket_path,
    build_admin_root_body, build_audit_root_body, build_reference_card_adf,
    build_registry_root_body, ensure_registry_record, ensure_scoped_page,
    ensure_scoped_structure_page, update_branch_index,
};
use crate::SourceKind;
```

- [ ] **Step 2: Add kind inference before the changes loop**

In `run_gold_publish`, find the start of the changes loop (around `for change in &change_proposal.proposed_changes`). Before the loop, add:

```rust
let source_kind = SourceKind::from_source_id(&page_id_arg);
```

- [ ] **Step 3: Branch on source kind for the published body**

Find the section that builds `updated_target_body` and calls `create_or_update_page`. It currently looks like:

```rust
let updated_target_body = apply_content_change(change, &current_target_body)?;
```

Replace with:
```rust
let updated_target_body = if source_kind.is_reference() {
    let origin_url = {
        let webui = curio_metadata_mut["source_page_webui"]
            .as_str()
            .map(|p| format!("{}{}", config.connection.confluence_url.trim_end_matches('/'), p));
        webui.or_else(|| source_kind.origin_url(&config.connection.confluence_url))
    };
    let summary = curio_metadata_mut["agent_analysis"]["summary"]
        .as_str()
        .or_else(|| curio_metadata_mut["subject_key"].as_str())
        .unwrap_or(&change.target_page_title)
        .to_string();
    let now = Utc::now().to_rfc3339();
    let metadata_rows: Vec<(String, String)> = vec![
        ("Source ID".to_string(), page_id_arg.clone()),
        ("Last verified".to_string(), now.clone()),
    ];
    let metadata_ref: Vec<(&str, &str)> = metadata_rows
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    build_reference_card_adf(
        &change.target_page_title,
        &page_id_arg,
        origin_url.as_deref(),
        Some(&summary),
        "Published",
        &now,
        &metadata_ref,
        &[],
    )
} else {
    apply_content_change(change, &current_target_body)?
};
```

- [ ] **Step 4: Build clean**

```bash
cd curio-rs && cargo build 2>&1
```

Fix any compile errors (likely the `config` parameter — check if `run_gold_publish` has access to it; it does via the `config: &Config` argument).

- [ ] **Step 5: Run all tests**

```bash
cd curio-rs && cargo test 2>&1 | tail -5
```

Expected: all pass.

- [ ] **Step 6: Commit**

```bash
git add curio-rs/src/commands/gold_publish.rs
git commit -m "feat: published reference stubs for Confluence/URL sources"
```

---

## Task 6: End-to-end smoke test

**Files:** No code changes — verification only.

- [ ] **Step 1: Full clean build and test**

```bash
cd curio-rs && cargo build 2>&1 && cargo test 2>&1
```

Expected: build clean, all tests pass.

- [ ] **Step 2: Dry-run a Confluence page intake**

```bash
cd /path/to/repo && curio intake-create --dry-run --url "https://<your-space>/wiki/spaces/CURIO/pages/<id>"
```

Expected: prints `(Dry run) Would create page: '<title>' under Intake` with no errors.

- [ ] **Step 3: Live ingest a single Confluence page**

```bash
curio intake-create --url "https://<your-space>/wiki/spaces/CURIO/pages/<id>"
```

Expected:
- Page created under Intake in Confluence
- Page body shows: title heading, "Curio lane: Intake", Source section with clickable link, Summary section (not "No summary available"), Metadata table
- No "sanitized intake copy" message anywhere

- [ ] **Step 4: Live ingest a URL**

```bash
curio intake-create --url "https://example.com"
```

Expected:
- Reference card with active link to `https://example.com`
- Summary extracted from page content
- No raw text dump

- [ ] **Step 5: Live ingest a file**

```bash
echo "This is a test file with enough content to verify the capture pipeline works correctly." > /tmp/curio_test.txt
curio intake-create --file /tmp/curio_test.txt
```

Expected:
- Content page under Intake with the file content in the body
- File metadata table (source, type, lines)

- [ ] **Step 6: Run process-intake and verify staged reference card**

```bash
curio process-intake --limit 3
```

Expected:
- Intake pages moved to Staged or Review
- Staged pages show reference card layout + routing metadata
- No raw text dump in any staged page

- [ ] **Step 7: Final commit with spec reference**

```bash
git add -A
git commit -m "chore: reference pipeline complete — see docs/superpowers/specs/2026-04-11-reference-pipeline-design.md"
```

---

## Self-Review Notes

**Spec coverage check:**
- ✅ Confluence sources → reference card: Tasks 3, 4, 5
- ✅ URL sources → reference card with active link: Task 3 (URL push) + Task 2 (body builder)
- ✅ File sources → capture page with content: Tasks 2, 3
- ✅ Summary extracted at collection time: Task 3, Step 3–9
- ✅ No "sanitized" message: Task 3, Step 12 (new create_scoped_intake_page)
- ✅ Published reference stub: Task 5
- ✅ `cargo build` clean: verified in each task
- ✅ All tests pass: verified in each task

**Type consistency check:**
- `SourceKind` defined Task 1, used Task 2 (body builders accept `&SourceKind`), used Task 3 (ContentItem.kind), used Task 4 and 5 (`SourceKind::from_source_id`)
- `ContentItem` defined Task 1, used Task 3 throughout
- `build_reference_card_adf` defined Task 2, called in Tasks 3, 4, 5 with identical signature
- `build_capture_intake_adf` defined Task 2, called in Tasks 3, 4 with identical signature
- `extract_summary_from_html` / `extract_summary_from_text` defined Task 2, called in Task 3

**One gap noted and addressed:** `process_intake` `build_stage_artifact_body` takes `source_page: &serde_json::Value` — for reference sources, this page may not have `_links.webui` if it came from a CQL result (which doesn't include body/links). The fallback `kind.origin_url(confluence_url)` handles this case in Task 4, Step 2.
