#[allow(unused_imports)]
use crate::{
    AgentAnalysis, Result, analyze_content_with_agent, config::Config,
    confluence::ConfluenceClient, get_page_id_by_title,
};
use anyhow::Context;
use chrono::Utc;
use serde_json::json;

pub async fn run_process_intake(config: &Config, dry_run: bool, limit: u32) -> Result<()> {
    println!("Running process-intake command with limit: {}", limit);

    let auth_token = std::env::var("CURIO_CONFLUENCE_TOKEN")
        .context("CURIO_CONFLUENCE_TOKEN environment variable not set")?;

    let client = ConfluenceClient::new(
        config.connection.confluence_url.clone(),
        config.connection.confluence_email.clone(),
        auth_token,
    )?;

    let space_key = &config.content_model.space_key;
    let root_folder_name = &config.content_model.root_folder_name;
    let label_namespace = &config.content_model.label_namespace;

    // Fetch required page IDs for stage transitions
    let root_page_id =
        get_page_id_by_title(&client, space_key, None, root_folder_name, "root page").await?;
    let _intake_page_id = get_page_id_by_title(
        &client,
        space_key,
        Some(&root_page_id),
        "Intake",
        "Intake page",
    )
    .await?;
    let staged_page_id: String = get_page_id_by_title(
        &client,
        space_key,
        Some(&root_page_id),
        "Staged",
        "Staged page",
    )
    .await?;
    let review_page_id: String = get_page_id_by_title(
        &client,
        space_key,
        Some(&root_page_id),
        "Review",
        "Review page",
    )
    .await?;
    let _published_page_id = get_page_id_by_title(
        &client,
        space_key,
        Some(&root_page_id),
        "Published",
        "Published page",
    )
    .await?;

    // 1. Find Intake Items
    let cql_query = format!(
        "label = \"{}-status-intake\" AND space = \"{}\" ORDER BY created ASC",
        label_namespace, space_key
    );
    println!("Searching for intake items with CQL: {}", cql_query);
    let intake_pages = client.execute_cql(&cql_query).await?;
    println!("Found {} intake items.", intake_pages.len());

    for page_json in intake_pages.into_iter().take(limit as usize) {
        let page_id = page_json["id"]
            .as_str()
            .context("Page ID not found")?
            .to_string();
        let page_title = page_json["title"]
            .as_str()
            .context("Page title not found")?
            .to_string();
        // Assuming body content can be directly read from page_json
        let page_body = page_json["body"]["storage"]["value"]
            .as_str()
            .unwrap_or("")
            .to_string();

        println!("Processing intake item: {} (ID: {})", page_title, page_id);

        // Retrieve existing metadata
        let current_metadata_json = client
            .get_content_property(&page_id, "curio_metadata")
            .await?
            .unwrap_or_else(|| json!({}));
        let curio_metadata = current_metadata_json;

        // 2. [Integration Point] Content Analysis (Placeholder)
        let analysis_result = analyze_content_with_agent(&page_body).await?;
        println!(
            "  - Analysis: Confidence {:.2}, Keywords: {:?}",
            analysis_result.confidence_score, analysis_result.keywords
        );

        // 3. Semantic Conflict Detection (Conceptual)
        let mut has_conflict = false;
        let mut conflict_details: Option<serde_json::Value> = None;

        // Simulate semantic conflict detection
        // In a real scenario, 'analysis_result.summary_embedding' or similar would be used
        // to query for semantically similar content in Staged/Published.
        if analysis_result.confidence_score < 0.7 {
            // Example: low confidence triggers review
            has_conflict = true;
            conflict_details = Some(json!({
                "reason_code": "low_confidence",
                "message": format!("Agent confidence score is low: {}", analysis_result.confidence_score),
            }));
        } else {
            // Simulate checking for semantic duplicates in Staged/Published
            // For now, a simple check if subject_key contains "duplicate"
            if curio_metadata["subject_key"]
                .as_str()
                .unwrap_or("")
                .contains("duplicate")
            {
                has_conflict = true;
                conflict_details = Some(json!({
                   "reason_code": "semantic_collision_simulated",
                   "message": "Simulated semantic collision detected based on subject key.",
                }));
            }
        }

        // 4. Routing Decision
        let new_status_label;
        let new_parent_id;
        let target_page_name;
        let mut labels_to_add = Vec::new();
        let labels_to_remove = vec![format!("{}-status-intake", label_namespace)];

        if !has_conflict && analysis_result.confidence_score >= 0.7 {
            new_status_label = format!("{}-status-staged", label_namespace);
            new_parent_id = &staged_page_id;
            target_page_name = "Staged";
        } else {
            new_status_label = format!("{}-status-review_required", label_namespace);
            new_parent_id = &review_page_id;
            target_page_name = "Review";
            labels_to_add.push(format!("{}-needs-review", label_namespace));
        }
        labels_to_add.push(new_status_label.clone());

        // 5. Perform Stage Transition
        if dry_run {
            println!(
                "(Dry run) Would move page '{}' (ID: {}) to '{}'",
                page_title, page_id, target_page_name
            );
            println!(
                "(Dry run) Would update curio_metadata for page {}: status={}, review_details={:?}",
                page_id, new_status_label, conflict_details
            );
            println!(
                "(Dry run) Would remove labels {:?} and add labels {:?}",
                labels_to_remove, labels_to_add
            );
        } else {
            println!(
                "Moving page '{}' (ID: {}) to '{}'",
                page_title, page_id, target_page_name
            );
            client.move_page(&page_id, new_parent_id).await?;

            // Update curio_metadata
            let updated_metadata = json!({
                "curio_version": "1.0", // Maintain version
                "source_id": curio_metadata["source_id"].as_str().unwrap_or_default(),
                "source_hash": curio_metadata["source_hash"].as_str().unwrap_or_default(),
                "dedupe_key": curio_metadata["dedupe_key"].as_str().unwrap_or_default(),
                "subject_key": curio_metadata["subject_key"].as_str().unwrap_or_default(),
                "status": if has_conflict { "review_required" } else { "staged" },
                "ingested_at": curio_metadata["ingested_at"].as_str().unwrap_or_default(),
                "processed_at": Utc::now().to_rfc3339(),
                "source_metadata": curio_metadata["source_metadata"].clone(),
                "agent_analysis": json!({
                    "summary": analysis_result.summary,
                    "keywords": analysis_result.keywords,
                    "confidence_score": analysis_result.confidence_score,
                }),
                "review_details": conflict_details,
            });
            println!("Updating curio_metadata for page {}", page_id);
            client
                .set_content_property(&page_id, "curio_metadata", updated_metadata)
                .await?;

            // Update labels
            println!("Updating labels for page {}", page_id);
            for label in labels_to_remove {
                client.remove_label(&page_id, &label).await?;
            }
            client.add_labels(&page_id, labels_to_add).await?;

            println!(
                "Successfully transitioned page '{}' to '{}'",
                page_title, target_page_name
            );
        }
    }

    println!("Process-intake command finished.");
    Ok(())
}
