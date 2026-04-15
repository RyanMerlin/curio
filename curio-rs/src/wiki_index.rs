use crate::{Frontmatter, PageStatus, WikiIndex, WikiIndexEntry, WikiPage, audit_store};
use anyhow::{Context, Result};
use chrono::Utc;
use std::collections::BTreeMap;
use std::path::Path;
use walkdir::WalkDir;

// ─── Registry (registry.json) ────────────────────────────────────────────

/// Load the wiki catalog from disk by walking the wiki tree.
pub fn load_registry(wiki_dir: &Path) -> Result<WikiIndex> {
    reindex_from_filesystem(wiki_dir)
}

/// Save the wiki catalog to disk.
///
/// The catalog is filesystem-derived, so this is a compatibility no-op.
pub fn save_registry(wiki_dir: &Path, index: &WikiIndex) -> Result<()> {
    let _ = (wiki_dir, index);
    Ok(())
}

/// Upsert a single entry in the registry (matched by `id`).
pub fn upsert_registry_entry(wiki_dir: &Path, entry: WikiIndexEntry) -> Result<()> {
    let mut index = load_registry(wiki_dir)?;
    if let Some(existing) = index.pages.iter_mut().find(|e| e.id == entry.id) {
        *existing = entry;
    } else {
        index.pages.push(entry);
    }
    save_registry(wiki_dir, &index)
}

/// Remove a registry entry by id.
pub fn remove_registry_entry(wiki_dir: &Path, id: &str) -> Result<()> {
    let mut index = load_registry(wiki_dir)?;
    index.pages.retain(|e| e.id != id);
    save_registry(wiki_dir, &index)
}

// ─── Reindex from filesystem ─────────────────────────────────────────────

/// Walk `wiki_dir/**/*.md`, parse frontmatter, and rebuild `WikiIndex`.
/// Skips `_config/` and generated `index.md` files.
pub fn reindex_from_filesystem(wiki_dir: &Path) -> Result<WikiIndex> {
    let mut pages = Vec::new();

    for entry in WalkDir::new(wiki_dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_file() && e.path().extension().map_or(false, |ext| ext == "md")
        })
    {
        let abs = entry.path();
        let rel = abs
            .strip_prefix(wiki_dir)
            .expect("walkdir entry is under wiki_dir");

        // Skip system directories
        let first = rel
            .components()
            .next()
            .map(|c| c.as_os_str().to_string_lossy().into_owned())
            .unwrap_or_default();
        if first == "_config" {
            continue;
        }

        // Skip co-located index.md files — they are generated artifacts, not content pages
        if rel.file_name().map_or(false, |f| f == "index.md") {
            continue;
        }

        match crate::wiki_fs::parse_wiki_page(abs) {
            Ok(page) => {
                pages.push(entry_from_page(&page, rel));
            }
            Err(e) => {
                eprintln!("Warning: skipping {} — {}", rel.display(), e);
            }
        }
    }

    Ok(WikiIndex { pages })
}

/// Build a `WikiIndexEntry` from a parsed wiki page.
pub fn entry_from_page(page: &WikiPage, rel_path: &Path) -> WikiIndexEntry {
    let summary = crate::wiki_fs::first_line_summary(&page.body, 200);
    WikiIndexEntry {
        path: rel_path.to_string_lossy().replace('\\', "/"),
        title: page.frontmatter.title.clone(),
        category: page.frontmatter.category.clone(),
        keywords: page.frontmatter.keywords.clone(),
        status: page.frontmatter.status.to_string(),
        summary,
        confidence: page.frontmatter.confidence,
        updated_at: page.frontmatter.updated_at.clone(),
        id: page.frontmatter.id.clone(),
    }
}

// ─── Hierarchical co-located index.md files ──────────────────────────────

