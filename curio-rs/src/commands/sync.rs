/// Sync command: push wiki/published/ to Confluence as a one-way mirror.
///
/// Algorithm (iterative, no async recursion):
/// 1. Walk wiki/published/ with walkdir in BFS order.
/// 2. Maintain a map of (relative_dir_path → confluence_page_id) to track hierarchy.
/// 3. For each directory entry: create/update a branch page under its parent.
/// 4. For each .md file: strip frontmatter, convert to HTML, upsert page.
/// 5. Track ownership via curio-sync content property (content_hash).
/// 6. Skip update if content_hash unchanged.
/// 7. After walk: delete pages with curio-sync property not seen this run.
use anyhow::{Context, Result};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::{
    config::Config,
    confluence::ConfluenceClient,
    output::emit_json,
    wiki_fs::{content_hash, strip_frontmatter},
    wiki_index::append_log,
};

const CHILDREN_MACRO: &str = r#"<ac:structured-macro ac:name="children" ac:schema-version="2"/>"#;
const SYNC_PROP_KEY: &str = "curio-sync";
const ICON_PROP_KEY: &str = "emoji-title-published";

/// Emoji icon for each well-known page title slug.
fn page_icon(slug: &str) -> Option<&'static str> {
    match slug {
        "curio-readme" | "readme"   => Some("atlassian-info"),
        "northstar"                  => Some("1f9ed"),   // 🧭
        "config"                     => Some("2699"),    // ⚙️
        "intake"                     => Some("1f4e5"),   // 📥
        "staged"                     => Some("atlassian-logo_projects"),
        "review"                     => Some("atlassian-logo_opsgenie"),
        "published"                  => Some("atlassian-check_mark"),
        "accounts" | "by-account" | "account-tree" => Some("1f3e2"),   // 🏢
        "by-product" | "product-tree"             => Some("1f4e6"),   // 📦
        "by-audience" | "audience-tree"           => Some("1f465"),   // 👥
        "by-use-case" | "use-case-tree"           => Some("1f527"),   // 🔧
        "by-topic" | "topic-tree"                 => Some("1f4da"),   // 📚
        "alteryx-server"   => Some("1f5a5"),   // 🖥️
        "alteryx-designer" => Some("1f3a8"),   // 🎨
        "intelligence-suite" => Some("1f9e0"), // 🧠
        "technical-cse"    => Some("1f527"),   // 🔧
        "executive-business" => Some("1f4ca"), // 📊
        _                            => None,
    }
}

