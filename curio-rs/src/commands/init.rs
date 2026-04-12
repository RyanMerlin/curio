use anyhow::Result;

use crate::{
    commands::sync::parse_northstar_blueprint,
    config::Config,
    output::emit_json,
    wiki_index::{append_log, rebuild_index_md, save_registry},
    WikiIndex,
};

/// Fallback top-level trees used when no NORTHSTAR blueprint is present.
const DEFAULT_TREES: &[(&str, &str)] = &[
    ("account-tree", "Customer- and account-specific intelligence"),
    ("product-tree", "Product-centric guidance and reference content"),
    ("topic-tree", "Subject matter pages when no stronger route applies"),
];

pub async fn run_init(config: &Config, dry_run: bool, json: bool, reset: bool) -> Result<()> {
    let wiki_dir = &config.wiki.wiki_dir;

    if dry_run {
        if json {
            let _ = emit_json("init", true, &serde_json::json!({ "wiki_dir": wiki_dir, "dry_run": true }));
        } else {
            println!("Would initialise wiki at {}", wiki_dir.display());
        }
        return Ok(());
    }

    // Create top-level directories
    let dirs = [
        wiki_dir.join("_schema"),
        wiki_dir.join("_index"),
        wiki_dir.join("_audit"),
        wiki_dir.join("intake"),
        wiki_dir.join("staged"),
        wiki_dir.join("review"),
        wiki_dir.join("published"),
    ];

    for dir in &dirs {
        std::fs::create_dir_all(dir)?;
    }

    // Seed _schema/northstar.md first — published tree dirs are derived from it
    let northstar_path = wiki_dir.join("_schema/northstar.md");
    if !northstar_path.exists() || reset {
        let repo_northstar = wiki_dir
            .parent()
            .map(|p| p.join("NORTHSTAR.md"))
            .filter(|p| p.exists());
        if let Some(src) = repo_northstar {
            std::fs::copy(&src, &northstar_path)?;
        } else {
            std::fs::write(
                &northstar_path,
                "# NORTHSTAR\n\nCurio is an enterprise intelligence workspace.\nEdit this file to define the project charter.\n",
            )?;
        }
    }

    // Create published/ top-level tree dirs from NORTHSTAR blueprint.
    // Subtree dirs are NOT pre-created — they exist only when content is published there.
    let ns_md = std::fs::read_to_string(&northstar_path).unwrap_or_default();
    let trees = parse_northstar_blueprint(&ns_md);
    let mut created_trees: Vec<String> = Vec::new();

    if trees.is_empty() {
        // No blueprint yet — seed with safe defaults
        for (slug, _) in DEFAULT_TREES {
            let dir = wiki_dir.join("published").join(slug);
            if !dir.exists() {
                std::fs::create_dir_all(&dir)?;
                created_trees.push(slug.to_string());
            }
        }
    } else {
        for tree in &trees {
            let dir = wiki_dir.join("published").join(&tree.slug);
            if !dir.exists() {
                std::fs::create_dir_all(&dir)?;
                created_trees.push(tree.slug.clone());
            }
        }
    }

    // Seed _schema/config.yaml
    let schema_config = wiki_dir.join("_schema/config.yaml");
    if !schema_config.exists() || reset {
        std::fs::write(
            &schema_config,
            "# Curio Wiki Configuration\n\n# Tree structure is defined in _schema/northstar.md\nauto_commit: true\n",
        )?;
    }

    // Seed _index files
    let empty_index = WikiIndex::default();
    save_registry(wiki_dir, &empty_index)?;
    rebuild_index_md(wiki_dir, &empty_index)?;

    let log_path = wiki_dir.join("_index/log.md");
    if !log_path.exists() || reset {
        std::fs::write(&log_path, "# Curio Operation Log\n\n")?;
    }

    // .gitkeep only in pipeline staging dirs (not in published/ — content is the placeholder)
    for dir in &[
        wiki_dir.join("intake"),
        wiki_dir.join("staged"),
        wiki_dir.join("review"),
    ] {
        let keep = dir.join(".gitkeep");
        if !keep.exists() {
            std::fs::write(&keep, "")?;
        }
    }

    append_log(wiki_dir, "init: wiki scaffold created")?;

    if json {
        let _ = emit_json("init", true, &serde_json::json!({ "wiki_dir": wiki_dir, "trees": created_trees }));
    } else {
        println!("Wiki initialised at {}", wiki_dir.display());
        for slug in &created_trees {
            println!("  published/{}", slug);
        }
        if !trees.is_empty() {
            println!("  (tree structure from NORTHSTAR — run `curio tree` to sync after changes)");
        }
    }
    Ok(())
}