/// Rebuild all co-located `index.md` files in `wiki/published/` from the catalog
/// and NORTHSTAR blueprint.
///
/// Generates:
///   wiki/published/index.md                        — root navigation
///   wiki/published/{tree}/index.md                 — tree overview
///   wiki/published/{tree}/{subtree}/index.md       — leaf page table
pub fn rebuild_colocated_indexes(
    wiki_dir: &Path,
    index: &WikiIndex,
    trees: &[crate::commands::sync::TreeNode],
) -> Result<()> {
    let published_dir = wiki_dir.join("published");
    if !published_dir.exists() {
        return Ok(());
    }

    let now = Utc::now().format("%Y-%m-%d %H:%M UTC").to_string();

    // Group published pages by tree and subtree
    let published_pages: Vec<&WikiIndexEntry> = index
        .pages
        .iter()
        .filter(|e| e.status == "published" && !e.path.starts_with("_"))
        .collect();

    // Count pages per tree/subtree
    let mut tree_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut subtree_counts: BTreeMap<(String, String), usize> = BTreeMap::new();
    for page in &published_pages {
        let tree = page
            .category
            .first()
            .cloned()
            .unwrap_or_else(|| "uncategorized".to_string());
        let subtree = page.category.get(1).cloned();
        *tree_counts.entry(tree.clone()).or_insert(0) += 1;
        if let Some(st) = subtree {
            *subtree_counts.entry((tree, st)).or_insert(0) += 1;
        }
    }

    let total: usize = published_pages.len();

    // ── Root index: wiki/published/index.md ─────────────────────────────
    let mut root_md = format!(
        "# Curio Knowledge Index\n> {} pages | updated {}\n\n",
        total, now
    );

    if trees.is_empty() {
        root_md.push_str("_No tree blueprint defined. Run `curio init` to seed from NORTHSTAR._\n");
    } else {
        root_md.push_str("## Trees\n\n");
        for tree in trees {
            let count = tree_counts.get(&tree.slug).copied().unwrap_or(0);
            let desc = strip_html_inline(&tree.description_html);
            root_md.push_str(&format!(
                "### [{title}]({slug}/index.md)\n",
                title = tree.title,
                slug = tree.slug
            ));
            if !desc.is_empty() {
                root_md.push_str(&format!("> {}\n", desc));
            }
            root_md.push_str(&format!("> **{}** pages\n\n", count));

            for sub in &tree.subtrees {
                let sub_count = subtree_counts
                    .get(&(tree.slug.clone(), sub.slug.clone()))
                    .copied()
                    .unwrap_or(0);
                let sub_desc = strip_html_inline(&sub.description_html);
                root_md.push_str(&format!(
                    "- [{title}]({tree}/{slug}/index.md)",
                    title = sub.title,
                    tree = tree.slug,
                    slug = sub.slug
                ));
                if !sub_desc.is_empty() {
                    root_md.push_str(&format!(" — _{}_", sub_desc));
                }
                root_md.push_str(&format!(" ({} pages)\n", sub_count));
            }
            root_md.push('\n');
        }
    }

    let root_path = published_dir.join("index.md");
    std::fs::write(&root_path, &root_md)
        .with_context(|| format!("Failed to write {}", root_path.display()))?;

    // ── Tree and leaf indexes ────────────────────────────────────────────
    let mut branch_issues: Vec<String> = Vec::new();
    for tree in trees {
        let tree_dir = published_dir.join(&tree.slug);
        if !tree_dir.exists() {
            std::fs::create_dir_all(&tree_dir)?;
        }

        // Validate branch description — branch nodes must have descriptions to be
        // useful in Confluence and to pass the branch-node first-class contract.
        let tree_desc = strip_html_inline(&tree.description_html);
        if tree_desc.trim().is_empty() {
            branch_issues.push(format!("branch node '{}' ({}) has no description — add description_markdown in NORTHSTAR.md", tree.title, tree.slug));
        }

        // Pages belonging to this tree but no subtree
        let tree_top_pages: Vec<&&WikiIndexEntry> = published_pages
            .iter()
            .filter(|e| {
                e.category.first().map(|s| s.as_str()) == Some(&tree.slug) && e.category.len() == 1
            })
            .collect();

        write_tree_index(
            &tree_dir,
            tree,
            &tree.subtrees,
            &tree_top_pages,
            now.as_str(),
        )?;

        // Leaf indexes for each subtree
        for sub in &tree.subtrees {
            let sub_dir = tree_dir.join(&sub.slug);
            if !sub_dir.exists() {
                std::fs::create_dir_all(&sub_dir)?;
            }

            let sub_desc = strip_html_inline(&sub.description_html);
            if sub_desc.trim().is_empty() {
                branch_issues.push(format!(
                    "  branch node '{}/{}' ({}/{}) has no description",
                    tree.slug, sub.slug, tree.title, sub.title
                ));
            }

            let sub_pages: Vec<&&WikiIndexEntry> = published_pages
                .iter()
                .filter(|e| {
                    e.category.first().map(|s| s.as_str()) == Some(tree.slug.as_str())
                        && e.category.get(1).map(|s| s.as_str()) == Some(sub.slug.as_str())
                })
                .collect();

            write_leaf_index(&sub_dir, sub, &sub_pages, now.as_str())?;
        }
    }

    // Report branch-node description defects so they are visible and actionable.
    // Branch nodes without descriptions produce thin Confluence pages that violate
    // the first-class branch-node contract in process.md.
    if !branch_issues.is_empty() {
        eprintln!("⚠ Branch node description gaps — add description_markdown to NORTHSTAR.md for:");
        for issue in &branch_issues {
            eprintln!("  {}", issue);
        }
        eprintln!("  (Re-run `curio reindex` after updating NORTHSTAR.md to clear these warnings)");
    }

    let unc_dir = published_dir.join("uncategorized");
    if unc_dir.exists() {
        let _ = std::fs::remove_file(unc_dir.join("index.md"));
        let _ = std::fs::remove_dir(&unc_dir);
    }

    Ok(())
}