pub async fn run_sync(
    config: &Config,
    dry_run: bool,
    json: bool,
    parent_page_id_override: Option<String>,
) -> Result<()> {
    config.connection.require_confluence()?;

    // parent_page_id is optional — None means sync to the space root
    let parent_page_id: Option<String> = parent_page_id_override
        .or_else(|| config.wiki.sync.confluence_parent_page_id.clone());

    let token = std::env::var("CURIO_CONFLUENCE_TOKEN")
        .context("CURIO_CONFLUENCE_TOKEN not set")?;
    let client = ConfluenceClient::new(
        config.connection.confluence_url.clone(),
        config.connection.confluence_email.clone(),
        token,
        config.content_model.space_key.clone(),
        None,
    )?;

    let wiki_dir = &config.wiki.wiki_dir;
    let published_dir = wiki_dir.join("published");

    if !published_dir.exists() {
        anyhow::bail!("wiki/published/ not found. Run `curio init` first.");
    }

    let space_key = &config.content_model.space_key;

    let mut upserted: Vec<String> = Vec::new();
    let mut skipped: Vec<String> = Vec::new();
    let mut errors: Vec<String> = Vec::new();
    let mut synced_page_ids: HashSet<String> = HashSet::new();

    // Sync top-level schema pages first (NORTHSTAR, config) at the space root
    let schema_dir = wiki_dir.join("_schema");
    if schema_dir.exists() && !dry_run {
        for entry in std::fs::read_dir(&schema_dir).into_iter().flatten().filter_map(|e| e.ok()) {
            let path = entry.path();
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if ext != "md" && ext != "yaml" && ext != "yml" { continue; }
            let stem = path.file_stem().unwrap_or_default().to_str().unwrap_or("");
            if stem.starts_with('_') { continue; }
            let page_title = to_title(stem);
            let raw = std::fs::read_to_string(&path)
                .with_context(|| format!("Failed to read {}", path.display()))?;
            let body_md = if ext == "md" { strip_frontmatter(&raw) } else { raw.as_str() };
            let html_body = if ext == "md" {
                markdown_to_html(body_md)
            } else {
                format!("<pre><code>{}</code></pre>", body_md)
            };
            let hash = content_hash(body_md);
            // Check existing
            if let Ok(Some(ref page)) = client.get_page_by_title(space_key, parent_page_id.as_deref(), &page_title).await {
                let page_id = page["id"].as_str().unwrap_or_default().to_string();
                if let Ok(Some(prop)) = client.get_content_property(&page_id, SYNC_PROP_KEY).await {
                    if prop["value"]["content_hash"].as_str() == Some(hash.as_str()) {
                        skipped.push(format!("[schema] {}", page_title));
                        synced_page_ids.insert(page_id);
                        continue;
                    }
                }
            }
            match client.create_or_update_page(space_key, parent_page_id.as_deref(), &page_title, "storage", &html_body).await {
                Ok(page_id) => {
                    let _ = set_sync_prop(&client, &page_id, &hash).await;
                    set_page_icon(&client, &page_id, stem).await;
                    synced_page_ids.insert(page_id);
                    upserted.push(format!("[schema] {}", page_title));
                }
                Err(e) => errors.push(format!("[schema] {}: {}", page_title, e)),
            }
        }
    }

    // Sync pipeline visibility pages (Intake, Staged, Review) at the space root
    if !dry_run {
        for (slug, title, desc) in &[
            ("intake", "Intake", "Incoming content captured for processing. Items here are awaiting analysis and routing into the wiki."),
            ("staged", "Staged", "Content analyzed and routed, awaiting human review or approval before publishing."),
            ("review", "Review", "Items that need human judgment before they can move forward in the pipeline."),
        ] {
            let body = format!("<p>{}</p><p>This page is maintained automatically by Curio. Manage content via <code>curio process</code>, <code>curio resolve</code>, and <code>curio publish</code>.</p>", desc);
            let hash = content_hash(slug);
            if let Ok(Some(ref page)) = client.get_page_by_title(space_key, parent_page_id.as_deref(), title).await {
                let page_id = page["id"].as_str().unwrap_or_default().to_string();
                if let Ok(Some(prop)) = client.get_content_property(&page_id, SYNC_PROP_KEY).await {
                    if prop["value"]["content_hash"].as_str() == Some(hash.as_str()) {
                        skipped.push(format!("[pipeline] {}", title));
                        synced_page_ids.insert(page_id);
                        continue;
                    }
                }
            }
            match client.create_or_update_page(space_key, parent_page_id.as_deref(), title, "storage", &body).await {
                Ok(page_id) => {
                    let _ = set_sync_prop(&client, &page_id, &hash).await;
                    set_page_icon(&client, &page_id, slug).await;
                    synced_page_ids.insert(page_id);
                    upserted.push(format!("[pipeline] {}", title));
                }
                Err(e) => errors.push(format!("[pipeline] {}: {}", title, e)),
            }
        }
    }

    // Parse NORTHSTAR blueprint for tree descriptions
    let northstar_path = wiki_dir.join("_schema").join("northstar.md");
    let northstar_trees: Vec<TreeNode> = if northstar_path.exists() {
        let ns_md = std::fs::read_to_string(&northstar_path).unwrap_or_default();
        parse_northstar_blueprint(&ns_md)
    } else {
        Vec::new()
    };
    // Build slug → (title, description_html, icon, subtree_slug → (title, desc_html, icon)) lookup
    type SubMap = HashMap<String, (String, String, Option<String>)>;
    type TreeMap = HashMap<String, (String, String, Option<String>, SubMap)>;
    let mut tree_info: TreeMap = HashMap::new();
    for t in &northstar_trees {
        let mut sub_map: SubMap = HashMap::new();
        for s in &t.subtrees {
            sub_map.insert(s.slug.clone(), (s.title.clone(), s.description_html.clone(), s.icon.clone()));
        }
        tree_info.insert(t.slug.clone(), (t.title.clone(), t.description_html.clone(), t.icon.clone(), sub_map));
    }

    // Map from relative directory path → confluence page id (None = parent_page_id).
    let mut dir_to_page_id: HashMap<PathBuf, Option<String>> = HashMap::new();
    dir_to_page_id.insert(PathBuf::new(), parent_page_id.clone());

    // Walk in sorted order so parent dirs are always processed before children.
    let entries = collect_sorted_entries(&published_dir)?;

    for (rel_path, is_dir) in &entries {
        let abs_path = published_dir.join(rel_path);
        let name = rel_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        // Skip hidden / system files
        if name.starts_with('.') || name.starts_with('_') {
            continue;
        }

        let parent_rel = rel_path.parent().map(PathBuf::from).unwrap_or_default();
        let parent_conf_id: Option<String> = dir_to_page_id
            .get(&parent_rel)
            .cloned()
            .unwrap_or_else(|| parent_page_id.clone());

        if *is_dir {
            // Look up NORTHSTAR title + description + icon for this dir slug
            let depth = rel_path.components().count();
            let (page_title, dir_body, ns_icon) = if depth == 1 {
                // Top-level tree dir: use NORTHSTAR title, description, and icon
                if let Some((ns_title, ns_desc, icon, _)) = tree_info.get(&name) {
                    let body = format!(
                        "{}<ac:structured-macro ac:name=\"children\" ac:schema-version=\"2\"/>",
                        if ns_desc.trim().is_empty() { String::new() } else { ns_desc.clone() }
                    );
                    (ns_title.clone(), body, icon.clone())
                } else {
                    (to_title(&name), CHILDREN_MACRO.to_string(), None)
                }
            } else if depth == 2 {
                // Subtree dir: look up in parent's subtree list
                let parent_name = parent_rel.file_name()
                    .and_then(|n| n.to_str()).unwrap_or("");
                let sub_info = tree_info.get(parent_name)
                    .and_then(|(_, _, _, subs)| subs.get(&name))
                    .map(|(t, d, i)| (t.clone(), d.clone(), i.clone()));
                if let Some((sub_title, sub_desc, sub_icon)) = sub_info {
                    let body = format!(
                        "{}<ac:structured-macro ac:name=\"children\" ac:schema-version=\"2\"/>",
                        if sub_desc.trim().is_empty() { String::new() } else { sub_desc }
                    );
                    (sub_title, body, sub_icon)
                } else {
                    (to_title(&name), CHILDREN_MACRO.to_string(), None)
                }
            } else {
                (to_title(&name), CHILDREN_MACRO.to_string(), None)
            };

            if dry_run {
                upserted.push(format!("[dir] {}", page_title));
                dir_to_page_id.insert(rel_path.clone(), Some(format!("dry-run:{}", page_title)));
            } else {
                match upsert_page(
                    &client, space_key, parent_conf_id.as_deref(), &page_title, &dir_body, &name,
                    ns_icon.as_deref(),
                )
                .await
                {
                    Ok(id) => {
                        synced_page_ids.insert(id.clone());
                        dir_to_page_id.insert(rel_path.clone(), Some(id));
                        upserted.push(format!("[dir] {}", page_title));
                    }
                    Err(e) => errors.push(format!("[dir] {}: {}", page_title, e)),
                }
            }
        } else if rel_path.extension().map_or(false, |ext| ext == "md") {
            let page_title = to_title(rel_path.file_stem().unwrap_or_default().to_str().unwrap_or(""));
            if dry_run {
                upserted.push(format!("[page] {}", page_title));
            } else {
                match sync_page(
                    &client, space_key, parent_conf_id.as_deref(), &abs_path, &page_title,
                    &mut synced_page_ids, &mut skipped,
                )
                .await
                {
                    Ok(()) => upserted.push(format!("[page] {}", page_title)),
                    Err(e) => errors.push(format!("[page] {}: {}", page_title, e)),
                }
            }
        }
    }

    // Delete stale pages
    if !dry_run {
        let stale =
            find_stale_pages(&client, space_key, parent_page_id.as_deref(), &synced_page_ids).await?;
        for page_id in &stale {
            match client.delete_page(page_id).await {
                Ok(()) => upserted.push(format!("[deleted] {}", page_id)),
                Err(e) => errors.push(format!("delete {} failed: {}", page_id, e)),
            }
        }

        append_log(
            wiki_dir,
            &format!(
                "sync: {} upserted, {} skipped, {} stale deleted, {} errors",
                upserted.len(),
                skipped.len(),
                stale.len(),
                errors.len()
            ),
        )?;
    }

    if json {
        let _ = emit_json(
            "sync",
            true,
            &serde_json::json!({
                "upserted": upserted,
                "skipped": skipped,
                "errors": errors,
                "dry_run": dry_run,
            }),
        );
    } else {
        println!(
            "Sync: {} upserted, {} skipped, {} errors{}",
            upserted.len(),
            skipped.len(),
            errors.len(),
            if dry_run { " (dry run)" } else { "" }
        );
        for e in &errors {
            eprintln!("  error: {}", e);
        }
    }
    Ok(())
}

