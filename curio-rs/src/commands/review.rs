use crate::{Result, config::Config, confluence::ConfluenceClient};
use anyhow::Context;
use chrono::Utc;
use serde_json::json;

pub async fn run_review_approve(
    config: &Config,
    dry_run: bool,
    page_id_arg: String, // The page to approve
) -> Result<()> {
    println!(
        "Running review approve command for page ID: {}",
        page_id_arg
    );

    let auth_token = std::env::var("CURIO_CONFLUENCE_TOKEN")
        .context("CURIO_CONFLUENCE_TOKEN environment variable not set")?;

    let client = ConfluenceClient::new(
        config.connection.confluence_url.clone(),
        config.connection.confluence_email.clone(),
        auth_token,
    )?;

    let space_key = &config.content_model.space_key;
    let label_namespace = &config.content_model.label_namespace;

    // 1. Fetch Source Page Content and Metadata
    let source_page = client
        .get_page_by_title(space_key, None, &page_id_arg)
        .await?
        .context(format!("Source page with ID {} not found", page_id_arg))?;
    let _source_page_content = source_page["body"]["storage"]["value"]
        .as_str()
        .unwrap_or_default()
        .to_string(); // Not used directly, but fetched
    let current_metadata_json = client
        .get_content_property(&page_id_arg, "curio_metadata")
        .await?
        .unwrap_or_else(|| json!({}));
    let mut curio_metadata_mut = current_metadata_json;

    // 2. Validate Status
    let current_status = curio_metadata_mut["status"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    if !matches!(&current_status as &str, "review_required" | "resolved") {
        anyhow::bail!(
            "Page {} is in '{}' status. Only 'review_required' or 'resolved' pages can be approved.",
            page_id_arg,
            current_status
        );
    }

    // 3. Update Metadata
    curio_metadata_mut["status"] = json!("approved_for_publish");
    curio_metadata_mut["review"] = json!({
        "approved_by": "unknown_reviewer", // TODO: Get from env/config
        "approved_at": Utc::now().to_rfc3339(),
    });

    if dry_run {
        println!(
            "(Dry run) Would update curio_metadata for page {} with status 'approved_for_publish' and review details.",
            page_id_arg
        );
        println!(
            "(Dry run) Would remove old labels and add `curio-status-approved_for_publish`."
        );
    } else {
        println!("Updating curio_metadata for page {}", page_id_arg);
        client
            .set_content_property(&page_id_arg, "curio_metadata", curio_metadata_mut.clone())
            .await?;

        // Update Labels
        println!("Updating labels for page {}", page_id_arg);
        // Remove existing review labels
        client
            .remove_label(&page_id_arg, &format!("{}-needs-review", label_namespace))
            .await?;
        client
            .remove_label(
                &page_id_arg,
                &format!("{}-status-{}", label_namespace, current_status),
            )
            .await?;
        client
            .add_labels(
                &page_id_arg,
                vec![format!("{}-status-approved_for_publish", label_namespace)],
            )
            .await?;
    }

    println!("Review approve command finished for page: {}", page_id_arg);
    Ok(())
}

pub async fn run_review_reject(
    config: &Config,
    dry_run: bool,
    page_id_arg: String, // The page to reject
    reason: String,      // The reason for rejection
) -> Result<()> {
    println!("Running review reject command for page ID: {}", page_id_arg);

    let auth_token = std::env::var("CURIO_CONFLUENCE_TOKEN")
        .context("CURIO_CONFLUENCE_TOKEN environment variable not set")?;

    let client = ConfluenceClient::new(
        config.connection.confluence_url.clone(),
        config.connection.confluence_email.clone(),
        auth_token,
    )?;

    let space_key = &config.content_model.space_key;
    let label_namespace = &config.content_model.label_namespace;

    // 1. Fetch Source Page Content and Metadata
    let source_page = client
        .get_page_by_title(space_key, None, &page_id_arg)
        .await?
        .context(format!("Source page with ID {} not found", page_id_arg))?;
    let _source_page_content = source_page["body"]["storage"]["value"]
        .as_str()
        .unwrap_or_default()
        .to_string(); // Not used directly, but fetched
    let current_metadata_json = client
        .get_content_property(&page_id_arg, "curio_metadata")
        .await?
        .unwrap_or_else(|| json!({}));
    let mut curio_metadata_mut = current_metadata_json;

    // 2. Validate Status
    let current_status = curio_metadata_mut["status"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    if !matches!(&current_status as &str, "review_required" | "resolved") {
        anyhow::bail!(
            "Page {} is in '{}' status. Only 'review_required' or 'resolved' pages can be rejected.",
            page_id_arg,
            current_status
        );
    }

    // 3. Update Metadata
    curio_metadata_mut["status"] = json!("rejected");
    curio_metadata_mut["review_details"] = json!({
        "rejection_reason": reason,
        "rejected_by": "unknown_reviewer", // TODO: Get from env/config
        "rejected_at": Utc::now().to_rfc3339(),
    });

    if dry_run {
        println!(
            "(Dry run) Would update curio_metadata for page {} with status 'rejected' and rejection reason.",
            page_id_arg
        );
        println!("(Dry run) Would remove old labels and add `curio-status-rejected`.");
    } else {
        println!("Updating curio_metadata for page {}", page_id_arg);
        client
            .set_content_property(&page_id_arg, "curio_metadata", curio_metadata_mut.clone())
            .await?;

        // Update Labels
        println!("Updating labels for page {}", page_id_arg);
        // Remove existing review labels
        client
            .remove_label(&page_id_arg, &format!("{}-needs-review", label_namespace))
            .await?;
        client
            .remove_label(
                &page_id_arg,
                &format!("{}-status-{}", label_namespace, current_status),
            )
            .await?;
        client
            .add_labels(
                &page_id_arg,
                vec![format!("{}-status-rejected", label_namespace)],
            )
            .await?;
    }

    println!("Review reject command finished for page: {}", page_id_arg);
    Ok(())
}
