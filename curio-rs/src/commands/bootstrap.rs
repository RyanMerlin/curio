use crate::{
    Result, config::Config, confluence::ConfluenceClient, resolve_managed_root_folder_id,
    resolve_or_create_scoped_child_page_id,
};
use anyhow::Context;

pub async fn run_bootstrap(config: &Config, dry_run: bool) -> Result<()> {
    println!("Running bootstrap command...");

    let auth_token = std::env::var("CURIO_CONFLUENCE_TOKEN")
        .context("CURIO_CONFLUENCE_TOKEN environment variable not set")?;

    let client = ConfluenceClient::new(
        config.connection.confluence_url.clone(),
        config.connection.confluence_email.clone(),
        auth_token,
        config.content_model.output_root_folder_id.clone(),
    )?;

    let space_key = &config.content_model.space_key;
    let root_folder_name = &config.content_model.root_folder_name;
    let root_folder_id = resolve_managed_root_folder_id(
        &client,
        space_key,
        root_folder_name,
        config.content_model.output_root_folder_id.as_deref(),
    )
    .await?;

    println!(
        "Using managed write root folder ID {} for space '{}'",
        root_folder_id, space_key
    );

    // Define the required sub-pages for the process-oriented lifecycle
    let lifecycle_pages = vec![
        "Intake",
        "Staged",
        "Review",
        "Published",
        "_templates",
        "_registry",
    ];

    for page_name in lifecycle_pages {
        println!("Checking for sub-page: '{}' under managed root", page_name);
        let body_content = match page_name {
            "Intake" => "<p>This page holds raw, unprocessed content that has been ingested.</p>",
            "Staged" => {
                "<p>Content on this page is high-confidence, conflict-free, and ready for publishing or final review.</p>"
            }
            "Review" => {
                "<p>Content on this page requires manual human intervention due to conflicts, missing information, or low confidence.</p>"
            }
            "Published" => {
                "<p>This page contains the final, published 'gold' content, optimized for agent consumption.</p>"
            }
            "_templates" => {
                "<p>This page is for storing templates for various content types managed by Curio.</p>"
            }
            "_registry" => {
                "<p>This page acts as a central registry for canonical 'Published' topics and their associated pages.</p>"
            }
            _ => "<p>Curio lifecycle stage page.</p>",
        };

        if dry_run {
            println!(
                "(Dry run) Would ensure sub-page: '{}' under managed root",
                page_name
            );
        } else {
            let page_id = resolve_or_create_scoped_child_page_id(
                &client,
                space_key,
                &root_folder_id,
                page_name,
                body_content,
            )
            .await?;
            println!("Ensured sub-page '{}' with ID: {}", page_name, page_id);
        }
    }

    println!("Bootstrap command finished.");
    Ok(())
}
