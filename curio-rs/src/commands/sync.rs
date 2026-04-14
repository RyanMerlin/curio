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
    config::{Config, upsert_repo_env_var},
    confluence::ConfluenceClient,
    northstar::{load_taxonomy, NorthstarTaxonomy, TaxonomyNode},
    output::emit_json,
    proposal::load_proposal_record,
    quality::assess_quality,
    reconcile::RoutingAnalysis,
    wiki_fs::{content_hash, parse_wiki_page, strip_frontmatter},
    wiki_index::append_log,
};

const SYNC_PROP_KEY: &str = "curio-sync";
const ICON_PROP_KEY: &str = "emoji-title-published";
pub const CURIO_ROOT_TITLE: &str = "CURIO";
const CURIO_HERO_FILENAME: &str = "Curio_curated_intelligence_operator.png";
const REQUIRED_CURIO_CHILDREN: &[&str] = &["Published", "Intake", "Staged", "Review", "Config"];
const REQUIRED_CONFIG_CHILDREN: &[&str] = &["Northstar", "CURIO Readme", "Settings"];

#[derive(Debug, Clone)]
pub struct CurioConfluenceTree {
    pub root_id: String,
    pub staged_id: String,
    pub review_id: String,
    pub published_id: String,
    pub config_id: String,
}

