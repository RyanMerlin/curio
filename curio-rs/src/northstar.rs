use anyhow::{Context, Result, bail};
use pulldown_cmark::{Options, Parser, html};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

pub const NORTHSTAR_FILENAME: &str = "NORTHSTAR.md";
pub const CONFIG_FILENAME: &str = "config.yaml";
pub const ADMIN_DIRNAME: &str = "_admin";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NorthstarTaxonomy {
    pub schema_version: u32,
    #[serde(default)]
    pub nodes: Vec<TaxonomyNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaxonomyNode {
    pub title: String,
    pub slug: String,
    #[serde(default)]
    pub description_markdown: String,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub children: Vec<TaxonomyNode>,
}

impl TaxonomyNode {
    pub fn flatten_paths(&self, parent: &[String], out: &mut Vec<Vec<String>>) {
        let mut current = parent.to_vec();
        current.push(self.slug.clone());
        out.push(current.clone());
        for child in &self.children {
            child.flatten_paths(&current, out);
        }
    }
}

// ── Path helpers ─────────────────────────────────────────────────────────────

pub fn northstar_path(repo_root: &Path) -> PathBuf {
    repo_root.join(NORTHSTAR_FILENAME)
}

/// Return the authoritative NORTHSTAR path for a workspace.
///
/// Prefer `wiki_dir/NORTHSTAR.md` so the workspace can be self-contained, but keep
/// the older repo-root fallback for compatibility with existing layouts.
pub fn workspace_northstar_path(wiki_dir: &Path) -> PathBuf {
    let nested = wiki_dir.join(NORTHSTAR_FILENAME);
    if nested.exists() {
        return nested;
    }
    wiki_dir.parent().map(northstar_path).unwrap_or(nested)
}

pub fn workspace_config_path(wiki_dir: &Path) -> PathBuf {
    wiki_dir.join(ADMIN_DIRNAME).join(CONFIG_FILENAME)
}

// ── Public load / save ────────────────────────────────────────────────────────

/// Load the taxonomy from the YAML block in NORTHSTAR.md.
/// `wiki_dir` is `<repo_root>/wiki/` — the repo root is derived automatically.
pub fn load_taxonomy(wiki_dir: &Path) -> Result<NorthstarTaxonomy> {
    let config_path = workspace_config_path(wiki_dir);
    if !config_path.exists() {
        bail!(
            "config.yaml not found at {}. Run `curio init` to create it.",
            config_path.display()
        );
    }
    let raw = fs::read_to_string(&config_path)
        .with_context(|| format!("Failed to read {}", config_path.display()))?;
    let parsed: crate::config::WorkspaceConfigFile =
        serde_yaml::from_str(&raw).with_context(|| {
            format!(
                "Failed to parse workspace config YAML at {}",
                config_path.display()
            )
        })?;
    Ok(NorthstarTaxonomy {
        schema_version: parsed.schema_version,
        nodes: parsed.nodes,
    })
}

/// Write a mutated taxonomy back into `_admin/config.yaml`, preserving non-taxonomy settings.
pub fn save_taxonomy(wiki_dir: &Path, taxonomy: &NorthstarTaxonomy) -> Result<()> {
    let config_path = workspace_config_path(wiki_dir);
    let mut current: crate::config::WorkspaceConfigFile = if config_path.exists() {
        let raw = fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read {}", config_path.display()))?;
        serde_yaml::from_str(&raw).with_context(|| {
            format!(
                "Failed to parse workspace config YAML at {}",
                config_path.display()
            )
        })?
    } else {
        crate::config::WorkspaceConfigFile::default()
    };
    current.schema_version = taxonomy.schema_version;
    current.nodes = taxonomy.nodes.clone();
    let updated =
        serde_yaml::to_string(&current).context("Failed to serialize workspace config YAML")?;
    fs::write(&config_path, updated)
        .with_context(|| format!("Failed to write {}", config_path.display()))
}

// ── Prose helpers (unchanged) ─────────────────────────────────────────────────

pub fn default_northstar_markdown() -> String {
    include_str!("../templates/NORTHSTAR.md").to_string()
}

pub fn ensure_northstar_markdown(repo_root: &Path, dry_run: bool) -> Result<()> {
    let path = northstar_path(repo_root);
    if path.is_file() {
        return Ok(());
    }
    if dry_run {
        println!("[WARN] northstar_doc :: missing {}", path.display());
        return Ok(());
    }
    fs::write(&path, default_northstar_markdown())
        .with_context(|| format!("Failed to write {}", path.display()))?;
    println!("[OK] northstar_doc :: created {}", path.display());
    Ok(())
}

pub fn read_northstar_markdown(wiki_dir: &Path) -> Result<String> {
    let path = workspace_northstar_path(wiki_dir);
    if !path.is_file() {
        bail!(
            "Missing NORTHSTAR.md at {}. Run `curio onboard` to create the charter file.",
            path.display()
        );
    }
    fs::read_to_string(&path).with_context(|| format!("Failed to read {}", path.display()))
}

pub fn render_northstar_markdown(markdown: &str) -> String {
    let mut html_output = String::new();
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    let parser = Parser::new_ext(markdown, options);
    html::push_html(&mut html_output, parser);
    html_output
}

// ── Taxonomy query helpers ────────────────────────────────────────────────────

pub fn taxonomy_paths(taxonomy: &NorthstarTaxonomy) -> Vec<Vec<String>> {
    let mut out = Vec::new();
    for node in &taxonomy.nodes {
        node.flatten_paths(&[], &mut out);
    }
    out
}

pub fn taxonomy_path_exists(taxonomy: &NorthstarTaxonomy, path: &[String]) -> bool {
    if path.is_empty() {
        return false;
    }
    let mut current_nodes = &taxonomy.nodes;
    for segment in path {
        let Some(node) = current_nodes.iter().find(|node| node.slug == *segment) else {
            return false;
        };
        current_nodes = &node.children;
    }
    true
}

pub fn taxonomy_root_titles(taxonomy: &NorthstarTaxonomy) -> HashSet<String> {
    taxonomy.nodes.iter().map(|n| n.title.clone()).collect()
}

pub fn slugify_title(title: &str) -> String {
    title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}
