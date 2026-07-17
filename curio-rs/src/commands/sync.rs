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
use chrono::Utc;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use crate::{
    config::{Config, upsert_repo_env_var},
    confluence::ConfluenceClient,
    northstar::{
        NorthstarTaxonomy, TaxonomyNode, load_taxonomy, read_northstar_markdown,
        workspace_config_path,
    },
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
const WORKSPACE_CONFIG_PAGE_TITLE: &str = "Config";
const REQUIRED_CURIO_CHILDREN: &[&str] = &["Published", "Intake", "Staged", "Review", "Admin"];
const REQUIRED_CONFIG_CHILDREN: &[&str] = &[
    "Northstar",
    "CURIO Readme",
    WORKSPACE_CONFIG_PAGE_TITLE,
    "Getting Started",
    "Log",
    "Downloads",
];

#[derive(Debug, Clone)]
pub struct CurioConfluenceTree {
    pub root_id: String,
    pub staged_id: String,
    pub review_id: String,
    pub published_id: String,
    pub config_id: String,
    pub downloads_id: String,
}

#[derive(Debug, Clone)]
pub struct CurioTreeValidation {
    pub root_id: String,
    pub checked_pages: usize,
}

#[derive(Debug, Default, serde::Serialize)]
struct CleanupReport {
    candidates_found: Vec<String>,
    owned_candidates: Vec<String>,
    preserved_unowned: Vec<String>,
    deleted_pages: Vec<String>,
    cleanup_skipped: bool,
    error: Option<String>,
}

/// Built-in icons for the **harness-managed** page slugs (intake / staged
/// / review / published / admin / etc.). Domain-specific product icons
/// live in the operator-supplied `Config::products` registry + the
/// `Config::category_icons` map — never hard-coded here.
fn page_icon_builtin(slug: &str) -> Option<&'static str> {
    match slug {
        "curio" => Some("1f9ed"), // 🧭
        "curio-readme" | "readme" => Some("atlassian-info"),
        "northstar" => Some("1f9ed"),                    // 🧭
        "admin" | "config" => Some("2699"),              // ⚙️
        "settings" | "workspace-config" => Some("2699"), // ⚙️
        "getting-started" => Some("1f680"),
        "log" => Some("1f4dd"),
        "downloads" => Some("1f4e6"),
        "intake" => Some("1f4e5"), // 📥
        "staged" => Some("atlassian-logo_projects"),
        "review" => Some("atlassian-logo_opsgenie"),
        "published" => Some("atlassian-check_mark"),
        _ => None,
    }
}

/// Resolve the icon for a page slug. Resolution order (operator-supplied
/// SSOT wins):
///   1. `Config::category_icons[slug]` — operator's per-KB icon map.
///   2. `Config::products[*]` — if any product has this slug AND an
///      `emoji` set, use it.
///   3. `page_icon_builtin(slug)` — harness-managed defaults.
fn page_icon_for(slug: &str, config: &Config) -> Option<String> {
    if let Some(icon) = config.category_icons.get(slug)
        && !icon.is_empty()
    {
        return Some(icon.clone());
    }
    for product in &config.products {
        if product.slug == slug
            && let Some(icon) = &product.emoji
            && !icon.is_empty()
        {
            return Some(icon.clone());
        }
    }
    page_icon_builtin(slug).map(|s| s.to_string())
}

