use anyhow::{Context, Result, bail};
use pulldown_cmark::{Options, Parser, html};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

pub const NORTHSTAR_FILENAME: &str = "NORTHSTAR.md";

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

/// Derive the repo root from wiki_dir (wiki_dir is always <repo_root>/wiki/).
pub fn repo_root_from_wiki(wiki_dir: &Path) -> PathBuf {
    wiki_dir.parent().unwrap_or(wiki_dir).to_path_buf()
}

// ── YAML-block parser and writer ─────────────────────────────────────────────

const YAML_FENCE_START: &str = "```yaml";
const YAML_FENCE_END: &str = "```";
const TAXONOMY_SECTION: &str = "## Taxonomy";

/// Extract the first ```yaml ... ``` block from a markdown string.
fn extract_yaml_block(markdown: &str) -> Option<&str> {
    // Find the opening fence
    let fence_start = markdown.find("```yaml")?;
    // Content starts after the opening fence line (past the newline)
    let content_start = markdown[fence_start..].find('\n')? + fence_start + 1;
    // Find the closing fence (must be on its own line after the opening)
    let after_open = &markdown[content_start..];
    let close_offset = after_open.find("\n```")?;
    Some(&after_open[..close_offset + 1]) // include the trailing newline
}

/// Parse a `NorthstarTaxonomy` from the YAML block embedded in a NORTHSTAR.md string.
pub fn parse_yaml_taxonomy(markdown: &str) -> Result<NorthstarTaxonomy> {
    let yaml = extract_yaml_block(markdown)
        .ok_or_else(|| anyhow::anyhow!(
            "No ```yaml block found in NORTHSTAR.md — add a ## Taxonomy section with a ```yaml block"
        ))?;
    serde_yaml::from_str(yaml).context("Failed to parse taxonomy YAML block in NORTHSTAR.md")
}

/// Replace the YAML block content in a NORTHSTAR.md string with serialized taxonomy.
/// If no block exists, appends a new ## Taxonomy section at the end.
fn replace_yaml_block(markdown: &str, taxonomy: &NorthstarTaxonomy) -> Result<String> {
    let yaml_body =
        serde_yaml::to_string(taxonomy).context("Failed to serialize taxonomy to YAML")?;

    // Find the ```yaml ... ``` span and replace it in-place
    let mut result = String::new();
    let mut iter = markdown.split_inclusive('\n').peekable();
    let mut replaced = false;

    while let Some(line) = iter.next() {
        let trimmed = line.trim_end();
        if !replaced && (trimmed == YAML_FENCE_START || trimmed.starts_with("```yaml")) {
            result.push_str(line); // keep the opening fence
            // Write new YAML content
            result.push_str(&yaml_body);
            // Skip old block content until closing fence
            for inner in iter.by_ref() {
                if inner.trim_end() == YAML_FENCE_END {
                    result.push_str(inner); // keep closing fence
                    break;
                }
            }
            replaced = true;
        } else {
            result.push_str(line);
        }
    }

    if !replaced {
        // Append a new Taxonomy section
        if !result.ends_with('\n') {
            result.push('\n');
        }
        result.push('\n');
        result.push_str(TAXONOMY_SECTION);
        result.push('\n');
        result.push_str("\n```yaml\n");
        result.push_str(&yaml_body);
        result.push_str("```\n");
    }

    Ok(result)
}

// ── Public load / save ────────────────────────────────────────────────────────

/// Load the taxonomy from the YAML block in NORTHSTAR.md.
/// `wiki_dir` is `<repo_root>/wiki/` — the repo root is derived automatically.
pub fn load_taxonomy(wiki_dir: &Path) -> Result<NorthstarTaxonomy> {
    let repo_root = repo_root_from_wiki(wiki_dir);
    let md_path = northstar_path(&repo_root);

    if !md_path.exists() {
        bail!(
            "NORTHSTAR.md not found at {}. Run `curio onboard` to create it.",
            md_path.display()
        );
    }

    let markdown = fs::read_to_string(&md_path)
        .with_context(|| format!("Failed to read {}", md_path.display()))?;

    parse_yaml_taxonomy(&markdown)
        .with_context(|| format!("Taxonomy parse failed for {}", md_path.display()))
}

/// Write a mutated taxonomy back into the YAML block in NORTHSTAR.md.
pub fn save_taxonomy(wiki_dir: &Path, taxonomy: &NorthstarTaxonomy) -> Result<()> {
    let repo_root = repo_root_from_wiki(wiki_dir);
    let md_path = northstar_path(&repo_root);

    let current = if md_path.exists() {
        fs::read_to_string(&md_path)
            .with_context(|| format!("Failed to read {}", md_path.display()))?
    } else {
        String::new()
    };

    let updated = replace_yaml_block(&current, taxonomy)?;
    fs::write(&md_path, updated).with_context(|| format!("Failed to write {}", md_path.display()))
}

// ── Prose helpers (unchanged) ─────────────────────────────────────────────────

pub fn default_northstar_markdown() -> String {
    include_str!("../../NORTHSTAR.md").to_string()
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

pub fn read_northstar_markdown(repo_root: &Path) -> Result<String> {
    let path = northstar_path(repo_root);
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
