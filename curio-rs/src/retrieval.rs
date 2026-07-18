//! Deterministic retrieval over canonical published Markdown pages.

use crate::{
    acl::{self, AccessContext},
    error::CliValidationError,
    wiki_fs,
    wiki_fs::parse_wiki_page,
};
use anyhow::{Context, Result};
use serde::Serialize;
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

const EXCERPT_MAX_CHARS: usize = 280;

/// Input to the published-page lexical retriever.
#[derive(Debug, Clone)]
pub struct RetrieveRequest {
    pub query: String,
    pub category: Option<String>,
    pub limit: usize,
}

/// The stable machine-readable result returned for one published page.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct RetrieveResult {
    /// Path-derived local ID; it is not a workspace-global identity.
    pub id: String,
    pub title: String,
    /// Unix-style path relative to wiki/published/.
    pub path: String,
    pub category: String,
    pub excerpt: String,
    pub score: u32,
    pub source_uri: Option<String>,
    pub curio_uri: String,
    pub content_hash: String,
    pub updated_at: String,
    pub authority: &'static str,
    pub last_commit: Option<LastCommit>,
}

/// Best-effort provenance for the last Git commit touching a page.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct LastCommit {
    pub hash: String,
    pub author: String,
    pub committed_at: String,
    pub subject: String,
}

/// Library response data. The CLI wraps this in the shared JSON envelope.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct RetrieveResponse {
    pub query: String,
    pub category: Option<String>,
    pub limit: usize,
    pub count: usize,
    pub results: Vec<RetrieveResult>,
}

/// Canonical published page fetch by stable retrieve id.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct FetchResponse {
    pub id: String,
    pub title: String,
    /// Unix-style path relative to wiki/published/.
    pub path: String,
    pub category: String,
    /// Canonical Markdown body without YAML frontmatter.
    pub body: String,
    pub source_uri: Option<String>,
    pub curio_uri: String,
    pub content_hash: String,
    pub updated_at: String,
    pub authority: &'static str,
    pub last_commit: Option<LastCommit>,
}

/// Metadata for a canonical published page that is visible to an access
/// context. This is intentionally smaller than a retrieval result so callers
/// that expose workspace metadata can share the same ACL filtering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessiblePublishedPage {
    pub path: String,
    pub category: String,
}

/// List canonical published pages visible to the supplied access context.
/// Pages with an ACL snapshot fail closed when no matching identity is given.
pub fn accessible_published_pages(
    wiki_dir: &Path,
    access: Option<&AccessContext>,
) -> Result<Vec<AccessiblePublishedPage>> {
    let published_dir = wiki_dir.join("published");
    let mut pages = Vec::new();
    for path in canonical_published_paths(&published_dir)? {
        let page = parse_wiki_page(&path)
            .with_context(|| format!("Failed to parse published page {}", path.display()))?;
        let relative_path = published_relative_path(&published_dir, &path);
        let acl_snapshot = acl::load_snapshot(wiki_dir, &page.frontmatter.source.id)?;
        if acl::can_read(acl_snapshot.as_ref(), access) {
            pages.push(AccessiblePublishedPage {
                category: page_category(&page.frontmatter.category, &relative_path),
                path: relative_path,
            });
        }
    }
    Ok(pages)
}

#[derive(Debug, Clone)]
struct ScoredResult {
    result: RetrieveResult,
    tier: u8,
}

/// Retrieve only canonical Markdown pages under wiki/published/.
pub fn retrieve_published(wiki_dir: &Path, request: &RetrieveRequest) -> Result<RetrieveResponse> {
    retrieve_published_with_access(wiki_dir, request, None)
}

