use crate::curio_docs::{
    ADMIN_TITLE, AUDIT_TITLE, AuditEntry, RegistryRecord, append_audit_entry, audit_bucket_path,
    build_admin_root_body, build_audit_root_body, build_registry_root_body, ensure_registry_record,
    ensure_scoped_page, ensure_scoped_structure_page, extract_page_text, infer_route_plan,
};
use crate::output::emit_json;
#[allow(unused_imports)]
use crate::{
    AgentAnalysis, ChangeProposal, Result, analyze_content_with_agent, compact_change_proposal,
    config::Config, confluence::ConfluenceClient, generate_change_proposal_with_agent,
    resolve_or_create_scoped_child_page_id,
};
use anyhow::Context;
use chrono::Utc;
use serde::Serialize;
use serde_json::json;

#[derive(Debug, Serialize)]
struct ProcessIntakeOutput {
    limit: u32,
    intake_found: usize,
    handled: usize,
    staged: usize,
    review_required: usize,
    dry_run: bool,
}

pub async fn run_process_intake(
    config: &Config,
    dry_run: bool,
    json_output: bool,
    limit: u32,
) -> Result<()> {
    if !json_output {
        println!("Running process-intake command with limit: {}", limit);
    }

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
    let label_namespace = &config.content_model.label_namespace;

    // Fetch required page IDs for stage transitions
    let _intake_page_id = ensure_scoped_structure_page(
        &client,
        space_key,
        "",
        "Intake",
        "<p>This page holds raw, unprocessed content that has been ingested.</p>",
    )
    .await?;
    let staged_page_id: String = ensure_scoped_structure_page(
        &client,
        space_key,
        "",
        "Staged",
        "<p>Content on this page is high-confidence, conflict-free, and ready for publishing or final review.</p>",
    )
    .await?;
    let review_page_id: String = ensure_scoped_structure_page(
        &client,
        space_key,
        "",
        "Review",
        "<p>Content on this page requires manual human intervention due to conflicts, missing information, or low confidence.</p>",
    )
    .await?;
    let _published_page_id = ensure_scoped_structure_page(
        &client,
        space_key,
        "",
        "Published",
        "<p>This page contains the final, published 'gold' content, optimized for agent consumption.</p>",
    )
    .await?;
    let admin_page_id = ensure_scoped_structure_page(
        &client,
        space_key,
        "",
        ADMIN_TITLE,
        &build_admin_root_body(),
    )
    .await?;
    let registry_root_id = ensure_scoped_page(
        &client,
        space_key,
        &admin_page_id,
        "_registry",
        &build_registry_root_body(),
    )
    .await?;
    let audit_root_id = ensure_scoped_page(
        &client,
        space_key,
        &admin_page_id,
        AUDIT_TITLE,
        &build_audit_root_body(),
    )
    .await?;

    // 1. Find Intake Items
    let cql_query = format!(
        "label = \"{}-status-intake\" AND space = \"{}\" ORDER BY created ASC",
        label_namespace, space_key
    );
    if !json_output {
        println!("Searching for intake items with CQL: {}", cql_query);
    }
    let intake_pages = client.execute_cql(&cql_query).await?;
    let mut scoped_intake_pages = Vec::new();
    scoped_intake_pages.extend(intake_pages);

    if scoped_intake_pages.is_empty() {
        if !json_output {
            println!(
                "Label search returned no intake items yet; falling back to descendants under the Intake page."
            );
        }

        let intake_descendants = client.get_page_descendants_v2(&_intake_page_id).await?;
        for descendant in intake_descendants {
            if descendant["type"].as_str().unwrap_or_default() != "page" {
                continue;
            }

            if descendant["parentId"].as_str().unwrap_or_default() != _intake_page_id {
                continue;
            }

            let page_id = descendant["id"].as_str().unwrap_or_default();
            match client.get_page_by_id_with_body_v1(page_id).await? {
                Some(page) => scoped_intake_pages.push(page),
                None => {
                    if !json_output {
                        println!(
                            "Skipping descendant page {} because it could not be reloaded for processing.",
                            page_id
                        );
                    }
                }
            }
        }
    }

    if !json_output {
        println!(
            "Found {} intake items in managed root subtree.",
            scoped_intake_pages.len()
        );
    }

    let intake_found = scoped_intake_pages.len();
    let mut handled = 0usize;
    let mut staged = 0usize;
    let mut review_required = 0usize;

    for page_json in scoped_intake_pages.into_iter().take(limit as usize) {
        let page_id = page_json["id"]
            .as_str()
            .context("Page ID not found")?
            .to_string();
        let page_title = page_json["title"]
            .as_str()
            .context("Page title not found")?
            .to_string();
        // Assuming body content can be directly read from page_json
        let page_body = extract_page_text(&page_json);

        if !json_output {
            println!("Processing intake item: {} (ID: {})", page_title, page_id);
        }

        // Retrieve existing metadata
        let current_metadata_json = client
            .get_content_property(&page_id, "curio_metadata")
            .await?
            .unwrap_or_else(|| json!({}));
        let curio_metadata = current_metadata_json;

        // 2. [Integration Point] Content Analysis (Placeholder)
        let analysis_result = analyze_content_with_agent(&page_body).await?;
        if !json_output {
            println!(
                "  - Analysis: Confidence {:.2}, Keywords: {:?}",
                analysis_result.confidence_score, analysis_result.keywords
            );
        }

        let route_plan = infer_route_plan(Some(&page_title), &page_body, Some(&curio_metadata));
        let mut proposal: ChangeProposal = generate_change_proposal_with_agent(
            Some(&page_title),
            &page_body,
            &[page_id.clone()],
            Some(&curio_metadata),
        )
        .await?;
        proposal.target_path = route_plan.target_path.clone();
        proposal.registry_path = route_plan.registry_path.clone();
        proposal.rationale = Some(route_plan.rationale.clone());
        proposal.validation_requirements = route_plan.validation_requirements.clone();
        proposal.sibling_context = route_plan.sibling_context.clone();
        proposal.model_used = Some("curio-route-plan-v1".to_string());

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

        let route_confident = proposal
            .validation_requirements
            .iter()
            .any(|req| req == "pre_change_snapshot")
            && proposal
                .sibling_context
                .iter()
                .any(|item| item.starts_with("confidence="));

        if !has_conflict && analysis_result.confidence_score >= 0.7 && route_confident {
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

        proposal.rationale = Some(
            conflict_details
                .as_ref()
                .and_then(|value| value["message"].as_str())
                .map(|message| format!("{}; {}", route_plan.rationale, message))
                .unwrap_or_else(|| route_plan.rationale.clone()),
        );

        // 5. Perform Stage Transition
        if dry_run {
            if !json_output {
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
            }
        } else {
            if !json_output {
                println!(
                    "Moving page '{}' (ID: {}) to '{}'",
                    page_title, page_id, target_page_name
                );
            }
            client.move_page(&page_id, new_parent_id).await?;

            let updated_page_body = build_stage_artifact_body(
                &page_title,
                &page_json,
                &page_body,
                &analysis_result,
                &proposal,
                target_page_name,
                conflict_details.as_ref(),
                &config.connection.confluence_url,
            );
            if !json_output {
                println!("Updating page body for {}", page_id);
            }
            client
                .update_page_body_by_id(&page_id, "atlas_doc_format", &updated_page_body)
                .await?;

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
                "change_proposal": compact_change_proposal(&proposal),
                "review_details": conflict_details,
            });
            if !json_output {
                println!("Updating curio_metadata for page {}", page_id);
            }
            client
                .set_content_property(&page_id, "curio_metadata", updated_metadata)
                .await?;

            // Update labels
            if !json_output {
                println!("Updating labels for page {}", page_id);
            }
            for label in labels_to_remove {
                client.remove_label(&page_id, &label).await?;
            }
            client.add_labels(&page_id, labels_to_add).await?;

            let registry_record = RegistryRecord {
                key: page_id.clone(),
                item_type: "confluence_page".to_string(),
                title: page_title.clone(),
                page_id: page_id.clone(),
                parent_id: new_parent_id.to_string(),
                status: if has_conflict {
                    "review_required".to_string()
                } else {
                    "staged".to_string()
                },
                source_id: curio_metadata["source_id"]
                    .as_str()
                    .unwrap_or_default()
                    .to_string(),
                summary: analysis_result.summary.clone(),
                updated_at: Utc::now().to_rfc3339(),
            };
            ensure_registry_record(
                &client,
                space_key,
                &registry_root_id,
                &proposal.registry_path,
                &registry_record,
            )
            .await?;

            let audit_entry = AuditEntry {
                actor: config.connection.confluence_email.clone(),
                command: "process-intake".to_string(),
                subject: page_title.clone(),
                action: format!("Moved intake page to {}", target_page_name),
                rationale: if has_conflict {
                    conflict_details
                        .as_ref()
                        .and_then(|value| value["message"].as_str())
                        .unwrap_or("Conflict detected")
                        .to_string()
                } else {
                    "Content was high-confidence and routed to Staged".to_string()
                },
                source: curio_metadata["source_id"]
                    .as_str()
                    .unwrap_or_default()
                    .to_string(),
                result: target_page_name.to_string(),
                detail_lines: vec![
                    format!("Page ID: {}", page_id),
                    format!("Status: {}", target_page_name),
                    format!("Confidence: {:.2}", analysis_result.confidence_score),
                ],
            };
            let audit_stamp = Utc::now().format("%Y%m%dT%H%M%S%.3fZ").to_string();
            append_audit_entry(
                &client,
                space_key,
                &audit_root_id,
                &audit_bucket_path(&audit_stamp),
                &audit_entry,
            )
            .await?;

            if !json_output {
                println!(
                    "Successfully transitioned page '{}' to '{}'",
                    page_title, target_page_name
                );
            }
        }

        handled += 1;
        if target_page_name == "Staged" {
            staged += 1;
        } else {
            review_required += 1;
        }
    }

    if json_output {
        emit_json(
            "process-intake",
            true,
            ProcessIntakeOutput {
                limit,
                intake_found,
                handled,
                staged,
                review_required,
                dry_run,
            },
        )?;
    } else {
        println!("Process-intake command finished.");
    }
    Ok(())
}

