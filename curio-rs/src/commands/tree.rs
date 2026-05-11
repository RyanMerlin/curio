use anyhow::Result;

use crate::{config::Config, git_ops, northstar::load_taxonomy, output::emit_json};

/// Sync the `wiki/published/` directory tree to match the NORTHSTAR blueprint.
///
/// - Creates dirs for trees and subtrees that exist in NORTHSTAR but not on disk.
/// - Removes dirs that are in `published/` but no longer in NORTHSTAR, **only if empty**.
/// - Leaves any dir containing content untouched (safe — content must be moved manually).
/// - Auto-commits if `wiki.auto_commit` is enabled.
pub async fn run_tree(config: &Config, dry_run: bool, json: bool) -> Result<()> {
    let wiki_dir = &config.wiki.wiki_dir;
    let config_path = crate::northstar::workspace_config_path(wiki_dir);

    if !config_path.exists() {
        anyhow::bail!(
            "No workspace config found at {}. Run `curio init` first.",
            config_path.display()
        );
    }

    let taxonomy = load_taxonomy(wiki_dir)?;
    let trees = taxonomy.nodes;

    if trees.is_empty() {
        if !json {
            println!("No published tree nodes found in _admin/config.yaml — nothing to do.");
            println!("Add taxonomy nodes to _admin/config.yaml and rerun `curio tree`.");
        }
        return Ok(());
    }

    let published_dir = wiki_dir.join("published");
    let mut created: Vec<String> = Vec::new();
    let mut removed: Vec<String> = Vec::new();
    let mut skipped: Vec<String> = Vec::new();

    // Build expected set: all slugs that should exist
    let mut expected: std::collections::HashSet<String> = std::collections::HashSet::new();
    collect_expected_paths(&trees, &mut expected, Vec::new());

    // Create missing dirs
    create_expected_dirs(&published_dir, &trees, &mut created, dry_run, Vec::new())?;

    // Remove dirs that are no longer in NORTHSTAR, but only if they are empty
    if published_dir.exists() {
        for entry in std::fs::read_dir(&published_dir)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let slug = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            if slug.starts_with('_') {
                continue;
            } // skip hidden workspace folders

            if !expected.contains(&slug) {
                let is_empty = std::fs::read_dir(&path)?.next().is_none();
                if is_empty {
                    if dry_run {
                        removed.push(format!("published/{} (empty, would remove)", slug));
                    } else {
                        std::fs::remove_dir(&path)?;
                        removed.push(format!("published/{}", slug));
                    }
                } else {
                    skipped.push(format!(
                        "published/{} (not in NORTHSTAR but has content — move manually)",
                        slug
                    ));
                }
            }
        }
    }
    prune_unexpected_dirs(
        &published_dir,
        &expected,
        dry_run,
        &mut removed,
        &mut skipped,
        Vec::new(),
    )?;

    // Auto-commit
    if !dry_run && config.wiki.auto_commit && (!created.is_empty() || !removed.is_empty()) {
        let msg = format!(
            "tree: sync published/ dirs from config.yaml ({} created, {} removed)",
            created.len(),
            removed.len()
        );
        let _ = git_ops::stage_and_commit(wiki_dir, &[wiki_dir.as_path()], &msg, true);
    }

    if json {
        let _ = emit_json(
            "tree",
            true,
            &serde_json::json!({ "created": created, "removed": removed, "skipped": skipped }),
        );
    } else {
        if created.is_empty() && removed.is_empty() && skipped.is_empty() {
            println!("Tree is already in sync with _admin/config.yaml.");
        }
        for s in &created {
            println!("  + {}", s);
        }
        for s in &removed {
            println!("  - {}", s);
        }
        for s in &skipped {
            println!("  ~ {}", s);
        }
        if dry_run && (!created.is_empty() || !removed.is_empty()) {
            println!("(dry-run — no changes made)");
        }
    }

    Ok(())
}

fn collect_expected_paths(
    nodes: &[crate::northstar::TaxonomyNode],
    expected: &mut std::collections::HashSet<String>,
    prefix: Vec<String>,
) {
    for node in nodes {
        let mut current = prefix.clone();
        current.push(node.slug.clone());
        expected.insert(current.join("/"));
        collect_expected_paths(&node.children, expected, current);
    }
}

fn create_expected_dirs(
    root: &std::path::Path,
    nodes: &[crate::northstar::TaxonomyNode],
    created: &mut Vec<String>,
    dry_run: bool,
    prefix: Vec<String>,
) -> Result<()> {
    for node in nodes {
        let mut current = prefix.clone();
        current.push(node.slug.clone());
        let rel = current.join("/");
        let dir = root.join(current.iter().collect::<std::path::PathBuf>());
        if !dir.exists() {
            if dry_run {
                created.push(format!("published/{}", rel));
            } else {
                std::fs::create_dir_all(&dir)?;
                created.push(format!("published/{}", rel));
            }
        }
        create_expected_dirs(root, &node.children, created, dry_run, current)?;
    }
    Ok(())
}

fn prune_unexpected_dirs(
    root: &std::path::Path,
    expected: &std::collections::HashSet<String>,
    dry_run: bool,
    removed: &mut Vec<String>,
    skipped: &mut Vec<String>,
    prefix: Vec<String>,
) -> Result<()> {
    if !root.exists() {
        return Ok(());
    }
    for entry in std::fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let slug = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        if slug.starts_with('_') {
            continue;
        }
        let mut current = prefix.clone();
        current.push(slug.clone());
        let key = current.join("/");
        prune_unexpected_dirs(&path, expected, dry_run, removed, skipped, current.clone())?;
        if !expected.contains(&key) {
            let is_empty = std::fs::read_dir(&path)?.next().is_none();
            if is_empty {
                if dry_run {
                    removed.push(format!("published/{} (empty, would remove)", key));
                } else {
                    std::fs::remove_dir(&path)?;
                    removed.push(format!("published/{}", key));
                }
            } else {
                skipped.push(format!(
                    "published/{} (not in taxonomy but has content — move manually)",
                    key
                ));
            }
        }
    }
    Ok(())
}
