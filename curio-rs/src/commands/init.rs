use anyhow::Result;

use crate::{
    WikiIndex,
    commands::sync::{
        ensure_curio_confluence_tree, reset_curio_confluence_tree, validate_curio_confluence_tree,
    },
    config::Config,
    confluence::ConfluenceClient,
    northstar::{NorthstarTaxonomy, save_taxonomy},
    output::emit_json,
    wiki_index::{append_log, rebuild_index_md},
};

/// Fallback top-level trees used when no NORTHSTAR blueprint is present.
const DEFAULT_TREES: &[(&str, &str)] = &[
    (
        "account-tree",
        "Customer- and account-specific intelligence",
    ),
    (
        "product-tree",
        "Product-centric guidance and reference content",
    ),
    (
        "topic-tree",
        "Subject matter pages when no stronger route applies",
    ),
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
                serde_json::json!({ "wiki_dir": wiki_dir, "dry_run": true, "reset": reset }),
            );
        } else {
            println!("Would initialise wiki at {}", wiki_dir.display());
        }
        return Ok(());
    }

    // Create top-level directories
    let dirs = [
        wiki_dir.join(crate::northstar::ADMIN_DIRNAME),
        wiki_dir.join("intake"),
        wiki_dir.join("staged"),
        wiki_dir.join("review"),
        wiki_dir.join("published"),
    ];

    for dir in &dirs {
        std::fs::create_dir_all(dir)?;
    }

    // Seed NORTHSTAR prose file.
    let northstar_path = wiki_dir.join("NORTHSTAR.md");
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
                "# NORTHSTAR\n\n## Name\n\nCurio Workspace\n\n## High-Level Description\n\nCurio is an enterprise intelligence workspace.\n\n## Charter\n\nUse this file to capture the purpose, vision, focus, and scope of this Curio instance.\n",
            )?;
        }
    }

    // Remove legacy mirrored NORTHSTAR file on reset.
    let legacy_northstar = wiki_dir.join("_config/northstar.md");
    if reset && legacy_northstar.exists() {
        std::fs::remove_file(&legacy_northstar)?;
    }

    // Seed unified config.yaml.
    let config_yaml_path = wiki_dir
        .join(crate::northstar::ADMIN_DIRNAME)
        .join("config.yaml");
    if !config_yaml_path.exists() || reset {
        let taxonomy = NorthstarTaxonomy {
            schema_version: 2,
            nodes: vec![],
        };
        save_taxonomy(wiki_dir, &taxonomy)?;
        let current = std::fs::read_to_string(&config_yaml_path)?;
        let merged = format!(
            "# Curio Workspace Configuration\n\n{}heal:\n  confidence_threshold: 0.85\n  show_auto_heal_callout: true\n  auto_heal_label: \"curio:auto-healed\"\n  max_pages_per_run: 20\n  stale_threshold_days: 240\n  overlap_threshold: 0.6\n  external_search_enabled: true\n  min_body_words: 50\n\nslack:\n  enabled: false\n  workspace_id: null\n  app_id: null\n  admin_user_ids: []\n  intake_channels: []\n  notification_channels: []\n  allowed_trigger_channels: []\n  job_provider_default: \"gemini\"\n  require_confirmation_for_actions: true\n",
            current
        );
        std::fs::write(&config_yaml_path, merged)?;
    }

    // Create published/ top-level tree dirs from NORTHSTAR blueprint.
    // Subtree dirs are NOT pre-created — they exist only when content is published there.
    let taxonomy = crate::northstar::load_taxonomy(wiki_dir)?;
    let trees = taxonomy.nodes;
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

    // Seed _admin/readme.md from repo README.md
    let config_readme = wiki_dir
        .join(crate::northstar::ADMIN_DIRNAME)
        .join("readme.md");
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

    // Remove legacy settings.yaml on reset.
    let legacy_settings = wiki_dir.join("_config/settings.yaml");
    if reset && legacy_settings.exists() {
        std::fs::remove_file(&legacy_settings)?;
    }

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

    if config.connection.require_confluence().is_ok()
        && let Ok(token) = config.connection.resolve_token()
    {
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
            println!(
                "  reset deleted {} managed descendant page(s)",
                deleted_descendants
            );
        }
        println!(
            "  validation passed: {} checked page(s)",
            validation.checked_pages
        );
    }

    if json {
        let _ = emit_json(
            "init",
            true,
            serde_json::json!({ "wiki_dir": wiki_dir, "trees": created_trees, "reset": reset }),
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
