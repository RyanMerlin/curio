use crate::curio_docs::{
    AUDIT_TITLE, AuditEntry, RegistryRecord, append_audit_entry, build_audit_root_body,
    build_registry_root_body, ensure_registry_record, ensure_scoped_page,
};
use crate::output::emit_json;
use crate::{
    ChangeProposal, Result, config::Config, confluence::ConfluenceClient,
    generate_change_proposal_with_agent, resolve_managed_root_folder_id,
};
use anyhow::Context;
use serde::Serialize;
use serde_json::json;

#[derive(Debug, Serialize)]
struct GoldResolveOutput {
    page_id: String,
    change_count: usize,
    dry_run: bool,
}

pub async fn run_gold_resolve(
    config: &Config,
    dry_run: bool,
    json_output: bool,
    page_id_arg: String, // The page to resolve
) -> Result<()> {
    if !json_output {
        println!("Running gold-resolve command for page ID: {}", page_id_arg);
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
    let source_page_content = source_page["body"]["storage"]["value"]
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

    // Ensure page is in 'analyzed' status
    if curio_metadata_mut["status"].as_str() != Some("analyzed") {
        anyhow::bail!(
            "Page {} is not in 'analyzed' status. It must be analyzed to be resolved.",
            page_id_arg
        );
    }

    // 2. [Integration Point] Trigger Analysis Agent to Generate Change Proposal
    if !json_output {
        println!(
            "Triggering agent to generate change proposal for page {}...",
            page_id_arg
        );
    }
    let change_proposal: ChangeProposal =
        generate_change_proposal_with_agent(&source_page_content).await?;
    if !json_output {
        println!(
            "  - Generated proposal summary: {}",
            change_proposal.summary
        );
        println!(
            "  - Proposed changes for {} target pages.",
            change_proposal.proposed_changes.len()
        );
    }

    // 3. Store the Change Proposal in Source Page Metadata
    curio_metadata_mut["status"] = json!("resolved");
    curio_metadata_mut["change_proposal"] = json!(change_proposal);

    if dry_run {
        if !json_output {
            println!(
                "(Dry run) Would update curio_metadata for page {} with status 'resolved' and change proposal: {:?}",
                page_id_arg, curio_metadata_mut["change_proposal"]
            );
            println!(
                "(Dry run) Would remove labels like `curio-status-staged` and add `curio-status-resolved`."
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
        let old_status_label = format!("{}-status-analyzed", label_namespace);
        client.remove_label(&page_id_arg, &old_status_label).await?;
        client
            .add_labels(
                &page_id_arg,
                vec![format!("{}-status-resolved", label_namespace)],
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
            status: "resolved".to_string(),
            source_id: page_id_arg.clone(),
            summary: change_proposal.summary.clone(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        };
        ensure_registry_record(&client, space_key, &registry_root_id, &registry_record).await?;

        let audit_entry = AuditEntry {
            actor: config.connection.confluence_email.clone(),
            command: "gold-resolve".to_string(),
            subject: source_page["title"]
                .as_str()
                .unwrap_or(&page_id_arg)
                .to_string(),
            action: "Generated a change proposal and marked the page resolved".to_string(),
            rationale: change_proposal.summary.clone(),
            source: page_id_arg.clone(),
            result: "resolved".to_string(),
            detail_lines: vec![
                format!("Page ID: {}", page_id_arg),
                format!(
                    "Proposed changes: {}",
                    change_proposal.proposed_changes.len()
                ),
            ],
        };
        append_audit_entry(&client, space_key, &audit_root_id, &audit_entry).await?;
    }

    if json_output {
        emit_json(
            "gold-resolve",
            true,
            GoldResolveOutput {
                page_id: page_id_arg,
                change_count: change_proposal.proposed_changes.len(),
                dry_run,
            },
        )?;
    } else {
        println!("Gold resolve command finished for page: {}", page_id_arg);
    }
    Ok(())
}
