use anyhow::Result;

use crate::{commands::sync::parse_northstar_blueprint, config::Config, git_ops, output::emit_json};

/// Sync the `wiki/published/` directory tree to match the NORTHSTAR blueprint.
///
/// - Creates dirs for trees and subtrees that exist in NORTHSTAR but not on disk.
/// - Removes dirs that are in `published/` but no longer in NORTHSTAR, **only if empty**.
/// - Leaves any dir containing content untouched (safe — content must be moved manually).
/// - Auto-commits if `wiki.auto_commit` is enabled.
pub async fn run_tree(config: &Config, dry_run: bool, json: bool) -> Result<()> {
    let wiki_dir = &config.wiki.wiki_dir;
    let northstar_path = wiki_dir.join("_config").join("northstar.md");

    if !northstar_path.exists() {
        anyhow::bail!(
            "No NORTHSTAR found at {}. Run `curio init` first.",
            northstar_path.display()
        );
    }

    let ns_md = std::fs::read_to_string(&northstar_path)?;
    let trees = parse_northstar_blueprint(&ns_md);

    if trees.is_empty() {
        if !json {
            println!("No Published Tree Blueprint found in NORTHSTAR — nothing to do.");
            println!("Add `## Published Tree Blueprint` with `###` tree headings to NORTHSTAR.");
        }
        return Ok(());
    }

    let published_dir = wiki_dir.join("published");
    let mut created: Vec<String> = Vec::new();
    let mut removed: Vec<String> = Vec::new();
    let mut skipped: Vec<String> = Vec::new();

    // Build expected set: all slugs that should exist
    let mut expected: std::collections::HashSet<String> = std::collections::HashSet::new();
    for tree in &trees {
        expected.insert(tree.slug.clone());
        for sub in &tree.subtrees {
            expected.insert(format!("{}/{}", tree.slug, sub.slug));
        }
    }

    // Create missing dirs
    for tree in &trees {
        let tree_dir = published_dir.join(&tree.slug);
        if !tree_dir.exists() {
            if dry_run {
                created.push(format!("published/{}", tree.slug));
            } else {
                std::fs::create_dir_all(&tree_dir)?;
                created.push(format!("published/{}", tree.slug));
            }
        }
        for sub in &tree.subtrees {
            let sub_dir = tree_dir.join(&sub.slug);
            if !sub_dir.exists() {
                if dry_run {
                    created.push(format!("published/{}/{}", tree.slug, sub.slug));
                } else {
                    std::fs::create_dir_all(&sub_dir)?;
                    created.push(format!("published/{}/{}", tree.slug, sub.slug));
                }
            }
        }
    }

    // Remove dirs that are no longer in NORTHSTAR, but only if they are empty
    if published_dir.exists() {
        for entry in std::fs::read_dir(&published_dir)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_dir() { continue; }
            let slug = path.file_name().unwrap_or_default().to_string_lossy().to_string();
            if slug.starts_with('_') { continue; } // skip hidden workspace folders

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
            } else {
                // Tree exists — check subtrees within it
                for sub_entry in std::fs::read_dir(&path)? {
                    let sub_entry = sub_entry?;
                    let sub_path = sub_entry.path();
                    if !sub_path.is_dir() { continue; }
                    let sub_slug = sub_path.file_name().unwrap_or_default().to_string_lossy().to_string();
                    let full_key = format!("{}/{}", slug, sub_slug);
                    if !expected.contains(&full_key) {
                        let is_empty = std::fs::read_dir(&sub_path)?.next().is_none();
                        if is_empty {
                            if dry_run {
                                removed.push(format!("published/{} (empty, would remove)", full_key));
                            } else {
                                std::fs::remove_dir(&sub_path)?;
                                removed.push(format!("published/{}", full_key));
                            }
                        } else {
                            skipped.push(format!(
                                "published/{} (not in NORTHSTAR but has content — move manually)",
                                full_key
                            ));
                        }
                    }
                }
            }
        }
    }

    // Also sync wiki/_config/northstar.md ← NORTHSTAR.md if repo root copy is newer
    let repo_northstar = wiki_dir.parent().map(|p| p.join("NORTHSTAR.md")).filter(|p| p.exists());
    if let Some(src) = repo_northstar {
        let src_mtime = src.metadata().and_then(|m| m.modified()).ok();
        let dst_mtime = northstar_path.metadata().and_then(|m| m.modified()).ok();
        if src_mtime > dst_mtime {
            if !dry_run {
                std::fs::copy(&src, &northstar_path)?;
            }
            created.push("_config/northstar.md (refreshed from NORTHSTAR.md)".to_string());
        }
    }

    // Auto-commit
    if !dry_run && config.wiki.auto_commit && (!created.is_empty() || !removed.is_empty()) {
        let msg = format!(
            "tree: sync published/ dirs from NORTHSTAR ({} created, {} removed)",
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
            println!("Tree is already in sync with NORTHSTAR.");
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
