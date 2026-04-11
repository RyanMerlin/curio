use crate::curio_docs::{
    AUDIT_TITLE, AuditEntry, RegistryRecord, append_audit_entry, build_audit_root_body,
    build_registry_root_body, ensure_registry_record, ensure_scoped_page,
};
use crate::output::emit_json;
use crate::{Result, config::Config, confluence::ConfluenceClient, resolve_managed_root_folder_id};
use anyhow::Context;
use chrono::Utc;
use serde::Serialize;
use serde_json::json;

#[derive(Debug, Serialize)]
struct ReviewOutput {
    page_id: String,
    status: String,
    dry_run: bool,
}

pub async fn run_review_approve(
    config: &Config,
    dry_run: bool,
    json_output: bool,
    page_id_arg: String, // The page to approve
) -> Result<()> {
    if !json_output {
        println!(
            "Running review approve command for page ID: {}",
            page_id_arg
        );
    }

    let auth_token = std::env::var("CURIO_CONFLUENCE_TOKEN")
        .context("CURIO_CONFLUENCE_TOKEN environment variable not set")?;

    let client = ConfluenceClient::new(
        config.connection.confluence_url.clone(),
        config.connection.confluence_email.clone(),
        auth_token,
        config.content_model.output_root_folder_id.clone(),
    )?;

    let space_key = &config.content_model.space_key;
    let label_namespace = &config.content_model.label_namespace;
    let root_folder_id = resolve_managed_root_folder_id(
        &client,
        space_key,
        &config.content_model.root_folder_name,
        config.content_model.output_root_folder_id.as_deref(),
        json_output,
    )
    .await?;
    let registry_root_id = ensure_scoped_page(
        &client,
        space_key,
        &root_folder_id,
        "_registry",
        &build_registry_root_body(),
    )
    .await?;
    let audit_root_id = ensure_scoped_page(
        &client,
        space_key,
        &root_folder_id,
        AUDIT_TITLE,
        &build_audit_root_body(),
    )
    .await?;

    // 1. Fetch Source Page Content and Metadata
    let source_page = client
        .get_page_by_id_v2(&page_id_arg)
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
        if !json_output {
            println!(
                "(Dry run) Would update curio_metadata for page {} with status 'approved_for_publish' and review details.",
                page_id_arg
            );
            println!(
                "(Dry run) Would remove old labels and add `curio-status-approved_for_publish`."
            );
        }
    } else {
        if !json_output {
            println!("Updating curio_metadata for page {}", page_id_arg);
        }
        client
            .set_content_property(&page_id_arg, "curio_metadata", curio_metadata_mut.clone())
            .await?;

        // Update Labels
        if !json_output {
            println!("Updating labels for page {}", page_id_arg);
        }
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

        let registry_record = RegistryRecord {
            key: page_id_arg.clone(),
            item_type: "review-page".to_string(),
            title: source_page["title"]
                .as_str()
                .unwrap_or(&page_id_arg)
                .to_string(),
            page_id: page_id_arg.clone(),
            parent_id: source_page["parentId"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
            status: "approved_for_publish".to_string(),
            source_id: page_id_arg.clone(),
            summary: "Approved for publishing".to_string(),
            updated_at: Utc::now().to_rfc3339(),
        };
        ensure_registry_record(&client, space_key, &registry_root_id, &registry_record).await?;

        let audit_entry = AuditEntry {
            actor: config.connection.confluence_email.clone(),
            command: "review-approve".to_string(),
            subject: source_page["title"]
                .as_str()
                .unwrap_or(&page_id_arg)
                .to_string(),
            action: "Approved the page for publishing".to_string(),
            rationale: "Human review accepted the staged content".to_string(),
            source: page_id_arg.clone(),
            result: "approved_for_publish".to_string(),
            detail_lines: vec![format!("Page ID: {}", page_id_arg)],
        };
        append_audit_entry(&client, space_key, &audit_root_id, &audit_entry).await?;
    }

    if json_output {
        emit_json(
            "review-approve",
            true,
            ReviewOutput {
                page_id: page_id_arg,
                status: "approved_for_publish".to_string(),
                dry_run,
            },
        )?;
    } else {
        println!("Review approve command finished for page: {}", page_id_arg);
    }
    Ok(())
}