fn build_stage_artifact_body(
    title: &str,
    source_page: &serde_json::Value,
    source_body: &str,
    analysis_result: &AgentAnalysis,
    proposal: &ChangeProposal,
    lane_name: &str,
    conflict_details: Option<&serde_json::Value>,
    confluence_url: &str,
) -> String {
    let target_path = if proposal.target_path.is_empty() {
        "Unresolved".to_string()
    } else {
        proposal.target_path.join(" / ")
    };
    let registry_path = if proposal.registry_path.is_empty() {
        "Unresolved".to_string()
    } else {
        proposal.registry_path.join(" / ")
    };
    let validation_requirements = if proposal.validation_requirements.is_empty() {
        "None".to_string()
    } else {
        proposal.validation_requirements.join(", ")
    };
    let sibling_context = if proposal.sibling_context.is_empty() {
        "None".to_string()
    } else {
        proposal.sibling_context.join(" | ")
    };
    let conflict_text = conflict_details
        .and_then(|value| value["message"].as_str())
        .unwrap_or("No conflict detected");
    let proposal_summary = proposal.summary.clone();
    let proposal_rationale = proposal
        .rationale
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("No route rationale was produced.");
    let proposed_changes = if proposal.proposed_changes.is_empty() {
        vec!["No proposed content changes were generated.".to_string()]
    } else {
        proposal
            .proposed_changes
            .iter()
            .map(|change| {
                format!(
                    "{} -> {} ({})",
                    change.target_page_title, change.target_page_id, change.summary_of_change
                )
            })
            .collect::<Vec<_>>()
    };
    let commands = [
        "curio review approve <page-id>",
        "curio review reject <page-id>",
        "curio gold-resolve <page-id>",
        "curio gold-publish <page-id>",
    ]
    .iter()
    .map(|command| command.to_string())
    .collect::<Vec<_>>();
    let source_reference = proposal
        .source_refs
        .first()
        .cloned()
        .unwrap_or_else(|| "Unknown source".to_string());
    let source_page_title = source_page["title"].as_str().unwrap_or(title);
    let source_page_url = source_page["_links"]["webui"]
        .as_str()
        .map(|webui| format!("{}{}", confluence_url.trim_end_matches('/'), webui));
    let source_excerpt_nodes = build_source_excerpt_nodes(source_page, source_body, 8);
    let quick_facts_rows = vec![
        vec!["Source reference".to_string(), source_reference.clone()],
        vec![
            "Source length".to_string(),
            if source_body.trim().is_empty() {
                "No source body was available.".to_string()
            } else {
                format!("{} line(s)", source_body.lines().count())
            },
        ],
        vec!["Target path".to_string(), target_path.clone()],
        vec!["Registry path".to_string(), registry_path.clone()],
        vec![
            "Confidence".to_string(),
            format!("{:.2}", analysis_result.confidence_score),
        ],
        vec!["Validation".to_string(), validation_requirements.clone()],
        vec!["Sibling context".to_string(), sibling_context.clone()],
        vec!["Review note".to_string(), conflict_text.to_string()],
    ];
    let routing_rows = vec![
        vec!["Target path".to_string(), target_path.clone()],
        vec!["Registry path".to_string(), registry_path.clone()],
        vec![
            "Confidence".to_string(),
            format!("{:.2}", analysis_result.confidence_score),
        ],
        vec!["Validation".to_string(), validation_requirements.clone()],
        vec!["Sibling context".to_string(), sibling_context.clone()],
        vec!["Review note".to_string(), conflict_text.to_string()],
    ];
    let adf = json!({
        "type": "doc",
        "version": 1,
        "content": [
            adf_heading(1, title),
            adf_paragraph_text(&format!("Curio lane: {}", lane_name)),
            adf_paragraph_text(&format!("Summary: {}", analysis_result.summary)),
            adf_paragraph_text(&format!("Rationale: {}", proposal_rationale)),
            adf_heading(2, "Quick Facts"),
            adf_table(quick_facts_rows),
            adf_heading(2, "Proposal Detail"),
            adf_paragraph_text(&proposal_summary),
            adf_heading(2, "Proposed Changes"),
            adf_bullet_list(proposed_changes),
            adf_heading(2, "Commands"),
            adf_bullet_list(commands),
            adf_heading(2, "Routing Proposal"),
            adf_table(routing_rows),
            adf_heading(2, "Source"),
            adf_paragraph_mixed(vec![
                adf_text("Original page: "),
                match source_page_url {
                    Some(ref url) => adf_link_text(source_page_title, url),
                    None => adf_text(source_page_title),
                }
            ]),
            adf_paragraph_text(&format!("Captured as ADF with {} top-level node(s) preserved.", source_excerpt_nodes.len())),
            adf_heading(2, "Source Excerpt"),
            adf_nodes(source_excerpt_nodes),
        ]
    });

    serde_json::to_string(&adf).unwrap_or_else(|_| {
        json!({
            "type": "doc",
            "version": 1,
            "content": [
                adf_heading(1, title),
                adf_paragraph_text("Curio was unable to build the staged artifact body.")
            ]
        })
        .to_string()
    })
}

