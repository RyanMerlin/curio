use anyhow::Result;

use crate::{
    commands::sync::{
        ensure_curio_confluence_tree, parse_northstar_blueprint, reset_curio_confluence_tree,
        validate_curio_confluence_tree,
    },
    config::Config,
    confluence::ConfluenceClient,
    output::emit_json,
    wiki_index::{append_log, rebuild_index_md},
    WikiIndex,
};

/// Fallback top-level trees used when no NORTHSTAR blueprint is present.
const DEFAULT_TREES: &[(&str, &str)] = &[
    ("account-tree", "Customer- and account-specific intelligence"),
    ("product-tree", "Product-centric guidance and reference content"),
    ("topic-tree", "Subject matter pages when no stronger route applies"),
];

pub async fn run_init(
    config: &Config,
    dry_run: bool,
    json: bool,
    reset: bool,
    confirm_nuke: bool,
) -> Result<()> {
    let wiki_dir = &config.wiki.wiki_dir;

    if reset && !confirm_nuke {
        anyhow::bail!(
            "Refusing destructive init reset without --confirm-nuke. Re-run with `curio init --reset --confirm-nuke`."
        );
    }

    if dry_run {
        if json {
            let _ = emit_json(
                "init",
                true,
                &serde_json::json!({ "wiki_dir": wiki_dir, "dry_run": true, "reset": reset }),
            );
        } else {
            println!("Would initialise wiki at {}", wiki_dir.display());
        }
        return Ok(());
    }

    // Create top-level directories
    let dirs = [
        wiki_dir.join("_config"),
        wiki_dir.join("intake"),
        wiki_dir.join("staged"),
        wiki_dir.join("review"),
        wiki_dir.join("published"),
    ];

    for dir in &dirs {
        std::fs::create_dir_all(dir)?;
    }

    // Seed _config/northstar.md first — published tree dirs are derived from it
    let northstar_path = wiki_dir.join("_config/northstar.md");
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

    // Seed _config/readme.md from repo README.md
    let config_readme = wiki_dir.join("_config/readme.md");
    if !config_readme.exists() || reset {
        let repo_readme = wiki_dir
            .parent()
            .map(|p| p.join("README.md"))
            .filter(|p| p.exists());
        if let Some(src) = repo_readme {
            std::fs::copy(&src, &config_readme)?;
        } else {
            std::fs::write(
                &config_readme,
                "# CURIO Readme\n\nCurio is a Git-native enterprise intelligence workspace.\n",
            )?;
        }
    }

    // Seed _config/settings.yaml
    let config_settings = wiki_dir.join("_config/settings.yaml");
    if !config_settings.exists() || reset {
        std::fs::write(
            &config_settings,
            "# Curio Wiki Configuration\n\n# Tree structure is defined in NORTHSTAR.md (repo root) as a YAML block.\nauto_commit: true\n",
        )?;
    }
    // Taxonomy is now embedded as YAML in NORTHSTAR.md — no separate file to write.

    // Generate co-located indexes from the seeded tree structure.
    let empty_index = WikiIndex::default();
    rebuild_index_md(wiki_dir, &empty_index)?;

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

    if config.connection.require_confluence().is_ok() {
        if let Ok(token) = std::env::var("CURIO_CONFLUENCE_TOKEN") {
            let client = ConfluenceClient::new(
                config.connection.confluence_url.clone(),
                config.connection.confluence_email.clone(),
                token,
                None,
            )?;
            let preferred_root_id = config.wiki.sync.confluence_parent_page_id.clone();
            let (tree, deleted_descendants) = if reset {
                reset_curio_confluence_tree(config, &client, preferred_root_id, true).await?
            } else {
                (
                    ensure_curio_confluence_tree(config, &client, preferred_root_id, true).await?,
                    0usize,
                )
            };
            let validation =
                validate_curio_confluence_tree(config, &client, Some(tree.root_id.clone())).await?;
            println!(
                "  confluence root: {} ({})",
                crate::commands::sync::CURIO_ROOT_TITLE,
                tree.root_id
            );
            if reset {
                println!("  reset deleted {} managed descendant page(s)", deleted_descendants);
            }
            println!(
                "  validation passed: {} checked page(s)",
                validation.checked_pages
            );
        }
    }

    if json {
        let _ = emit_json(
            "init",
            true,
            &serde_json::json!({ "wiki_dir": wiki_dir, "trees": created_trees, "reset": reset }),
        );
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
