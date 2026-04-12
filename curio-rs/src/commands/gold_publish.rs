/// Publish command: move a staged/ page to published/.
use anyhow::Result;
use chrono::Utc;
use std::path::PathBuf;

use crate::{
    config::Config,
    output::emit_json,
    wiki_fs::{parse_wiki_page, update_frontmatter},
    wiki_index::{
        append_log, load_registry, rebuild_index_md, save_registry,
        update_entry_path, update_entry_status,
    },
    PageStatus,
};

pub async fn run_publish(
    config: &Config,
    dry_run: bool,
    json: bool,
    slug: String,
    category: Option<String>,
) -> Result<()> {
    let wiki_dir = &config.wiki.wiki_dir;
    let staged_dir = wiki_dir.join("staged");

    // Find the staged file
    let src_path = {
        let direct = staged_dir.join(format!("{}.md", slug));
        if direct.exists() {
            direct
        } else {
            find_in_dir(&staged_dir, &slug)?
                .ok_or_else(|| anyhow::anyhow!("No staged page found for slug: {}", slug))?
        }
    };

    let mut page = parse_wiki_page(&src_path)?;

    // Empty/"-" category means top-level published/ (no subdirectory).
    let cat_segments: Vec<String> = category
        .as_deref()
        .map(|c| {
            if c.is_empty() || c == "-" || c == "." {
                vec![]
            } else {
                c.split('/').filter(|s| !s.is_empty()).map(|s| s.to_string()).collect()
            }
        })
        .unwrap_or_else(|| {
            if page.frontmatter.category.is_empty() {
                vec!["by-topic".to_string()]
            } else {
                page.frontmatter.category.clone()
            }
        });

    let cat_path: PathBuf = cat_segments.iter().collect();
    let filename = format!("{}.md", slug);
    let dest_dir = wiki_dir.join("published").join(&cat_path);
    let dest_path = dest_dir.join(&filename);

    if dry_run {
        let msg = format!(
            "Would publish {} → published/{}",
            slug,
            dest_path.strip_prefix(wiki_dir).unwrap_or(&dest_path).display()
        );
        if json {
            let _ = emit_json("publish", true, &serde_json::json!({ "slug": slug, "would_publish_to": dest_path, "dry_run": true }));
        } else {
            println!("{}", msg);
        }
        return Ok(());
    }

    // Update frontmatter
    let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    page.frontmatter.status = PageStatus::Published;
    page.frontmatter.category = cat_segments;
    page.frontmatter.updated_at = now;
    update_frontmatter(&src_path, &page.frontmatter)?;

    // git mv staged/{...}/{slug}.md → published/{cat}/{slug}.md
    let repo_root = wiki_dir.parent().unwrap_or(wiki_dir);
    let rel_src = src_path.strip_prefix(repo_root).unwrap_or(&src_path);
    let rel_dest = dest_path.strip_prefix(repo_root).unwrap_or(&dest_path);
    crate::git_ops::git_mv(repo_root, rel_src, rel_dest)?;

    // Update registry
    let new_rel = dest_path
        .strip_prefix(wiki_dir)
        .unwrap_or(&dest_path)
        .to_string_lossy()
        .replace('\\', "/");
    update_entry_path(wiki_dir, &page.frontmatter.id, &new_rel)?;
    update_entry_status(wiki_dir, &page.frontmatter.id, &PageStatus::Published)?;

    // Update cross-refs in other published pages (best-effort)
    update_cross_refs(wiki_dir, &slug, &new_rel)?;

    let registry = load_registry(wiki_dir)?;
    save_registry(wiki_dir, &registry)?;
    rebuild_index_md(wiki_dir, &registry)?;
    append_log(wiki_dir, &format!("publish: {} published to {}", slug, new_rel))?;

    if config.wiki.auto_commit {
        crate::git_ops::git_add(repo_root, wiki_dir)?;
        if crate::git_ops::git_has_staged(repo_root) {
            crate::git_ops::git_commit(repo_root, &format!("curio: publish {}", slug))?;
        }
    }

    if json {
        let _ = emit_json("publish", true, &serde_json::json!({ "slug": slug, "published_to": new_rel }));
    } else {
        println!("Published: {} → {}", slug, new_rel);
    }
    Ok(())
}

/// Best-effort: scan published/ pages and fix cross_refs that reference the slug without the full path.
fn update_cross_refs(wiki_dir: &std::path::Path, slug: &str, new_path: &str) -> Result<()> {
    let published_dir = wiki_dir.join("published");
    if !published_dir.exists() {
        return Ok(());
    }
    for entry in walkdir::WalkDir::new(&published_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "md"))
    {
        let path = entry.path();
        if let Ok(mut page) = crate::wiki_fs::parse_wiki_page(path) {
            let old_ref = format!("{}.md", slug);
            if page.frontmatter.cross_refs.iter().any(|r| r == &old_ref) {
                page.frontmatter
                    .cross_refs
                    .iter_mut()
                    .filter(|r| r.as_str() == old_ref)
                    .for_each(|r| *r = new_path.to_string());
                let _ = crate::wiki_fs::update_frontmatter(path, &page.frontmatter);
            }
        }
    }
    Ok(())
}

fn find_in_dir(dir: &std::path::Path, slug: &str) -> Result<Option<std::path::PathBuf>> {
    if !dir.exists() {
        return Ok(None);
    }
    for entry in walkdir::WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.extension().map_or(false, |e| e == "md") {
            if path.file_stem().map_or(false, |s| s == slug) {
                return Ok(Some(path.to_path_buf()));
            }
        }
    }
    Ok(None)
}