pub async fn run_review_reject(
    config: &Config,
    dry_run: bool,
    json_output: bool,
    page_id_arg: String, // The page to reject
    reason: String,      // The reason for rejection
) -> Result<()> {
    if !json_output {
        println!("Running review reject command for page ID: {}", page_id_arg);
    }

    let auth_token = std::env::var("CURIO_CONFLUENCE_TOKEN")
        .context("CURIO_CONFLUENCE_TOKEN environment variable not set")?;

    let client = ConfluenceClient::new(
        config.connection.confluence_url.clone(),
        config.connection.confluence_email.clone(),
        auth_token,
        config.content_model.output_root_folder_id.clone(),
    )?;

    let space_key = &config.content_model.space_key;
    let label_namespace = &config.content_model.label_namespace;
    let root_folder_id = resolve_managed_root_folder_id(
        &client,
        space_key,
        &config.content_model.root_folder_name,
        config.content_model.output_root_folder_id.as_deref(),
        json_output,
    )
    .await?;
    let registry_root_id = ensure_scoped_page(
        &client,
        space_key,
        &root_folder_id,
        "_registry",
        &build_registry_root_body(),
    )
    .await?;
    let audit_root_id = ensure_scoped_page(
        &client,
        space_key,
        &root_folder_id,
        AUDIT_TITLE,
        &build_audit_root_body(),
    )
    .await?;

    // 1. Fetch Source Page Content and Metadata
    let source_page = client
        .get_page_by_id_v2(&page_id_arg)
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
        if !json_output {
            println!(
                "(Dry run) Would update curio_metadata for page {} with status 'rejected' and rejection reason.",
                page_id_arg
            );
            println!("(Dry run) Would remove old labels and add `curio-status-rejected`.");
        }
    } else {
        if !json_output {
            println!("Updating curio_metadata for page {}", page_id_arg);
        }
        client
            .set_content_property(&page_id_arg, "curio_metadata", curio_metadata_mut.clone())
            .await?;

        // Update Labels
        if !json_output {
            println!("Updating labels for page {}", page_id_arg);
        }
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

        let registry_record = RegistryRecord {
            key: page_id_arg.clone(),
            item_type: "review-page".to_string(),
            title: source_page["title"]
                .as_str()
                .unwrap_or(&page_id_arg)
                .to_string(),
            page_id: page_id_arg.clone(),
            parent_id: source_page["parentId"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
            status: "rejected".to_string(),
            source_id: page_id_arg.clone(),
            summary: format!("Rejected: {}", reason),
            updated_at: Utc::now().to_rfc3339(),
        };
        ensure_registry_record(&client, space_key, &registry_root_id, &registry_record).await?;

        let audit_entry = AuditEntry {
            actor: config.connection.confluence_email.clone(),
            command: "review-reject".to_string(),
            subject: source_page["title"]
                .as_str()
                .unwrap_or(&page_id_arg)
                .to_string(),
            action: "Rejected the page".to_string(),
            rationale: reason.clone(),
            source: page_id_arg.clone(),
            result: "rejected".to_string(),
            detail_lines: vec![format!("Page ID: {}", page_id_arg)],
        };
        append_audit_entry(&client, space_key, &audit_root_id, &audit_entry).await?;
    }

    if json_output {
        emit_json(
            "review-reject",
            true,
            ReviewOutput {
                page_id: page_id_arg,
                status: "rejected".to_string(),
                dry_run,
            },
        )?;
    } else {
        println!("Review reject command finished for page: {}", page_id_arg);
    }
    Ok(())
}
