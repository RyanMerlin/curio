pub mod cli;
pub mod commands;
pub mod config;
pub mod confluence;
pub mod error;
pub mod harness;

use anyhow::Result as AnyhowResult;
pub use error::Result;
use serde::{Deserialize, Serialize};

// --- Shared Placeholder for External Agent Integration ---
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AgentAnalysis {
    pub summary: String,
    pub keywords: Vec<String>,
    pub confidence_score: f32, // 0.0 to 1.0
                               // potentially embedding vectors for semantic search
}

pub async fn analyze_content_with_agent(_content: &str) -> AnyhowResult<AgentAnalysis> {
    // This is a placeholder. In a real scenario, this would make an HTTP call
    // to an external AI agent service.

    // Simulate some analysis results
    Ok(AgentAnalysis {
        summary: "This is a simulated summary of the content.".to_string(),
        keywords: vec![
            "simulated".to_string(),
            "analysis".to_string(),
            "curio".to_string(),
        ],
        confidence_score: 0.85, // Simulate high confidence
    })
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Change {
    pub target_page_id: String,
    pub target_page_title: String,
    pub change_type: String, // e.g., "update_section", "add_note"
    pub summary_of_change: String,
    pub proposed_content_diff: String, // A diff or the full new section content
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChangeProposal {
    pub summary: String,
    pub proposed_changes: Vec<Change>,
}

pub async fn generate_change_proposal_with_agent(_content: &str) -> AnyhowResult<ChangeProposal> {
    // This is a placeholder. In a real scenario, this would make an HTTP call
    // to an external AI agent service to get change proposals.

    // Simulate some analysis results
    Ok(ChangeProposal {
        summary: "This is a simulated change proposal.".to_string(),
        proposed_changes: vec![Change {
            target_page_id: "4218060932".to_string(),
            target_page_title: "publish-test - publish-".to_string(),
            change_type: "update_section".to_string(),
            summary_of_change: "Adds simulated new detail to section.".to_string(),
            proposed_content_diff: "<p>Simulated new content for section.</p>".to_string(),
        }],
    })
}

// Helper to get page ID (moved from commands)
pub async fn get_page_id_by_title(
    client: &crate::confluence::ConfluenceClient,
    space_key: &str,
    parent_id: Option<&str>,
    title: &str,
    page_type: &str,
) -> AnyhowResult<String> {
    client
        .get_page_by_title(space_key, parent_id, title)
        .await?
        .and_then(|page| page["id"].as_str().map(|s| s.to_string()))
        .ok_or_else(|| {
            anyhow::anyhow!(format!(
                "Could not find {} page '{}' in space '{}'",
                page_type, title, space_key
            ))
        })
}

pub async fn resolve_managed_root_folder_id(
    client: &crate::confluence::ConfluenceClient,
    space_key: &str,
    root_folder_name: &str,
    output_root_folder_id: Option<&str>,
) -> AnyhowResult<String> {
    if let Some(explicit_output_root_folder_id) = output_root_folder_id {
        let explicit_output_root_folder = client
            .get_folder_by_id_v2(explicit_output_root_folder_id)
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!(format!(
                    "Configured CURIO_CONFLUENCE_OUTPUT_ROOT_FOLDER_ID {} does not exist",
                    explicit_output_root_folder_id
                ))
            })?;

        let explicit_output_root_title = explicit_output_root_folder["title"]
            .as_str()
            .unwrap_or(explicit_output_root_folder_id);
        println!(
            "Using configured CURIO output root folder {} ('{}')",
            explicit_output_root_folder_id, explicit_output_root_title
        );

        Ok(explicit_output_root_folder_id.to_string())
    } else if root_folder_name.is_empty() {
        anyhow::bail!(
            "Confluence output root folder is not configured. Set CURIO_CONFLUENCE_OUTPUT_ROOT_FOLDER_ID in .env or CURIO_ROOT_FOLDER_NAME as a fallback."
        );
    } else {
        get_page_id_by_title(client, space_key, None, root_folder_name, "root folder").await
    }
}

pub async fn resolve_or_create_scoped_child_page_id(
    client: &crate::confluence::ConfluenceClient,
    space_key: &str,
    parent_id: &str,
    title: &str,
    body_content: &str,
) -> AnyhowResult<String> {
    if let Some(page) = client
        .get_page_by_title(space_key, Some(parent_id), title)
        .await?
    {
        if let Some(page_id) = page["id"].as_str() {
            if client.page_is_descendant_of(page_id, parent_id).await? {
                return Ok(page_id.to_string());
            }
            client.migrate_page_to_parent(page_id, parent_id).await?;
            return Ok(page_id.to_string());
        }
    }

    client
        .create_or_update_page(space_key, Some(parent_id), title, "storage", body_content)
        .await
}
