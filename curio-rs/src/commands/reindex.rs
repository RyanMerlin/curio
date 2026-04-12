use crate::curio_docs::{
    ADMIN_TITLE, build_admin_root_body, ensure_scoped_page, ensure_scoped_structure_page,
    update_branch_index,
};
use crate::output::emit_json;
use crate::{Result, config::Config, confluence::ConfluenceClient};
use anyhow::Context;
use serde::Serialize;

#[derive(Debug, Serialize)]
struct ReindexOutput {
    branches_reindexed: usize,
    dry_run: bool,
}

pub async fn run_reindex(
    config: &Config,
    dry_run: bool,
    json_output: bool,
) -> Result<()> {
    let auth_token = std::env::var("CURIO_CONFLUENCE_TOKEN")
        .context("CURIO_CONFLUENCE_TOKEN environment variable not set")?;

    let client = ConfluenceClient::new(
        config.connection.confluence_url.clone(),
        config.connection.confluence_email.clone(),
        auth_token,
        config.content_model.space_key.clone(),
        None,
    )?;

    let space_key = &config.content_model.space_key;

    // Locate (but do not create) the top-level structural pages.
    // Use the same ensure_* helpers that bootstrap uses — they are idempotent.
    let admin_page_id = ensure_scoped_structure_page(
        &client, space_key, "", ADMIN_TITLE, &build_admin_root_body(),
    ).await?;

    let intake_id = ensure_scoped_structure_page(
        &client, space_key, "", "Intake",
        "<p>Curio intake lane.</p>",
    ).await?;
    let staged_id = ensure_scoped_structure_page(
        &client, space_key, "", "Staged",
        "<p>Curio staged lane.</p>",
    ).await?;
    let review_id = ensure_scoped_structure_page(
        &client, space_key, "", "Review",
        "<p>Curio review lane.</p>",
    ).await?;
    let published_id = ensure_scoped_structure_page(
        &client, space_key, "", "Published",
        "<p>This page contains the final, published Curio content tree.</p>",
    ).await?;
    let registry_id = ensure_scoped_page(
        &client, space_key, &admin_page_id, "_registry",
        "<p>Curio registry.</p>",
    ).await?;

    let mut count = 0usize;

    if dry_run {
        if !json_output {
            println!("(Dry run) Would reindex: Intake, Staged, Review, Published, _registry");
        }
    } else {
        // Reindex flat lanes
        for (page_id, title, root_label) in [
            (&intake_id, "Intake", "Intake"),
            (&staged_id, "Staged", "Staged"),
            (&review_id, "Review", "Review"),
        ] {
            if !json_output { println!("Reindexing {}...", title); }
            update_branch_index(&client, page_id, title, &[title.to_string()], root_label, None).await?;
            count += 1;
        }

        // Reindex Published and all its sub-branches recursively
        if !json_output { println!("Reindexing Published tree..."); }
        reindex_tree(&client, &published_id, "Published", &["Published".to_string()], "Published", None, &mut count, json_output).await?;

        // Reindex _registry and its sub-branches
        if !json_output { println!("Reindexing _registry tree..."); }
        reindex_tree(&client, &registry_id, "_registry", &["Admin".to_string(), "_registry".to_string()], "Admin", Some(&admin_page_id), &mut count, json_output).await?;
    }

    if json_output {
        emit_json("reindex", true, ReindexOutput { branches_reindexed: count, dry_run })?;
    } else {
        println!("Reindex complete. {} branch pages updated.", count);
    }
    Ok(())
}

/// Recursively reindex a branch page and all its children that are themselves branches.
async fn reindex_tree(
    client: &ConfluenceClient,
    page_id: &str,
    title: &str,
    path: &[String],
    root_label: &str,
    parent_page_id: Option<&str>,
    count: &mut usize,
    json_output: bool,
) -> Result<()> {
    // Update this branch's index
    update_branch_index(client, page_id, title, path, root_label, parent_page_id).await?;
    *count += 1;

    // Recurse into children that have a curio_branch_index property (i.e., are branch pages)
    let children = client.get_direct_children_v2(page_id).await?;
    for child in &children {
        let child_id = child["id"].as_str().unwrap_or_default();
        let child_title = child["title"].as_str().unwrap_or_default();
        if child_id.is_empty() { continue; }

        // Only recurse into pages that already have a branch index (are branches, not leaves)
        if let Ok(Some(_)) = client.get_content_property(child_id, "curio_branch_index").await {
            let mut child_path = path.to_vec();
            child_path.push(child_title.to_string());
            if !json_output {
                println!("  Reindexing {}...", child_path.join(" / "));
            }
            // Box the future to allow recursion
            reindex_tree_boxed(client, child_id, child_title, &child_path, root_label, Some(page_id), count, json_output).await?;
        }
    }
    Ok(())
}

// Boxed wrapper to allow async recursion
fn reindex_tree_boxed<'a>(
    client: &'a ConfluenceClient,
    page_id: &'a str,
    title: &'a str,
    path: &'a [String],
    root_label: &'a str,
    parent_page_id: Option<&'a str>,
    count: &'a mut usize,
    json_output: bool,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
    Box::pin(reindex_tree(client, page_id, title, path, root_label, parent_page_id, count, json_output))
}