fn write_tree_index(
    tree_dir: &Path,
    tree: &crate::commands::sync::TreeNode,
    subtrees: &[crate::commands::sync::TreeNode],
    top_pages: &[&&WikiIndexEntry],
    now: &str,
) -> Result<()> {
    let desc = strip_html_inline(&tree.description_html);
    let mut md = format!("# {}\n", tree.title);
    if !desc.is_empty() {
        md.push_str(&format!("> {}\n", desc));
    }
    md.push_str(&format!("> updated {}\n\n", now));

    if !subtrees.is_empty() {
        md.push_str("## Subtrees\n\n");
        for sub in subtrees {
            let sub_desc = strip_html_inline(&sub.description_html);
            md.push_str(&format!("- **[{}]({}/index.md)**", sub.title, sub.slug));
            if !sub_desc.is_empty() {
                md.push_str(&format!(" — {}", sub_desc));
            }
            md.push('\n');
        }
        md.push('\n');
    }

    if !top_pages.is_empty() {
        md.push_str("## Pages\n\n");
        md.push_str("| Title | Summary | Updated |\n|-------|---------|--------|\n");
        for e in top_pages {
            let fname = e.path.split('/').last().unwrap_or(&e.path);
            md.push_str(&format!(
                "| [{}]({}) | {} | {} |\n",
                e.title,
                fname,
                e.summary,
                short_date(&e.updated_at)
            ));
        }
    }

    std::fs::write(tree_dir.join("index.md"), md)
        .with_context(|| format!("Failed to write tree index at {}", tree_dir.display()))
}

fn write_leaf_index(
    sub_dir: &Path,
    sub: &crate::commands::sync::TreeNode,
    pages: &[&&WikiIndexEntry],
    now: &str,
) -> Result<()> {
    let desc = strip_html_inline(&sub.description_html);
    let mut md = format!("# {}\n", sub.title);
    if !desc.is_empty() {
        md.push_str(&format!("> {}\n", desc));
    }
    md.push_str(&format!(
        "> **{}** pages | updated {}\n\n",
        pages.len(),
        now
    ));

    if pages.is_empty() {
        md.push_str("_No pages yet._\n");
    } else {
        md.push_str(
            "| Title | Summary | Keywords | Updated |\n|-------|---------|----------|--------|\n",
        );
        let mut sorted = pages.to_vec();
        sorted.sort_by(|a, b| a.title.cmp(&b.title));
        for e in sorted {
            let fname = e.path.split('/').last().unwrap_or(&e.path);
            let kw = e.keywords.join(", ");
            md.push_str(&format!(
                "| [{}]({}) | {} | {} | {} |\n",
                e.title,
                fname,
                e.summary,
                kw,
                short_date(&e.updated_at)
            ));
        }
    }

    std::fs::write(sub_dir.join("index.md"), md)
        .with_context(|| format!("Failed to write leaf index at {}", sub_dir.display()))
}