fn adf_heading(level: u8, text: &str) -> serde_json::Value {
    json!({
        "type": "heading",
        "attrs": { "level": level },
        "content": [adf_text(text)]
    })
}

fn adf_paragraph_text(text: &str) -> serde_json::Value {
    json!({
        "type": "paragraph",
        "content": [adf_text(text)]
    })
}

fn adf_paragraph_from_lines(lines: Vec<String>) -> serde_json::Value {
    let mut content = Vec::new();
    for (index, line) in lines.into_iter().enumerate() {
        if index > 0 {
            content.push(json!({ "type": "hardBreak" }));
        }
        content.push(adf_text(&line));
    }
    json!({
        "type": "paragraph",
        "content": content
    })
}

fn adf_paragraph_mixed(runs: Vec<serde_json::Value>) -> serde_json::Value {
    json!({
        "type": "paragraph",
        "content": runs
    })
}

fn adf_text(text: &str) -> serde_json::Value {
    json!({
        "type": "text",
        "text": text
    })
}

fn adf_link_text(text: &str, href: &str) -> serde_json::Value {
    json!({
        "type": "text",
        "text": text,
        "marks": [
            {
                "type": "link",
                "attrs": {
                    "href": href
                }
            }
        ]
    })
}

fn adf_nodes(nodes: Vec<serde_json::Value>) -> serde_json::Value {
    json!({
        "type": "expand",
        "attrs": {
            "title": "Open source excerpt"
        },
        "content": nodes
    })
}

