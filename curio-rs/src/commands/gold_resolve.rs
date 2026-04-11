use crate::{
    ChangeProposal, Result, config::Config, confluence::ConfluenceClient,
    generate_change_proposal_with_agent,
};
use anyhow::Context;
use serde_json::json;

pub async fn run_gold_resolve(
    config: &Config,
    dry_run: bool,
    page_id_arg: String, // The page to resolve
) -> Result<()> {
    println!("Running gold-resolve command for page ID: {}", page_id_arg);

    let auth_token = std::env::var("CURIO_CONFLUENCE_TOKEN")
        .context("CURIO_CONFLUENCE_TOKEN environment variable not set")?;

    let client = ConfluenceClient::new(
        config.connection.confluence_url.clone(),
        config.connection.confluence_email.clone(),
        auth_token,
        config.content_model.output_root_folder_id.clone(),
    )?;

    let label_namespace = &config.content_model.label_namespace;

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
    println!(
        "Triggering agent to generate change proposal for page {}...",
        page_id_arg
    );
    let change_proposal: ChangeProposal =
        generate_change_proposal_with_agent(&source_page_content).await?;
    println!(
        "  - Generated proposal summary: {}",
        change_proposal.summary
    );
    println!(
        "  - Proposed changes for {} target pages.",
        change_proposal.proposed_changes.len()
    );

    // 3. Store the Change Proposal in Source Page Metadata
    curio_metadata_mut["status"] = json!("resolved");
    curio_metadata_mut["change_proposal"] = json!(change_proposal);

    if dry_run {
        println!(
            "(Dry run) Would update curio_metadata for page {} with status 'resolved' and change proposal: {:?}",
            page_id_arg, curio_metadata_mut["change_proposal"]
        );
        println!(
            "(Dry run) Would remove labels like `curio-status-staged` and add `curio-status-resolved`."
        );
    } else {
        println!("Updating curio_metadata for page {}", page_id_arg);
        client
            .set_content_property(&page_id_arg, "curio_metadata", curio_metadata_mut.clone())
            .await?;

        // Update Labels
        println!("Updating labels for page {}", page_id_arg);
        let old_status_label = format!("{}-status-analyzed", label_namespace);
        client.remove_label(&page_id_arg, &old_status_label).await?;
        client
            .add_labels(
                &page_id_arg,
                vec![format!("{}-status-resolved", label_namespace)],
            )
            .await?;
    }

    println!("Gold resolve command finished for page: {}", page_id_arg);
    Ok(())
}
