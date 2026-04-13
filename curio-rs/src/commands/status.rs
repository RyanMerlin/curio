/// Pipeline status command.
///
/// Shows:
///  - Page counts per pipeline stage (intake / staged / review / published)
///  - Last sync timestamp (from `wiki/.curio/audit.jsonl`)
///  - Index freshness warning when published pages are newer than published/index.md
use anyhow::Result;
use std::path::Path;
use walkdir::WalkDir;

use crate::{audit_store, config::Config, output::emit_json};

pub async fn run_status(config: &Config, json: bool) -> Result<()> {
    let wiki_dir = &config.wiki.wiki_dir;

    if !wiki_dir.exists() {
        if json {
            let _ = emit_json("status", false, &serde_json::json!({ "error": "wiki not initialised" }));
        } else {
            eprintln!("Wiki not initialised. Run `curio init` first.");
        }
        return Ok(());
    }

    let intake   = count_md(wiki_dir, "intake");
    let staged   = count_md_recursive(wiki_dir, "staged");
    let review   = count_md_recursive(wiki_dir, "review");
    let published = count_md_content(wiki_dir, "published"); // excludes index.md

    let last_sync   = read_last_sync(wiki_dir);
    let stale       = is_index_stale(wiki_dir);
    let stale_hint  = if stale {
        Some("Index may be stale — run `curio reindex` to rebuild")
    } else {
        None
    };

    if json {
        let _ = emit_json("status", true, &serde_json::json!({
            "intake": intake,
            "staged": staged,
            "review": review,
            "published": published,
            "last_sync": last_sync,
            "index_stale": stale,
        }));
        return Ok(());
    }

    println!();
    println!("  Curio Pipeline");
    println!("  ──────────────────────────────────");
    println!("  intake      {:>4}  (wiki/intake/)", intake);
    println!("  staged      {:>4}  (wiki/staged/)", staged);
    println!("  review      {:>4}  (wiki/review/)", review);
    println!("  published   {:>4}  (wiki/published/)", published);
    println!();

    match &last_sync {
        Some(ts) => println!("  last sync   {}", ts),
        None      => println!("  last sync   never"),
    }

    if let Some(hint) = stale_hint {
        println!();
        println!("  ⚠  {}", hint);
    }

    println!();
    Ok(())
}

// ─── Helpers ─────────────────────────────────────────────────────────────

fn count_md(wiki_dir: &Path, subdir: &str) -> usize {
    let dir = wiki_dir.join(subdir);
    if !dir.exists() { return 0; }
    std::fs::read_dir(&dir)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |x| x == "md"))
        .count()
}

fn count_md_recursive(wiki_dir: &Path, subdir: &str) -> usize {
    let dir = wiki_dir.join(subdir);
    if !dir.exists() { return 0; }
    WalkDir::new(&dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_file()
                && e.path().extension().map_or(false, |x| x == "md")
        })
        .count()
}

/// Count content .md files in published/, excluding co-located index.md files.
fn count_md_content(wiki_dir: &Path, subdir: &str) -> usize {
    let dir = wiki_dir.join(subdir);
    if !dir.exists() { return 0; }
    WalkDir::new(&dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_file()
                && e.path().extension().map_or(false, |x| x == "md")
                && e.file_name() != "index.md"
        })
        .count()
}

/// Read the last sync timestamp from the Git-tracked audit log.
fn read_last_sync(wiki_dir: &Path) -> Option<String> {
    audit_store::read_last_sync(wiki_dir).ok().flatten()
}

/// Returns true if any published .md file is newer than `published/index.md`.
/// A simple staleness signal — doesn't guarantee full accuracy.
fn is_index_stale(wiki_dir: &Path) -> bool {
    let index_md = wiki_dir.join("published/index.md");
    let index_mtime = match std::fs::metadata(&index_md)
        .and_then(|m| m.modified())
    {
        Ok(t) => t,
        Err(_) => return false, // no index yet → not stale (not initialised)
    };

    let published_dir = wiki_dir.join("published");
    if !published_dir.exists() { return false; }

    WalkDir::new(&published_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_file()
                && e.path().extension().map_or(false, |x| x == "md")
                && e.file_name() != "index.md"
        })
        .any(|e| {
            std::fs::metadata(e.path())
                .and_then(|m| m.modified())
                .map(|mtime| mtime > index_mtime)
                .unwrap_or(false)
        })
}
