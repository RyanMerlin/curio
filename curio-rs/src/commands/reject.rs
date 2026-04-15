/// `curio reject <slug>` — locally reject a wiki page without requiring a Confluence signal.
///
/// Useful for pages that failed to sync (no .sync-refs.json) or that are clearly wrong
/// and should never reach the feedback→approve loop.  Works on any lane: review, staged,
/// intake, or published (published requires --force).
///
/// Deletes:
///   <slug>.md
///   <slug>.analysis.json       (if present)
///   <slug>.sync-refs.json      (if present)
///   <slug>.feedback.md         (if present)
///   <slug>.md.proposal.json    (if present)
///
/// Logs the rejection to wiki/_config/log.md.
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::{config::Config, wiki_fs::parse_wiki_page, wiki_index::append_log};

pub async fn run_reject(
    config: &Config,
    dry_run: bool,
    slug_or_path: String,
    reason: Option<String>,
    force: bool,
) -> Result<()> {
    let wiki_dir = &config.wiki.wiki_dir;

    // Resolve to an absolute path.  Accept either:
    //   - a bare slug ("orasi-labs")
    //   - a relative path from wiki root ("review/product-tree/alteryx-server/orasi-labs")
    //   - an absolute path
    let md_path = resolve_path(wiki_dir, &slug_or_path)?;

    // Safety: warn loudly about published pages.
    if md_path.starts_with(wiki_dir.join("published")) && !force {
        anyhow::bail!(
            "Refusing to reject a published page without --force.\n  Path: {}\n  Use --force if you are certain.",
            md_path.display()
        );
    }

    let page = parse_wiki_page(&md_path)
        .with_context(|| format!("Failed to parse {}", md_path.display()))?;

    let lane = detect_lane(wiki_dir, &md_path);
    let reason_str = reason.as_deref().unwrap_or("manually rejected");

    if dry_run {
        println!(
            "[dry-run] REJECT  {} ({}) — {}",
            page.frontmatter.title, lane, reason_str
        );
        println!("[dry-run]   path: {}", md_path.display());
        return Ok(());
    }

    // Delete the page and all sidecars.
    std::fs::remove_file(&md_path)
        .with_context(|| format!("Failed to remove {}", md_path.display()))?;

    let sidecars = [
        md_path.with_extension("analysis.json"),
        md_path.with_extension("sync-refs.json"),
        md_path.with_extension("feedback.md"),
        // .md.proposal.json sits alongside as <slug>.md.proposal.json
        {
            let mut p = md_path.clone();
            let fname = p.file_name().unwrap().to_string_lossy().to_string();
            p.set_file_name(format!("{}.proposal.json", fname));
            p
        },
    ];
    for sidecar in &sidecars {
        if sidecar.exists() {
            std::fs::remove_file(sidecar).ok();
        }
    }

    let timestamp = chrono::Utc::now().to_rfc3339();
    let log_entry = format!(
        "[{}] reject {} ({}) — {}",
        timestamp, page.frontmatter.title, lane, reason_str
    );
    append_log(wiki_dir, &log_entry)?;

    println!("REJECT  {} ({})", page.frontmatter.title, lane);
    println!("  reason : {}", reason_str);
    println!("  removed: {}", md_path.display());

    Ok(())
}

/// Walk all lanes to find the page by slug or path.
fn resolve_path(wiki_dir: &Path, slug_or_path: &str) -> Result<PathBuf> {
    // Absolute path — use directly.
    let p = Path::new(slug_or_path);
    if p.is_absolute() {
        if p.exists() && p.extension().map_or(false, |e| e == "md") {
            return Ok(p.to_path_buf());
        }
        anyhow::bail!("Path not found or not a .md file: {}", p.display());
    }

    // Relative path that looks like a wiki-relative path (contains a slash).
    if slug_or_path.contains('/') || slug_or_path.contains('\\') {
        // Try relative to wiki_dir, then relative to cwd.
        let from_wiki = wiki_dir.join(slug_or_path);
        let with_ext = if from_wiki.extension().is_some() {
            from_wiki.clone()
        } else {
            from_wiki.with_extension("md")
        };
        if with_ext.exists() {
            return Ok(with_ext);
        }
        let from_cwd = PathBuf::from(slug_or_path);
        let with_ext_cwd = if from_cwd.extension().is_some() {
            from_cwd.clone()
        } else {
            from_cwd.with_extension("md")
        };
        if with_ext_cwd.exists() {
            return Ok(with_ext_cwd);
        }
        anyhow::bail!("Could not resolve path: {}", slug_or_path);
    }

    // Bare slug — search all lanes.
    let slug = slug_or_path.trim_end_matches(".md");
    for lane in &["review", "staged", "intake", "published"] {
        let lane_dir = wiki_dir.join(lane);
        if let Some(found) = find_by_slug(&lane_dir, slug)? {
            return Ok(found);
        }
    }
    anyhow::bail!("No wiki page found for slug: {}", slug);
}

fn find_by_slug(dir: &Path, slug: &str) -> Result<Option<PathBuf>> {
    if !dir.exists() {
        return Ok(None);
    }
    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.extension().map_or(false, |e| e == "md") {
            if path.file_stem().map_or(false, |s| s == slug) {
                return Ok(Some(path.to_path_buf()));
            }
        }
    }
    Ok(None)
}

fn detect_lane(wiki_dir: &Path, md_path: &Path) -> &'static str {
    let rel = md_path.strip_prefix(wiki_dir).unwrap_or(md_path);
    let first = rel
        .components()
        .next()
        .and_then(|c| c.as_os_str().to_str())
        .unwrap_or("");
    match first {
        "review" => "review",
        "staged" => "staged",
        "intake" => "intake",
        "published" => "published",
        _ => "unknown",
    }
}