fn adf_bullet_list(items: Vec<String>) -> serde_json::Value {
    json!({
        "type": "bulletList",
        "content": items.into_iter().map(|item| json!({
            "type": "listItem",
            "content": [
                {
                    "type": "paragraph",
                    "content": [adf_text(&item)]
                }
            ]
        })).collect::<Vec<_>>()
    })
}

fn adf_table(rows: Vec<Vec<String>>) -> serde_json::Value {
    json!({
        "type": "table",
        "content": rows.into_iter().map(|row| json!({
            "type": "tableRow",
            "content": row.into_iter().map(|cell| json!({
                "type": "tableCell",
                "content": [
                    {
                        "type": "paragraph",
                        "content": [adf_text(&cell)]
                    }
                ]
            })).collect::<Vec<_>>()
        })).collect::<Vec<_>>()
    })
}

fn build_source_excerpt_nodes(
    source_page: &serde_json::Value,
    source_body: &str,
    max_nodes: usize,
) -> Vec<serde_json::Value> {
    let mut nodes = Vec::new();
    if let Some(adf_value) = source_page["body"]["atlas_doc_format"]["value"].as_str() {
        if let Ok(adf_json) = serde_json::from_str::<serde_json::Value>(adf_value) {
            if let Some(content) = adf_json["content"].as_array() {
                nodes.extend(content.iter().take(max_nodes).cloned());
            }
        }
    }

    if nodes.is_empty() {
        let lines = if source_body.trim().is_empty() {
            vec!["No source body was available.".to_string()]
        } else {
            source_body
                .lines()
                .take(40)
                .map(|line| line.to_string())
                .collect::<Vec<_>>()
        };
        nodes.push(adf_paragraph_from_lines(lines));
    }

    if nodes.len() >= max_nodes {
        nodes.push(adf_paragraph_text(
            "Excerpt truncated. Open the source page for the full structured body.",
        ));
    }

    nodes
}
