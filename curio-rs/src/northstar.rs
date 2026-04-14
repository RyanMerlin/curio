use anyhow::{Context, Result, bail};
use pulldown_cmark::{Options, Parser, html};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

pub const NORTHSTAR_FILENAME: &str = "NORTHSTAR.md";
pub const NORTHSTAR_CONFIG_MD: &str = "northstar.md";
pub const NORTHSTAR_CONFIG_JSON: &str = "northstar.json";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NorthstarTaxonomy {
    pub schema_version: u32,
    #[serde(default)]
    pub generated_from: String,
    #[serde(default)]
    pub generated_at: String,
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

pub fn northstar_path(repo_root: &Path) -> PathBuf {
    repo_root.join(NORTHSTAR_FILENAME)
}

pub fn northstar_markdown_path(wiki_dir: &Path) -> PathBuf {
    wiki_dir.join("_config").join(NORTHSTAR_CONFIG_MD)
}

pub fn northstar_json_path(wiki_dir: &Path) -> PathBuf {
    wiki_dir.join("_config").join(NORTHSTAR_CONFIG_JSON)
}

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

pub fn parse_markdown_taxonomy(markdown: &str) -> NorthstarTaxonomy {
    let mut nodes: Vec<TaxonomyNode> = Vec::new();
    let mut in_blueprint = false;
    let mut current_tree: Option<TaxonomyNode> = None;
    let mut current_sub: Option<TaxonomyNode> = None;
    let mut desc_lines: Vec<String> = Vec::new();

    let flush_markdown = |lines: &mut Vec<String>| -> String {
        let md = lines.join("\n").trim().to_string();
        lines.clear();
        md
    };

    for line in markdown.lines() {
        if line.starts_with("## Published Tree Blueprint") {
            in_blueprint = true;
            continue;
        }
        if in_blueprint && line.starts_with("## ") {
            break;
        }
        if !in_blueprint {
            continue;
        }

        if line.starts_with("### ") {
            if let Some(mut sub) = current_sub.take() {
                sub.description_markdown = flush_markdown(&mut desc_lines);
                if let Some(ref mut tree) = current_tree {
                    tree.children.push(sub);
                }
            } else if let Some(ref mut tree) = current_tree {
                tree.description_markdown = flush_markdown(&mut desc_lines);
            } else {
                desc_lines.clear();
            }
            if let Some(tree) = current_tree.take() {
                nodes.push(tree);
            }
            let title = line[4..].trim().to_string();
            current_tree = Some(TaxonomyNode {
                slug: slugify_title(&title),
                title,
                ..Default::default()
            });
        } else if line.starts_with("#### ") {
            if let Some(mut sub) = current_sub.take() {
                sub.description_markdown = flush_markdown(&mut desc_lines);
                if let Some(ref mut tree) = current_tree {
                    tree.children.push(sub);
                }
            } else if let Some(ref mut tree) = current_tree {
                tree.description_markdown = flush_markdown(&mut desc_lines);
            }
            let title = line[5..].trim().to_string();
            current_sub = Some(TaxonomyNode {
                slug: slugify_title(&title),
                title,
                ..Default::default()
            });
        } else if let Some(rest) = line.trim().strip_prefix("**Icon:**") {
            let icon = rest.trim().to_string();
            if let Some(ref mut sub) = current_sub {
                sub.icon = Some(icon);
            } else if let Some(ref mut tree) = current_tree {
                tree.icon = Some(icon);
            }
        } else {
            desc_lines.push(line.to_string());
        }
    }

    if let Some(mut sub) = current_sub.take() {
        sub.description_markdown = flush_markdown(&mut desc_lines);
        if let Some(ref mut tree) = current_tree {
            tree.children.push(sub);
        }
    } else if let Some(ref mut tree) = current_tree {
        tree.description_markdown = flush_markdown(&mut desc_lines);
    }
    if let Some(tree) = current_tree.take() {
        nodes.push(tree);
    }

    NorthstarTaxonomy {
        schema_version: 1,
        generated_from: NORTHSTAR_CONFIG_MD.to_string(),
        generated_at: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        nodes,
    }
}

pub fn load_taxonomy(wiki_dir: &Path) -> Result<NorthstarTaxonomy> {
    let json_path = northstar_json_path(wiki_dir);
    if json_path.exists() {
        let raw = fs::read_to_string(&json_path)
            .with_context(|| format!("Failed to read {}", json_path.display()))?;
        let taxonomy: NorthstarTaxonomy = serde_json::from_str(&raw)
            .with_context(|| format!("Failed to parse {}", json_path.display()))?;
        return Ok(taxonomy);
    }

    let md_path = northstar_markdown_path(wiki_dir);
    if !md_path.exists() {
        bail!("Missing taxonomy source at {}", md_path.display());
    }
    let markdown = fs::read_to_string(&md_path)
        .with_context(|| format!("Failed to read {}", md_path.display()))?;
    let taxonomy = parse_markdown_taxonomy(&markdown);
    save_taxonomy(wiki_dir, &taxonomy)?;
    Ok(taxonomy)
}

pub fn save_taxonomy(wiki_dir: &Path, taxonomy: &NorthstarTaxonomy) -> Result<()> {
    let json_path = northstar_json_path(wiki_dir);
    if let Some(parent) = json_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create {}", parent.display()))?;
    }
    fs::write(&json_path, serde_json::to_string_pretty(taxonomy)?)
        .with_context(|| format!("Failed to write {}", json_path.display()))
}

pub fn sync_taxonomy_from_markdown(wiki_dir: &Path) -> Result<NorthstarTaxonomy> {
    let md_path = northstar_markdown_path(wiki_dir);
    let markdown = fs::read_to_string(&md_path)
        .with_context(|| format!("Failed to read {}", md_path.display()))?;
    let taxonomy = parse_markdown_taxonomy(&markdown);
    save_taxonomy(wiki_dir, &taxonomy)?;
    Ok(taxonomy)
}

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