// ─── Entry collection ─────────────────────────────────────────────────────

/// Return all entries under `root` as (relative_path, is_dir) in sorted BFS order.
fn collect_sorted_entries(root: &Path) -> Result<Vec<(PathBuf, bool)>> {
    use walkdir::WalkDir;
    let mut entries = Vec::new();
    for e in WalkDir::new(root)
        .sort_by_file_name()
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path() != root)
    {
        let rel = e
            .path()
            .strip_prefix(root)
            .map(PathBuf::from)
            .unwrap_or_default();
        entries.push((rel, e.file_type().is_dir()));
    }
    Ok(entries)
}

// ─── Page operations ──────────────────────────────────────────────────────

async fn sync_page(
    client: &ConfluenceClient,
    space_key: &str,
    parent_id: Option<&str>,
    path: &Path,
    page_title: &str,
    synced_ids: &mut HashSet<String>,
    skipped: &mut Vec<String>,
) -> Result<()> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    let body_md = strip_frontmatter(&raw);
    let hash = content_hash(body_md);
    let html_body = markdown_to_html(body_md);

    // Check if page already exists and content hasn't changed
    if let Some(ref page) = client.get_page_by_title(space_key, parent_id, page_title).await? {
        let page_id = page["id"].as_str().unwrap_or_default().to_string();
        if let Ok(Some(prop)) = client.get_content_property(&page_id, SYNC_PROP_KEY).await {
            if prop["value"]["content_hash"].as_str() == Some(hash.as_str()) {
                skipped.push(page_title.to_string());
                synced_ids.insert(page_id);
                return Ok(());
            }
        }
        // Content changed — use create_or_update_page which handles versioning
    }

    // create_or_update_page handles both create and update
    let slug = path.file_stem().unwrap_or_default().to_str().unwrap_or("");
    let page_id = client
        .create_or_update_page(space_key, parent_id, page_title, "storage", &html_body)
        .await?;
    set_sync_prop(client, &page_id, &hash).await?;
    set_page_icon(client, &page_id, slug).await;
    synced_ids.insert(page_id);
    Ok(())
}