#[derive(Debug, Clone)]
pub struct CurioTreeValidation {
    pub root_id: String,
    pub checked_pages: usize,
}

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
        "accounts" | "account-tree" => Some("1f3e2"),   // 🏢
        "product-tree"              => Some("1f4e6"),   // 📦
        "audience-tree"             => Some("1f465"),   // 👥
        "use-case-tree"             => Some("1f527"),   // 🔧
        "topic-tree"                => Some("1f4da"),   // 📚
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

    let wiki_dir = &config.wiki.wiki_dir;
    let published_dir = wiki_dir.join("published");
    if !published_dir.exists() {
        anyhow::bail!("wiki/published/ not found. Run `curio init` first.");
    }

    let space_key = &config.content_model.space_key;
    let token = std::env::var("CURIO_CONFLUENCE_TOKEN")
        .context("CURIO_CONFLUENCE_TOKEN not set")?;
    let bootstrap_client = ConfluenceClient::new(
        config.connection.confluence_url.clone(),
        config.connection.confluence_email.clone(),
        token.clone(),
        None,
    )?;
    let tree = ensure_curio_confluence_tree(
        config,
        &bootstrap_client,
        parent_page_id_override
            .or_else(|| config.wiki.sync.confluence_parent_page_id.clone()),
        !dry_run,
    )
    .await?;
    let client = ConfluenceClient::new(
        config.connection.confluence_url.clone(),
        config.connection.confluence_email.clone(),
        token,
        Some(tree.root_id.clone()),
    )?;

    let mut upserted: Vec<String> = Vec::new();
    let mut skipped: Vec<String> = Vec::new();
    let mut errors: Vec<String> = Vec::new();
    let mut synced_page_ids: HashSet<String> = HashSet::new();
    // Sync top-level config pages first (settings, northstar, readme) under CURIO/Config
    let config_dir = wiki_dir.join("_config");
    if config_dir.exists() && !dry_run {
        // Sort entries so config is processed first (alphabetically: config < northstar < readme)
        let mut config_entries: Vec<_> = std::fs::read_dir(&config_dir)
            .into_iter().flatten().filter_map(|e| e.ok()).collect();
        config_entries.sort_by_key(|e| e.file_name());

        for entry in config_entries {
            let path = entry.path();
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if ext != "md" && ext != "yaml" && ext != "yml" { continue; }
            let stem = path.file_stem().unwrap_or_default().to_str().unwrap_or("");
            if stem.starts_with('_') { continue; }

            let page_title = config_page_title(stem);

            let raw = std::fs::read_to_string(&path)
                .with_context(|| format!("Failed to read {}", path.display()))?;
            let body_md = if ext == "md" { strip_frontmatter(&raw) } else { raw.as_str() };
            let html_body = if stem == "northstar" {
                let trees = taxonomy_to_tree_nodes(&load_taxonomy(wiki_dir)?);
                render_northstar_for_confluence(body_md, &trees)
            } else if ext == "md" {
                markdown_to_html(body_md)
            } else {
                format!("<pre><code>{}</code></pre>", body_md)
            };
            let hash = content_hash(body_md);

            let existing = client
                .get_page_by_title(space_key, Some(tree.config_id.as_str()), &page_title)
                .await
                .ok()
                .flatten();

            if let Some(ref page) = existing {
                let page_id = page["id"].as_str().unwrap_or_default().to_string();
                let page_v2 = client.get_page_by_id_v2(&page_id).await.ok().flatten();
                let current_title = page_v2.as_ref()
                    .and_then(|p| p["title"].as_str())
                    .unwrap_or("");
                let title_matches = current_title == page_title;
                if title_matches {
                    if let Ok(Some(prop)) = client.get_content_property(&page_id, SYNC_PROP_KEY).await {
                        if prop["value"]["content_hash"].as_str() == Some(hash.as_str()) {
                            skipped.push(format!("[config] {}", page_title));
                            synced_page_ids.insert(page_id);
                            continue;
                        }
                    }
                }
            }

            match client.create_or_update_page(space_key, Some(tree.config_id.as_str()), &page_title, "storage", &html_body).await {
                Ok(page_id) => {
                    let _ = set_sync_prop(&client, &page_id, &hash).await;
                    set_page_icon(&client, &page_id, stem).await;
                    synced_page_ids.insert(page_id);
                    upserted.push(format!("[config] {}", page_title));
                }
                Err(e) => errors.push(format!("[config] {}: {}", page_title, e)),
            }
        }
    }

    let taxonomy = load_taxonomy(wiki_dir)?;
    let northstar_trees: Vec<TreeNode> = taxonomy_to_tree_nodes(&taxonomy);
    validate_published_sync_inputs(&published_dir, &northstar_trees)?;
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

    // Map from relative directory path → confluence page id (root = CURIO/Published).
    let mut dir_to_page_id: HashMap<PathBuf, Option<String>> = HashMap::new();
    dir_to_page_id.insert(PathBuf::new(), Some(tree.published_id.clone()));

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
            .unwrap_or_else(|| Some(tree.published_id.clone()));

        if *is_dir {
            // Look up NORTHSTAR title + description + icon for this dir slug
            let depth = rel_path.components().count();
            let (page_title, dir_body, ns_icon) = if depth == 1 {
                // Top-level tree dir: use NORTHSTAR title, description, and icon
                if let Some((ns_title, ns_desc, icon, _)) = tree_info.get(&name) {
                    let page_title = ns_title.clone();
                    let body = render_branch_page_body(&abs_path, ns_desc, &page_title)?;
                    (page_title, body, icon.clone())
                } else {
                    let fallback_title = to_title(&name);
                    let body = render_branch_page_body(&abs_path, "", &fallback_title)?;
                    (fallback_title, body, None)
                }
            } else if depth == 2 {
                // Subtree dir: look up in parent's subtree list
                let parent_name = parent_rel.file_name()
                    .and_then(|n| n.to_str()).unwrap_or("");
                let sub_info = tree_info.get(parent_name)
                    .and_then(|(_, _, _, subs)| subs.get(&name))
                    .map(|(t, d, i)| (t.clone(), d.clone(), i.clone()));
                if let Some((sub_title, sub_desc, sub_icon)) = sub_info {
                    let body = render_branch_page_body(&abs_path, &sub_desc, &sub_title)?;
                    (sub_title, body, sub_icon)
                } else {
                    let fallback_title = to_title(&name);
                    let body = render_branch_page_body(&abs_path, "", &fallback_title)?;
                    (fallback_title, body, None)
                }
            } else {
                let fallback_title = to_title(&name);
                let body = render_branch_page_body(&abs_path, "", &fallback_title)?;
                (fallback_title, body, None)
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
            if dry_run {
                let page_title = published_page_title(&abs_path)?;
                upserted.push(format!("[page] {}", page_title));
            } else {
                match sync_page(
                    &client, space_key, parent_conf_id.as_deref(), &abs_path,
                    &mut synced_page_ids, &mut skipped,
                )
                .await
                {
                    Ok(page_title) => upserted.push(format!("[page] {}", page_title)),
                    Err(e) => errors.push(format!("[page] {}: {}", abs_path.display(), e)),
                }
            }
        }
    }

    sync_lane_directory(
        &client,
        space_key,
        &wiki_dir.join("staged"),
        tree.staged_id.as_str(),
        "staged",
        &mut synced_page_ids,
        &mut skipped,
        &mut upserted,
        &mut errors,
    )
    .await?;
    sync_lane_directory(
        &client,
        space_key,
        &wiki_dir.join("review"),
        tree.review_id.as_str(),
        "review",
        &mut synced_page_ids,
        &mut skipped,
        &mut upserted,
        &mut errors,
    )
    .await?;
    let proposals_dir = wiki_dir.join("_config").join("sharpening-proposals");
    sync_review_proposals(
        &client,
        space_key,
        &proposals_dir,
        tree.review_id.as_str(),
        &mut synced_page_ids,
        &mut skipped,
        &mut upserted,
        &mut errors,
    )
    .await?;

    // Taxonomy reconciliation index: group all review pages that propose the same
    // new subtree into a single summary page so reviewers can approve or reject as
    // a coherent batch rather than visiting each proposal in isolation.
    sync_taxonomy_reconciliation_index(
        &client,
        space_key,
        &wiki_dir.join("review"),
        tree.review_id.as_str(),
        &mut synced_page_ids,
        &mut upserted,
        &mut errors,
    )
    .await?;

    // Delete stale pages
    if !dry_run {
        let mut stale_deleted = 0usize;
        for root_id in [&tree.published_id, &tree.staged_id, &tree.review_id] {
            let stale =
                find_stale_pages(&client, Some(root_id.as_str()), &synced_page_ids).await?;
            for page_id in &stale {
                match client.delete_page(page_id).await {
                    Ok(()) => {
                        stale_deleted += 1;
                        upserted.push(format!("[deleted] {}", page_id))
                    }
                    Err(e) => errors.push(format!("delete {} failed: {}", page_id, e)),
                }
            }
        }

        let legacy_pages =
            find_legacy_sync_pages(&client, tree.published_id.as_str(), &northstar_trees).await?;
        for page_id in &legacy_pages {
            match client.delete_page(page_id).await {
                Ok(()) => upserted.push(format!("[deleted legacy] {}", page_id)),
                Err(e) => errors.push(format!("delete legacy {} failed: {}", page_id, e)),
            }
        }

        append_log(
            wiki_dir,
            &format!(
                "sync: {} upserted, {} skipped, {} stale deleted, {} legacy deleted, {} errors",
                upserted.len(),
                skipped.len(),
                stale_deleted,
                legacy_pages.len(),
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

pub async fn ensure_curio_confluence_tree(
    config: &Config,
    client: &ConfluenceClient,
    preferred_root_id: Option<String>,
    persist_root_id: bool,
) -> Result<CurioConfluenceTree> {
    let space_key = config.content_model.space_key.as_str();
    let root_id = ensure_curio_root_page(config, client, space_key, preferred_root_id.as_deref()).await?;

    let published_id = upsert_static_page(
        client,
        space_key,
        Some(root_id.as_str()),
        "Published",
        &published_root_body(),
        "published",
        Some(page_icon("published").unwrap_or("atlassian-check_mark")),
    )
    .await?;
    let _intake_id = upsert_static_page(
        client,
        space_key,
        Some(root_id.as_str()),
        "Intake",
        &pipeline_body(
            "Incoming content captured for processing.",
            "Items here are awaiting analysis and routing into the wiki.",
        ),
        "intake",
        None,
    )
    .await?;
    let staged_id = upsert_static_page(
        client,
        space_key,
        Some(root_id.as_str()),
        "Staged",
        &pipeline_body(
            "Content analyzed and routed, awaiting human review or approval before publishing.",
            "Manage staged content with <code>curio resolve</code> and <code>curio publish</code>.",
        ),
        "staged",
        None,
    )
    .await?;
    let review_id = upsert_static_page(
        client,
        space_key,
        Some(root_id.as_str()),
        "Review",
        &pipeline_body(
            "Items that need human judgment before they can move forward in the pipeline.",
            "Use <code>curio review</code> to inspect them and <code>curio resolve</code> to route them.",
        ),
        "review",
        None,
    )
    .await?;
    let config_id = upsert_static_page(
        client,
        space_key,
        Some(root_id.as_str()),
        "Config",
        &config_root_body(),
        "config",
        None,
    )
    .await?;

    let config_dir = config.wiki.wiki_dir.join("_config");
    if config_dir.exists() {
        let mut config_entries: Vec<_> = std::fs::read_dir(&config_dir)
            .into_iter()
            .flatten()
            .filter_map(|entry| entry.ok())
            .collect();
        config_entries.sort_by_key(|entry| entry.file_name());
        for entry in config_entries {
            let path = entry.path();
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if ext != "md" && ext != "yaml" && ext != "yml" {
                continue;
            }
            let stem = path.file_stem().unwrap_or_default().to_str().unwrap_or("");
            if stem.starts_with('_') {
                continue;
            }

            let raw = std::fs::read_to_string(&path)
                .with_context(|| format!("Failed to read {}", path.display()))?;
            let body_md = if ext == "md" { strip_frontmatter(&raw) } else { raw.as_str() };
            let html_body = if stem == "northstar" {
                let trees = taxonomy_to_tree_nodes(&load_taxonomy(&config.wiki.wiki_dir)?);
                render_northstar_for_confluence(body_md, &trees)
            } else if ext == "md" {
                markdown_to_html(body_md)
            } else {
                format!("<pre><code>{}</code></pre>", html_escape(body_md))
            };
            let page_title = config_page_title(stem);
            let _ = upsert_static_page(
                client,
                space_key,
                Some(config_id.as_str()),
                &page_title,
                &html_body,
                stem,
                None,
            )
            .await?;
        }
    }

    let taxonomy = load_taxonomy(&config.wiki.wiki_dir)?;
    let trees = taxonomy_to_tree_nodes(&taxonomy);
    for tree in &trees {
        let body = render_branch_summary_only(&tree.title, &tree.description_html, &tree.subtrees);
        let _ = upsert_static_page(
            client,
            space_key,
            Some(published_id.as_str()),
            &tree.title,
            &body,
            &tree.slug,
            tree.icon.as_deref(),
        )
        .await?;
    }

    upload_root_hero(config, client, &root_id).await?;
    let root_body = curio_root_body(space_key);
    let root_page_id = upsert_static_page(
        client,
        space_key,
        None,
        CURIO_ROOT_TITLE,
        &root_body,
        "curio",
        None,
    )
    .await?;

    if persist_root_id {
        upsert_repo_env_var("CURIO_CONFLUENCE_PARENT_PAGE_ID", &root_page_id)?;
    }

    Ok(CurioConfluenceTree {
        root_id: root_page_id,
        staged_id,
        review_id,
        published_id,
        config_id,
    })
}

pub async fn reset_curio_confluence_tree(
    config: &Config,
    client: &ConfluenceClient,
    preferred_root_id: Option<String>,
    persist_root_id: bool,
) -> Result<(CurioConfluenceTree, usize)> {
    let space_key = config.content_model.space_key.as_str();
    let root_id =
        ensure_curio_root_page(config, client, space_key, preferred_root_id.as_deref()).await?;
    let descendants = client.get_page_descendants_v2(&root_id).await.unwrap_or_default();

    let mut pages_to_delete: Vec<(String, usize)> = descendants
        .into_iter()
        .filter_map(|page| {
            let page_id = page["id"].as_str()?.to_string();
            let parent_id = page["parentId"].as_str().unwrap_or_default().to_string();
            let depth = descendant_depth(&page, &root_id, &parent_id);
            Some((page_id, depth))
        })
        .collect();

    pages_to_delete.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

    let mut deleted = 0usize;
    for (page_id, _) in pages_to_delete {
        client.delete_page(&page_id).await?;
        deleted += 1;
    }

    let tree = ensure_curio_confluence_tree(config, client, Some(root_id), persist_root_id).await?;
    Ok((tree, deleted))
}

pub async fn validate_curio_confluence_tree(
    config: &Config,
    client: &ConfluenceClient,
    preferred_root_id: Option<String>,
) -> Result<CurioTreeValidation> {
    let space_key = config.content_model.space_key.as_str();
    let expected_space_id = client.get_numeric_space_id(space_key).await?;
    let root_id =
        ensure_curio_root_page(config, client, space_key, preferred_root_id.as_deref()).await?;
    let root_page = client
        .get_page_by_id_v2(&root_id)
        .await?
        .context("CURIO root page missing during validation")?;

    let root_title = root_page["title"].as_str().unwrap_or_default();
    if root_title != CURIO_ROOT_TITLE {
        anyhow::bail!(
            "CURIO root validation failed: expected title '{}' but found '{}'",
            CURIO_ROOT_TITLE,
            root_title
        );
    }
    let root_space_id = root_page["spaceId"].as_str().unwrap_or_default();
    if root_space_id != expected_space_id {
        anyhow::bail!(
            "CURIO root validation failed: root page {} is in space {} instead of {}",
            root_id,
            root_space_id,
            expected_space_id
        );
    }
    validate_page_body_loaded(client, &root_id, CURIO_ROOT_TITLE).await?;
    if client
        .get_attachment_by_filename(&root_id, CURIO_HERO_FILENAME)
        .await?
        .is_none()
    {
        anyhow::bail!(
            "CURIO root validation failed: hero attachment '{}' is missing on page {}",
            CURIO_HERO_FILENAME,
            root_id
        );
    }

    let root_descendants = client.get_page_descendants_v2(&root_id).await?;
    let root_children = direct_children_from_descendants(&root_descendants, &root_id);
    let root_children_by_title = map_children_by_title(&root_children);
    let root_titles: HashSet<String> = root_children_by_title.keys().cloned().collect();
    let expected_root_titles: HashSet<String> =
        REQUIRED_CURIO_CHILDREN.iter().map(|title| title.to_string()).collect();
    if root_titles != expected_root_titles {
        anyhow::bail!(
            "CURIO root validation failed: direct children were {:?}, expected {:?}",
            sorted_titles(&root_titles),
            sorted_titles(&expected_root_titles)
        );
    }

    let mut checked_pages = 1usize;
    let config_id = root_children_by_title
        .get("Config")
        .cloned()
        .context("CURIO validation failed: Config page missing")?;

    for title in REQUIRED_CURIO_CHILDREN {
        let page_id = root_children_by_title
            .get(*title)
            .cloned()
            .with_context(|| format!("CURIO validation failed: missing direct child '{}'", title))?;
        let page = client
            .get_page_by_id_v2(&page_id)
            .await?
            .with_context(|| format!("CURIO validation failed: child page '{}' missing", title))?;
        if page["parentId"].as_str() != Some(root_id.as_str()) {
            anyhow::bail!(
                "CURIO validation failed: '{}' is not a direct child of CURIO",
                title
            );
        }
        validate_page_body_loaded(client, &page_id, title).await?;
        checked_pages += 1;
    }

    let config_children = direct_children_from_descendants(&root_descendants, &config_id);
    let config_children_by_title = map_children_by_title(&config_children);
    let config_titles: HashSet<String> = config_children_by_title.keys().cloned().collect();
    let expected_config_titles: HashSet<String> =
        REQUIRED_CONFIG_CHILDREN.iter().map(|title| title.to_string()).collect();
    if config_titles != expected_config_titles {
        anyhow::bail!(
            "CURIO validation failed: Config children were {:?}, expected {:?}",
            sorted_titles(&config_titles),
            sorted_titles(&expected_config_titles)
        );
    }

    for title in REQUIRED_CONFIG_CHILDREN {
        let page_id = config_children_by_title
            .get(*title)
            .cloned()
            .with_context(|| format!("CURIO validation failed: missing Config child '{}'", title))?;
        let page = client
            .get_page_by_id_v2(&page_id)
            .await?
            .with_context(|| format!("CURIO validation failed: Config child '{}' missing", title))?;
        if page["parentId"].as_str() != Some(config_id.as_str()) {
            anyhow::bail!(
                "CURIO validation failed: Config child '{}' is not under Config",
                title
            );
        }
        validate_page_body_loaded(client, &page_id, title).await?;
        checked_pages += 1;
    }

    Ok(CurioTreeValidation { root_id, checked_pages })
}


async fn ensure_curio_root_page(
    config: &Config,
    client: &ConfluenceClient,
    space_key: &str,
    preferred_root_id: Option<&str>,
) -> Result<String> {
    let expected_space_id = client.get_numeric_space_id(space_key).await?;
    if let Some(root_id) = preferred_root_id {
        if let Some(page) = client.get_page_by_id_v2(root_id).await? {
            let title_matches = page["title"].as_str() == Some(CURIO_ROOT_TITLE);
            let space_matches = page["spaceId"].as_str() == Some(expected_space_id.as_str());
            let status_current = page["status"].as_str() == Some("current");
            if title_matches && space_matches && status_current {
                return Ok(root_id.to_string());
            }
        }
    }

    if let Some(page) = client.get_page_by_title(space_key, None, CURIO_ROOT_TITLE).await? {
        return Ok(page["id"].as_str().unwrap_or_default().to_string());
    }

    let root_id = client
        .create_or_update_page(
            space_key,
            None,
            CURIO_ROOT_TITLE,
            "storage",
            "<p>Initializing CURIO workspace…</p>",
        )
        .await?;

    let _ = upload_root_hero(config, client, &root_id).await;
    Ok(root_id)
}

fn descendant_depth(page: &serde_json::Value, root_id: &str, parent_id: &str) -> usize {
    if parent_id == root_id {
        return 1;
    }
    page["depth"].as_u64().unwrap_or(1) as usize
}

fn map_children_by_title(children: &[serde_json::Value]) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for child in children {
        let title = child["title"].as_str().unwrap_or_default().to_string();
        let id = child["id"].as_str().unwrap_or_default().to_string();
        if !title.is_empty() && !id.is_empty() {
            map.insert(title, id);
        }
    }
    map
}

fn direct_children_from_descendants(
    descendants: &[serde_json::Value],
    parent_id: &str,
) -> Vec<serde_json::Value> {
    descendants
        .iter()
        .filter(|page| page["parentId"].as_str() == Some(parent_id))
        .cloned()
        .collect()
}

fn sorted_titles(titles: &HashSet<String>) -> Vec<String> {
    let mut values: Vec<String> = titles.iter().cloned().collect();
    values.sort();
    values
}

async fn validate_page_body_loaded(
    client: &ConfluenceClient,
    page_id: &str,
    label: &str,
) -> Result<()> {
    let page = client
        .get_page_by_id_with_body_v1(page_id)
        .await?
        .with_context(|| format!("Failed to load page body for '{}'", label))?;
    let body = page["body"]["storage"]["value"].as_str().unwrap_or_default();
    let text = strip_html_tags(body);
    if text.trim().len() < 20 {
        anyhow::bail!(
            "CURIO validation failed: page '{}' has no meaningful body content",
            label
        );
    }
    Ok(())
}

fn strip_html_tags(html: &str) -> String {
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

async fn upload_root_hero(config: &Config, client: &ConfluenceClient, root_id: &str) -> Result<()> {
    let repo_root = config
        .wiki
        .wiki_dir
        .parent()
        .map(PathBuf::from)
        .unwrap_or_else(|| crate::config::repo_root());
    let hero_path = repo_root.join("docs").join("assets").join(CURIO_HERO_FILENAME);
    if !hero_path.exists() {
        return Ok(());
    }
    let bytes = std::fs::read(&hero_path)
        .with_context(|| format!("Failed to read hero image: {}", hero_path.display()))?;
    client
        .upload_attachment(root_id, CURIO_HERO_FILENAME, "image/png", bytes)
        .await
}

fn curio_root_body(space_key: &str) -> String {
    format!(
        concat!(
            "<ac:image ac:layout=\"center\" ac:width=\"230\"><ri:attachment ri:filename=\"{hero}\" /></ac:image>",
            "<h1>CURIO</h1>",
            "<p>Curated Intelligence Operator is the Confluence-facing workspace for intake, review, publication, and operating configuration.</p>",
            "<ac:structured-macro ac:name=\"info\"><ac:rich-text-body>",
            "<p>Use CURIO as the landing page for teams who need a clear view of what is published, what is in-flight, and how the workspace is configured.</p>",
            "</ac:rich-text-body></ac:structured-macro>",
            "<table><tbody>",
            "<tr><th>Section</th><th>Purpose</th></tr>",
            "<tr><td>{published}</td><td>Published knowledge organized into explicit tree structures.</td></tr>",
            "<tr><td>{intake}</td><td>Fresh source material collected for processing.</td></tr>",
            "<tr><td>{staged}</td><td>Routed content awaiting final publication.</td></tr>",
            "<tr><td>{review}</td><td>Items needing human judgment.</td></tr>",
            "<tr><td>{config}</td><td>Northstar, readme, and runtime settings that define the workspace.</td></tr>",
            "</tbody></table>"
        ),
        hero = CURIO_HERO_FILENAME,
        published = page_link(space_key, "Published", "Published"),
        intake = page_link(space_key, "Intake", "Intake"),
        staged = page_link(space_key, "Staged", "Staged"),
        review = page_link(space_key, "Review", "Review"),
        config = page_link(space_key, "Config", "Config"),
    )
}

fn published_root_body() -> String {
    "<p>Published knowledge from the Curio workspace. Tree pages below mirror the filesystem and NORTHSTAR blueprint.</p>".to_string()
}

fn config_root_body() -> String {
    "<p>Configuration and workspace reference pages for Curio. This branch is machine-maintained from <code>wiki/_config</code>.</p>".to_string()
}

fn pipeline_body(summary: &str, detail: &str) -> String {
    format!(
        "<p>{}</p><p>{}</p><p>This page is maintained automatically by Curio.</p>",
        summary, detail
    )
}

fn render_branch_summary_only(title: &str, description_html: &str, children: &[TreeNode]) -> String {
    let mut body = String::new();
    body.push_str(&format!("<h1>{}</h1>", html_escape(title)));
    if !description_html.trim().is_empty() {
        body.push_str(description_html);
    }
    if !children.is_empty() {
        body.push_str("<h2>Child Sections</h2><ul>");
        for child in children {
            body.push_str(&format!(
                "<li><strong>{}</strong>{}</li>",
                html_escape(&child.title),
                if child.description_html.trim().is_empty() {
                    String::new()
                } else {
                    format!(" — {}", html_escape(&inline_desc(&child.description_html)))
                }
            ));
        }
        body.push_str("</ul>");
    }
    body
}

fn render_branch_page_body(dir_path: &Path, fallback_description_html: &str, title: &str) -> Result<String> {
    let index_path = dir_path.join("index.md");
    if index_path.exists() {
        let raw = std::fs::read_to_string(&index_path)
            .with_context(|| format!("Failed to read {}", index_path.display()))?;
        let stripped = strip_frontmatter(&raw);
        let html = markdown_to_html(stripped);
        if !html.trim().is_empty() {
            return Ok(html);
        }
    }
    Ok(render_branch_summary_only(title, fallback_description_html, &[]))
}

fn page_link(space_key: &str, title: &str, label: &str) -> String {
    format!(
        "<ac:link><ri:page ri:space-key=\"{}\" ri:content-title=\"{}\" /><ac:plain-text-link-body><![CDATA[{}]]></ac:plain-text-link-body></ac:link>",
        html_escape(space_key),
        html_escape(title),
        label
    )
}

fn config_page_title(stem: &str) -> String {
    match stem {
        "readme" => "CURIO Readme".to_string(),
        "settings" => "Settings".to_string(),
        _ => to_title(stem),
    }
}

async fn upsert_static_page(
    client: &ConfluenceClient,
    space_key: &str,
    parent_id: Option<&str>,
    title: &str,
    body: &str,
    slug: &str,
    icon_override: Option<&str>,
) -> Result<String> {
    let hash = content_hash(body);
    if let Some(page) = find_existing_page_for_sync(client, space_key, parent_id, title).await? {
        let page_id = page["id"].as_str().unwrap_or_default().to_string();
        if let Some(target_parent_id) = parent_id {
            if let Some(current_page) = client.get_page_by_id_v2(&page_id).await? {
                let current_parent_id = current_page["parentId"].as_str();
                if current_parent_id != Some(target_parent_id) {
                    let _ = client.migrate_page_to_parent(&page_id, target_parent_id).await;
                }
            }
        }
        if let Ok(Some(prop)) = client.get_content_property(&page_id, SYNC_PROP_KEY).await {
            if prop["value"]["content_hash"].as_str() == Some(hash.as_str()) {
                if let Some(icon_val) = icon_override {
                    set_page_icon_value(client, &page_id, icon_val).await;
                } else {
                    set_page_icon(client, &page_id, slug).await;
                }
                return Ok(page_id);
            }
        }

        client
            .update_page_body_by_id(&page_id, "storage", body)
            .await?;
        set_sync_prop(client, &page_id, &hash).await?;
        if let Some(icon_val) = icon_override {
            set_page_icon_value(client, &page_id, icon_val).await;
        } else {
            set_page_icon(client, &page_id, slug).await;
        }
        return Ok(page_id);
    }

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

// ─── Entry collection ─────────────────────────────────────────────────────

/// Return all entries under `root` as (relative_path, is_dir) in sorted BFS order.
/// Excludes:
///   - `.analysis.json` sidecars (machine provenance, never synced to Confluence)
///   - `.gitkeep` placeholder files
fn collect_sorted_entries(root: &Path) -> Result<Vec<(PathBuf, bool)>> {
    use walkdir::WalkDir;
    let mut entries = Vec::new();
    for e in WalkDir::new(root)
        .sort_by_file_name()
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path() != root)
    {
        let path = e.path();

        // Skip analysis sidecars and git placeholders
        if e.file_type().is_file() {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            if name == ".gitkeep" {
                continue;
            }
            if name == "index.md" {
                continue;
            }
            // Skip *.analysis.json — machine provenance, never synced
            if name.ends_with(".analysis.json") || name.ends_with(".proposal.json") {
                continue;
            }
        }

        let rel = path
            .strip_prefix(root)
            .map(PathBuf::from)
            .unwrap_or_default();
        if rel
            .components()
            .next()
            .and_then(|c| c.as_os_str().to_str())
            == Some("uncategorized")
        {
            continue;
        }
        entries.push((rel, e.file_type().is_dir()));
    }
    Ok(entries)
}

fn validate_published_sync_inputs(root: &Path, trees: &[TreeNode]) -> Result<()> {
    use walkdir::WalkDir;

    let valid_routes = valid_published_routes(trees);
    let mut errors = Vec::new();

    for entry in WalkDir::new(root).into_iter().filter_map(|entry| entry.ok()) {
        let path = entry.path();
        if !entry.file_type().is_file() || path.extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }

        let name = path.file_name().unwrap_or_default().to_string_lossy();
        if name == "index.md" || name == ".gitkeep" || name.ends_with(".analysis.json") || name.ends_with(".proposal.json") {
            continue;
        }

        let rel = path.strip_prefix(root).unwrap_or(path);
        if rel
            .components()
            .next()
            .and_then(|component| component.as_os_str().to_str())
            == Some("uncategorized")
        {
            errors.push(format!(
                "{} is under published/uncategorized, which is invalid. Send it to review with a subtree proposal instead.",
                rel.display()
            ));
            continue;
        }

        let page = parse_wiki_page(path)
            .with_context(|| format!("Failed to parse {}", path.display()))?;
        if page.frontmatter.title.trim().is_empty() {
            errors.push(format!("{} is missing a frontmatter title", rel.display()));
        }
        if page.frontmatter.category.is_empty() {
            errors.push(format!(
                "{} has no category. Published content must already be curated into a NORTHSTAR route.",
                rel.display()
            ));
            continue;
        }

        let route = page.frontmatter.category.join("/");
        if !valid_routes.contains(&route) {
            errors.push(format!(
                "{} uses invalid published route '{}'. Update NORTHSTAR or move the page back to review.",
                rel.display(),
                route
            ));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        anyhow::bail!("Published sync validation failed:\n- {}", errors.join("\n- "));
    }
}

fn valid_published_routes(trees: &[TreeNode]) -> HashSet<String> {
    let mut routes = HashSet::new();
    for tree in trees {
        routes.insert(tree.slug.clone());
        for subtree in &tree.subtrees {
            routes.insert(format!("{}/{}", tree.slug, subtree.slug));
        }
    }
    routes
}

async fn sync_lane_directory(
    client: &ConfluenceClient,
    space_key: &str,
    root_dir: &Path,
    root_page_id: &str,
    lane: &str,
    synced_ids: &mut HashSet<String>,
    skipped: &mut Vec<String>,
    upserted: &mut Vec<String>,
    errors: &mut Vec<String>,
) -> Result<()> {
    if !root_dir.exists() {
        return Ok(());
    }

    let mut dir_to_page_id: HashMap<PathBuf, Option<String>> = HashMap::new();
    dir_to_page_id.insert(PathBuf::new(), Some(root_page_id.to_string()));

    for (rel_path, is_dir) in collect_sorted_entries(root_dir)? {
        let abs_path = root_dir.join(&rel_path);
        let name = rel_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        if name.starts_with('.') || name.starts_with('_') {
            continue;
        }

        let parent_rel = rel_path.parent().map(PathBuf::from).unwrap_or_default();
        let parent_conf_id = dir_to_page_id
            .get(&parent_rel)
            .cloned()
            .unwrap_or_else(|| Some(root_page_id.to_string()));

        if is_dir {
            if !subtree_has_markdown(&abs_path) {
                continue;
            }
            let page_title = lane_display_title(lane, &rel_path);
            let branch_body = render_lane_branch_body(root_dir, &rel_path, lane)?;
            match upsert_page(
                client,
                space_key,
                parent_conf_id.as_deref(),
                &page_title,
                &branch_body,
                &name,
                None,
            )
            .await
            {
                Ok(id) => {
                    synced_ids.insert(id.clone());
                    dir_to_page_id.insert(rel_path, Some(id));
                    upserted.push(format!("[{} dir] {}", lane, page_title));
                }
                Err(e) => errors.push(format!("[{} dir] {}: {}", lane, page_title, e)),
            }
        } else if rel_path.extension().map_or(false, |ext| ext == "md") {
            match sync_lane_page(
                client,
                space_key,
                parent_conf_id.as_deref(),
                &abs_path,
                lane,
                synced_ids,
                skipped,
            )
            .await
            {
                Ok(title) => upserted.push(format!("[{} page] {}", lane, title)),
                Err(e) => errors.push(format!("[{} page] {}: {}", lane, abs_path.display(), e)),
            }
        }
    }

    Ok(())
}

async fn sync_review_proposals(
    client: &ConfluenceClient,
    space_key: &str,
    proposals_dir: &Path,
    parent_id: &str,
    synced_ids: &mut HashSet<String>,
    skipped: &mut Vec<String>,
    upserted: &mut Vec<String>,
    errors: &mut Vec<String>,
) -> Result<()> {
    if !proposals_dir.exists() {
        return Ok(());
    }

    let mut entries: Vec<_> = std::fs::read_dir(proposals_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().and_then(|ext| ext.to_str()) == Some("json"))
        .collect();
    entries.sort_by_key(|entry| entry.file_name());

    for entry in entries {
        let path = entry.path();
        match sync_proposal_page(client, space_key, Some(parent_id), &path, synced_ids, skipped).await {
            Ok(title) => upserted.push(format!("[review proposal] {}", title)),
            Err(e) => errors.push(format!("[review proposal] {}: {}", path.display(), e)),
        }
    }

    Ok(())
}

/// Build a Confluence "Taxonomy Reconciliation" index page that groups all review
/// pages proposing the same new subtree into one place.  This lets a reviewer
/// approve or reject a whole batch of related pages in a single decision rather
/// than visiting 20 individual proposals that each ask for the same new node.
async fn sync_taxonomy_reconciliation_index(
    client: &ConfluenceClient,
    space_key: &str,
    review_dir: &Path,
    parent_id: &str,
    synced_ids: &mut HashSet<String>,
    upserted: &mut Vec<String>,
    errors: &mut Vec<String>,
) -> Result<()> {
    if !review_dir.exists() {
        return Ok(());
    }

    // Collect all .analysis.json files under the review dir and group by proposed_new_subtree
    let mut groups: std::collections::BTreeMap<String, Vec<(String, String, f32)>> =
        std::collections::BTreeMap::new();

    for entry in walkdir::WalkDir::new(review_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().extension().and_then(|x| x.to_str()) == Some("json")
                && e.path().to_str().map_or(false, |s| s.ends_with(".analysis.json"))
        })
    {
        let path = entry.path();
        let Ok(raw) = std::fs::read_to_string(path) else { continue };
        let Ok(analysis) = serde_json::from_str::<serde_json::Value>(&raw) else { continue };
        let Some(subtree) = analysis["routing"]["proposed_new_subtree"].as_str() else { continue };
        let title = analysis["title"].as_str().unwrap_or("Untitled").to_string();
        let slug = analysis["slug"].as_str().unwrap_or("").to_string();
        let confidence = analysis["routing"]["confidence"].as_f64().unwrap_or(0.0) as f32;
        groups
            .entry(subtree.to_string())
            .or_default()
            .push((title, slug, confidence));
    }

    if groups.is_empty() {
        return Ok(());
    }

    // Build the index page body
    let mut body = String::new();
    body.push_str("<ac:structured-macro ac:name=\"info\"><ac:rich-text-body><p><strong>Taxonomy Reconciliation Index</strong> — pages grouped by the new subtree they propose. Approve or reject each group as a batch. Once a subtree is approved, add it to <code>NORTHSTAR.md</code> and re-process the pages.</p></ac:rich-text-body></ac:structured-macro>");
    body.push_str(&format!("<p><strong>{}</strong> proposed new subtrees across <strong>{}</strong> review items.</p>",
        groups.len(),
        groups.values().map(|v| v.len()).sum::<usize>()
    ));

    for (subtree, pages) in &mut groups {
        pages.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
        body.push_str(&format!("<h2>{}</h2>", html_escape(subtree)));
        body.push_str(&format!("<p>{} page(s) propose this subtree:</p>", pages.len()));
        body.push_str("<table><tbody><tr><th>Page</th><th>Confidence</th></tr>");
        for (title, _slug, confidence) in pages.iter() {
            body.push_str(&format!(
                "<tr><td>{}</td><td>{:.0}%</td></tr>",
                html_escape(title),
                confidence * 100.0,
            ));
        }
        body.push_str("</tbody></table>");
        body.push_str("<p><strong>Decision options:</strong></p><ul>");
        body.push_str("<li>✅ <strong>Approve:</strong> add this path to <code>NORTHSTAR.md</code> children, then move pages from review → staged</li>");
        body.push_str("<li>✏️ <strong>Reroute:</strong> pick an existing path for these pages instead and update their category</li>");
        body.push_str("<li>🗑️ <strong>Reject:</strong> mark pages as out-of-scope and archive</li>");
        body.push_str("</ul><hr/>");
    }

    let hash = content_hash(&body);
    let page_title = "Taxonomy Proposals — Reconciliation Index";
    match client
        .create_or_update_page(space_key, Some(parent_id), page_title, "storage", &body)
        .await
    {
        Ok(page_id) => {
            set_sync_prop(client, &page_id, &hash).await?;
            synced_ids.insert(page_id);
            upserted.push(format!("[taxonomy reconciliation index] {} groups", groups.len()));
        }
        Err(e) => errors.push(format!("[taxonomy reconciliation index]: {}", e)),
    }

    Ok(())
}

// ─── Page operations ──────────────────────────────────────────────────────

async fn sync_page(
    client: &ConfluenceClient,
    space_key: &str,
    parent_id: Option<&str>,
    path: &Path,
    synced_ids: &mut HashSet<String>,
    skipped: &mut Vec<String>,
) -> Result<String> {
    let page = parse_wiki_page(path)
        .with_context(|| format!("Failed to parse {}", path.display()))?;
    let page_title = page.frontmatter.title.trim().to_string();
    if page_title.is_empty() {
        anyhow::bail!("Page title is empty for {}", path.display());
    }
    let body_md = page.body.as_str();
    let hash = content_hash(body_md);
    let html_body = markdown_to_html(body_md);
    sync_page_html(
        client,
        space_key,
        parent_id,
        path,
        &page_title,
        &hash,
        &html_body,
        synced_ids,
        skipped,
        true,
    )
    .await
}

async fn sync_lane_page(
    client: &ConfluenceClient,
    space_key: &str,
    parent_id: Option<&str>,
    path: &Path,
    lane: &str,
    synced_ids: &mut HashSet<String>,
    skipped: &mut Vec<String>,
) -> Result<String> {
    let page = parse_wiki_page(path)
        .with_context(|| format!("Failed to parse {}", path.display()))?;
    let page_title = format!("{} - {}", to_title(lane), page.frontmatter.title.trim());
    if page_title.is_empty() {
        anyhow::bail!("Page title is empty for {}", path.display());
    }
    let html_body = render_lane_page_body(path, &page, lane)?;
    let hash = content_hash(&html_body);
    let result_title = sync_page_html(
        client,
        space_key,
        parent_id,
        path,
        &page_title,
        &hash,
        &html_body,
        synced_ids,
        skipped,
        true,
    )
    .await?;

    // For review-lane pages: persist the Confluence page ID in a .sync-refs.json sidecar
    // and post/update the single pinned reaction-instruction footer comment so reviewers
    // can signal approve/reject/rewrite without editing the page.
    if lane == "review" {
        if let Ok(Some(existing)) = client.get_page_by_title(space_key, parent_id, &page_title).await {
            if let Some(page_id) = existing["id"].as_str() {
                // Upsert the pinned comment ──────────────────────────────────────────
                let pinned_body = "<p><em>Curio review signals</em>: react \
                    \u{1F44D} to <strong>approve</strong>, \
                    \u{1F44E} to <strong>reject</strong>, \
                    \u{2753} to request a <strong>rewrite</strong>. \
                    Or apply labels <code>curio:approve</code> / <code>curio:reject</code> / <code>curio:rewrite</code>. \
                    Free-form comments are captured as reviewer feedback.</p>";

                // Check whether we already have a persisted pinned comment ID
                let refs_path = path.with_extension("sync-refs.json");
                let existing_refs: serde_json::Value = if refs_path.exists() {
                    std::fs::read_to_string(&refs_path)
                        .ok()
                        .and_then(|s| serde_json::from_str(&s).ok())
                        .unwrap_or(serde_json::json!({}))
                } else {
                    serde_json::json!({})
                };
                let pinned_comment_id = if let Some(existing_id) = existing_refs["pinned_comment_id"].as_str() {
                    // Try to update existing; if it fails (deleted), create a new one
                    match client.update_footer_comment(existing_id, pinned_body).await {
                        Ok(_) => existing_id.to_string(),
                        Err(_) => {
                            client.create_footer_comment(page_id, pinned_body).await
                                .unwrap_or_else(|_| String::new())
                        }
                    }
                } else {
                    client.create_footer_comment(page_id, pinned_body).await
                        .unwrap_or_else(|_| String::new())
                };

                let pinned_id_opt = if pinned_comment_id.is_empty() { None } else { Some(pinned_comment_id.as_str()) };
                write_sync_refs(path, page_id, pinned_id_opt);
            }
        }
    }

    Ok(result_title)
}

/// Write (or update) the .sync-refs.json sidecar next to the wiki page.
/// This persists the Confluence review page ID (and optionally the pinned
/// comment ID) so `curio feedback` can read labels/reactions without
/// performing expensive title lookups.
fn write_sync_refs(wiki_page_path: &Path, confluence_page_id: &str, pinned_comment_id: Option<&str>) {
    let refs_path = wiki_page_path.with_extension("sync-refs.json");
    // Merge with any existing refs so we don't overwrite the pinned_comment_id on a hash-skip update
    let mut refs_value: serde_json::Value = if refs_path.exists() {
        std::fs::read_to_string(&refs_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({})
    };
    refs_value["confluence_review_page_id"] = serde_json::json!(confluence_page_id);
    if let Some(comment_id) = pinned_comment_id {
        refs_value["pinned_comment_id"] = serde_json::json!(comment_id);
    }
    refs_value["updated_at"] = serde_json::json!(chrono::Utc::now().to_rfc3339());
    if let Ok(json) = serde_json::to_string_pretty(&refs_value) {
        let _ = std::fs::write(&refs_path, json);
    }
}

async fn sync_proposal_page(
    client: &ConfluenceClient,
    space_key: &str,
    parent_id: Option<&str>,
    path: &Path,
    synced_ids: &mut HashSet<String>,
    skipped: &mut Vec<String>,
) -> Result<String> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    let payload: serde_json::Value = serde_json::from_str(&raw)
        .with_context(|| format!("Failed to parse proposal JSON {}", path.display()))?;
    let title = format!(
        "Sharpening Proposal {}",
        path.file_stem().and_then(|s| s.to_str()).unwrap_or("untitled")
    );
    let html_body = render_proposal_body(&payload);
    let hash = content_hash(&raw);
    sync_page_html(
        client,
        space_key,
        parent_id,
        path,
        &title,
        &hash,
        &html_body,
        synced_ids,
        skipped,
        false,
    )
    .await
}

async fn sync_page_html(
    client: &ConfluenceClient,
    space_key: &str,
    parent_id: Option<&str>,
    path: &Path,
    page_title: &str,
    hash: &str,
    html_body: &str,
    synced_ids: &mut HashSet<String>,
    skipped: &mut Vec<String>,
    allow_duplicate_fallback: bool,
) -> Result<String> {

    // Check if page already exists and content hasn't changed
    if let Some(ref existing_page) = client.get_page_by_title(space_key, parent_id, page_title).await? {
        let page_id = existing_page["id"].as_str().unwrap_or_default().to_string();
        if let Ok(Some(prop)) = client.get_content_property(&page_id, SYNC_PROP_KEY).await {
            if prop["value"]["content_hash"].as_str() == Some(hash) {
                skipped.push(page_title.to_string());
                synced_ids.insert(page_id);
                return Ok(page_title.to_string());
            }
        }
        let slug = path.file_stem().unwrap_or_default().to_str().unwrap_or("");
        let page_id = client
            .create_or_update_page(space_key, parent_id, page_title, "storage", html_body)
            .await?;
        set_sync_prop(client, &page_id, hash).await?;
        set_page_icon(client, &page_id, slug).await;
        synced_ids.insert(page_id);
        return Ok(page_title.to_string());
    }

    if allow_duplicate_fallback {
        if let Some(conflicting_page) = client.get_page_by_title(space_key, None, page_title).await? {
            let conflicting_id = conflicting_page["id"].as_str().unwrap_or_default().to_string();
            let duplicate_title = format!("{} (dup)", page_title);
            let duplicate_html = duplicate_notice_body(client, &conflicting_id, page_title, html_body);
            if let Some(existing_dup_page) =
                find_existing_page_for_sync(client, space_key, parent_id, &duplicate_title).await?
            {
                let page_id = existing_dup_page["id"].as_str().unwrap_or_default().to_string();
                if let Some(target_parent_id) = parent_id {
                    if let Some(current_page) = client.get_page_by_id_v2(&page_id).await? {
                        let current_parent_id = current_page["parentId"].as_str();
                        if current_parent_id != Some(target_parent_id) {
                            let _ = client.migrate_page_to_parent(&page_id, target_parent_id).await;
                        }
                    }
                }
                client
                    .update_page_body_by_id(&page_id, "storage", &duplicate_html)
                    .await?;
                set_sync_prop(client, &page_id, hash).await?;
                set_page_icon(
                    client,
                    &page_id,
                    path.file_stem().unwrap_or_default().to_str().unwrap_or(""),
                )
                .await;
                synced_ids.insert(page_id);
                return Ok(duplicate_title);
            }
            let page_id = client
                .create_or_update_page(space_key, parent_id, &duplicate_title, "storage", &duplicate_html)
                .await?;
            set_sync_prop(client, &page_id, hash).await?;
            set_page_icon(client, &page_id, path.file_stem().unwrap_or_default().to_str().unwrap_or("")).await;
            synced_ids.insert(page_id);
            return Ok(duplicate_title);
        }
    } else if let Some(existing_global_page) = client.get_page_by_title(space_key, None, page_title).await? {
        let page_id = existing_global_page["id"].as_str().unwrap_or_default().to_string();
        if let Some(target_parent_id) = parent_id {
            if let Some(current_page) = client.get_page_by_id_v2(&page_id).await? {
                let current_parent_id = current_page["parentId"].as_str();
                if current_parent_id != Some(target_parent_id) {
                    let _ = client.migrate_page_to_parent(&page_id, target_parent_id).await;
                }
            }
        }
        client
            .update_page_body_by_id(&page_id, "storage", html_body)
            .await?;
        set_sync_prop(client, &page_id, hash).await?;
        set_page_icon(client, &page_id, path.file_stem().unwrap_or_default().to_str().unwrap_or("")).await;
        synced_ids.insert(page_id);
        return Ok(page_title.to_string());
    }

    let slug = path.file_stem().unwrap_or_default().to_str().unwrap_or("");
    let page_id = client
        .create_or_update_page(space_key, parent_id, page_title, "storage", html_body)
        .await?;
    set_sync_prop(client, &page_id, hash).await?;
    set_page_icon(client, &page_id, slug).await;
    synced_ids.insert(page_id);
    Ok(page_title.to_string())
}

fn render_lane_page_body(path: &Path, page: &crate::WikiPage, lane: &str) -> Result<String> {
    let quality = assess_quality(&page.frontmatter.title, &page.body);
    let analysis = read_analysis_sidecar(path);
    let proposal = load_proposal_record(path).ok().flatten();
    let route = if page.frontmatter.category.is_empty() {
        "unrouted".to_string()
    } else {
        page.frontmatter.category.join(" / ")
    };
    let confidence = page.frontmatter.confidence.unwrap_or(0.0) * 100.0;
    let mut body = String::new();
    let lane_label = if lane == "review" { "Review" } else { "Staged" };
    body.push_str(&format!(
        "<ac:structured-macro ac:name=\"info\"><ac:rich-text-body><p><strong>{}</strong> item in the Curio workflow.</p></ac:rich-text-body></ac:structured-macro>",
        lane_label
    ));
    body.push_str("<table><tbody>");
    body.push_str(&format!("<tr><th>Status</th><td>{}</td></tr>", html_escape(page.frontmatter.status.as_str())));
    body.push_str(&format!("<tr><th>Category</th><td>{}</td></tr>", html_escape(&route)));
    body.push_str(&format!("<tr><th>Confidence</th><td>{:.0}%</td></tr>", confidence));
    body.push_str(&format!(
        "<tr><th>Information quality</th><td>{:.0}%</td></tr>",
        quality.information_quality * 100.0
    ));
    body.push_str(&format!(
        "<tr><th>Usability</th><td>{:.0}%</td></tr>",
        quality.usability * 100.0
    ));
    if let Some(ref proposal) = proposal {
        body.push_str(&format!(
            "<tr><th>Proposal kind</th><td>{}</td></tr>",
            html_escape(&format!("{:?}", proposal.kind))
        ));
        body.push_str(&format!(
            "<tr><th>Recommended action</th><td>{}</td></tr>",
            html_escape(&proposal.recommended_action)
        ));
        body.push_str(&format!(
            "<tr><th>Hierarchy fit</th><td>{:.0}%</td></tr>",
            proposal.scores.hierarchy_fit_confidence * 100.0
        ));
        body.push_str(&format!(
            "<tr><th>Overlap risk</th><td>{:.0}%</td></tr>",
            proposal.scores.overlap_risk * 100.0
        ));
    }
    if !quality.flags.is_empty() {
        body.push_str(&format!(
            "<tr><th>Quality flags</th><td>{}</td></tr>",
            html_escape(&quality.flags.join(", "))
        ));
    }
    if let Some(ref analysis) = analysis {
        if !analysis.routing.rationale.trim().is_empty() {
            body.push_str(&format!(
                "<tr><th>Rationale</th><td>{}</td></tr>",
                html_escape(&analysis.routing.rationale)
            ));
        }
        if let Some(ref reason) = analysis.routing.review_reason {
            body.push_str(&format!(
                "<tr><th>Review reason</th><td>{}</td></tr>",
                html_escape(reason)
            ));
        }
        if let Some(ref subtree) = analysis.routing.proposed_new_subtree {
            body.push_str(&format!(
                "<tr><th>Proposed subtree</th><td>{}</td></tr>",
                html_escape(subtree)
            ));
        }
        if let Some(ref rationale) = analysis.routing.proposal_rationale {
            body.push_str(&format!(
                "<tr><th>Proposal rationale</th><td>{}</td></tr>",
                html_escape(rationale)
            ));
        }
    }
    body.push_str("</tbody></table>");
    body.push_str(&markdown_to_html(&page.body));
    Ok(body)
}

fn render_lane_branch_body(root_dir: &Path, rel_path: &Path, lane: &str) -> Result<String> {
    let abs_dir = root_dir.join(rel_path);
    let mut children: Vec<(String, String)> = Vec::new();
    for entry in std::fs::read_dir(&abs_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file()
            || path.extension().and_then(|ext| ext.to_str()) != Some("md")
            || path.file_name().and_then(|name| name.to_str()) == Some("index.md")
        {
            continue;
        }
        let page = match parse_wiki_page(&path) {
            Ok(page) => page,
            Err(_) => continue,
        };
        children.push((page.frontmatter.title, crate::wiki_fs::first_line_summary(&page.body, 160)));
    }
    children.sort_by(|a, b| a.0.cmp(&b.0));
    let mut body = format!(
        "<p>{} proposal branch for <code>{}</code>.</p>",
        html_escape(&to_title(lane)),
        html_escape(&rel_path.display().to_string())
    );
    if !children.is_empty() {
        body.push_str("<h2>Child Proposals</h2><ul>");
        for (title, summary) in children {
            body.push_str(&format!(
                "<li><strong>{}</strong>{}</li>",
                html_escape(&title),
                if summary.is_empty() {
                    String::new()
                } else {
                    format!(" — {}", html_escape(&summary))
                }
            ));
        }
        body.push_str("</ul>");
    }
    Ok(body)
}

fn render_proposal_body(payload: &serde_json::Value) -> String {
    let generated_at = payload["generated_at"].as_str().unwrap_or("");
    let proposals = payload["proposals"].as_array().cloned().unwrap_or_default();
    let mut out = String::new();
    out.push_str("<h1>Sharpening Proposal Set</h1>");
    if !generated_at.is_empty() {
        out.push_str(&format!("<p>Generated at {}</p>", html_escape(generated_at)));
    }
    for proposal in proposals {
        out.push_str("<ac:structured-macro ac:name=\"info\"><ac:rich-text-body>");
        out.push_str(&format!(
            "<p><strong>{}</strong>: {}</p>",
            html_escape(proposal["type"].as_str().unwrap_or("proposal")),
            html_escape(proposal["recommended_action"].as_str().unwrap_or(""))
        ));
        out.push_str("</ac:rich-text-body></ac:structured-macro>");
        out.push_str("<table><tbody>");
        if let Some(paths) = proposal["affected_paths"].as_array() {
            let joined = paths.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join("<br/>");
            out.push_str(&format!("<tr><th>Affected paths</th><td>{}</td></tr>", joined));
        }
        if let Some(rationale) = proposal["rationale"].as_str() {
            out.push_str(&format!("<tr><th>Rationale</th><td>{}</td></tr>", html_escape(rationale)));
        }
        if let Some(confidence) = proposal["confidence"].as_f64() {
            out.push_str(&format!("<tr><th>Confidence</th><td>{:.0}%</td></tr>", confidence * 100.0));
        }
        if let Some(gain) = proposal["expected_signal_gain"].as_str() {
            out.push_str(&format!("<tr><th>Expected signal gain</th><td>{}</td></tr>", html_escape(gain)));
        }
        out.push_str("</tbody></table>");
    }
    out
}

fn read_analysis_sidecar(path: &Path) -> Option<RoutingAnalysis> {
    let sidecar_path = path.with_extension("analysis.json");
    let raw = std::fs::read_to_string(sidecar_path).ok()?;
    serde_json::from_str(&raw).ok()
}

fn lane_display_title(lane: &str, rel_path: &Path) -> String {
    let parts: Vec<String> = rel_path
        .components()
        .filter_map(|component| component.as_os_str().to_str())
        .map(to_title)
        .collect();
    format!("{} - {}", to_title(lane), parts.join(" - "))
}

fn subtree_has_markdown(root: &Path) -> bool {
    walkdir::WalkDir::new(root)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .any(|entry| {
            entry.file_type().is_file()
                && entry.path().extension().and_then(|ext| ext.to_str()) == Some("md")
                && entry.path().file_name().and_then(|name| name.to_str()) != Some("index.md")
        })
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
    if let Some(page) = find_existing_page_for_sync(client, space_key, parent_id, title).await? {
        let page_id = page["id"].as_str().unwrap_or_default().to_string();
        if let Some(target_parent_id) = parent_id {
            if let Some(current_page) = client.get_page_by_id_v2(&page_id).await? {
                let current_parent_id = current_page["parentId"].as_str();
                if current_parent_id != Some(target_parent_id) {
                    let _ = client.migrate_page_to_parent(&page_id, target_parent_id).await;
                }
            }
        }
        client
            .update_page_body_by_id(&page_id, "storage", body)
            .await?;
        set_sync_prop(client, &page_id, &hash).await?;
        if let Some(icon_val) = icon_override {
            set_page_icon_value(client, &page_id, icon_val).await;
        } else {
            set_page_icon(client, &page_id, slug).await;
        }
        return Ok(page_id);
    }

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

async fn find_existing_page_for_sync(
    client: &ConfluenceClient,
    space_key: &str,
    parent_id: Option<&str>,
    title: &str,
) -> Result<Option<serde_json::Value>> {
    if let Some(page) = client.get_page_by_title(space_key, parent_id, title).await? {
        return Ok(Some(page));
    }
    client.get_page_by_title(space_key, None, title).await
}

async fn set_sync_prop(client: &ConfluenceClient, page_id: &str, hash: &str) -> Result<()> {
    let value = serde_json::json!({ "content_hash": hash, "synced_by": "curio" });
    client.set_content_property(page_id, SYNC_PROP_KEY, value).await
}

async fn find_stale_pages(
    client: &ConfluenceClient,
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
        stale.push(page_id);
    }
    Ok(stale)
}

async fn find_legacy_sync_pages(
    client: &ConfluenceClient,
    published_root_id: &str,
    trees: &[TreeNode],
) -> Result<Vec<String>> {
    let descendants = client
        .get_page_descendants_v2(published_root_id)
        .await
        .unwrap_or_default();
    let tree_titles: HashSet<String> = trees.iter().map(|tree| tree.title.clone()).collect();
    let mut legacy = Vec::new();

    for page in descendants {
        let page_id = page["id"].as_str().unwrap_or_default().to_string();
        let title = page["title"].as_str().unwrap_or_default();
        if !is_legacy_sync_title(title, &tree_titles) {
            continue;
        }
        legacy.push(page_id);
    }

    Ok(legacy)
}

fn is_legacy_sync_title(title: &str, tree_titles: &HashSet<String>) -> bool {
    if title.ends_with(" Index") {
        return true;
    }
    tree_titles
        .iter()
        .any(|tree_title| title.starts_with(&format!("{} - ", tree_title)))
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

fn published_page_title(path: &Path) -> Result<String> {
    let page = parse_wiki_page(path)
        .with_context(|| format!("Failed to parse {}", path.display()))?;
    let title = page.frontmatter.title.trim().to_string();
    if title.is_empty() {
        anyhow::bail!("Published page {} is missing a title", path.display());
    }
    Ok(title)
}

fn duplicate_notice_body(client: &ConfluenceClient, conflicting_page_id: &str, original_title: &str, html_body: &str) -> String {
    let conflicting_url = client.page_web_url(conflicting_page_id);
    format!(
        concat!(
            "<ac:structured-macro ac:name=\"info\"><ac:rich-text-body>",
            "<p>This page was published with a duplicate-title fallback because another CURIO page already used the title <strong>{title}</strong>.</p>",
            "<p>Conflicting page reference: <a href=\"{url}\">{title}</a> (pageId {page_id}).</p>",
            "</ac:rich-text-body></ac:structured-macro>",
            "{body}"
        ),
        title = html_escape(original_title),
        url = html_escape(&conflicting_url),
        page_id = html_escape(conflicting_page_id),
        body = html_body
    )
}

// ─── NORTHSTAR rich Confluence renderer ──────────────────────────────────

/// Render the NORTHSTAR page as rich Confluence storage format.
///
/// The `## Published Tree Blueprint` section is replaced with a custom visual
/// nested list built from parsed `TreeNode` data, so parent/child relationships
/// are immediately obvious. Everything else passes through `markdown_to_html`.
fn render_northstar_for_confluence(northstar_md: &str, trees: &[TreeNode]) -> String {
    // Split at the blueprint section boundary
    let blueprint_start = northstar_md.find("## Published Tree Blueprint");
    let after_blueprint = blueprint_start.and_then(|s| {
        // Find the next `## ` section after the blueprint
        northstar_md[s + 1..].find("\n## ").map(|o| s + 1 + o)
    });

    let (pre_md, post_md) = match (blueprint_start, after_blueprint) {
        (Some(start), Some(end)) => (
            &northstar_md[..start],
            &northstar_md[end..],
        ),
        (Some(start), None) => (&northstar_md[..start], ""),
        _ => (northstar_md, ""),
    };

    let mut out = String::new();

    // Pre-blueprint sections (Name, High-Level Description, What Curio Curates)
    out.push_str(&markdown_to_html(pre_md));

    // Blueprint section heading + intro
    out.push_str("<h2>Published Tree Blueprint</h2>\n");
    out.push_str("<p>Tree definitions below drive the <code>published/</code> wiki structure and the Confluence hierarchy. \
        Run <code>curio tree</code> after editing <code>NORTHSTAR.md</code> to sync directory structure and Confluence pages.</p>\n");

    if trees.is_empty() {
        out.push_str("<p><em>No tree definitions found. Add <code>### TreeName</code> headings under <code>## Published Tree Blueprint</code> in NORTHSTAR.md.</em></p>\n");
    } else {
        out.push_str("<ul>\n");
        for tree in trees {
            let icon = tree.icon.as_deref()
                .and_then(|cp| u32::from_str_radix(cp, 16).ok())
                .and_then(char::from_u32)
                .map(|c| format!("{} ", c))
                .unwrap_or_default();

            let desc = inline_desc(&tree.description_html);
            out.push_str(&format!(
                "<li><strong>{icon}{title}</strong> &mdash; <code>{slug}/</code>",
                icon = icon,
                title = html_escape(&tree.title),
                slug = html_escape(&tree.slug),
            ));
            if !desc.is_empty() {
                out.push_str(&format!("<br/><em>{desc}</em>"));
            }

            if !tree.subtrees.is_empty() {
                out.push_str("<ul>\n");
                for sub in &tree.subtrees {
                    let sub_icon = sub.icon.as_deref()
                        .and_then(|cp| u32::from_str_radix(cp, 16).ok())
                        .and_then(char::from_u32)
                        .map(|c| format!("{} ", c))
                        .unwrap_or_default();
                    let sub_desc = inline_desc(&sub.description_html);
                    out.push_str(&format!(
                        "<li>{sub_icon}<strong>{title}</strong> &mdash; <code>{slug}/</code>",
                        sub_icon = sub_icon,
                        title = html_escape(&sub.title),
                        slug = html_escape(&sub.slug),
                    ));
                    if !sub_desc.is_empty() {
                        out.push_str(&format!("<br/><em>{sub_desc}</em>"));
                    }
                    out.push_str("</li>\n");
                }
                out.push_str("</ul>\n");
            }

            out.push_str("</li>\n");
        }
        out.push_str("</ul>\n");
    }

    // Post-blueprint sections (Structure, Helpful Guidance)
    if !post_md.is_empty() {
        out.push_str(&markdown_to_html(post_md));
    }

    out
}

/// Flatten block-level HTML (p, blockquote) to a single inline text string.
/// Strips all block tags and collapses whitespace so descriptions render
/// inline inside <li> elements without causing wide block containers.
fn inline_desc(html: &str) -> String {
    let text = html
        .replace("<blockquote>", "").replace("</blockquote>", " ")
        .replace("<p>", "").replace("</p>", " ")
        .replace("<br/>", " ").replace("<br />", " ");
    // Collapse runs of whitespace / newlines
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

fn taxonomy_to_tree_nodes(taxonomy: &NorthstarTaxonomy) -> Vec<TreeNode> {
    taxonomy.nodes.iter().map(tree_node_from_taxonomy).collect()
}

pub(crate) fn tree_node_from_taxonomy(node: &TaxonomyNode) -> TreeNode {
    TreeNode {
        title: node.title.clone(),
        slug: node.slug.clone(),
        description_html: if node.description_markdown.trim().is_empty() {
            String::new()
        } else {
            markdown_to_html(&node.description_markdown)
        },
        icon: node.icon.clone(),
        subtrees: node.children.iter().map(tree_node_from_taxonomy).collect(),
    }
}

// ─── NORTHSTAR blueprint parser ───────────────────────────────────────────

#[derive(Debug, Default, Clone)]
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
                if let Some(ref mut t) = current_tree {
                    t.description_html = html;
                }
            }
            if current_tree.is_none() {
                desc_lines.clear();
            }
            // Flush current tree
            if let Some(t) = current_tree.take() {
                trees.push(t);
            }
            let title = line[4..].trim().to_string();
            let slug = title.to_lowercase().replace(' ', "-");
            current_tree = Some(TreeNode { title, slug, ..Default::default() });
        } else if line.starts_with("#### ") {
            // Flush previous subtree (or tree description if this is the first subtree)
            if let Some(mut sub) = current_sub.take() {
                sub.description_html = flush_desc(&mut desc_lines);
                if let Some(ref mut t) = current_tree { t.subtrees.push(sub); }
            } else {
                // First subtree — flush accumulated lines as the parent tree's description
                let html = flush_desc(&mut desc_lines);
                if let Some(ref mut t) = current_tree {
                    if t.description_html.is_empty() { t.description_html = html; }
                }
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
