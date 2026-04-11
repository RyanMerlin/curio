use crate::curio_docs::{
    AUDIT_TITLE, AuditEntry, RegistryRecord, append_audit_entry, build_audit_root_body,
    build_registry_root_body, ensure_registry_record, ensure_scoped_page,
};
use crate::output::emit_json;
#[allow(unused_imports)]
use crate::{
    AgentAnalysis, Result, analyze_content_with_agent, config::Config,
    confluence::ConfluenceClient, resolve_managed_root_folder_id,
};
use anyhow::Context;
use serde::Serialize;
use serde_json::json;

#[derive(Debug, Serialize)]
struct AgentAnalyzeOutput {
    requested: usize,
    analyzed: usize,
    dry_run: bool,
}

pub async fn run_agent_analyze(
    config: &Config,
    dry_run: bool,
    json_output: bool,
    page_id_arg: &Option<String>,
    status_arg: &Option<String>,
    limit: u32,
) -> Result<()> {
    if !json_output {
        println!("Running agent-analyze command...");
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

    let mut pages_to_analyze: Vec<serde_json::Value> = Vec::new();

    if let Some(pid) = page_id_arg {
        if !json_output {
            println!("Attempting to fetch page with ID: {}", pid);
        }
        match client.get_page_by_id_v2(pid).await {
            Ok(Some(page)) => {
                pages_to_analyze.push(page);
            }
            Ok(None) => {
                if !json_output {
                    println!("Page with ID {} not found.", pid);
                }
                return Ok(());
            }
            Err(e) => {
                if !json_output {
                    eprintln!("Error fetching page with ID {}: {:?}", pid, e);
                }
                return Err(e);
            }
        }
    } else {
        // Otherwise, query for pages with a specific status or intake by default
        let target_status = status_arg.as_deref().unwrap_or("intake"); // Default to intake
        let cql_query = format!(
            "label = \"{}-status-{}\" AND space = \"{}\" ORDER BY created ASC",
            label_namespace, target_status, space_key
        );

        if !json_output {
            println!("Searching for pages to analyze with CQL: {}", cql_query);
        }
        let pages = client
            .execute_cql_with_limit(&cql_query, Some(limit))
            .await?;
        if let Some(write_root_folder_id) = config.content_model.output_root_folder_id.as_deref() {
            for page in pages {
                let page_id = page["id"].as_str().unwrap_or_default();
                if client
                    .page_is_descendant_of(page_id, write_root_folder_id)
                    .await?
                {
                    pages_to_analyze.push(page);
                }
            }
        } else {
            pages_to_analyze = pages;
        }
        if !json_output {
            println!("Found {} pages to analyze.", pages_to_analyze.len());
        }
    }
    if pages_to_analyze.is_empty() {
        if json_output {
            emit_json(
                "agent-analyze",
                true,
                AgentAnalyzeOutput {
                    requested: limit as usize,
                    analyzed: 0,
                    dry_run,
                },
            )?;
        } else {
            println!("No pages found matching criteria for analysis.");
        }
        return Ok(());
    }

    let analyzed = pages_to_analyze.len();
    for page_json in pages_to_analyze {
        let page_id = page_json["id"]
            .as_str()
            .context("Page ID not found")?
            .to_string();
        let page_title = page_json["title"]
            .as_str()
            .context("Page title not found")?
            .to_string();
        let page_body = page_json["body"]["storage"]["value"]
            .as_str()
            .unwrap_or("")
            .to_string();

        if !json_output {
            println!("Analyzing page: {} (ID: {})", page_title, page_id);
        }

        // Update status to 'analyzing' first
        if !dry_run {
            if !json_output {
                println!("  - Setting status to 'analyzing' for page {}", page_id);
            }
            let current_metadata_json = client
                .get_content_property(&page_id, "curio_metadata")
                .await?;
            let mut curio_metadata_mut = if let Some(metadata) = current_metadata_json {
                metadata["value"].clone()
            } else {
                json!({})
            };
            curio_metadata_mut["status"] = json!("analyzing");
            client
                .set_content_property(&page_id, "curio_metadata", curio_metadata_mut.clone())
                .await?;

            // Remove old status label, add 'analyzing' label
            let old_status_label = format!(
                "{}-status-{}",
                label_namespace,
                curio_metadata_mut["status"].as_str().unwrap_or_default()
            );
            client.remove_label(&page_id, &old_status_label).await?;
            client
                .add_labels(
                    &page_id,
                    vec![format!("{}-status-analyzing", label_namespace)],
                )
                .await?;
        } else {
            if !json_output {
                println!(
                    "(Dry run) Would set status to 'analyzing' for page {}",
                    page_id
                );
            }
        }

        // Perform analysis (placeholder)
        let analysis_result = analyze_content_with_agent(&page_body).await?;
        if !json_output {
            println!(
                "  - Analysis Complete: Confidence {:.2}, Keywords: {:?}",
                analysis_result.confidence_score, analysis_result.keywords
            );
        }

        // Update with analysis results and final status
        if !dry_run {
            if !json_output {
                println!(
                    "  - Updating page {} with analysis results and status 'analyzed'",
                    page_id
                );
            }
            let current_metadata_json = client
                .get_content_property(&page_id, "curio_metadata")
                .await?;
            let mut curio_metadata_mut = if let Some(metadata) = current_metadata_json {
                metadata["value"].clone()
            } else {
                json!({})
            };
            curio_metadata_mut["status"] = json!("analyzed");
            curio_metadata_mut["agent_analysis"] = json!({
                "summary": analysis_result.summary,
                "keywords": analysis_result.keywords,
                "confidence_score": analysis_result.confidence_score,
            });
            client
                .set_content_property(&page_id, "curio_metadata", curio_metadata_mut.clone())
                .await?;

            // Remove 'analyzing' status label, add 'analyzed' label
            client
                .remove_label(&page_id, &format!("{}-status-analyzing", label_namespace))
                .await?;
            client
                .add_labels(
                    &page_id,
                    vec![format!("{}-status-analyzed", label_namespace)],
                )
                .await?;

            let registry_record = RegistryRecord {
                key: page_id.clone(),
                item_type: "confluence_page".to_string(),
                title: page_title.clone(),
                page_id: page_id.clone(),
                parent_id: page_json["parentId"]
                    .as_str()
                    .unwrap_or_default()
                    .to_string(),
                status: "analyzed".to_string(),
                source_id: page_json["id"].as_str().unwrap_or_default().to_string(),
                summary: analysis_result.summary.clone(),
                updated_at: chrono::Utc::now().to_rfc3339(),
            };
            ensure_registry_record(&client, space_key, &registry_root_id, &registry_record).await?;

            let audit_entry = AuditEntry {
                actor: config.connection.confluence_email.clone(),
                command: "agent-analyze".to_string(),
                subject: page_title.clone(),
                action: "Analyzed content and marked the page analyzed".to_string(),
                rationale: "Content analysis completed successfully".to_string(),
                source: page_id.clone(),
                result: "analyzed".to_string(),
                detail_lines: vec![
                    format!("Page ID: {}", page_id),
                    format!("Confidence: {:.2}", analysis_result.confidence_score),
                ],
            };
            append_audit_entry(&client, space_key, &audit_root_id, &audit_entry).await?;
        } else {
            if !json_output {
                println!(
                    "(Dry run) Would update page {} with analysis results and status 'analyzed'",
                    page_id
                );
            }
        }
    }

    if json_output {
        emit_json(
            "agent-analyze",
            true,
            AgentAnalyzeOutput {
                requested: limit as usize,
                analyzed,
                dry_run,
            },
        )?;
    } else {
        println!("Agent analyze command finished.");
    }
    Ok(())
}
