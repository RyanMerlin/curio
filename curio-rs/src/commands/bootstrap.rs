use crate::{Result, config::Config, confluence::ConfluenceClient};
use anyhow::Context;

pub async fn run_bootstrap(config: &Config, dry_run: bool) -> Result<()> {
    println!("Running bootstrap command...");

    let auth_token = std::env::var("CURIO_CONFLUENCE_TOKEN")
        .context("CURIO_CONFLUENCE_TOKEN environment variable not set")?;

    let client = ConfluenceClient::new(
        config.connection.confluence_url.clone(),
        config.connection.confluence_email.clone(),
        auth_token,
    )?;

    let space_key = &config.content_model.space_key;
    let root_folder_name = &config.content_model.root_folder_name;

    // Ensure the root folder exists
    println!(
        "Checking for root page: '{}' in space '{}'",
        root_folder_name, space_key
    );
    let root_page_id = if let Some(root_page) = client
        .get_page_by_title(space_key, None, root_folder_name)
        .await?
    {
        println!(
            "Root page '{}' already exists with ID: {}",
            root_folder_name,
            root_page["id"].as_str().unwrap_or_default()
        );
        root_page["id"]
            .as_str()
            .map(|s| s.to_string())
            .context("Root page ID not found")?
    } else {
        if dry_run {
            println!("(Dry run) Would create root page: '{}'", root_folder_name);
            "dry_run_id".to_string() // Placeholder for dry run
        } else {
            println!("Creating root page: '{}'", root_folder_name);
            let page_id = client
                .create_or_update_page(
                    space_key,
                    None,
                    root_folder_name,
                    "storage",
                    "<p>This is the root page for Curio knowledge base content.</p>",
                )
                .await?;
            println!(
                "Created root page '{}' with ID: {}",
                root_folder_name, page_id
            );
            page_id
        }
    };

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
        println!(
            "Checking for sub-page: '{}' under '{}'",
            page_name, root_folder_name
        );
        if let Some(sub_page) = client
            .get_page_by_title(space_key, Some(&root_page_id), page_name)
            .await?
        {
            println!(
                "Sub-page '{}' already exists with ID: {}",
                page_name,
                sub_page["id"].as_str().unwrap_or_default()
            );
        } else {
            if dry_run {
                println!(
                    "(Dry run) Would create sub-page: '{}' under '{}'",
                    page_name, root_folder_name
                );
            } else {
                println!(
                    "Creating sub-page: '{}' under '{}'",
                    page_name, root_folder_name
                );
                let body_content = match page_name {
                    "Intake" => {
                        "<p>This page holds raw, unprocessed content that has been ingested.</p>"
                    }
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
                let page_id = client
                    .create_or_update_page(
                        space_key,
                        Some(&root_page_id),
                        page_name,
                        "storage",
                        body_content,
                    )
                    .await?;
                println!("Created sub-page '{}' with ID: {}", page_name, page_id);
            }
        }
    }

    println!("Bootstrap command finished.");
    Ok(())
}
