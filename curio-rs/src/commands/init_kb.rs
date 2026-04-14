/// `curio init-kb` — scaffold a new KB store at an arbitrary path.
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

use crate::workspace::{expand_tilde, upsert_workspace};

const DEFAULT_KB_PATH: &str = "~/curio-kb";

const NORTHSTAR_TEMPLATE: &str = r#"# NORTHSTAR — Taxonomy Blueprint

This file defines the KB taxonomy. The `yaml` block below is the
single source of truth — `curio tree`, `curio process`, and `curio sync`
all read it. Edit the YAML block to add or rename trees and subtrees,
then run `curio tree` to sync the `wiki/published/` directory structure.

## Taxonomy

```yaml
schema_version: 2
nodes: []
```

## How to add a tree

Add an entry under `nodes`:

```yaml
nodes:
  - title: My Tree
    slug: my-tree
    description: "What this tree covers"
    children: []
```

Run `curio tree` after saving to create the directory scaffold.
"#;

const CURIO_YAML_TEMPLATE: &str = r#"# Curio KB configuration
# See: https://github.com/RyanMerlin/curio for full reference

connection:
  confluence_url: ""        # e.g. https://yourorg.atlassian.net
  confluence_email: ""      # your Atlassian account email

content_model:
  space_key: ""             # Confluence space key (e.g. WIKI)
  label_namespace: curio

wiki:
  wiki_dir: wiki
  auto_commit: true
  sync:
    enabled: false
    confluence_parent_page_id: ""  # Confluence numeric page ID for KB root

# Secrets: never put CURIO_CONFLUENCE_TOKEN here.
# Add it to .env in this directory (which is .gitignore'd).
"#;

const GITIGNORE_CONTENT: &str = r#"# Curio KB — gitignore
.env
*.env.local
/wiki/_config/last-sync.txt
"#;

pub async fn run_init_kb(
    path: Option<PathBuf>,
    name: Option<String>,
    description: Option<String>,
    dry_run: bool,
) -> Result<()> {
    let kb_path = path
        .map(|p| if p.is_absolute() { p } else {
            std::env::current_dir().unwrap_or_default().join(p)
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
        kb_path.join("wiki").join("_config"),
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
    write_if_missing(&kb_path.join("NORTHSTAR.md"), NORTHSTAR_TEMPLATE)?;

    // Write curio.yaml
    write_if_missing(&kb_path.join("curio.yaml"), CURIO_YAML_TEMPLATE)?;

    // Write .gitignore
    write_if_missing(&kb_path.join(".gitignore"), GITIGNORE_CONTENT)?;

    // Write .env stub (with comment only, never commit secrets)
    let env_stub = "# KB secrets — never commit this file\n# CURIO_CONFLUENCE_TOKEN=your-token-here\n";
    write_if_missing(&kb_path.join(".env"), env_stub)?;

    println!("\nKB store created:");
    print_scaffold(&kb_path);

    // Register as named workspace if name was given
    if let Some(ref ws_name) = name {
        let ws_file = upsert_workspace(ws_name, &kb_path, description.as_deref())?;
        println!("\nRegistered as workspace '{ws_name}' in {}", ws_file.display());
        println!("  Use: curio --workspace {ws_name} <command>");
    } else {
        println!("\nTo register as a named workspace:");
        println!("  curio workspace add --name <name> --path {}", kb_path.display());
    }

    println!("\nNext steps:");
    println!("  1. Edit {} — add your Confluence URL and space key", kb_path.join("curio.yaml").display());
    println!("  2. Add CURIO_CONFLUENCE_TOKEN to {}", kb_path.join(".env").display());
    println!("  3. Edit {} — define your taxonomy", kb_path.join("NORTHSTAR.md").display());
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
        "wiki/_config/",
        "NORTHSTAR.md",
        "curio.yaml",
        ".gitignore",
        ".env",
    ];
    for item in &items {
        println!("  {}", kb_path.join(item).display());
    }
}
