use crate::curio_docs::{
    AUDIT_TITLE, AuditEntry, RegistryRecord, append_audit_entry, build_audit_root_body,
    build_registry_root_body, ensure_registry_record, ensure_scoped_page,
};
use crate::output::emit_json;
use crate::{
    Change, ChangeProposal, Result, config::Config, confluence::ConfluenceClient,
    resolve_managed_root_folder_id,
};
use anyhow::Context;
use chrono::Utc;
use serde::Serialize;
use serde_json::json;

#[derive(Debug, Serialize)]
struct GoldPublishOutput {
    page_id: String,
    changes_applied: usize,
    dry_run: bool,
}

pub async fn run_gold_publish(
    config: &Config,
    dry_run: bool,
    json_output: bool,
    page_id_arg: String, // The resolved page to publish
) -> Result<()> {
    if !json_output {
        println!("Running gold-publish command for page ID: {}", page_id_arg);
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
    let _source_page_body = source_page["body"]["storage"]["value"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    let current_metadata_json = client
        .get_content_property(&page_id_arg, "curio_metadata")
        .await?;
    let mut curio_metadata_mut = if let Some(metadata) = current_metadata_json {
        metadata["value"].clone()
    } else {
        json!({})
    };

    // Ensure page is in 'resolved' status
    if curio_metadata_mut["status"].as_str() != Some("resolved") {
        anyhow::bail!(
            "Page {} is not in 'resolved' status. It must be resolved to be published.",
            page_id_arg
        );
    }

    // Extract Change Proposal
    let change_proposal: ChangeProposal =
        serde_json::from_value(curio_metadata_mut["change_proposal"].clone())
            .context("Change proposal missing or invalid in metadata")?;

    if !json_output {
        println!(
            "Executing change proposal (summary: {}) for page {}",
            change_proposal.summary, page_id_arg
        );
    }

    // 2. Loop Through and Apply Changes
    let changes_applied = change_proposal.proposed_changes.len();
    for change in change_proposal.proposed_changes {
        if !json_output {
            println!(
                "  - Applying change to target page {} ({}): {}",
                change.target_page_title, change.target_page_id, change.summary_of_change
            );
        }

        // Fetch target page
        let target_page_current = client
            .get_page_by_id_v2(&change.target_page_id)
            .await?
            .context(format!(
                "Target gold page with ID {} not found",
                change.target_page_id
            ))?;
        let target_page_current_body = target_page_current["body"]["storage"]["value"]
            .as_str()
            .unwrap_or_default()
            .to_string();

        // [Integration Point] Apply Change (Placeholder)
        let updated_target_body = apply_content_change(&target_page_current_body, &change).await?;

        if !dry_run {
            if !json_output {
                println!("    - Updating target page {}", change.target_page_id);
            }
            client
                .create_or_update_page(
                    space_key,
                    None, // Parent will be determined by the existing page
                    &change.target_page_title,
                    "storage",
                    &updated_target_body,
                )
                .await?;

            // Update target page's curio_metadata (e.g., processed_at)
            let current_target_metadata_json = client
                .get_content_property(&change.target_page_id, "curio_metadata")
                .await?
                .unwrap_or_else(|| json!({}));
            let mut target_metadata_mut = current_target_metadata_json;
            target_metadata_mut["processed_at"] = json!(Utc::now().to_rfc3339());
            client
                .set_content_property(
                    &change.target_page_id,
                    "curio_metadata",
                    target_metadata_mut.clone(),
                )
                .await?;
        } else {
            if !json_output {
                println!(
                    "    (Dry run) Would update target page {} with new content.",
                    change.target_page_id
                );
            }
        }
    }

    // 3. Update Source Page (Post-Publish)
    if !dry_run {
        if !json_output {
            println!("Updating source page {} status to 'published'", page_id_arg);
        }
        curio_metadata_mut["status"] = json!("published");
        client
            .set_content_property(&page_id_arg, "curio_metadata", curio_metadata_mut.clone())
            .await?;

        // Update Labels
        if !json_output {
            println!("Updating labels for source page {}", page_id_arg);
        }
        let old_status_label = format!("{}-status-resolved", label_namespace);
        client.remove_label(&page_id_arg, &old_status_label).await?;
        client
            .add_labels(
                &page_id_arg,
                vec![format!("{}-status-published", label_namespace)],
            )
            .await?;

        let registry_record = RegistryRecord {
            key: page_id_arg.clone(),
            item_type: "gold-source-page".to_string(),
            title: source_page["title"]
                .as_str()
                .unwrap_or(&page_id_arg)
                .to_string(),
            page_id: page_id_arg.clone(),
            parent_id: source_page["parentId"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
            status: "published".to_string(),
            source_id: page_id_arg.clone(),
            summary: change_proposal.summary.clone(),
            updated_at: Utc::now().to_rfc3339(),
        };
        ensure_registry_record(&client, space_key, &registry_root_id, &registry_record).await?;

        let audit_entry = AuditEntry {
            actor: config.connection.confluence_email.clone(),
            command: "gold-publish".to_string(),
            subject: source_page["title"]
                .as_str()
                .unwrap_or(&page_id_arg)
                .to_string(),
            action: "Applied change proposal and marked the page published".to_string(),
            rationale: change_proposal.summary.clone(),
            source: page_id_arg.clone(),
            result: "published".to_string(),
            detail_lines: vec![
                format!("Page ID: {}", page_id_arg),
                format!("Changes applied: {}", changes_applied),
            ],
        };
        append_audit_entry(&client, space_key, &audit_root_id, &audit_entry).await?;

        // (Optional) Move source page to _archive
        // Not implemented in initial version
    } else {
        if !json_output {
            println!(
                "(Dry run) Would update source page {} status to 'published' and update labels.",
                page_id_arg
            );
        }
    }

    if json_output {
        emit_json(
            "gold-publish",
            true,
            GoldPublishOutput {
                page_id: page_id_arg,
                changes_applied,
                dry_run,
            },
        )?;
    } else {
        println!("Gold publish command finished for page: {}", page_id_arg);
    }
    Ok(())
}

// --- Placeholder for Content Merge Integration ---
async fn apply_content_change(original_content: &str, change: &Change) -> Result<String> {
    // This is a placeholder. In a real scenario, this would involve a complex merge
    // or a call to an AI agent for surgical content updates.

    // For initial implementation, we'll do a simple overwrite with proposed_content_diff
    // or append if change_type suggests.
    // This needs to be robustly implemented based on actual change_type.
    // For now, if "update_section", just return the diff as content.
    // If "add_note", append to original.

    match change.change_type.as_str() {
        "update_section" => {
            // In a real scenario, this would apply the diff to a specific section.
            // For now, we'll just return the proposed content as if it's the full body update.
            Ok(change.proposed_content_diff.clone())
        }
        "add_note" => {
            // Append the note to the original content
            Ok(format!(
                "{}
{}",
                original_content, change.proposed_content_diff
            ))
        }
        _ => anyhow::bail!(
            "Unsupported change type for initial implementation: {}",
            change.change_type
        ),
    }
}
