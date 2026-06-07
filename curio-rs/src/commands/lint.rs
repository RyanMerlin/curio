/// Lint command: scan the wiki for contradictions, stale claims, orphaned cross-refs.
///
/// The LLM agent drives the actual analysis. This command reads the wiki index
/// and relevant pages, then outputs a structured report the agent can act on.
use anyhow::Result;

use crate::{
    config::Config,
    output::emit_json,
    wiki_fs::parse_wiki_page,
    wiki_index::{append_log, load_registry},
};

pub async fn run_lint(config: &Config, _dry_run: bool, json: bool, fix: bool) -> Result<()> {
    let wiki_dir = &config.wiki.wiki_dir;
    let registry = load_registry(wiki_dir)?;

    let mut issues: Vec<serde_json::Value> = Vec::new();
    let mut fixed = 0usize;

    // ── Check 1: Orphaned cross-references ───────────────────────────────
    let all_paths: std::collections::HashSet<String> =
        registry.pages.iter().map(|e| e.path.clone()).collect();

    for entry in &registry.pages {
        let page_path = wiki_dir.join(&entry.path);
        if !page_path.exists() {
            issues.push(serde_json::json!({
                "type": "missing_file",
                "path": entry.path,
                "detail": "Registry entry exists but file is missing"
            }));
            continue;
        }

        if let Ok(page) = parse_wiki_page(&page_path) {
            for xref in &page.frontmatter.cross_refs {
                let xref_path = if xref.contains('/') {
                    xref.clone()
                } else {
                    // Try to find by filename
                    format!("published/{}", xref)
                };
                if !all_paths.contains(&xref_path) {
                    let issue = serde_json::json!({
                        "type": "orphaned_cross_ref",
                        "source": entry.path,
                        "broken_ref": xref,
                        "detail": "Cross-reference points to a non-existent page"
                    });

                    if fix {
                        // Remove the broken ref
                        let mut page = page.clone();
                        page.frontmatter.cross_refs.retain(|r| r != xref);
                        if let Err(e) =
                            crate::wiki_fs::update_frontmatter(&page_path, &page.frontmatter)
                        {
                            eprintln!("Warning: could not fix {}: {}", entry.path, e);
                        } else {
                            fixed += 1;
                        }
                    }
                    issues.push(issue);
                }
            }
        }
    }

    // ── Check 2: Pages with no keywords ──────────────────────────────────
    for entry in &registry.pages {
        if entry.keywords.is_empty() && entry.status == "published" {
            issues.push(serde_json::json!({
                "type": "missing_keywords",
                "path": entry.path,
                "detail": "Published page has no keywords — harder to discover"
            }));
        }
    }

    // ── Check 3: Low-confidence published pages ───────────────────────────
    for entry in &registry.pages {
        if entry.status == "published"
            && let Some(conf) = entry.confidence
            && conf < 0.5
        {
            issues.push(serde_json::json!({
                "type": "low_confidence",
                "path": entry.path,
                "confidence": conf,
                "detail": "Published page has low routing confidence — may be miscategorised"
            }));
        }
    }

    append_log(wiki_dir, &format!("lint: {} issues found", issues.len()))?;

    if fix && fixed > 0 {
        // Commit fixes
        if config.wiki.auto_commit {
            let repo_root = wiki_dir.parent().unwrap_or(wiki_dir);
            crate::git_ops::git_add(repo_root, wiki_dir)?;
            if crate::git_ops::git_has_staged(repo_root) {
                crate::git_ops::git_commit(
                    repo_root,
                    &format!("curio: lint --fix ({} cross-refs removed)", fixed),
                )?;
            }
        }
    }

    if json {
        let _ = emit_json(
            "lint",
            true,
            serde_json::json!({
                "issues": issues,
                "fixed": fixed,
                "total_pages": registry.pages.len(),
            }),
        );
    } else {
        if issues.is_empty() {
            println!("No issues found ({} pages checked)", registry.pages.len());
        } else {
            println!(
                "{} issue(s) found across {} pages:{}",
                issues.len(),
                registry.pages.len(),
                if fix {
                    format!(" {} fixed", fixed)
                } else {
                    String::new()
                }
            );
            for issue in &issues {
                let kind = issue["type"].as_str().unwrap_or("unknown");
                let path = issue["path"]
                    .as_str()
                    .or_else(|| issue["source"].as_str())
                    .unwrap_or("?");
                let detail = issue["detail"].as_str().unwrap_or("");
                println!("  [{:^20}] {} — {}", kind, path, detail);
            }
        }
    }
    Ok(())
}