/// Permission-aware retrieval. `None` retains legacy unrestricted behavior for
/// pages without an ACL snapshot; pages with a snapshot fail closed.
pub fn retrieve_published_with_access(
    wiki_dir: &Path,
    request: &RetrieveRequest,
    access: Option<&AccessContext>,
) -> Result<RetrieveResponse> {
    let terms = normalized_query_terms(&request.query)?;
    let category_filter = normalize_category(request.category.as_deref());
    let published_dir = wiki_dir.join("published");
    let paths = canonical_published_paths(&published_dir)?;

    let git_root = find_git_root(wiki_dir);
    let mut scored = Vec::new();
    for path in paths {
        let page = parse_wiki_page(&path)
            .with_context(|| format!("Failed to parse published page {}", path.display()))?;
        let relative_path = published_relative_path(&published_dir, &path);
        let category = page_category(&page.frontmatter.category, &relative_path);
        let acl_snapshot = acl::load_snapshot(wiki_dir, &page.frontmatter.source.id)?;
        if !acl::can_read(acl_snapshot.as_ref(), access) {
            continue;
        }
        if let Some(filter) = category_filter.as_deref()
            && !category_matches(&category, filter)
        {
            continue;
        }

        let title_tokens = token_set(&page.frontmatter.title);
        let keyword_tokens = token_set(&page.frontmatter.keywords.join(" "));
        let summary = page.frontmatter.source.summary.as_deref().unwrap_or("");
        let summary_tokens = token_set(summary);
        let body_tokens = token_set(&page.body);
        let title_hits = matching_terms(&terms, &title_tokens);
        let keyword_hits = matching_terms(&terms, &keyword_tokens);
        let summary_hits = matching_terms(&terms, &summary_tokens);
        let body_hits = matching_terms(&terms, &body_tokens);
        let tier = if title_hits > 0 {
            3
        } else if keyword_hits > 0 {
            2
        } else if summary_hits > 0 {
            1
        } else {
            0
        };
        if title_hits + keyword_hits + summary_hits + body_hits == 0 {
            continue;
        }

        let score = title_hits * 100 + keyword_hits * 60 + summary_hits * 30 + body_hits * 10;
        let excerpt = query_excerpt(&page.body, summary, &terms);
        let id = local_retrieval_id(&relative_path);
        let source_uri = page
            .frontmatter
            .source
            .origin_url
            .clone()
            .filter(|uri| !uri.trim().is_empty());
        let content_hash = if page.frontmatter.content_hash.trim().is_empty() {
            wiki_fs::content_hash(&page.body)
        } else {
            page.frontmatter.content_hash.clone()
        };
        let last_commit = git_root
            .as_deref()
            .and_then(|root| git_last_commit(root, &path).ok().flatten());

        scored.push(ScoredResult {
            result: RetrieveResult {
                id,
                title: page.frontmatter.title,
                path: relative_path.clone(),
                category,
                excerpt,
                score,
                source_uri,
                curio_uri: curio_uri(wiki_dir, &relative_path),
                content_hash,
                updated_at: page.frontmatter.updated_at,
                authority: "published",
                last_commit,
            },
            tier,
        });
    }

    scored.sort_by(|left, right| {
        right
            .tier
            .cmp(&left.tier)
            .then_with(|| right.result.score.cmp(&left.result.score))
            .then_with(|| left.result.path.cmp(&right.result.path))
            .then_with(|| left.result.id.cmp(&right.result.id))
    });
    let results: Vec<_> = scored
        .into_iter()
        .take(request.limit)
        .map(|item| item.result)
        .collect();

    Ok(RetrieveResponse {
        query: request.query.clone(),
        category: request.category.clone(),
        limit: request.limit,
        count: results.len(),
        results,
    })
}

/// Fetch a canonical published page by the stable local id emitted by retrieve.
pub fn fetch_published(wiki_dir: &Path, id: &str) -> Result<FetchResponse> {
    fetch_published_with_access(wiki_dir, id, None)
}

pub fn fetch_published_with_access(
    wiki_dir: &Path,
    id: &str,
    access: Option<&AccessContext>,
) -> Result<FetchResponse> {
    validate_fetch_id(id)?;
    let published_dir = wiki_dir.join("published");
    let git_root = find_git_root(wiki_dir);

    for path in canonical_published_paths(&published_dir)? {
        let relative_path = published_relative_path(&published_dir, &path);
        let candidate_id = local_retrieval_id(&relative_path);
        if candidate_id != id {
            continue;
        }

        let page = parse_wiki_page(&path)
            .with_context(|| format!("Failed to parse published page {}", path.display()))?;
        let acl_snapshot = acl::load_snapshot(wiki_dir, &page.frontmatter.source.id)?;
        if !acl::can_read(acl_snapshot.as_ref(), access) {
            return Err(CliValidationError::new(
                "fetch_not_found",
                "No accessible canonical published page exists for this retrieval id.",
                "Use an id returned by search for the current access context.",
            )
            .into());
        }
        let content_hash = if page.frontmatter.content_hash.trim().is_empty() {
            wiki_fs::content_hash(&page.body)
        } else {
            page.frontmatter.content_hash.clone()
        };
        let source_uri = page
            .frontmatter
            .source
            .origin_url
            .clone()
            .filter(|uri| !uri.trim().is_empty());
        let last_commit = git_root
            .as_deref()
            .and_then(|root| git_last_commit(root, &path).ok().flatten());

        return Ok(FetchResponse {
            id: candidate_id,
            title: page.frontmatter.title,
            path: relative_path.clone(),
            category: page_category(&page.frontmatter.category, &relative_path),
            body: page.body,
            source_uri,
            curio_uri: curio_uri(wiki_dir, &relative_path),
            content_hash,
            updated_at: page.frontmatter.updated_at,
            authority: "published",
            last_commit,
        });
    }

    Err(CliValidationError::new(
        "fetch_not_found",
        format!("No canonical published page exists for retrieval id {id}."),
        "Run curio retrieve --query <text> --json to discover valid ids, then retry with --id.",
    )
    .into())
}