/// Strip HTML tags and collapse whitespace for use in markdown text.
fn strip_html_inline(html: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn short_date(ts: &str) -> &str {
    // "2026-04-13T02:03:24Z" → "2026-04-13"
    if ts.len() >= 10 { &ts[..10] } else { ts }
}

/// Read the root navigation index for passing to LLM context.
pub fn read_index_md(wiki_dir: &Path) -> Result<String> {
    let new_path = wiki_dir.join("published/index.md");
    if new_path.exists() {
        return std::fs::read_to_string(&new_path)
            .with_context(|| format!("Failed to read {}", new_path.display()));
    }
    Ok(String::new())
}

// ─── Backwards-compat shim ───────────────────────────────────────────────

/// Rebuild the filesystem-derived catalog and regenerate the co-located indexes.
pub fn rebuild_index_md(wiki_dir: &Path, index: &WikiIndex) -> Result<()> {
    let trees = crate::northstar::load_taxonomy(wiki_dir)
        .map(|taxonomy| {
            taxonomy
                .nodes
                .iter()
                .map(|node| crate::commands::sync::TreeNode {
                    title: node.title.clone(),
                    slug: node.slug.clone(),
                    description_html: if node.description_markdown.trim().is_empty() {
                        String::new()
                    } else {
                        crate::md_to_confluence::markdown_to_storage(&node.description_markdown)
                            .unwrap_or_default()
                    },
                    icon: node.icon.clone(),
                    subtrees: node
                        .children
                        .iter()
                        .map(|child| crate::commands::sync::TreeNode {
                            title: child.title.clone(),
                            slug: child.slug.clone(),
                            description_html: if child.description_markdown.trim().is_empty() {
                                String::new()
                            } else {
                                crate::md_to_confluence::markdown_to_storage(
                                    &child.description_markdown,
                                )
                                .unwrap_or_default()
                            },
                            icon: child.icon.clone(),
                            subtrees: vec![],
                        })
                        .collect(),
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let catalog = if index.pages.is_empty() {
        reindex_from_filesystem(wiki_dir)?
    } else {
        index.clone()
    };
    rebuild_colocated_indexes(wiki_dir, &catalog, &trees)
}

// ─── audit.jsonl ─────────────────────────────────────────────────────────

/// Append an entry to the Git-tracked audit log (JSONL) and the human-readable log.md.
pub fn append_log(wiki_dir: &Path, entry: &str) -> Result<()> {
    audit_store::append_entry(wiki_dir, entry)?;
    // Best-effort: log.md write failure does not abort the primary operation
    if let Err(e) = audit_store::append_log_md(wiki_dir, entry) {
        eprintln!("warning: could not append to log.md: {}", e);
    }
    Ok(())
}

// ─── Lookup helpers ──────────────────────────────────────────────────────

/// Find entries by status.
pub fn entries_by_status<'a>(index: &'a WikiIndex, status: &str) -> Vec<&'a WikiIndexEntry> {
    index.pages.iter().filter(|e| e.status == status).collect()
}

pub fn is_duplicate_hash(_index: &WikiIndex, _content_hash: &str) -> bool {
    false
}

/// Build a `WikiIndexEntry` directly from `Frontmatter` + relative path + summary.
pub fn entry_from_frontmatter(fm: &Frontmatter, rel_path: &str, summary: &str) -> WikiIndexEntry {
    WikiIndexEntry {
        path: rel_path.replace('\\', "/"),
        title: fm.title.clone(),
        category: fm.category.clone(),
        keywords: fm.keywords.clone(),
        status: fm.status.to_string(),
        summary: summary.to_string(),
        confidence: fm.confidence,
        updated_at: fm.updated_at.clone(),
        id: fm.id.clone(),
    }
}

/// Update an entry's path in the registry (after a git mv).
pub fn update_entry_path(wiki_dir: &Path, id: &str, new_path: &str) -> Result<()> {
    let _ = (wiki_dir, id, new_path);
    Ok(())
}

/// Update an entry's status in the registry.
pub fn update_entry_status(wiki_dir: &Path, id: &str, status: &PageStatus) -> Result<()> {
    let _ = (wiki_dir, id, status);
    Ok(())
}