async fn upsert_page(
    client: &ConfluenceClient,
    space_key: &str,
    parent_id: Option<&str>,
    title: &str,
    body: &str,
    slug: &str,
    icon_override: Option<&str>,
) -> Result<String> {
    let hash = content_hash(slug);
    let page_id = client
        .create_or_update_page(space_key, parent_id, title, "storage", body)
        .await?;
    set_sync_prop(client, &page_id, &hash).await?;
    if let Some(icon_val) = icon_override {
        set_page_icon_value(client, &page_id, icon_val).await;
    } else {
        set_page_icon(client, &page_id, slug).await;
    }
    Ok(page_id)
}

async fn set_sync_prop(client: &ConfluenceClient, page_id: &str, hash: &str) -> Result<()> {
    let value = serde_json::json!({ "content_hash": hash, "synced_by": "curio" });
    client.set_content_property(page_id, SYNC_PROP_KEY, value).await
}

async fn find_stale_pages(
    client: &ConfluenceClient,
    _space_key: &str,
    parent_id: Option<&str>,
    synced_ids: &HashSet<String>,
) -> Result<Vec<String>> {
    let Some(parent_id) = parent_id else {
        // No parent anchor — can't enumerate stale pages at space root safely
        return Ok(vec![]);
    };
    let all = client
        .get_page_descendants_v2(parent_id)
        .await
        .unwrap_or_default();

    let mut stale = Vec::new();
    for page in all {
        let page_id = page["id"].as_str().unwrap_or_default().to_string();
        if synced_ids.contains(&page_id) {
            continue;
        }
        if let Ok(Some(_)) = client.get_content_property(&page_id, SYNC_PROP_KEY).await {
            stale.push(page_id);
        }
    }
    Ok(stale)
}