fn curio_uri(wiki_dir: &Path, relative_path: &str) -> String {
    let workspace = workspace_name(wiki_dir);
    format!("curio://{workspace}/published/{relative_path}")
}

fn workspace_name(wiki_dir: &Path) -> String {
    let workspace_dir = if wiki_dir.file_name().and_then(|name| name.to_str()) == Some("wiki") {
        wiki_dir.parent()
    } else {
        Some(wiki_dir)
    };
    workspace_dir
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("workspace")
        .to_string()
}

fn canonical_published_paths(published_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    if published_dir.exists() {
        for entry in WalkDir::new(published_dir).follow_links(false) {
            let entry = entry.with_context(|| {
                format!(
                    "Failed to walk published pages under {}",
                    published_dir.display()
                )
            })?;
            if entry.file_type().is_file() && is_canonical_page(entry.path()) {
                paths.push(entry.into_path());
            }
        }
    }
    paths.sort_by_key(|path| published_relative_path(published_dir, path));
    Ok(paths)
}

fn local_retrieval_id(relative_path: &str) -> String {
    format!(
        "local:{}",
        wiki_fs::generate_id(&format!("published/{relative_path}"))
    )
}

fn validate_fetch_id(id: &str) -> Result<()> {
    let Some(raw) = id.strip_prefix("local:") else {
        return Err(CliValidationError::new(
            "invalid_fetch_id",
            "Fetch id must start with local: and match a retrieve result exactly.",
            "Use the id field returned by curio retrieve --json, for example --id local:0123456789abcdef.",
        )
        .into());
    };
    if raw.len() != 16
        || !raw
            .chars()
            .all(|ch| ch.is_ascii_hexdigit() && !ch.is_ascii_uppercase())
    {
        return Err(CliValidationError::new(
            "invalid_fetch_id",
            "Fetch id must match local:<16 lowercase hex characters>; path-like or uppercase ids are rejected.",
            "Use the id field returned by curio retrieve --json, for example --id local:0123456789abcdef.",
        )
        .into());
    }
    Ok(())
}

fn normalized_query_terms(query: &str) -> Result<Vec<String>> {
    let terms: BTreeSet<String> = token_set(query)
        .into_iter()
        .filter(|term| !is_stopword(term))
        .filter(|term| term.chars().count() >= 2)
        .collect();
    if terms.is_empty() {
        return Err(CliValidationError::new(
            "invalid_query",
            "Query must contain at least one meaningful search term; empty and stopword-only queries are rejected.",
            "Use a specific content term, for example --query deployment runbook.",
        )
        .into());
    }
    Ok(terms.into_iter().collect())
}

