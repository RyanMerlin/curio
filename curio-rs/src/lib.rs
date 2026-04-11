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
        proposed_changes: vec![
            Change {
                target_page_id: "mock_page_id_1".to_string(),
                target_page_title: "Simulated Gold Page 1".to_string(),
                change_type: "update_section".to_string(),
                summary_of_change: "Adds simulated new detail to section.".to_string(),
                proposed_content_diff: "<p>Simulated new content for section.</p>".to_string(),
            },
            Change {
                target_page_id: "mock_page_id_2".to_string(),
                target_page_title: "Simulated Gold Page 2".to_string(),
                change_type: "add_note".to_string(),
                summary_of_change: "Adds a simulated note about impact.".to_string(),
                proposed_content_diff: "<p>Simulated note about impact.</p>".to_string(),
            },
        ],
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
