/// `curio init-kb` — scaffold a new KB store at an arbitrary path.
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

use crate::workspace::{expand_tilde, upsert_workspace};

const DEFAULT_KB_PATH: &str = "~/curio-kb";

const NORTHSTAR_TEMPLATE: &str = r#"# NORTHSTAR

## Name

New Curio Workspace

## High-Level Description

This file is the human-readable charter for this Curio instance.

## Charter

Describe the purpose, vision, focus, intended audience, and scope of this workspace.
"#;

const WORKSPACE_CONFIG_TEMPLATE: &str = r#"# Curio workspace configuration
schema_version: 2
nodes: []

heal:
  confidence_threshold: 0.85
  show_auto_heal_callout: true
  auto_heal_label: "curio:auto-healed"
  max_pages_per_run: 20
  stale_threshold_days: 240
  overlap_threshold: 0.6
  external_search_enabled: true
  min_body_words: 50

slack:
  enabled: false
  workspace_id: null
  app_id: null
  admin_user_ids: []
  intake_channels: []
  notification_channels: []
  allowed_trigger_channels: []
  job_provider_default: "gemini"
  require_confirmation_for_actions: true
"#;

const CURIO_YAML_TEMPLATE: &str = r#"# Curio KB configuration
# See: https://github.com/<your-org>/curio for full reference

connection:
  confluence_url: ""        # e.g. https://yourorg.atlassian.net/wiki
  confluence_email: ""      # your Atlassian account email
  # Name of the env var that holds this KB's Confluence API token.
  # A single Curio harness instance can manage multiple KBs with distinct
  # credentials by giving each one a different env var name here.
  # Defaults to CURIO_CONFLUENCE_TOKEN if omitted.
  token_env: "CURIO_CONFLUENCE_TOKEN"

content_model:
  space_key: ""             # Confluence space key (e.g. WIKI)
  label_namespace: curio

wiki:
  wiki_dir: wiki
  auto_commit: true
  sync:
    enabled: false
    confluence_parent_page_id: ""  # Confluence numeric page ID for KB root

# Secrets: never commit tokens. Add them to .env in this directory
# (which is .gitignore'd) using the env-var name set above in connection.token_env.
"#;

const GITIGNORE_CONTENT: &str = r#"# Curio KB — gitignore
.env
*.env.local
/wiki/_admin/last-sync.txt
"#;

pub async fn run_init_kb(
    path: Option<PathBuf>,
    name: Option<String>,
    description: Option<String>,
    dry_run: bool,
) -> Result<()> {
    let kb_path = path
        .map(|p| {
            if p.is_absolute() {
                p
            } else {
                std::env::current_dir().unwrap_or_default().join(p)
            }
        })
        .unwrap_or_else(|| expand_tilde(DEFAULT_KB_PATH));

    println!("Initializing KB store at: {}", kb_path.display());

    if dry_run {
        println!("[dry-run] Would create:");
        print_scaffold(&kb_path);
        return Ok(());
    }

    // Create directories
    let dirs = [
        kb_path.join("wiki").join("intake"),
        kb_path.join("wiki").join("staged"),
        kb_path.join("wiki").join("review"),
        kb_path.join("wiki").join("published"),
        kb_path.join("wiki").join(crate::northstar::ADMIN_DIRNAME),
    ];
    for dir in &dirs {
        std::fs::create_dir_all(dir)
            .with_context(|| format!("Failed to create {}", dir.display()))?;
        // .gitkeep so empty dirs are tracked
        let gitkeep = dir.join(".gitkeep");
        if !gitkeep.exists() {
            std::fs::write(&gitkeep, "").ok();
        }
    }

    // Write NORTHSTAR.md
    write_if_missing(
        &kb_path.join("wiki").join("NORTHSTAR.md"),
        NORTHSTAR_TEMPLATE,
    )?;

    // Write workspace config YAML
    write_if_missing(
        &kb_path
            .join("wiki")
            .join(crate::northstar::ADMIN_DIRNAME)
            .join("config.yaml"),
        WORKSPACE_CONFIG_TEMPLATE,
    )?;

    // Write curio.yaml
    write_if_missing(&kb_path.join("curio.yaml"), CURIO_YAML_TEMPLATE)?;

    // Write .gitignore
    write_if_missing(&kb_path.join(".gitignore"), GITIGNORE_CONTENT)?;

    // Write .env stub (with comment only, never commit secrets)
    let env_stub =
        "# KB secrets — never commit this file\n# CURIO_CONFLUENCE_TOKEN=your-token-here\n";
    write_if_missing(&kb_path.join(".env"), env_stub)?;

    // Write a colleague-facing README at the KB root.
    let readme_body = format!(
        "# {}\n\nThis is a Curio knowledge base.\n\n- **Source of truth:** the `wiki/` tree, in this git repo.\n- **Confluence mirror:** configured via `.curio.yaml` (`content_model.space_key` and `wiki.sync.confluence_parent_page_id`).\n- **Charter:** see `wiki/NORTHSTAR.md`.\n\n## First steps\n\n1. Edit `wiki/NORTHSTAR.md` — author your editorial charter.\n2. Edit `wiki/_admin/config.yaml` — declare your taxonomy nodes.\n3. Edit `.curio.yaml` — set `connection.confluence_url`, `connection.confluence_email`, and `content_model.space_key`.\n4. Copy `.env.example` → `.env` and paste your Confluence API token under the env var named in `connection.token_env`.\n5. Run `curio --kb-dir <this-dir> doctor`. All infrastructure checks should pass.\n6. See `curio-agent/docs/runbook.md` for the full intake → process → publish → sync flow.\n",
        kb_path.file_name().and_then(|n| n.to_str()).unwrap_or("kb")
    );
    write_if_missing(&kb_path.join("README.md"), &readme_body)?;

    println!("\nKB store created:");
    print_scaffold(&kb_path);

    // Register as named workspace if name was given
    if let Some(ref ws_name) = name {
        let ws_file = upsert_workspace(ws_name, &kb_path, description.as_deref())?;
        println!(
            "\nRegistered as workspace '{ws_name}' in {}",
            ws_file.display()
        );
        println!("  Use: curio --workspace {ws_name} <command>");
    } else {
        println!("\nTo register as a named workspace:");
        println!(
            "  curio workspace add --name <name> --path {}",
            kb_path.display()
        );
    }

    println!("\nNext steps:");
    println!(
        "  1. Edit {} — add your Confluence URL and space key",
        kb_path.join("curio.yaml").display()
    );
    println!(
        "  2. Add CURIO_CONFLUENCE_TOKEN to {}",
        kb_path.join(".env").display()
    );
    println!(
        "  3. Edit {} — define your taxonomy",
        kb_path.join("NORTHSTAR.md").display()
    );
    println!("  4. Run: curio --kb-dir {} init", kb_path.display());

    Ok(())
}

fn write_if_missing(path: &Path, content: &str) -> Result<()> {
    if !path.exists() {
        std::fs::write(path, content)
            .with_context(|| format!("Failed to write {}", path.display()))?;
        println!("  created  {}", path.display());
    } else {
        println!("  exists   {}", path.display());
    }
    Ok(())
}

fn print_scaffold(kb_path: &Path) {
    let items = [
        "wiki/intake/",
        "wiki/staged/",
        "wiki/review/",
        "wiki/published/",
        "wiki/_admin/",
        "NORTHSTAR.md",
        "curio.yaml",
        ".gitignore",
        ".env",
        "README.md",
    ];
    for item in &items {
        println!("  {}", kb_path.join(item).display());
    }
}