fn token_set(text: &str) -> BTreeSet<String> {
    let mut tokens = BTreeSet::new();
    let mut current = String::new();
    for ch in text.chars() {
        if ch.is_alphanumeric() {
            current.extend(ch.to_lowercase());
        } else if !current.is_empty() {
            tokens.insert(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        tokens.insert(current);
    }
    tokens
}

fn matching_terms(terms: &[String], haystack: &BTreeSet<String>) -> u32 {
    terms.iter().filter(|term| haystack.contains(*term)).count() as u32
}

fn is_stopword(term: &str) -> bool {
    matches!(
        term,
        "a" | "an"
            | "and"
            | "are"
            | "as"
            | "at"
            | "be"
            | "by"
            | "can"
            | "do"
            | "does"
            | "for"
            | "from"
            | "how"
            | "in"
            | "into"
            | "is"
            | "it"
            | "of"
            | "on"
            | "or"
            | "that"
            | "the"
            | "these"
            | "this"
            | "those"
            | "to"
            | "what"
            | "when"
            | "where"
            | "which"
            | "who"
            | "why"
            | "with"
            | "you"
            | "your"
    )
}

fn normalize_category(category: Option<&str>) -> Option<String> {
    category
        .map(|value| value.trim().trim_matches('/').to_ascii_lowercase())
        .filter(|value| !value.is_empty())
}

fn category_matches(category: &str, filter: &str) -> bool {
    let category = category.to_ascii_lowercase();
    category == filter || category.starts_with(&format!("{filter}/"))
}

fn page_category(frontmatter_category: &[String], relative_path: &str) -> String {
    if !frontmatter_category.is_empty() {
        return frontmatter_category.join("/");
    }
    Path::new(relative_path)
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .map(|parent| parent.to_string_lossy().replace('\\', "/"))
        .unwrap_or_default()
}

fn is_canonical_page(path: &Path) -> bool {
    let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    path.extension().and_then(|ext| ext.to_str()) == Some("md")
        && file_name != "index.md"
        && !file_name.starts_with('.')
        && !file_name.ends_with(".feedback.md")
        && !file_name.ends_with(".analysis.md")
        && !file_name.ends_with(".proposal.md")
}

fn published_relative_path(published_dir: &Path, path: &Path) -> String {
    path.strip_prefix(published_dir)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn query_excerpt(body: &str, summary: &str, terms: &[String]) -> String {
    let mut best: Option<(u32, usize, String)> = None;
    for (index, raw_line) in body.lines().enumerate() {
        let line = clean_markdown_line(raw_line);
        if line.is_empty() {
            continue;
        }
        let hits = matching_terms(terms, &token_set(&line));
        if hits == 0 {
            continue;
        }
        let replace = best
            .as_ref()
            .map(|(best_hits, best_index, _)| {
                hits > *best_hits || (hits == *best_hits && index < *best_index)
            })
            .unwrap_or(true);
        if replace {
            best = Some((hits, index, line));
        }
    }
    best.map(|(_, _, line)| truncate_chars(&line, EXCERPT_MAX_CHARS))
        .or_else(|| {
            let summary = clean_markdown_line(summary);
            (!summary.is_empty()).then(|| truncate_chars(&summary, EXCERPT_MAX_CHARS))
        })
        .or_else(|| {
            let first = wiki_fs::first_line_summary(body, EXCERPT_MAX_CHARS);
            (!first.is_empty()).then_some(first)
        })
        .unwrap_or_default()
}

fn clean_markdown_line(line: &str) -> String {
    line.trim()
        .trim_start_matches('>')
        .trim()
        .trim_start_matches('#')
        .trim()
        .trim_matches('*')
        .trim()
        .to_string()
}

fn truncate_chars(text: &str, max_chars: usize) -> String {
    let mut chars = text.chars();
    let truncated: String = chars.by_ref().take(max_chars).collect();
    if chars.next().is_some() {
        format!("{truncated}...")
    } else {
        truncated
    }
}

fn find_git_root(start: &Path) -> Option<PathBuf> {
    let mut current = start;
    loop {
        if current.join(".git").exists() {
            return Some(current.to_path_buf());
        }
        current = current.parent()?;
    }
}

fn git_last_commit(repo_root: &Path, path: &Path) -> Result<Option<LastCommit>> {
    let relative = path.strip_prefix(repo_root).with_context(|| {
        format!(
            "{} is outside Git root {}",
            path.display(),
            repo_root.display()
        )
    })?;
    let output = std::process::Command::new("git")
        .args(["log", "-1", "--format=%H%x1f%an%x1f%aI%x1f%s", "--"])
        .arg(relative)
        .current_dir(repo_root)
        .output()
        .context("Failed to query Git history")?;
    if !output.status.success() {
        return Ok(None);
    }
    let line = String::from_utf8_lossy(&output.stdout);
    let fields: Vec<_> = line.trim_end().split('\x1f').collect();
    if fields.len() != 4 || fields.iter().any(|field| field.is_empty()) {
        return Ok(None);
    }
    Ok(Some(LastCommit {
        hash: fields[0].to_string(),
        author: fields[1].to_string(),
        committed_at: fields[2].to_string(),
        subject: fields[3].to_string(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stopwords_are_rejected() {
        let error = normalized_query_terms("the and of").expect_err("query should fail");
        assert_eq!(
            error.downcast_ref::<CliValidationError>().unwrap().code,
            "invalid_query"
        );
    }

    #[test]
    fn path_is_stable_and_platform_normalized() {
        assert_eq!(
            published_relative_path(
                Path::new("/kb/published"),
                Path::new("/kb/published/a/b.md")
            ),
            "a/b.md"
        );
    }

    #[test]
    fn provenance_workspace_name_supports_nested_and_kb_root_layouts() {
        assert_eq!(
            curio_uri(Path::new("/stores/example/wiki"), "ops/runbook.md"),
            "curio://example/published/ops/runbook.md"
        );
        assert_eq!(
            curio_uri(Path::new("/stores/example"), "ops/runbook.md"),
            "curio://example/published/ops/runbook.md"
        );
    }

    #[test]
    fn category_filter_includes_descendants() {
        assert!(category_matches("product-tree/server", "product-tree"));
        assert!(!category_matches("topic-tree/server", "product-tree"));
    }

    #[test]
    fn fetch_id_validation_rejects_non_local_or_path_like_values() {
        for id in [
            "alpha.md",
            "local:../../etc/passwd",
            "local:ABCDEF0123456789",
            "local:abc123",
        ] {
            let error = validate_fetch_id(id).expect_err("id should fail");
            assert_eq!(
                error.downcast_ref::<CliValidationError>().unwrap().code,
                "invalid_fetch_id"
            );
        }
    }
}