// ─── Helpers ──────────────────────────────────────────────────────────────

fn markdown_to_html(md: &str) -> String {
    crate::md_to_confluence::markdown_to_storage(md)
        .unwrap_or_else(|_| {
            // Fallback: plain pulldown_cmark if macro parsing fails
            use pulldown_cmark::{html, Options, Parser};
            let mut opts = Options::empty();
            opts.insert(Options::ENABLE_TABLES);
            let parser = Parser::new_ext(md, opts);
            let mut out = String::new();
            html::push_html(&mut out, parser);
            out
        })
}

/// Set the emoji icon on a Confluence page using a hardcoded slug → emoji map. Best-effort.
async fn set_page_icon(client: &ConfluenceClient, page_id: &str, slug: &str) {
    if let Some(icon) = page_icon(slug) {
        set_page_icon_value(client, page_id, icon).await;
    }
}

/// Set the emoji icon on a Confluence page using a raw emoji value (e.g. "1f4e6"). Best-effort.
async fn set_page_icon_value(client: &ConfluenceClient, page_id: &str, icon: &str) {
    let val = serde_json::json!(icon);
    let _ = client.set_content_property(page_id, ICON_PROP_KEY, val).await;
}

/// Convert slug/filename to a display title.
/// For tree dirs the NORTHSTAR blueprint drives the name; fallback is title-case.
fn to_title(name: &str) -> String {
    // Tree dirs end in "-tree" — preserve that as-is (title-cased)
    // Other slugs: title-case each word
    name.split('-')
        .map(|word| {
            let mut c = word.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
        // "Account Tree" → "Account-tree" etc. is handled by the slug naming convention
        // We want "Account-tree" not "Account Tree" for tree nodes
        .replace(" Tree", "-tree")
}

// ─── NORTHSTAR blueprint parser ───────────────────────────────────────────

#[derive(Debug, Default)]
pub struct TreeNode {
    /// Display title (e.g. "Account-tree")
    pub title: String,
    /// Filesystem slug (e.g. "account-tree")
    pub slug: String,
    /// HTML description for the Confluence page body
    pub description_html: String,
    /// Confluence emoji icon value (e.g. "1f4e6"). Parsed from `**Icon:** 1f4e6` in NORTHSTAR.
    pub icon: Option<String>,
    /// Named subtrees (from `####` headings)
    pub subtrees: Vec<TreeNode>,
}

/// Parse the `## Published Tree Blueprint` section of a NORTHSTAR markdown file.
/// Returns a list of top-level tree nodes, each with optional subtree children.
pub fn parse_northstar_blueprint(northstar_md: &str) -> Vec<TreeNode> {
    let mut trees: Vec<TreeNode> = Vec::new();
    let mut in_blueprint = false;
    let mut current_tree: Option<TreeNode> = None;
    let mut current_sub: Option<TreeNode> = None;
    // Collect description lines for the current node
    let mut desc_lines: Vec<String> = Vec::new();

    let flush_desc = |lines: &mut Vec<String>| -> String {
        let md = lines.join("\n");
        lines.clear();
        if md.trim().is_empty() { return String::new(); }
        crate::md_to_confluence::markdown_to_storage(&md).unwrap_or_default()
    };

    for line in northstar_md.lines() {
        // Detect blueprint section
        if line.starts_with("## Published Tree Blueprint") {
            in_blueprint = true;
            continue;
        }
        // Next `##` section ends the blueprint
        if in_blueprint && line.starts_with("## ") {
            break;
        }
        if !in_blueprint { continue; }

        if line.starts_with("### ") {
            // Flush previous subtree into current tree
            if let Some(mut sub) = current_sub.take() {
                sub.description_html = flush_desc(&mut desc_lines);
                if let Some(ref mut t) = current_tree { t.subtrees.push(sub); }
            } else {
                // Flush description into current tree
                let html = flush_desc(&mut desc_lines);
                if let Some(ref mut t) = current_tree { t.description_html = html; }
            }
            // Flush current tree
            if let Some(t) = current_tree.take() {
                trees.push(t);
            }
            let title = line[4..].trim().to_string();
            let slug = title.to_lowercase().replace(' ', "-");
            current_tree = Some(TreeNode { title, slug, ..Default::default() });
        } else if line.starts_with("#### ") {
            // Flush previous subtree
            if let Some(mut sub) = current_sub.take() {
                sub.description_html = flush_desc(&mut desc_lines);
                if let Some(ref mut t) = current_tree { t.subtrees.push(sub); }
            } else {
                flush_desc(&mut desc_lines); // discard — pre-subtree desc already flushed or belongs to tree
            }
            let title = line[5..].trim().to_string();
            // "Technical / CSE" → "technical-cse", collapsing runs of non-alpha chars
            let slug: String = title.to_lowercase()
                .chars()
                .map(|c| if c.is_alphanumeric() { c } else { '-' })
                .collect::<String>()
                .split('-')
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
                .join("-");
            current_sub = Some(TreeNode { title, slug, ..Default::default() });
        } else {
            // Check for **Icon:** metadata line — extract icon value, don't add to desc
            if let Some(rest) = line.trim().strip_prefix("**Icon:**") {
                let icon_val = rest.trim().to_string();
                if let Some(ref mut sub) = current_sub {
                    sub.icon = Some(icon_val);
                } else if let Some(ref mut t) = current_tree {
                    t.icon = Some(icon_val);
                }
            } else {
                desc_lines.push(line.to_string());
            }
        }
    }
    // Flush trailing
    if let Some(mut sub) = current_sub.take() {
        sub.description_html = flush_desc(&mut desc_lines);
        if let Some(ref mut t) = current_tree { t.subtrees.push(sub); }
    } else {
        let html = flush_desc(&mut desc_lines);
        if let Some(ref mut t) = current_tree { t.description_html = html; }
    }
    if let Some(t) = current_tree.take() {
        trees.push(t);
    }
    trees
}
