/// Resolve command: move a review/ page to staged/.
use anyhow::Result;
use chrono::Utc;
use std::path::PathBuf;

use crate::{
    PageStatus,
    config::Config,
    northstar::{load_taxonomy, taxonomy_path_exists},
    output::emit_json,
    proposal::{ProposalLane, load_proposal_record, save_proposal_record},
    wiki_fs::{parse_wiki_page, update_frontmatter},
    wiki_index::{append_log, rebuild_index_md},
};

pub async fn run_resolve(
    config: &Config,
    dry_run: bool,
    json: bool,
    slug: String,
    category: Option<String>,
) -> Result<()> {
    let wiki_dir = &config.wiki.wiki_dir;
    let review_dir = wiki_dir.join("review");

    let src_path = review_dir.join(format!("{}.md", slug));
    if !src_path.exists() {
        // Also check review subdirs
        let found = find_in_dir(&review_dir, &slug)?;
        let src_path =
            found.ok_or_else(|| anyhow::anyhow!("No review page found for slug: {}", slug))?;

        return do_resolve(config, &src_path, &slug, category, dry_run, json).await;
    }

    do_resolve(config, &src_path, &slug, category, dry_run, json).await
}

async fn do_resolve(
    config: &Config,
    src_path: &std::path::Path,
    slug: &str,
    category: Option<String>,
    dry_run: bool,
    json: bool,
) -> Result<()> {
    let wiki_dir = &config.wiki.wiki_dir;

    let mut page = parse_wiki_page(src_path)?;
    let taxonomy = load_taxonomy(wiki_dir)?;

    let cat_segments: Vec<String> = category
        .as_deref()
        .map(|c| c.split('/').map(|s| s.to_string()).collect())
        .unwrap_or_else(|| page.frontmatter.category.clone());
    if cat_segments.is_empty() {
        anyhow::bail!(
            "Cannot resolve '{}' to staged without a category. Keep it in review and attach a subtree proposal instead.",
            slug
        );
    }
    if !taxonomy_path_exists(&taxonomy, &cat_segments) {
        anyhow::bail!(
            "Cannot resolve '{}' to staged with invalid taxonomy path '{}'. Keep it in review and attach an approved taxonomy change proposal instead.",
            slug,
            cat_segments.join("/")
        );
    }
    if let Some(mut proposal) = load_proposal_record(src_path)? {
        proposal.target_path = cat_segments.clone();
        proposal.lane = ProposalLane::Staged;
        proposal.review_reason = None;
        save_proposal_record(src_path, &proposal)?;
    }

    let cat_path: PathBuf = cat_segments.iter().collect();
    let filename = format!("{}.md", slug);
    let dest_dir = wiki_dir.join("staged").join(&cat_path);
    let dest_path = dest_dir.join(&filename);

    if dry_run {
        let msg = format!(
            "Would move {} → staged/{}/{}",
            slug,
            cat_path.display(),
            filename
        );
        if json {
            let _ = emit_json(
                "resolve",
                true,
                serde_json::json!({ "slug": slug, "would_move_to": dest_path, "dry_run": true }),
            );
        } else {
            println!("{}", msg);
        }
        return Ok(());
    }

    // Update frontmatter
    let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    page.frontmatter.status = PageStatus::Staged;
    page.frontmatter.category = cat_segments;
    page.frontmatter.updated_at = now;
    update_frontmatter(src_path, &page.frontmatter)?;

    // git mv
    let repo_root = if wiki_dir.join(".git").exists() {
        wiki_dir
    } else {
        wiki_dir.parent().unwrap_or(wiki_dir)
    };
    let rel_src = src_path.strip_prefix(repo_root).unwrap_or(src_path);
    let rel_dest = dest_path.strip_prefix(repo_root).unwrap_or(&dest_path);
    crate::git_ops::git_mv(repo_root, rel_src, rel_dest)?;
    let proposal_src = crate::proposal::proposal_sidecar_path(src_path);
    if proposal_src.exists() {
        let proposal_dest = crate::proposal::proposal_sidecar_path(&dest_path);
        let rel_psrc = proposal_src
            .strip_prefix(repo_root)
            .unwrap_or(&proposal_src);
        let rel_pdest = proposal_dest
            .strip_prefix(repo_root)
            .unwrap_or(&proposal_dest);
        crate::git_ops::git_mv(repo_root, rel_psrc, rel_pdest)?;
    }

    let new_rel = dest_path
        .strip_prefix(wiki_dir)
        .unwrap_or(&dest_path)
        .to_string_lossy()
        .replace('\\', "/");

    rebuild_index_md(wiki_dir, &crate::WikiIndex::default())?;
    append_log(wiki_dir, &format!("resolve: {} moved to staged", slug))?;

    if config.wiki.auto_commit {
        crate::git_ops::git_add(repo_root, wiki_dir)?;
        if crate::git_ops::git_has_staged(repo_root) {
            crate::git_ops::git_commit(repo_root, &format!("curio: resolve {}", slug))?;
        }
    }

    if json {
        let _ = emit_json(
            "resolve",
            true,
            serde_json::json!({ "slug": slug, "moved_to": new_rel }),
        );
    } else {
        println!("Resolved: {} → staged/{}", slug, new_rel);
    }
    Ok(())
}

fn find_in_dir(dir: &std::path::Path, slug: &str) -> Result<Option<std::path::PathBuf>> {
    if !dir.exists() {
        return Ok(None);
    }
    for entry in walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "md")
            && path.file_stem().is_some_and(|s| s == slug)
        {
            return Ok(Some(path.to_path_buf()));
        }
    }
    Ok(None)
}
