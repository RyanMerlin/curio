use anyhow::Result;

use crate::{config::Config, output::emit_json, wiki_index::load_registry};

#[allow(clippy::too_many_arguments)]
pub async fn run_search(
    config: &Config,
    _dry_run: bool,
    json: bool,
    keywords: Option<String>,
    category: Option<String>,
    status: Option<String>,
    text: Option<String>,
    limit: u32,
) -> Result<()> {
    let wiki_dir = &config.wiki.wiki_dir;
    let registry = load_registry(wiki_dir)?;

    let kw_terms: Vec<String> = keywords
        .as_deref()
        .unwrap_or("")
        .split(',')
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect();

    let cat_filter = category.as_deref().map(|s| s.to_lowercase());
    let status_filter = status.as_deref();
    let text_lower = text.as_deref().map(|s| s.to_lowercase());

    let mut results: Vec<_> = registry
        .pages
        .iter()
        .filter(|e| {
            // Status filter
            if let Some(sf) = status_filter
                && e.status != sf
            {
                return false;
            }
            // Category filter
            if let Some(ref cf) = cat_filter {
                let cat_str = e.category.join("/").to_lowercase();
                if !cat_str.contains(cf.as_str()) {
                    return false;
                }
            }
            // Keyword filter (any keyword matches in title/keywords/summary)
            if !kw_terms.is_empty() {
                let haystack = format!(
                    "{} {} {}",
                    e.title.to_lowercase(),
                    e.keywords.join(" ").to_lowercase(),
                    e.summary.to_lowercase()
                );
                let matches_any = kw_terms.iter().any(|kw| haystack.contains(kw.as_str()));
                if !matches_any {
                    return false;
                }
            }
            // Text filter (search in summary/title; full-body grep handled separately below)
            if let Some(ref tl) = text_lower {
                let haystack = format!("{} {}", e.title.to_lowercase(), e.summary.to_lowercase());
                if !haystack.contains(tl.as_str()) {
                    return false;
                }
            }
            true
        })
        .take(limit as usize)
        .collect();

    // If text search and not many registry results, also try grep on file bodies
    if let Some(ref txt) = text
        && results.len() < limit as usize
    {
        let extra = grep_wiki_bodies(wiki_dir, txt, limit as usize - results.len())?;
        let existing_paths: std::collections::HashSet<_> =
            results.iter().map(|e| e.path.as_str()).collect();
        for path in extra {
            if !existing_paths.contains(path.as_str()) {
                // Find the registry entry for this path
                if let Some(e) = registry.pages.iter().find(|e| e.path == path) {
                    results.push(e);
                }
            }
        }
    }

    if json {
        let _ = emit_json(
            "search",
            true,
            serde_json::json!({ "results": results, "count": results.len() }),
        );
    } else {
        if results.is_empty() {
            println!("No results found");
        } else {
            for r in &results {
                println!("[{}] {} — {}", r.status, r.title, r.path);
                if !r.summary.is_empty() {
                    println!("    {}", r.summary);
                }
            }
        }
    }
    Ok(())
}

fn grep_wiki_bodies(wiki_dir: &std::path::Path, text: &str, limit: usize) -> Result<Vec<String>> {
    let output = std::process::Command::new("rg")
        .args([
            "--files-with-matches",
            "--glob",
            "**/*.md",
            "--no-heading",
            text,
            wiki_dir.to_str().unwrap_or("."),
        ])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let lines = String::from_utf8_lossy(&out.stdout);
            let paths: Vec<String> = lines
                .lines()
                .take(limit)
                .filter_map(|l| {
                    std::path::Path::new(l)
                        .strip_prefix(wiki_dir)
                        .ok()
                        .map(|rel| rel.to_string_lossy().replace('\\', "/"))
                })
                .collect();
            Ok(paths)
        }
        _ => Ok(vec![]), // rg not available or no matches — silently skip
    }
}