pub async fn run_sync(
    config: &Config,
    dry_run: bool,
    json: bool,
    parent_page_id_override: Option<String>,
    full_refresh: bool,
    docs_only: bool,
    downloads_only: bool,
) -> Result<()> {
    config.connection.require_confluence()?;

    let wiki_dir = &config.wiki.wiki_dir;
    let published_dir = wiki_dir.join("published");
    if !published_dir.exists() {
        anyhow::bail!("wiki/published/ not found. Run `curio init` first.");
    }

    // Dry-run is a SAFE short-circuit: enumerate what would be synced
    // from the filesystem and exit without constructing a Confluence
    // client. The deeper sync pipeline has many helpers (set_page_icon,
    // set_content_property, ensure_curio_root_page, taxonomy index, etc.)
    // that issue API writes from inside read-looking paths — gating each
    // individually is brittle. A clean short-circuit at the top is the
    // only way to guarantee --dry-run is truly read-only. Bug surfaced
    // 2026-05-10 against curio/wiki demo KB.
    if dry_run {
        return run_sync_dry_run(config, json, docs_only, downloads_only).await;
    }

    let space_key = &config.content_model.space_key;
    let token = config.connection.resolve_token()?;
    let bootstrap_client = ConfluenceClient::new(
        config.connection.confluence_url.clone(),
        config.connection.confluence_email.clone(),
        token.clone(),
        None,
    )?;
    let tree = ensure_curio_confluence_tree(
        config,
        &bootstrap_client,
        parent_page_id_override.or_else(|| config.wiki.sync.confluence_parent_page_id.clone()),
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
    let mut cleanup_report: Option<serde_json::Value> = None;
    // Sync top-level admin pages first under CURIO/Admin.
    let config_dir = wiki_dir.join(crate::northstar::ADMIN_DIRNAME);
    if config_dir.exists() && !dry_run {
        let northstar_md = read_northstar_markdown(wiki_dir)?;
        let config_yaml_path = workspace_config_path(wiki_dir);
        let config_yaml = std::fs::read_to_string(&config_yaml_path)
            .with_context(|| format!("Failed to read {}", config_yaml_path.display()))?;
        let mut config_pages = vec![
            (
                "northstar",
                "Northstar".to_string(),
                render_northstar_for_confluence(&northstar_md, &config_yaml),
                content_hash(&format!("{northstar_md}\n{config_yaml}")),
            ),
            (
                "workspace-config",
                WORKSPACE_CONFIG_PAGE_TITLE.to_string(),
                render_yaml_for_confluence(&config_yaml),
                content_hash(&config_yaml),
            ),
        ];
        // These operator-facing reference pages are optional. Their absence
        // must not prevent the content mirror from syncing.
        for (filename, stem, title) in [
            ("readme.md", "readme", "CURIO Readme"),
            ("getting-started.md", "getting-started", "Getting Started"),
            ("log.md", "log", "Log"),
        ] {
            let path = config_dir.join(filename);
            if let Some(raw) = path
                .exists()
                .then(|| std::fs::read_to_string(&path))
                .transpose()?
            {
                let body = markdown_to_html(strip_frontmatter(&raw));
                config_pages.push((stem, title.to_string(), body.clone(), content_hash(&body)));
            }
        }

        for (stem, page_title, html_body, hash) in config_pages {
            let existing = client
                .get_page_by_title(space_key, Some(tree.config_id.as_str()), &page_title)
                .await
                .ok()
                .flatten();

            if let Some(ref page) = existing {
                let page_id = page["id"].as_str().unwrap_or_default().to_string();
                let page_v2 = client.get_page_by_id_v2(&page_id).await.ok().flatten();
                let current_title = page_v2
                    .as_ref()
                    .and_then(|p| p["title"].as_str())
                    .unwrap_or("");
                let title_matches = current_title == page_title;
                if title_matches
                    && let Ok(Some(prop)) =
                        client.get_content_property(&page_id, SYNC_PROP_KEY).await
                    && prop["value"]["content_hash"].as_str() == Some(hash.as_str())
                {
                    skipped.push(format!("[admin] {}", page_title));
                    synced_page_ids.insert(page_id);
                    continue;
                }
            }

            match client
                .create_or_update_page(
                    space_key,
                    Some(tree.config_id.as_str()),
                    &page_title,
                    "storage",
                    &html_body,
                )
                .await
            {
                Ok(page_id) => {
                    let _ = set_sync_prop(&client, &page_id, &hash).await;
                    set_page_icon(&client, config, &page_id, stem).await;
                    synced_page_ids.insert(page_id);
                    upserted.push(format!("[admin] {}", page_title));
                }
                Err(e) => errors.push(format!("[admin] {}: {}", page_title, e)),
            }
        }
    }

    if !docs_only && !dry_run {
        refresh_download_bundles(config, &client, tree.downloads_id.as_str(), docs_only).await?;
    }

    if !downloads_only {
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
                sub_map.insert(
                    s.slug.clone(),
                    (s.title.clone(), s.description_html.clone(), s.icon.clone()),
                );
            }
            tree_info.insert(
                t.slug.clone(),
                (
                    t.title.clone(),
                    t.description_html.clone(),
                    t.icon.clone(),
                    sub_map,
                ),
            );
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
                        let body =
                            render_branch_page_body(&abs_path, space_key, ns_desc, &page_title)?;
                        (page_title, body, icon.clone())
                    } else {
                        let fallback_title = to_title(&name);
                        let body =
                            render_branch_page_body(&abs_path, space_key, "", &fallback_title)?;
                        (fallback_title, body, None)
                    }
                } else if depth == 2 {
                    // Subtree dir: look up in parent's subtree list
                    let parent_name = parent_rel
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("");
                    let sub_info = tree_info
                        .get(parent_name)
                        .and_then(|(_, _, _, subs)| subs.get(&name))
                        .map(|(t, d, i)| (t.clone(), d.clone(), i.clone()));
                    if let Some((sub_title, sub_desc, sub_icon)) = sub_info {
                        let body =
                            render_branch_page_body(&abs_path, space_key, &sub_desc, &sub_title)?;
                        (sub_title, body, sub_icon)
                    } else {
                        let fallback_title = to_title(&name);
                        let body =
                            render_branch_page_body(&abs_path, space_key, "", &fallback_title)?;
                        (fallback_title, body, None)
                    }
                } else {
                    let fallback_title = to_title(&name);
                    let body = render_branch_page_body(&abs_path, space_key, "", &fallback_title)?;
                    (fallback_title, body, None)
                };

                if dry_run {
                    upserted.push(format!("[dir] {}", page_title));
                    dir_to_page_id
                        .insert(rel_path.clone(), Some(format!("dry-run:{}", page_title)));
                } else {
                    match upsert_page(
                        &client,
                        config,
                        space_key,
                        parent_conf_id.as_deref(),
                        &page_title,
                        &dir_body,
                        &name,
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
            } else if rel_path.extension().is_some_and(|ext| ext == "md") {
                if dry_run {
                    let page_title = published_page_title(&abs_path)?;
                    upserted.push(format!("[page] {}", page_title));
                } else {
                    match sync_page(
                        &client,
                        config,
                        space_key,
                        parent_conf_id.as_deref(),
                        &abs_path,
                        &mut synced_page_ids,
                        &mut skipped,
                    )
                    .await
                    {
                        Ok(page_title) => upserted.push(format!("[page] {}", page_title)),
                        Err(e) => errors.push(format!("[page] {}: {}", abs_path.display(), e)),
                    }
                }
            }
        }

        let auto_heal_label = config.heal.auto_heal_label();
        sync_lane_directory(
            &client,
            config,
            space_key,
            &wiki_dir.join("staged"),
            tree.staged_id.as_str(),
            "staged",
            full_refresh,
            &mut synced_page_ids,
            &mut skipped,
            &mut upserted,
            &mut errors,
            auto_heal_label,
        )
        .await?;
        sync_lane_directory(
            &client,
            config,
            space_key,
            &wiki_dir.join("review"),
            tree.review_id.as_str(),
            "review",
            full_refresh,
            &mut synced_page_ids,
            &mut skipped,
            &mut upserted,
            &mut errors,
            auto_heal_label,
        )
        .await?;
        let proposals_dir = wiki_dir
            .join(crate::northstar::ADMIN_DIRNAME)
            .join("sharpening-proposals");
        sync_review_proposals(
            &client,
            config,
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
            config,
            space_key,
            &wiki_dir.join("review"),
            tree.review_id.as_str(),
            &mut synced_page_ids,
            &mut upserted,
            &mut errors,
        )
        .await?;

        // Prune stale pages — only in full-refresh mode to avoid slow Confluence tree walks
        // on every incremental sync. Run `curio sync --all` to prune deleted pages.
        if !dry_run && full_refresh {
            let mut cleanup = CleanupReport::default();
            for root_id in [&tree.published_id, &tree.staged_id, &tree.review_id] {
                match find_owned_stale_pages(&client, root_id, &synced_page_ids).await {
                    Ok((mut candidates, mut owned, mut preserved)) => {
                        cleanup.candidates_found.append(&mut candidates);
                        cleanup.owned_candidates.append(&mut owned);
                        cleanup.preserved_unowned.append(&mut preserved);
                    }
                    Err(e) => {
                        cleanup.cleanup_skipped = true;
                        cleanup.error = Some(e.to_string());
                        errors.push(format!("cleanup skipped: {}", e));
                    }
                }
            }
            if !cleanup.cleanup_skipped {
                for page_id in cleanup.owned_candidates.clone() {
                    match client.delete_page(&page_id).await {
                        Ok(()) => {
                            cleanup.deleted_pages.push(page_id.clone());
                            upserted.push(format!("[deleted] {}", page_id));
                        }
                        Err(e) => errors.push(format!("delete {} failed: {}", page_id, e)),
                    }
                }
            }

            append_log(
                wiki_dir,
                &format!(
                    "sync: {} upserted, {} skipped, {} owned stale deleted, {} unowned preserved, {} errors",
                    upserted.len(),
                    skipped.len(),
                    cleanup.deleted_pages.len(),
                    cleanup.preserved_unowned.len(),
                    errors.len()
                ),
            )?;

            cleanup_report = Some(serde_json::to_value(&cleanup)?);
        }
    }

    if json {
        let _ = emit_json(
            "sync",
            true,
            serde_json::json!({
                "upserted": upserted,
                "skipped": skipped,
                "errors": errors,
                "dry_run": dry_run,
                "cleanup": cleanup_report,
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
    let root_id =
        ensure_curio_root_page(config, client, space_key, preferred_root_id.as_deref()).await?;

    let published_id = upsert_static_page(
        client,
        config,
        space_key,
        Some(root_id.as_str()),
        "Published",
        &published_root_body(),
        "published",
        Some(page_icon_builtin("published").unwrap_or("atlassian-check_mark")),
    )
    .await?;
    let _intake_id = upsert_static_page(
        client,
        config,
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
        config,
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
        config,
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
        config,
        space_key,
        Some(root_id.as_str()),
        "Admin",
        &config_root_body(),
        "admin",
        Some(page_icon_builtin("admin").unwrap_or("2699")),
    )
    .await?;
    let downloads_id = upsert_static_page(
        client,
        config,
        space_key,
        Some(config_id.as_str()),
        "Downloads",
        &downloads_root_body(),
        "downloads",
        Some(page_icon_builtin("downloads").unwrap_or("1f4e6")),
    )
    .await?;

    let config_dir = config.wiki.wiki_dir.join(crate::northstar::ADMIN_DIRNAME);
    sync_config_reference_pages(config, client, space_key, &config_id, &config_dir).await?;

    let taxonomy = load_taxonomy(&config.wiki.wiki_dir)?;
    let trees = taxonomy_to_tree_nodes(&taxonomy);
    for tree in &trees {
        let body = render_branch_summary_only(
            space_key,
            &tree.title,
            &tree.description_html,
            &tree.subtrees,
        );
        let _ = upsert_static_page(
            client,
            config,
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
    let root_body = curio_root_body(space_key, config);
    let root_page_id = upsert_static_page(
        client,
        config,
        space_key,
        None,
        CURIO_ROOT_TITLE,
        &root_body,
        "curio",
        None,
    )
    .await?;
    ensure_space_homepage_best_effort(client, space_key, &root_page_id).await;

    if persist_root_id {
        upsert_repo_env_var("CURIO_CONFLUENCE_PARENT_PAGE_ID", &root_page_id)?;
    }

    Ok(CurioConfluenceTree {
        root_id: root_page_id,
        staged_id,
        review_id,
        published_id,
        config_id,
        downloads_id,
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
    let descendants = client
        .get_page_descendants_v2(&root_id)
        .await
        .unwrap_or_default();

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
    let expected_root_titles: HashSet<String> = REQUIRED_CURIO_CHILDREN
        .iter()
        .map(|title| title.to_string())
        .collect();
    if root_titles != expected_root_titles {
        anyhow::bail!(
            "CURIO root validation failed: direct children were {:?}, expected {:?}",
            sorted_titles(&root_titles),
            sorted_titles(&expected_root_titles)
        );
    }

    let mut checked_pages = 1usize;
    let config_id = root_children_by_title
        .get("Admin")
        .cloned()
        .context("CURIO validation failed: Admin page missing")?;

    for title in REQUIRED_CURIO_CHILDREN {
        let page_id = root_children_by_title
            .get(*title)
            .cloned()
            .with_context(|| {
                format!("CURIO validation failed: missing direct child '{}'", title)
            })?;
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
    let expected_config_titles: HashSet<String> = REQUIRED_CONFIG_CHILDREN
        .iter()
        .map(|title| title.to_string())
        .collect();
    if config_titles != expected_config_titles {
        anyhow::bail!(
            "CURIO validation failed: Admin children were {:?}, expected {:?}",
            sorted_titles(&config_titles),
            sorted_titles(&expected_config_titles)
        );
    }

    for title in REQUIRED_CONFIG_CHILDREN {
        let page_id = config_children_by_title
            .get(*title)
            .cloned()
            .with_context(|| format!("CURIO validation failed: missing Admin child '{}'", title))?;
        let page = client
            .get_page_by_id_v2(&page_id)
            .await?
            .with_context(|| format!("CURIO validation failed: Admin child '{}' missing", title))?;
        if page["parentId"].as_str() != Some(config_id.as_str()) {
            anyhow::bail!(
                "CURIO validation failed: Admin child '{}' is not under Admin",
                title
            );
        }
        validate_page_body_loaded(client, &page_id, title).await?;
        checked_pages += 1;
    }

    Ok(CurioTreeValidation {
        root_id,
        checked_pages,
    })
}

async fn ensure_curio_root_page(
    config: &Config,
    client: &ConfluenceClient,
    space_key: &str,
    preferred_root_id: Option<&str>,
) -> Result<String> {
    let expected_space_id = client.get_numeric_space_id(space_key).await?;
    if let Some(root_id) = preferred_root_id
        && let Some(page) = client.get_page_by_id_v2(root_id).await?
    {
        let title_matches = page["title"].as_str() == Some(CURIO_ROOT_TITLE);
        let space_matches = page["spaceId"].as_str() == Some(expected_space_id.as_str());
        let status_current = page["status"].as_str() == Some("current");
        if title_matches && space_matches && status_current {
            return Ok(root_id.to_string());
        }
    }

    if let Some(page) = client
        .get_page_by_title(space_key, None, CURIO_ROOT_TITLE)
        .await?
    {
        let root_id = page["id"].as_str().unwrap_or_default().to_string();
        set_page_icon(client, config, &root_id, "curio").await;
        return Ok(root_id);
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

    set_page_icon(client, config, &root_id, "curio").await;
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
    let body = page["body"]["storage"]["value"]
        .as_str()
        .unwrap_or_default();
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

async fn sync_config_reference_pages(
    config: &Config,
    client: &ConfluenceClient,
    space_key: &str,
    config_id: &str,
    config_dir: &Path,
) -> Result<()> {
    let northstar_md = read_northstar_markdown(&config.wiki.wiki_dir)?;
    let config_yaml_path = workspace_config_path(&config.wiki.wiki_dir);
    let config_yaml = std::fs::read_to_string(&config_yaml_path)
        .with_context(|| format!("Failed to read {}", config_yaml_path.display()))?;

    let northstar_body = render_northstar_for_confluence(&northstar_md, &config_yaml);
    let _ = upsert_static_page(
        client,
        config,
        space_key,
        Some(config_id),
        "Northstar",
        &northstar_body,
        "northstar",
        Some(page_icon_builtin("northstar").unwrap_or("1f9ed")),
    )
    .await?;

    for (filename, page_title, slug) in [
        ("readme.md", "CURIO Readme", "readme"),
        ("getting-started.md", "Getting Started", "getting-started"),
        ("log.md", "Log", "log"),
    ] {
        let path = config_dir.join(filename);
        if !path.exists() {
            continue;
        }
        let raw = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        let html_body = markdown_to_html(strip_frontmatter(&raw));
        let _ = upsert_static_page(
            client,
            config,
            space_key,
            Some(config_id),
            page_title,
            &html_body,
            slug,
            None,
        )
        .await?;
    }

    let workspace_config_body = render_yaml_for_confluence(&config_yaml);
    let _ = upsert_static_page(
        client,
        config,
        space_key,
        Some(config_id),
        WORKSPACE_CONFIG_PAGE_TITLE,
        &workspace_config_body,
        "config",
        Some(page_icon_builtin("config").unwrap_or("2699")),
    )
    .await?;

    Ok(())
}

async fn ensure_space_homepage_best_effort(
    client: &ConfluenceClient,
    space_key: &str,
    root_page_id: &str,
) {
    match client.ensure_space_homepage(space_key, root_page_id).await {
        Ok(true) => println!("CURIO sync: updated Confluence space homepage to CURIO root"),
        Ok(false) => {}
        Err(err) => eprintln!(
            "curio: warning: could not update Confluence space homepage for {}: {}",
            space_key, err
        ),
    }
}

async fn upload_root_hero(config: &Config, client: &ConfluenceClient, root_id: &str) -> Result<()> {
    let repo_root = config
        .wiki
        .wiki_dir
        .parent()
        .map(PathBuf::from)
        .unwrap_or_else(crate::config::repo_root);
    let hero_path = repo_root
        .join("docs")
        .join("assets")
        .join(CURIO_HERO_FILENAME);
    if !hero_path.exists() {
        return Ok(());
    }
    let bytes = std::fs::read(&hero_path)
        .with_context(|| format!("Failed to read hero image: {}", hero_path.display()))?;
    client
        .upload_attachment(root_id, CURIO_HERO_FILENAME, "image/png", bytes)
        .await
}

fn curio_root_body(space_key: &str, config: &Config) -> String {
    // Operator-supplied related repos render under "Source Repositories".
    // Defaults to empty — public Curio repo does not hard-code any host.
    let related_repos_html: String = if config.admin_related_repos.is_empty() {
        String::new()
    } else {
        let mut s = String::from("<h2>Source Repositories</h2><ul>");
        for repo in &config.admin_related_repos {
            let desc = if repo.description.trim().is_empty() {
                String::new()
            } else {
                format!(" — {}", html_escape(&repo.description))
            };
            s.push_str(&format!(
                "<li><a href=\"{}\">{}</a>{}</li>",
                html_escape(&repo.url),
                html_escape(&repo.title),
                desc
            ));
        }
        s.push_str("</ul>");
        s
    };
    format!(
        concat!(
            "<ac:image ac:layout=\"center\" ac:width=\"230\"><ri:attachment ri:filename=\"{hero}\" /></ac:image>",
            "<h1>CURIO</h1>",
            "<p>Curated Intelligence Operator is a Git-native knowledge curation system that turns raw source material into a governed, navigable Confluence space.</p>",
            "<ac:structured-macro ac:name=\"info\"><ac:rich-text-body>",
            "<p>Use CURIO as the landing page for teams who need a clear view of what is published, what is in-flight, how the workspace is configured, and where to start.</p>",
            "</ac:rich-text-body></ac:structured-macro>",
            "<h2>Why CURIO</h2>",
            "<p>CURIO keeps Git as the source of truth while publishing a readable Confluence mirror for broader audiences. It helps teams capture intake, route content into an explicit taxonomy, review ambiguous material, and publish durable knowledge without losing provenance.</p>",
            "<table><tbody>",
            "<tr><th>Feature</th><th>Benefit</th></tr>",
            "<tr><td>Git-native workspace</td><td>Auditable edits, repeatable automation, and durable source control.</td></tr>",
            "<tr><td>Structured editorial pipeline</td><td>Separates intake, staging, review, and publication so quality gates stay visible.</td></tr>",
            "<tr><td>Explicit taxonomy</td><td>Keeps published knowledge organized by clear branches instead of ad hoc page sprawl.</td></tr>",
            "<tr><td>Confluence mirror</td><td>Lets less-technical users consume curated output without working in the repo.</td></tr>",
            "</tbody></table>",
            "<h2>Get Started</h2>",
            "<p>New to CURIO? Start with ",
            "{getting_started}",
            " for setup, then use ",
            "{downloads}",
            " if you need a packaged bundle instead of a Git checkout. Read ",
            "{northstar}",
            " for the workspace charter and publishing model, or jump straight to ",
            "{published}",
            " to browse the curated output.</p>",
            "<h2>Deployment Prerequisites</h2>",
            "<ul>",
            "<li>Confluence URL, account email, and API token available to Curio.</li>",
            "<li>Access to the target Confluence space; space-admin rights are needed if Curio should repair the homepage setting automatically.</li>",
            "<li>A local Git workspace for the Curio harness and the active knowledge base.</li>",
            "<li>Curio CLI access for onboarding, sync, and editorial workflows.</li>",
            "</ul>",
            "{related_repos}",
            "<h2>How It Works</h2>",
            "<p>Content moves through a simple editorial loop: capture source material, route it into the right branch, review ambiguous or weak items, then publish only content that is useful and durable.</p>",
            "<table><tbody>",
            "<tr><th>Section</th><th>Purpose</th></tr>",
            "<tr><td>{published}</td><td>Published knowledge organized into explicit tree structures.</td></tr>",
            "<tr><td>{intake}</td><td>Fresh source material collected for processing.</td></tr>",
            "<tr><td>{staged}</td><td>Routed content awaiting final publication.</td></tr>",
            "<tr><td>{review}</td><td>Items needing human judgment.</td></tr>",
            "<tr><td>{config}</td><td>Northstar, readme, config YAML, onboarding docs, and downloads that define the workspace.</td></tr>",
            "</tbody></table>"
        ),
        hero = CURIO_HERO_FILENAME,
        related_repos = related_repos_html,
        published = page_link(space_key, "Published", "Published"),
        intake = page_link(space_key, "Intake", "Intake"),
        staged = page_link(space_key, "Staged", "Staged"),
        review = page_link(space_key, "Review", "Review"),
        config = page_link(space_key, "Admin", "Admin"),
        northstar = page_link(space_key, "Northstar", "Northstar"),
        getting_started = page_link(space_key, "Getting Started", "Getting Started"),
        downloads = page_link(space_key, "Downloads", "Downloads"),
    )
}

fn published_root_body() -> String {
    "<p>Published knowledge from the Curio workspace. Tree pages below mirror the filesystem and NORTHSTAR blueprint.</p>".to_string()
}

fn config_root_body() -> String {
    "<p>Administration, onboarding, and workspace reference pages for Curio. This branch is machine-maintained from <code>wiki/_admin</code> plus the workspace <code>NORTHSTAR.md</code>.</p><ul><li>Northstar: workspace charter plus published config YAML</li><li>CURIO Readme: workspace overview and operating model</li><li>Config: deterministic YAML for taxonomy and runtime settings</li><li>Getting Started: setup and first-run guide</li><li>Log: concise run history</li><li>Downloads: bundle snapshots for operators</li></ul>".to_string()
}

fn downloads_root_body() -> String {
    "<p>Bundle snapshots for operators who need a direct download path instead of a Git checkout.</p><p>Curio refreshes these attachments during sync.</p>".to_string()
}

#[derive(Debug, Clone)]
struct BundleArtifact {
    filename: String,
    path: PathBuf,
    description: String,
}

async fn refresh_download_bundles(
    config: &Config,
    client: &ConfluenceClient,
    downloads_page_id: &str,
    docs_only: bool,
) -> Result<()> {
    if docs_only {
        return Ok(());
    }

    let workspace_root = config.wiki.wiki_dir.clone();
    let harness_root = crate::config::repo_root();

    let stamp = Utc::now().format("%Y%m%d-%H%M%S").to_string();
    let bundle_root = std::env::temp_dir().join(format!("curio-downloads-{stamp}"));
    fs::create_dir_all(&bundle_root)
        .with_context(|| format!("Failed to create {}", bundle_root.display()))?;

    let mut bundles = Vec::new();
    let harness_zip = bundle_root.join("curio-harness-source.zip");
    create_zip_bundle(
        &harness_root,
        &harness_zip,
        &[],
        &[".git", "target", "curio-rs/target", "docs/wiki-demo/.git"],
    )?;
    bundles.push(BundleArtifact {
        filename: "curio-harness-source.zip".to_string(),
        path: harness_zip,
        description: "Harness source bundle".to_string(),
    });

    let kb_zip = bundle_root.join("curio-active-kb.zip");
    create_zip_bundle(&workspace_root, &kb_zip, &[], &[".git", "target"])?;
    bundles.push(BundleArtifact {
        filename: "curio-active-kb.zip".to_string(),
        path: kb_zip,
        description: "Active knowledge base bundle".to_string(),
    });

    if let Some(exe_path) = find_local_curio_executable(&harness_root) {
        let windows_zip = bundle_root.join("curio-harness-windows.zip");
        let extra: [(&Path, &str); 1] = [(exe_path.as_path(), "bin/curio.exe")];
        create_zip_bundle(
            &harness_root,
            &windows_zip,
            &extra,
            &[".git", "target", "curio-rs/target", "docs/wiki-demo/.git"],
        )?;
        bundles.push(BundleArtifact {
            filename: "curio-harness-windows.zip".to_string(),
            path: windows_zip,
            description: "Harness bundle with Windows executable".to_string(),
        });
    }

    let mut body = String::from("<p>Download snapshots generated by Curio during sync.</p><ul>");
    for bundle in &bundles {
        let bytes = fs::read(&bundle.path)
            .with_context(|| format!("Failed to read bundle {}", bundle.path.display()))?;
        client
            .upload_attachment(
                downloads_page_id,
                &bundle.filename,
                "application/zip",
                bytes,
            )
            .await?;
        body.push_str(&format!(
            "<li>{} — {}</li>",
            attachment_link(
                &config.connection.confluence_url,
                downloads_page_id,
                &bundle.filename
            ),
            html_escape(&bundle.description)
        ));
    }
    body.push_str("</ul><p>Curio updates these attachments in place when sync runs.</p>");
    client
        .update_page_body_by_id(downloads_page_id, "storage", &body)
        .await?;
    Ok(())
}

fn attachment_link(base_url: &str, page_id: &str, filename: &str) -> String {
    let url = format!(
        "{}/wiki/download/attachments/{}/{}?api=v2",
        base_url.trim_end_matches('/'),
        html_escape(page_id),
        html_escape(filename)
    );
    format!("<a href=\"{}\">{}</a>", url, html_escape(filename))
}

fn pipeline_body(summary: &str, detail: &str) -> String {
    format!(
        "<p>{}</p><p>{}</p><p>This page is maintained automatically by Curio.</p>",
        summary, detail
    )
}

fn render_branch_summary_only(
    space_key: &str,
    title: &str,
    description_html: &str,
    children: &[TreeNode],
) -> String {
    let mut body = String::new();
    body.push_str(&format!("<h1>{}</h1>", html_escape(title)));
    if !description_html.trim().is_empty() {
        body.push_str(description_html);
    }
    if !children.is_empty() {
        body.push_str("<h2>Child Sections</h2><ul>");
        for child in children {
            let link = page_link(space_key, &child.title, &child.title);
            body.push_str(&format!(
                "<li><strong>{}</strong>{}</li>",
                link,
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

fn render_branch_page_body(
    dir_path: &Path,
    space_key: &str,
    fallback_description_html: &str,
    title: &str,
) -> Result<String> {
    let index_path = dir_path.join("index.md");
    let mut body = if index_path.exists() {
        let raw = std::fs::read_to_string(&index_path)
            .with_context(|| format!("Failed to read {}", index_path.display()))?;
        let stripped = strip_frontmatter(&raw);
        let html = markdown_to_html(stripped);
        if !html.trim().is_empty() {
            html
        } else {
            render_branch_summary_only(space_key, title, fallback_description_html, &[])
        }
    } else {
        render_branch_summary_only(space_key, title, fallback_description_html, &[])
    };

    if let Ok(child_section) = render_immediate_child_links(dir_path, space_key)
        && !child_section.trim().is_empty()
    {
        body.push_str(&child_section);
    }
    Ok(body)
}

fn page_link(space_key: &str, title: &str, label: &str) -> String {
    format!(
        "<ac:link><ri:page ri:space-key=\"{}\" ri:content-title=\"{}\" /><ac:plain-text-link-body><![CDATA[{}]]></ac:plain-text-link-body></ac:link>",
        html_escape(space_key),
        html_escape(title),
        label
    )
}

/// Render a numeric score (0.0–1.0) as a compact text bar plus percent,
/// suitable for inline rendering in a Confluence storage-format table cell.
/// Example: `0.62` → `62% [▰▰▰▰▰▰▱▱▱▱]`.
fn score_bar(score: f32) -> String {
    let clamped = score.clamp(0.0, 1.0);
    let pct = (clamped * 100.0).round() as u32;
    let filled = (clamped * 10.0).round() as usize;
    let filled = filled.min(10);
    let bar: String = (0..10)
        .map(|i| if i < filled { '▰' } else { '▱' })
        .collect();
    format!("{}% <code>{}</code>", pct, bar)
}

/// Read a child page's proposal sidecar (if present) and produce a short
/// status badge: kind + route confidence + recommended action.
/// Returns an empty string when no sidecar is available — caller renders
/// without a badge in that case.
fn child_status_badge(path: &Path) -> String {
    let proposal = match load_proposal_record(path).ok().flatten() {
        Some(p) => p,
        None => return String::new(),
    };
    let conf = (proposal.scores.route_confidence * 100.0).round() as u32;
    let mut chips = format!("<code>{:?}</code>", proposal.kind);
    chips.push_str(&format!(" <em>{}%</em>", conf));
    if proposal.scores.overlap_risk >= 0.7 {
        chips.push_str(" <strong>⚠ overlap</strong>");
    }
    if proposal.taxonomy_mutation.is_some() {
        chips.push_str(" <strong>+ new node</strong>");
    }
    if let Some(ref merge) = proposal.merge_target {
        chips.push_str(&format!(" — merge → <code>{}</code>", html_escape(merge)));
    }
    format!(" {}", chips)
}

fn render_immediate_child_links(dir_path: &Path, space_key: &str) -> Result<String> {
    // (title, summary, kind, badge_html)
    let mut children: Vec<(String, String, String, String)> = Vec::new();
    for entry in std::fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if name.starts_with('.') || name.starts_with('_') || name == "index.md" {
            continue;
        }

        if path.is_dir() {
            let index = path.join("index.md");
            let title = if index.exists() {
                parse_wiki_page(&index)
                    .ok()
                    .map(|p| p.frontmatter.title)
                    .unwrap_or_else(|| to_title(name))
            } else {
                to_title(name)
            };
            let summary = if index.exists() {
                parse_wiki_page(&index)
                    .ok()
                    .map(|p| crate::wiki_fs::first_line_summary(&p.body, 160))
                    .unwrap_or_default()
            } else {
                String::new()
            };
            let badge = if index.exists() {
                child_status_badge(&index)
            } else {
                String::new()
            };
            children.push((title, summary, "branch".to_string(), badge));
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("md")
            && let Ok(page) = parse_wiki_page(&path)
        {
            let badge = child_status_badge(&path);
            children.push((
                page.frontmatter.title,
                crate::wiki_fs::first_line_summary(&page.body, 160),
                "leaf".to_string(),
                badge,
            ));
        }
    }

    children.sort_by(|a, b| a.0.cmp(&b.0));
    if children.is_empty() {
        return Ok(String::new());
    }

    let mut body = String::from("<h2>Child Pages</h2><ul>");
    for (title, summary, kind, badge) in children {
        let link = page_link(space_key, &title, &title);
        let role = if kind == "branch" { "branch" } else { "page" };
        body.push_str(&format!(
            "<li>{} <em>({})</em>{}{}</li>",
            link,
            role,
            badge,
            if summary.is_empty() {
                String::new()
            } else {
                format!(" — {}", html_escape(&summary))
            }
        ));
    }
    body.push_str("</ul>");
    Ok(body)
}

#[allow(clippy::too_many_arguments)]
async fn upsert_static_page(
    client: &ConfluenceClient,
    config: &Config,
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
        if let Some(target_parent_id) = parent_id
            && let Some(current_page) = client.get_page_by_id_v2(&page_id).await?
        {
            let current_parent_id = current_page["parentId"].as_str();
            if current_parent_id != Some(target_parent_id) {
                anyhow::bail!(
                    "Refusing to update same-title Confluence page {} outside target parent {}",
                    page_id,
                    target_parent_id
                );
            }
        }
        if let Ok(Some(prop)) = client.get_content_property(&page_id, SYNC_PROP_KEY).await
            && prop["value"]["content_hash"].as_str() == Some(hash.as_str())
        {
            if let Some(icon_val) = icon_override {
                set_page_icon_value(client, &page_id, icon_val).await;
            } else {
                set_page_icon(client, config, &page_id, slug).await;
            }
            return Ok(page_id);
        }

        client
            .update_page_body_by_id(&page_id, "storage", body)
            .await?;
        set_sync_prop(client, &page_id, &hash).await?;
        if let Some(icon_val) = icon_override {
            set_page_icon_value(client, &page_id, icon_val).await;
        } else {
            set_page_icon(client, config, &page_id, slug).await;
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
        set_page_icon(client, config, &page_id, slug).await;
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
        if rel.components().next().and_then(|c| c.as_os_str().to_str()) == Some("uncategorized") {
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

    for entry in WalkDir::new(root)
        .into_iter()
        .filter_map(|entry| entry.ok())
    {
        let path = entry.path();
        if !entry.file_type().is_file()
            || path.extension().and_then(|ext| ext.to_str()) != Some("md")
        {
            continue;
        }

        let name = path.file_name().unwrap_or_default().to_string_lossy();
        if name == "index.md"
            || name == ".gitkeep"
            || name.ends_with(".analysis.json")
            || name.ends_with(".proposal.json")
        {
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

        let page =
            parse_wiki_page(path).with_context(|| format!("Failed to parse {}", path.display()))?;
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
        anyhow::bail!(
            "Published sync validation failed:\n- {}",
            errors.join("\n- ")
        );
    }
}

fn valid_published_routes(trees: &[TreeNode]) -> HashSet<String> {
    let mut routes = HashSet::new();
    for tree in trees {
        collect_valid_routes(tree, &mut Vec::new(), &mut routes);
    }
    routes
}

fn collect_valid_routes(
    node: &TreeNode,
    parent_segments: &mut Vec<String>,
    routes: &mut HashSet<String>,
) {
    parent_segments.push(node.slug.clone());
    routes.insert(parent_segments.join("/"));
    for child in &node.subtrees {
        collect_valid_routes(child, parent_segments, routes);
    }
    parent_segments.pop();
}

#[allow(clippy::too_many_arguments)]
async fn sync_lane_directory(
    client: &ConfluenceClient,
    config: &Config,
    space_key: &str,
    root_dir: &Path,
    root_page_id: &str,
    lane: &str,
    full_refresh: bool,
    synced_ids: &mut HashSet<String>,
    skipped: &mut Vec<String>,
    upserted: &mut Vec<String>,
    errors: &mut Vec<String>,
    auto_heal_label: &str,
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
            let branch_body = render_lane_branch_body(root_dir, space_key, &rel_path, lane)?;
            match upsert_page(
                client,
                config,
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
        } else if rel_path.extension().is_some_and(|ext| ext == "md") {
            match sync_lane_page(
                client,
                config,
                space_key,
                parent_conf_id.as_deref(),
                &abs_path,
                lane,
                full_refresh,
                synced_ids,
                skipped,
                auto_heal_label,
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

#[allow(clippy::too_many_arguments)]
async fn sync_review_proposals(
    client: &ConfluenceClient,
    config: &Config,
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
        match sync_proposal_page(
            client,
            config,
            space_key,
            Some(parent_id),
            &path,
            synced_ids,
            skipped,
        )
        .await
        {
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
#[allow(clippy::too_many_arguments)]
async fn sync_taxonomy_reconciliation_index(
    client: &ConfluenceClient,
    config: &Config,
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
                && e.path()
                    .to_str()
                    .is_some_and(|s| s.ends_with(".analysis.json"))
        })
    {
        let path = entry.path();
        let Ok(raw) = std::fs::read_to_string(path) else {
            continue;
        };
        let Ok(analysis) = serde_json::from_str::<serde_json::Value>(&raw) else {
            continue;
        };
        let Some(subtree) = analysis["routing"]["proposed_new_subtree"].as_str() else {
            continue;
        };
        // Title lives under inputs.title (the original source page title)
        let title = analysis["inputs"]["title"]
            .as_str()
            .or_else(|| analysis["title"].as_str())
            .unwrap_or("Untitled")
            .to_string();
        // Slug is derived from the .analysis.json filename (strip the suffix)
        let slug = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .trim_end_matches(".analysis.json")
            .to_string();
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
    body.push_str(&format!(
        "<p><strong>{}</strong> proposed new subtrees across <strong>{}</strong> review items.</p>",
        groups.len(),
        groups.values().map(|v| v.len()).sum::<usize>()
    ));

    for (subtree, pages) in &mut groups {
        pages.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
        body.push_str(&format!("<h2>{}</h2>", html_escape(subtree)));
        body.push_str(&format!(
            "<p>{} page(s) propose this subtree:</p>",
            pages.len()
        ));
        body.push_str("<table><tbody><tr><th>Page</th><th>Confidence</th></tr>");
        for (title, slug, confidence) in pages.iter() {
            // Try to load the .sync-refs.json sidecar to get the Confluence review page ID.
            // The analysis file path is slug-derived; the sidecar sits next to the .md file.
            let conf_page_id: Option<String> = {
                // Walk review_dir to find a sync-refs sidecar whose stem matches slug
                walkdir::WalkDir::new(review_dir)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .find(|e| {
                        e.path()
                            .file_name()
                            .and_then(|n| n.to_str())
                            .map(|n| n == format!("{}.sync-refs.json", slug))
                            .unwrap_or(false)
                    })
                    .and_then(|e| std::fs::read_to_string(e.path()).ok())
                    .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
                    .and_then(|v| {
                        v["confluence_review_page_id"]
                            .as_str()
                            .map(|s| s.to_string())
                    })
            };
            let confluence_base = config
                .connection
                .confluence_url
                .trim_end_matches("/wiki")
                .trim_end_matches('/');
            let cell = if let Some(page_id) = conf_page_id {
                format!(
                    "<a href=\"{}/wiki/spaces/{}/pages/{}\"><strong>{}</strong></a>",
                    confluence_base,
                    space_key,
                    page_id,
                    html_escape(title)
                )
            } else {
                html_escape(title).to_string()
            };
            body.push_str(&format!(
                "<tr><td>{}</td><td>{:.0}%</td></tr>",
                cell,
                confidence * 100.0,
            ));
        }
        body.push_str("</tbody></table>");
        body.push_str("<p><strong>To signal a decision:</strong> open an individual review page above and react 👍 (approve), 👎 (reject), or ❓ (rewrite) on the pinned Curio comment at the bottom of the page. Then run <code>curio feedback</code> to apply.</p>");
        body.push_str("<p>To approve the entire subtree at once, add the label <code>curio:approve</code> to each page in the group, then run <code>curio feedback</code>.</p><hr/>");
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
            upserted.push(format!(
                "[taxonomy reconciliation index] {} groups",
                groups.len()
            ));
        }
        Err(e) => errors.push(format!("[taxonomy reconciliation index]: {}", e)),
    }

    Ok(())
}

// ─── Page operations ──────────────────────────────────────────────────────

async fn sync_page(
    client: &ConfluenceClient,
    config: &Config,
    space_key: &str,
    parent_id: Option<&str>,
    path: &Path,
    synced_ids: &mut HashSet<String>,
    skipped: &mut Vec<String>,
) -> Result<String> {
    let page =
        parse_wiki_page(path).with_context(|| format!("Failed to parse {}", path.display()))?;
    let page_title = page.frontmatter.title.trim().to_string();
    if page_title.is_empty() {
        anyhow::bail!("Page title is empty for {}", path.display());
    }
    let body_md = page.body.as_str();
    let hash = content_hash(body_md);
    let html_body = markdown_to_html(body_md);
    sync_page_html(
        client,
        config,
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

#[allow(clippy::too_many_arguments)]
async fn sync_lane_page(
    client: &ConfluenceClient,
    config: &Config,
    space_key: &str,
    parent_id: Option<&str>,
    path: &Path,
    lane: &str,
    full_refresh: bool,
    synced_ids: &mut HashSet<String>,
    skipped: &mut Vec<String>,
    auto_heal_label: &str,
) -> Result<String> {
    let page =
        parse_wiki_page(path).with_context(|| format!("Failed to parse {}", path.display()))?;
    let raw_title = page
        .frontmatter
        .title
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    let safe_title = raw_title.trim_start_matches('/').trim().to_string();
    let page_title = format!("{} - {}", to_title(lane), safe_title);
    if page_title.is_empty() {
        anyhow::bail!("Page title is empty for {}", path.display());
    }
    let html_body = render_lane_page_body(path, &page, lane)?;
    let hash = content_hash(&html_body);
    let result_title = sync_page_html(
        client,
        config,
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
    // and post/update the single pinned reaction-instruction footer comment.
    // In incremental mode, skip this block entirely when the sidecar already has both IDs —
    // the comment body is static and never needs updating unless --all is passed.
    if lane == "review" {
        let refs_path = path.with_extension("sync-refs.json");
        let existing_refs: serde_json::Value = if refs_path.exists() {
            std::fs::read_to_string(&refs_path)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or(serde_json::json!({}))
        } else {
            serde_json::json!({})
        };
        let already_set = existing_refs["confluence_review_page_id"].is_string()
            && existing_refs["pinned_comment_id"].is_string();
        if !full_refresh && already_set {
            // Incremental: sidecar is complete, skip API calls.
        } else if let Ok(Some(existing)) = client
            .get_page_by_title(space_key, parent_id, &result_title)
            .await
            && let Some(page_id) = existing["id"].as_str()
        {
            // Upsert the pinned comment ──────────────────────────────────────────
            let pinned_body = "<p><strong>Curio review signals.</strong> \
                    React to <em>this comment</em> with \
                    \u{1F44D} <strong>approve</strong> (promotes the proposal toward published), \
                    \u{1F44E} <strong>reject</strong> (deletes the page and records the reason), \
                    or \u{2753} <strong>rewrite</strong> (sends it back for another curation pass). \
                    Equivalent labels on the page: <code>curio:approve</code>, <code>curio:reject</code>, <code>curio:rewrite</code>. \
                    Free-form replies are captured as reviewer feedback for the agent. \
                    Run <code>curio feedback</code> to apply pending signals.</p>";

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
            let pinned_comment_id =
                if let Some(existing_id) = existing_refs["pinned_comment_id"].as_str() {
                    // Try to update existing; if it fails (deleted), create a new one
                    match client.update_footer_comment(existing_id, pinned_body).await {
                        Ok(_) => existing_id.to_string(),
                        Err(e) => {
                            eprintln!(
                                "  [warn] update pinned comment {} failed ({}), creating new one",
                                existing_id, e
                            );
                            match client.create_footer_comment(page_id, pinned_body).await {
                                Ok(id) => id,
                                Err(e2) => {
                                    eprintln!(
                                        "  [warn] create_footer_comment for page {} failed: {}",
                                        page_id, e2
                                    );
                                    String::new()
                                }
                            }
                        }
                    }
                } else {
                    match client.create_footer_comment(page_id, pinned_body).await {
                        Ok(id) => id,
                        Err(e) => {
                            eprintln!(
                                "  [warn] create_footer_comment for page {} failed: {}",
                                page_id, e
                            );
                            String::new()
                        }
                    }
                };

            let pinned_id_opt = if pinned_comment_id.is_empty() {
                None
            } else {
                Some(pinned_comment_id.as_str())
            };
            write_sync_refs(path, page_id, pinned_id_opt);
            // Apply auto-heal label if this page was auto-healed.
            if page.frontmatter.auto_healed_at.is_some()
                && !auto_heal_label.is_empty()
                && let Err(e) = client
                    .add_labels(page_id, vec![auto_heal_label.to_string()])
                    .await
            {
                eprintln!(
                    "  [warn] Failed to apply auto-heal label to page {}: {}",
                    page_id, e
                );
            }
        }
    }

    Ok(result_title)
}

/// Write (or update) the .sync-refs.json sidecar next to the wiki page.
/// This persists the Confluence review page ID (and optionally the pinned
/// comment ID) so `curio feedback` can read labels/reactions without
/// performing expensive title lookups.
fn write_sync_refs(
    wiki_page_path: &Path,
    confluence_page_id: &str,
    pinned_comment_id: Option<&str>,
) {
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
    config: &Config,
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
        path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("untitled")
    );
    let html_body = render_proposal_body(&payload);
    let hash = content_hash(&raw);
    sync_page_html(
        client, config, space_key, parent_id, path, &title, &hash, &html_body, synced_ids, skipped,
        false,
    )
    .await
}

#[allow(clippy::too_many_arguments)]
async fn sync_page_html(
    client: &ConfluenceClient,
    config: &Config,
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
    if let Some(ref existing_page) = client
        .get_page_by_title(space_key, parent_id, page_title)
        .await?
    {
        let page_id = existing_page["id"].as_str().unwrap_or_default().to_string();
        if let Ok(Some(prop)) = client.get_content_property(&page_id, SYNC_PROP_KEY).await
            && prop["value"]["content_hash"].as_str() == Some(hash)
        {
            skipped.push(page_title.to_string());
            synced_ids.insert(page_id);
            return Ok(page_title.to_string());
        }
        let slug = path.file_stem().unwrap_or_default().to_str().unwrap_or("");
        let page_id = client
            .create_or_update_page(space_key, parent_id, page_title, "storage", html_body)
            .await?;
        set_sync_prop(client, &page_id, hash).await?;
        set_page_icon(client, config, &page_id, slug).await;
        synced_ids.insert(page_id);
        return Ok(page_title.to_string());
    }

    if allow_duplicate_fallback {
        if let Some(conflicting_page) = client
            .get_page_by_title(space_key, None, page_title)
            .await?
        {
            let conflicting_id = conflicting_page["id"]
                .as_str()
                .unwrap_or_default()
                .to_string();
            if let Some(target_parent_id) = parent_id
                && let Some(current_page) = client.get_page_by_id_v2(&conflicting_id).await?
            {
                let current_parent_id = current_page["parentId"].as_str();
                if current_parent_id != Some(target_parent_id) {
                    anyhow::bail!(
                        "Refusing to update same-title Confluence page {} outside target parent {}",
                        conflicting_id,
                        target_parent_id
                    );
                }
            }
            client
                .update_page_body_by_id(&conflicting_id, "storage", html_body)
                .await?;
            set_sync_prop(client, &conflicting_id, hash).await?;
            set_page_icon(
                client,
                config,
                &conflicting_id,
                path.file_stem().unwrap_or_default().to_str().unwrap_or(""),
            )
            .await;
            synced_ids.insert(conflicting_id);
            return Ok(page_title.to_string());
        }
    } else if let Some(existing_global_page) = client
        .get_page_by_title(space_key, None, page_title)
        .await?
    {
        let page_id = existing_global_page["id"]
            .as_str()
            .unwrap_or_default()
            .to_string();
        if let Some(target_parent_id) = parent_id
            && let Some(current_page) = client.get_page_by_id_v2(&page_id).await?
        {
            let current_parent_id = current_page["parentId"].as_str();
            if current_parent_id != Some(target_parent_id) {
                anyhow::bail!(
                    "Refusing to update same-title Confluence page {} outside target parent {}",
                    page_id,
                    target_parent_id
                );
            }
        }
        client
            .update_page_body_by_id(&page_id, "storage", html_body)
            .await?;
        set_sync_prop(client, &page_id, hash).await?;
        set_page_icon(
            client,
            config,
            &page_id,
            path.file_stem().unwrap_or_default().to_str().unwrap_or(""),
        )
        .await;
        synced_ids.insert(page_id);
        return Ok(page_title.to_string());
    }

    let slug = path.file_stem().unwrap_or_default().to_str().unwrap_or("");
    let page_id = match client
        .create_or_update_page(space_key, parent_id, page_title, "storage", html_body)
        .await
    {
        Ok(page_id) => page_id,
        Err(err) => {
            let err_text = err.to_string();
            if allow_duplicate_fallback
                && (err_text.contains("same TITLE in this space")
                    || err_text.contains("title already exists"))
                && let Some(conflicting_page) = client
                    .get_page_by_title(space_key, None, page_title)
                    .await?
            {
                let conflicting_id = conflicting_page["id"]
                    .as_str()
                    .unwrap_or_default()
                    .to_string();
                if let Some(target_parent_id) = parent_id
                    && let Some(current_page) = client.get_page_by_id_v2(&conflicting_id).await?
                {
                    let current_parent_id = current_page["parentId"].as_str();
                    if current_parent_id != Some(target_parent_id) {
                        anyhow::bail!(
                            "Refusing to update same-title Confluence page {} outside target parent {}",
                            conflicting_id,
                            target_parent_id
                        );
                    }
                }
                client
                    .update_page_body_by_id(&conflicting_id, "storage", html_body)
                    .await?;
                set_sync_prop(client, &conflicting_id, hash).await?;
                set_page_icon(client, config, &conflicting_id, slug).await;
                synced_ids.insert(conflicting_id);
                return Ok(page_title.to_string());
            }
            return Err(err);
        }
    };
    set_sync_prop(client, &page_id, hash).await?;
    set_page_icon(client, config, &page_id, slug).await;
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
    let route_confidence = page.frontmatter.confidence.unwrap_or(0.0);
    let mut body = String::new();
    let lane_label = if lane == "review" { "Review" } else { "Staged" };

    // ── Banner ───────────────────────────────────────────────────────────
    let banner_kind = if lane == "review" { "warning" } else { "info" };
    let banner_blurb = if lane == "review" {
        "This page needs human judgment. Read the curation decision below, then react or label to signal an outcome (see the pinned Curio comment at the bottom)."
    } else {
        "This page is a curated proposal awaiting publication. The curation decision below records the agent's rationale, scores, and alternatives. Approve to promote to published."
    };
    body.push_str(&format!(
        "<ac:structured-macro ac:name=\"{}\"><ac:rich-text-body><p><strong>{} item in the Curio workflow.</strong> {}</p></ac:rich-text-body></ac:structured-macro>",
        banner_kind, lane_label, banner_blurb
    ));

    // ── Section 1: Curation Decision (7-dim scores) ──────────────────────
    body.push_str("<h2>Curation Decision</h2>");
    body.push_str("<table><tbody>");
    body.push_str(&format!(
        "<tr><th>Status</th><td>{}</td></tr>",
        html_escape(page.frontmatter.status.as_str())
    ));
    body.push_str(&format!(
        "<tr><th>Route</th><td>{}</td></tr>",
        html_escape(&route)
    ));
    body.push_str(&format!(
        "<tr><th>Route confidence</th><td>{}</td></tr>",
        score_bar(route_confidence)
    ));
    body.push_str(&format!(
        "<tr><th>Information quality</th><td>{}</td></tr>",
        score_bar(quality.information_quality)
    ));
    body.push_str(&format!(
        "<tr><th>Usability</th><td>{}</td></tr>",
        score_bar(quality.usability)
    ));
    if let Some(ref proposal) = proposal {
        body.push_str(&format!(
            "<tr><th>Hierarchy fit</th><td>{}</td></tr>",
            score_bar(proposal.scores.hierarchy_fit_confidence)
        ));
        body.push_str(&format!(
            "<tr><th>Overlap risk</th><td>{}</td></tr>",
            score_bar(proposal.scores.overlap_risk)
        ));
        body.push_str(&format!(
            "<tr><th>Evidence completeness</th><td>{}</td></tr>",
            score_bar(proposal.scores.evidence_completeness)
        ));
        body.push_str(&format!(
            "<tr><th>Freshness</th><td>{}</td></tr>",
            score_bar(proposal.scores.freshness_confidence)
        ));
    }
    if !quality.flags.is_empty() {
        body.push_str(&format!(
            "<tr><th>Quality flags</th><td>{}</td></tr>",
            html_escape(&quality.flags.join(", "))
        ));
    }
    body.push_str("</tbody></table>");

    // ── Section 2: Recommendation + structural mutations ─────────────────
    if let Some(ref proposal) = proposal {
        body.push_str("<h2>Recommendation</h2>");
        body.push_str("<table><tbody>");
        body.push_str(&format!(
            "<tr><th>Proposal kind</th><td>{}</td></tr>",
            html_escape(&format!("{:?}", proposal.kind))
        ));
        body.push_str(&format!(
            "<tr><th>Recommended action</th><td>{}</td></tr>",
            html_escape(&proposal.recommended_action)
        ));
        if let Some(ref merge) = proposal.merge_target {
            body.push_str(&format!(
                "<tr><th>Merge target</th><td><code>{}</code></td></tr>",
                html_escape(merge)
            ));
        }
        if let Some(ref reason) = proposal.review_reason {
            body.push_str(&format!(
                "<tr><th>Review reason</th><td>{}</td></tr>",
                html_escape(reason)
            ));
        }
        body.push_str("</tbody></table>");

        // Taxonomy mutation, if proposed — its own panel because reviewers
        // approving this affects the whole KB tree.
        if let Some(ref tm) = proposal.taxonomy_mutation {
            body.push_str("<h3>Taxonomy mutation proposed</h3>");
            body.push_str("<ac:structured-macro ac:name=\"note\"><ac:rich-text-body>");
            body.push_str(
                "<p>Approving this proposal also adds a new node to <code>NORTHSTAR.md</code>.</p>",
            );
            body.push_str("<table><tbody>");
            body.push_str(&format!(
                "<tr><th>New node title</th><td>{}</td></tr>",
                html_escape(&tm.proposed_node_title)
            ));
            body.push_str(&format!(
                "<tr><th>Slug</th><td><code>{}</code></td></tr>",
                html_escape(&tm.proposed_node_slug)
            ));
            if !tm.proposed_parent_path.is_empty() {
                body.push_str(&format!(
                    "<tr><th>Parent path</th><td><code>{}</code></td></tr>",
                    html_escape(&tm.proposed_parent_path.join(" / "))
                ));
            }
            if !tm.node_description.trim().is_empty() {
                body.push_str(&format!(
                    "<tr><th>Description</th><td>{}</td></tr>",
                    html_escape(&tm.node_description)
                ));
            }
            if !tm.rationale.trim().is_empty() {
                body.push_str(&format!(
                    "<tr><th>Rationale</th><td>{}</td></tr>",
                    html_escape(&tm.rationale)
                ));
            }
            if !tm.rejected_nearby_nodes.is_empty() {
                body.push_str(&format!(
                    "<tr><th>Considered + rejected</th><td>{}</td></tr>",
                    html_escape(&tm.rejected_nearby_nodes.join(", "))
                ));
            }
            body.push_str("</tbody></table>");
            body.push_str("</ac:rich-text-body></ac:structured-macro>");
        }

        // Alternatives the agent considered.
        if !proposal.dossier.alternatives_considered.is_empty() {
            body.push_str("<h3>Alternatives considered</h3><ul>");
            for alt in &proposal.dossier.alternatives_considered {
                body.push_str(&format!("<li>{}</li>", html_escape(alt)));
            }
            body.push_str("</ul>");
        }
        if !proposal.dossier.unresolved_questions.is_empty() {
            body.push_str("<h3>Unresolved questions</h3><ul>");
            for q in &proposal.dossier.unresolved_questions {
                body.push_str(&format!("<li>{}</li>", html_escape(q)));
            }
            body.push_str("</ul>");
        }
        if !proposal.dossier.rationale.trim().is_empty() {
            body.push_str(&format!(
                "<h3>Rationale</h3><p>{}</p>",
                html_escape(&proposal.dossier.rationale)
            ));
        }

        // Body-rewrite badge — what the agent did to produce the body below.
        if let Some(ref kind) = proposal.dossier.body_rewrite_kind
            && (kind != "none" || proposal.dossier.decision_section_present)
        {
            body.push_str(&format!(
                "<p><em>Body rewrite: <code>{}</code>{}</em></p>",
                html_escape(kind),
                if proposal.dossier.decision_section_present {
                    " (with structured decision section)"
                } else {
                    ""
                }
            ));
        }
    } else if let Some(ref analysis) = analysis {
        // No proposal sidecar but analysis exists — surface what we have.
        body.push_str("<h2>Routing analysis</h2><table><tbody>");
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
        body.push_str("</tbody></table>");
    }

    body.push_str("<h2>Proposed page</h2>");
    body.push_str(&markdown_to_html(&page.body));
    // If this page was auto-healed, append a visible info callout.
    if let Some(ref healed_at) = page.frontmatter.auto_healed_at {
        let confidence = page.frontmatter.auto_healed_confidence.unwrap_or(0.0);
        // Show only YYYY-MM-DD
        let date_str = &healed_at[..healed_at.len().min(10)];
        let callout = format!(
            "<ac:structured-macro ac:name=\"info\"><ac:rich-text-body><p>\
            ⚡ Auto-healed by Curio on {} | confidence: {:.0}%\
            </p></ac:rich-text-body></ac:structured-macro>",
            date_str,
            confidence * 100.0,
        );
        body.push_str(&callout);
    }
    Ok(body)
}

fn render_lane_branch_body(
    root_dir: &Path,
    space_key: &str,
    rel_path: &Path,
    lane: &str,
) -> Result<String> {
    let abs_dir = root_dir.join(rel_path);
    let mut body = format!(
        "<p>{} proposal branch for <code>{}</code>.</p>",
        html_escape(&to_title(lane)),
        html_escape(&rel_path.display().to_string())
    );
    if let Ok(child_outline) = render_immediate_child_links(&abs_dir, space_key)
        && !child_outline.trim().is_empty()
    {
        body.push_str(&child_outline);
    }
    Ok(body)
}

fn render_proposal_body(payload: &serde_json::Value) -> String {
    let generated_at = payload["generated_at"].as_str().unwrap_or("");
    let proposals = payload["proposals"].as_array().cloned().unwrap_or_default();
    let mut out = String::new();
    out.push_str("<h1>Sharpening Proposal Set</h1>");
    if !generated_at.is_empty() {
        out.push_str(&format!(
            "<p>Generated at {}</p>",
            html_escape(generated_at)
        ));
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
            let joined = paths
                .iter()
                .filter_map(|v| v.as_str())
                .collect::<Vec<_>>()
                .join("<br/>");
            out.push_str(&format!(
                "<tr><th>Affected paths</th><td>{}</td></tr>",
                joined
            ));
        }
        if let Some(rationale) = proposal["rationale"].as_str() {
            out.push_str(&format!(
                "<tr><th>Rationale</th><td>{}</td></tr>",
                html_escape(rationale)
            ));
        }
        if let Some(confidence) = proposal["confidence"].as_f64() {
            out.push_str(&format!(
                "<tr><th>Confidence</th><td>{:.0}%</td></tr>",
                confidence * 100.0
            ));
        }
        if let Some(gain) = proposal["expected_signal_gain"].as_str() {
            out.push_str(&format!(
                "<tr><th>Expected signal gain</th><td>{}</td></tr>",
                html_escape(gain)
            ));
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

#[allow(clippy::too_many_arguments)]
async fn upsert_page(
    client: &ConfluenceClient,
    config: &Config,
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
        if let Some(target_parent_id) = parent_id
            && let Some(current_page) = client.get_page_by_id_v2(&page_id).await?
        {
            let current_parent_id = current_page["parentId"].as_str();
            if current_parent_id != Some(target_parent_id) {
                anyhow::bail!(
                    "Refusing to update same-title Confluence page {} outside target parent {}",
                    page_id,
                    target_parent_id
                );
            }
        }
        client
            .update_page_body_by_id(&page_id, "storage", body)
            .await?;
        set_sync_prop(client, &page_id, &hash).await?;
        if let Some(icon_val) = icon_override {
            set_page_icon_value(client, &page_id, icon_val).await;
        } else {
            set_page_icon(client, config, &page_id, slug).await;
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
        set_page_icon(client, config, &page_id, slug).await;
    }
    Ok(page_id)
}

async fn find_existing_page_for_sync(
    client: &ConfluenceClient,
    space_key: &str,
    parent_id: Option<&str>,
    title: &str,
) -> Result<Option<serde_json::Value>> {
    if let Some(page) = client
        .get_page_by_title(space_key, parent_id, title)
        .await?
    {
        return Ok(Some(page));
    }
    client.get_page_by_title(space_key, None, title).await
}

async fn set_sync_prop(client: &ConfluenceClient, page_id: &str, hash: &str) -> Result<()> {
    let value = serde_json::json!({ "content_hash": hash, "synced_by": "curio" });
    client
        .set_content_property(page_id, SYNC_PROP_KEY, value)
        .await
}

async fn find_owned_stale_pages(
    client: &ConfluenceClient,
    parent_id: &str,
    synced_ids: &HashSet<String>,
) -> Result<(Vec<String>, Vec<String>, Vec<String>)> {
    let all = client
        .get_page_descendants_v2(parent_id)
        .await
        .with_context(|| {
            format!("failed to enumerate descendants below managed root {parent_id}")
        })?;

    let mut candidates = Vec::new();
    let mut owned = Vec::new();
    let mut preserved = Vec::new();
    for page in all {
        let Some(page_id) = page["id"].as_str() else {
            continue;
        };
        let page_id = page_id.to_string();
        if synced_ids.contains(&page_id) {
            continue;
        }
        candidates.push(page_id.clone());
        match client.get_content_property(&page_id, SYNC_PROP_KEY).await? {
            Some(prop) if prop["value"]["synced_by"].as_str() == Some("curio") => {
                owned.push(page_id);
            }
            _ => preserved.push(page_id),
        }
    }
    Ok((candidates, owned, preserved))
}

// ─── Helpers ──────────────────────────────────────────────────────────────

fn markdown_to_html(md: &str) -> String {
    crate::md_to_confluence::markdown_to_storage(md).unwrap_or_else(|_| {
        // Fallback: plain pulldown_cmark if macro parsing fails
        use pulldown_cmark::{Options, Parser, html};
        let mut opts = Options::empty();
        opts.insert(Options::ENABLE_TABLES);
        let parser = Parser::new_ext(md, opts);
        let mut out = String::new();
        html::push_html(&mut out, parser);
        out
    })
}

fn find_local_curio_executable(harness_root: &Path) -> Option<PathBuf> {
    let candidates = [
        harness_root
            .join("target")
            .join("release")
            .join("curio.exe"),
        harness_root
            .join("curio-rs")
            .join("target")
            .join("release")
            .join("curio.exe"),
        harness_root.join("target").join("release").join("curio"),
        harness_root
            .join("curio-rs")
            .join("target")
            .join("release")
            .join("curio"),
    ];
    candidates.into_iter().find(|path| path.exists())
}

fn create_zip_bundle(
    root: &Path,
    out_path: &Path,
    extra_files: &[(&Path, &str)],
    excluded_prefixes: &[&str],
) -> Result<()> {
    let file = fs::File::create(out_path)
        .with_context(|| format!("Failed to create bundle {}", out_path.display()))?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o644);

    for entry in walkdir::WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path == out_path {
            continue;
        }
        let rel = match path.strip_prefix(root) {
            Ok(rel) => rel,
            Err(_) => continue,
        };
        if rel.as_os_str().is_empty() {
            continue;
        }
        let rel_str = rel.to_string_lossy().replace('\\', "/");
        if excluded_prefixes
            .iter()
            .any(|prefix| rel_str == *prefix || rel_str.starts_with(&format!("{prefix}/")))
        {
            continue;
        }
        if entry.file_type().is_dir() {
            zip.add_directory(rel_str, options)?;
        } else {
            zip.start_file(rel_str, options)?;
            let mut src = fs::File::open(path)
                .with_context(|| format!("Failed to read {}", path.display()))?;
            let mut buf = Vec::new();
            src.read_to_end(&mut buf)?;
            zip.write_all(&buf)?;
        }
    }

    for (src_path, zip_name) in extra_files {
        if !src_path.exists() {
            continue;
        }
        zip.start_file(*zip_name, options)?;
        let mut src = fs::File::open(src_path)
            .with_context(|| format!("Failed to read {}", src_path.display()))?;
        let mut buf = Vec::new();
        src.read_to_end(&mut buf)?;
        zip.write_all(&buf)?;
    }

    zip.finish()?;
    Ok(())
}

/// Set the emoji icon on a Confluence page. Resolution goes through
/// `page_icon_for` — operator-supplied registry first, builtin defaults
/// last. Best-effort: silently no-op on API errors and on slugs with no
/// configured icon.
async fn set_page_icon(client: &ConfluenceClient, config: &Config, page_id: &str, slug: &str) {
    if let Some(icon) = page_icon_for(slug, config) {
        set_page_icon_value(client, page_id, &icon).await;
    }
}

/// Set the emoji icon on a Confluence page using a raw emoji value (e.g. "1f4e6"). Best-effort.
async fn set_page_icon_value(client: &ConfluenceClient, page_id: &str, icon: &str) {
    let val = serde_json::json!(icon);
    let _ = client
        .set_content_property(page_id, ICON_PROP_KEY, val)
        .await;
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
    let page =
        parse_wiki_page(path).with_context(|| format!("Failed to parse {}", path.display()))?;
    let title = page.frontmatter.title.trim().to_string();
    if title.is_empty() {
        anyhow::bail!("Published page {} is missing a title", path.display());
    }
    Ok(title)
}

// ─── NORTHSTAR rich Confluence renderer ──────────────────────────────────

/// Render the Northstar page as prose followed by the deterministic config YAML.
fn render_northstar_for_confluence(northstar_md: &str, config_yaml: &str) -> String {
    let mut out = String::new();
    out.push_str(&markdown_to_html(northstar_md));
    out.push_str(
        "<h2>Config</h2>\
         <p>The YAML below is the deterministic Curio config used for taxonomy and runtime settings.</p>",
    );
    out.push_str(&render_yaml_for_confluence(config_yaml));
    out
}

fn render_yaml_for_confluence(yaml: &str) -> String {
    format!(
        "<ac:structured-macro ac:name=\"code\"><ac:parameter ac:name=\"language\">yaml</ac:parameter><ac:plain-text-body><![CDATA[{}]]></ac:plain-text-body></ac:structured-macro>",
        yaml
    )
}

fn inline_desc(html: &str) -> String {
    let text = html
        .replace("<blockquote>", "")
        .replace("</blockquote>", " ")
        .replace("<p>", "")
        .replace("</p>", " ")
        .replace("<br/>", " ")
        .replace("<br />", " ");
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
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
        if md.trim().is_empty() {
            return String::new();
        }
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
        if !in_blueprint {
            continue;
        }

        if let Some(stripped) = line.strip_prefix("### ") {
            // Flush previous subtree into current tree
            if let Some(mut sub) = current_sub.take() {
                sub.description_html = flush_desc(&mut desc_lines);
                if let Some(ref mut t) = current_tree {
                    t.subtrees.push(sub);
                }
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
            let title = stripped.trim().to_string();
            let slug = title.to_lowercase().replace(' ', "-");
            current_tree = Some(TreeNode {
                title,
                slug,
                ..Default::default()
            });
        } else if let Some(stripped) = line.strip_prefix("#### ") {
            // Flush previous subtree (or tree description if this is the first subtree)
            if let Some(mut sub) = current_sub.take() {
                sub.description_html = flush_desc(&mut desc_lines);
                if let Some(ref mut t) = current_tree {
                    t.subtrees.push(sub);
                }
            } else {
                // First subtree — flush accumulated lines as the parent tree's description
                let html = flush_desc(&mut desc_lines);
                if let Some(ref mut t) = current_tree
                    && t.description_html.is_empty()
                {
                    t.description_html = html;
                }
            }
            let title = stripped.trim().to_string();
            // "Technical / CSE" → "technical-cse", collapsing runs of non-alpha chars
            let slug: String = title
                .to_lowercase()
                .chars()
                .map(|c| if c.is_alphanumeric() { c } else { '-' })
                .collect::<String>()
                .split('-')
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
                .join("-");
            current_sub = Some(TreeNode {
                title,
                slug,
                ..Default::default()
            });
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
        if let Some(ref mut t) = current_tree {
            t.subtrees.push(sub);
        }
    } else {
        let html = flush_desc(&mut desc_lines);
        if let Some(ref mut t) = current_tree {
            t.description_html = html;
        }
    }
    if let Some(t) = current_tree.take() {
        trees.push(t);
    }
    trees
}

/// Dry-run path for `curio sync --dry-run`. Enumerates the published /
/// staged / review lanes from the filesystem and prints what WOULD be
/// pushed to Confluence — never constructs a Confluence client and
/// never makes API calls. Operators can safely run this against a live
/// KB to preview a sync before committing.
async fn run_sync_dry_run(
    config: &Config,
    json: bool,
    docs_only: bool,
    downloads_only: bool,
) -> Result<()> {
    let wiki_dir = &config.wiki.wiki_dir;
    let mut planned: Vec<String> = Vec::new();
    let mut count_published = 0usize;
    let mut count_staged = 0usize;
    let mut count_review = 0usize;
    let mut count_admin = 0usize;

    let count_lane = |lane_dir: &std::path::Path, lane: &str, planned: &mut Vec<String>| -> usize {
        if !lane_dir.exists() {
            return 0;
        }
        let mut count = 0usize;
        for entry in walkdir::WalkDir::new(lane_dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            let rel = path.strip_prefix(lane_dir).unwrap_or(path);
            let name = rel.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name.is_empty() || name.starts_with('.') || name.starts_with('_') {
                continue;
            }
            if path.is_dir() {
                planned.push(format!("[{lane}/dir]  {}", rel.display()));
                count += 1;
            } else if path.extension().and_then(|e| e.to_str()) == Some("md") && name != "index.md"
            {
                let title = crate::wiki_fs::parse_wiki_page(path)
                    .map(|p| p.frontmatter.title)
                    .unwrap_or_else(|_| to_title(name.trim_end_matches(".md")));
                planned.push(format!("[{lane}/page] {} — {}", title, rel.display()));
                count += 1;
            }
        }
        count
    };

    if !downloads_only {
        count_admin = count_lane(
            &wiki_dir.join(crate::northstar::ADMIN_DIRNAME),
            "admin",
            &mut planned,
        );
        count_published = count_lane(&wiki_dir.join("published"), "published", &mut planned);
        count_staged = count_lane(&wiki_dir.join("staged"), "staged", &mut planned);
        count_review = count_lane(&wiki_dir.join("review"), "review", &mut planned);
        planned.push("[taxonomy reconciliation index] (would generate from review/)".to_string());
    }
    if !docs_only {
        planned.push("[downloads] (would refresh download bundles)".to_string());
    }

    let total =
        count_admin + count_published + count_staged + count_review + if docs_only { 0 } else { 1 };

    if json {
        let _ = emit_json(
            "sync",
            true,
            serde_json::json!({
                "dry_run": true,
                "would_push": total,
                "by_lane": {
                    "admin": count_admin,
                    "published": count_published,
                    "staged": count_staged,
                    "review": count_review,
                },
                "planned": planned,
            }),
        );
    } else {
        for line in &planned {
            println!("  {}", line);
        }
        println!();
        println!(
            "Sync (dry run): {} item(s) would be pushed — admin {} / published {} / staged {} / review {}. No Confluence API calls made.",
            total, count_admin, count_published, count_staged, count_review
        );
    }
    Ok(())
}

#[cfg(test)]
mod review_tree_tests {
    use super::*;
    use crate::proposal::{
        ProposalDossier, ProposalKind, ProposalLane, ProposalRecord, ProposalScores,
        ProposalTaxonomyMutation,
    };

    fn make_page(title: &str, body: &str, category: &[&str], confidence: f32) -> crate::WikiPage {
        crate::WikiPage {
            path: std::path::PathBuf::from("/tmp/test.md"),
            frontmatter: crate::Frontmatter {
                id: "test".into(),
                title: title.into(),
                status: crate::PageStatus::Review,
                source: crate::SourceRef {
                    kind: "web_page".into(),
                    id: "src".into(),
                    origin_url: Some("https://x".into()),
                    summary: None,
                    acl: None,
                },
                category: category.iter().map(|s| s.to_string()).collect(),
                keywords: vec![],
                created_at: "2026-01-01T00:00:00Z".into(),
                updated_at: "2026-01-01T00:00:00Z".into(),
                confidence: Some(confidence),
                cross_refs: vec![],
                content_hash: "h".into(),
                confluence_page_id: None,
                model_used: None,
                auto_healed_at: None,
                auto_healed_confidence: None,
                intake_request_id: None,
            },
            body: body.into(),
        }
    }

    fn write_proposal(path: &Path, proposal: &ProposalRecord) {
        let sidecar = crate::proposal::proposal_sidecar_path(path);
        std::fs::write(&sidecar, serde_json::to_string_pretty(proposal).unwrap()).unwrap();
    }

    fn proposal_with(
        kind: ProposalKind,
        scores: ProposalScores,
        merge_target: Option<String>,
        taxonomy_mutation: Option<ProposalTaxonomyMutation>,
        alternatives: Vec<String>,
    ) -> ProposalRecord {
        ProposalRecord {
            schema_version: 1,
            proposal_id: "p".into(),
            generated_at: "2026-01-01T00:00:00Z".into(),
            lane: ProposalLane::Review,
            kind,
            subject_slug: "s".into(),
            title: "T".into(),
            target_path: vec!["tree".into(), "leaf".into()],
            summary: "sum".into(),
            body_markdown: "b".into(),
            recommended_action: "review for merge".into(),
            scores,
            review_reason: Some("ambiguous topic".into()),
            merge_target,
            taxonomy_mutation,
            dossier: ProposalDossier {
                source_ids: vec!["sid".into()],
                source_locations: vec![],
                fetched_artifacts: vec![],
                compared_pages: vec![],
                alternatives_considered: alternatives,
                unresolved_questions: vec!["is this duplicate?".into()],
                overlap_candidates: vec![],
                rationale: "the agent chose review because two paths fit".into(),
                body_rewrite_kind: Some("full_synthesis".into()),
                decision_section_present: true,
            },
        }
    }

    #[test]
    fn score_bar_renders_correctly() {
        assert!(score_bar(0.0).starts_with("0%"));
        assert!(score_bar(1.0).starts_with("100%"));
        let mid = score_bar(0.5);
        assert!(mid.contains("50%"), "got {mid}");
        // 10-char bar with ~5 filled
        assert!(mid.contains("▰"));
        assert!(mid.contains("▱"));
        // Clamps out-of-range
        assert!(score_bar(-0.5).starts_with("0%"));
        assert!(score_bar(1.5).starts_with("100%"));
    }

    #[test]
    fn child_status_badge_returns_empty_without_sidecar() {
        let tmp = tempfile::tempdir().unwrap();
        let p = tmp.path().join("no-sidecar.md");
        std::fs::write(&p, "---\nid: x\ntitle: x\nstatus: review\nsource:\n  kind: web_page\n  id: s\n  origin_url: null\ncategory: []\nkeywords: []\ncreated_at: \"\"\nupdated_at: \"\"\ncross_refs: []\ncontent_hash: \"\"\n---\n\nb\n").unwrap();
        assert_eq!(child_status_badge(&p), "");
    }

    #[test]
    fn child_status_badge_surfaces_overlap_merge_and_taxonomy() {
        let tmp = tempfile::tempdir().unwrap();
        let page_path = tmp.path().join("p.md");
        std::fs::write(&page_path, "x").unwrap();
        let proposal = proposal_with(
            ProposalKind::Merge,
            ProposalScores {
                route_confidence: 0.82,
                quality_confidence: 0.7,
                hierarchy_fit_confidence: 0.7,
                overlap_risk: 0.85, // triggers overlap warning
                evidence_completeness: 0.6,
                usability: 0.7,
                freshness_confidence: 1.0,
            },
            Some("published/x.md".into()),
            Some(ProposalTaxonomyMutation {
                proposed_node_title: "Y".into(),
                proposed_node_slug: "y".into(),
                proposed_parent_path: vec!["tree".into()],
                node_description: "d".into(),
                rationale: "r".into(),
                rejected_nearby_nodes: vec![],
            }),
            vec![],
        );
        write_proposal(&page_path, &proposal);

        let badge = child_status_badge(&page_path);
        assert!(badge.contains("Merge"), "badge must show kind; got {badge}");
        assert!(
            badge.contains("82%"),
            "badge must show route conf; got {badge}"
        );
        assert!(
            badge.contains("overlap"),
            "badge must flag overlap; got {badge}"
        );
        assert!(
            badge.contains("new node"),
            "badge must flag taxonomy mutation; got {badge}"
        );
        assert!(
            badge.contains("published/x.md"),
            "badge must show merge target; got {badge}"
        );
    }

    #[test]
    fn branch_child_outline_surfaces_index_proposal_badges() {
        let tmp = tempfile::tempdir().unwrap();
        let branch = tmp.path().join("branch");
        let index = branch.join("index.md");
        std::fs::create_dir_all(&branch).unwrap();
        std::fs::write(
            &index,
            "---\nid: branch-index\ntitle: Branch Index\nstatus: review\nsource:\n  kind: web_page\n  id: src\n  origin_url: https://x\ncategory:\n  - tree\nkeywords: []\ncreated_at: \"2026-01-01T00:00:00Z\"\nupdated_at: \"2026-01-01T00:00:00Z\"\ncross_refs: []\ncontent_hash: \"h\"\n---\n\nBranch summary body.\n",
        )
        .unwrap();
        let proposal = proposal_with(
            ProposalKind::Split,
            ProposalScores {
                route_confidence: 0.81,
                quality_confidence: 0.7,
                hierarchy_fit_confidence: 0.86,
                overlap_risk: 0.2,
                evidence_completeness: 0.74,
                usability: 0.8,
                freshness_confidence: 0.9,
            },
            None,
            None,
            vec!["alternate branch path".into()],
        );
        write_proposal(&index, &proposal);

        let html = render_immediate_child_links(tmp.path(), "TEST").unwrap();
        assert!(html.contains("Branch Index"));
        assert!(
            html.contains("Split"),
            "branch children must show proposal kind"
        );
        assert!(
            html.contains("81%"),
            "branch children must show route confidence"
        );
    }

    #[test]
    fn lane_page_body_includes_curation_decision_and_recommendation() {
        let tmp = tempfile::tempdir().unwrap();
        let page_path = tmp.path().join("p.md");
        let page = make_page("Test page", "Body content.", &["tree", "leaf"], 0.92);
        write_proposal(
            &page_path,
            &proposal_with(
                ProposalKind::NewPage,
                ProposalScores {
                    route_confidence: 0.92,
                    quality_confidence: 0.75,
                    hierarchy_fit_confidence: 0.88,
                    overlap_risk: 0.1,
                    evidence_completeness: 0.7,
                    usability: 0.8,
                    freshness_confidence: 1.0,
                },
                None,
                None,
                vec!["alt path A (rejected: too shallow)".into()],
            ),
        );

        let html = render_lane_page_body(&page_path, &page, "review").unwrap();
        assert!(
            html.contains("Curation Decision"),
            "missing curation section"
        );
        assert!(
            html.contains("Recommendation"),
            "missing recommendation section"
        );
        assert!(html.contains("Hierarchy fit"), "missing hierarchy fit row");
        assert!(
            html.contains("Evidence completeness"),
            "missing evidence row (T2-B addition)"
        );
        assert!(
            html.contains("Freshness"),
            "missing freshness row (T2-B addition)"
        );
        assert!(
            html.contains("Alternatives considered"),
            "must surface alternatives"
        );
        assert!(html.contains("alt path A (rejected: too shallow)"));
        assert!(
            html.contains("Body rewrite"),
            "must surface body rewrite badge from T2-A"
        );
        assert!(
            html.contains("Proposed page"),
            "must label the rendered body"
        );
    }

    #[test]
    fn lane_page_body_renders_taxonomy_mutation_panel() {
        let tmp = tempfile::tempdir().unwrap();
        let page_path = tmp.path().join("p.md");
        let page = make_page("Test", "B", &["tree"], 0.6);
        write_proposal(
            &page_path,
            &proposal_with(
                ProposalKind::TaxonomyChange,
                ProposalScores::default(),
                None,
                Some(ProposalTaxonomyMutation {
                    proposed_node_title: "New Subtree".into(),
                    proposed_node_slug: "new-subtree".into(),
                    proposed_parent_path: vec!["tree".into()],
                    node_description: "what it's for".into(),
                    rationale: "no existing node fits".into(),
                    rejected_nearby_nodes: vec!["near-a".into(), "near-b".into()],
                }),
                vec![],
            ),
        );

        let html = render_lane_page_body(&page_path, &page, "review").unwrap();
        assert!(html.contains("Taxonomy mutation proposed"));
        assert!(html.contains("New Subtree"));
        assert!(html.contains("new-subtree"));
        assert!(html.contains("no existing node fits"));
        assert!(html.contains("near-a, near-b"));
    }
}
