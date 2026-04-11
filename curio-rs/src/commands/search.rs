use crate::{Result, config::Config, confluence::ConfluenceClient, output::emit_json};
use anyhow::Context;
use serde::Serialize;

#[derive(Debug, Serialize)]
struct SearchData {
    query: String,
    count: usize,
    results: Vec<serde_json::Value>,
}

pub async fn run_search(
    config: &Config,
    _dry_run: bool, // Not directly used in search, but part of global CLI context
    json_output: bool,
    labels: Vec<String>,
    text: Option<String>,
    content_type: Option<String>,
    limit: u32,
) -> Result<()> {
    if !json_output {
        println!("Running search command...");
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
    let _label_namespace = &config.content_model.label_namespace;

    let mut cql_parts = Vec::new();
    cql_parts.push(format!("space = \"{}\"", space_key));

    // Add labels to CQL
    for label in labels {
        cql_parts.push(format!("label = \"{}\"", label));
    }

    // Add text search to CQL
    if let Some(t) = text {
        cql_parts.push(format!("text ~ \"{}\"", t));
    }

    // Add content type to CQL
    if let Some(ct) = content_type {
        cql_parts.push(format!("type = \"{}\"", ct));
    }

    // Combine all parts with AND
    let cql_query = cql_parts.join(" AND ");

    if !json_output {
        println!("Executing Confluence search with CQL: {}", cql_query);
    }
    let search_results = client
        .execute_cql_with_limit(&cql_query, Some(limit))
        .await?;

    if json_output {
        emit_json(
            "search",
            true,
            SearchData {
                query: cql_query,
                count: search_results.len(),
                results: search_results,
            },
        )?;
    } else {
        if search_results.is_empty() {
            println!("No results found for your search query.");
        } else {
            println!("Found {} results:", search_results.len());
            let results_json = serde_json::to_string_pretty(&search_results)
                .context("Failed to serialize search results to JSON")?;
            println!("{}", results_json);
        }
        println!("Search command finished.");
    }

    Ok(())
}
