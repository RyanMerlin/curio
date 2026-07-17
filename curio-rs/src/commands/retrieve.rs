use anyhow::Result;

use crate::{
    config::Config,
    output::emit_json,
    retrieval::{RetrieveRequest, retrieve_published},
};

/// Thin CLI adapter for deterministic published-page retrieval.
pub async fn run_retrieve(
    config: &Config,
    _dry_run: bool,
    json: bool,
    query: String,
    category: Option<String>,
    limit: u32,
) -> Result<()> {
    let response = retrieve_published(
        &config.wiki.wiki_dir,
        &RetrieveRequest {
            query,
            category,
            limit: limit as usize,
        },
    )?;

    if json {
        emit_json("retrieve", true, response)?;
    } else if response.results.is_empty() {
        println!("No published matches for {:?}.", response.query);
    } else {
        for result in response.results {
            println!("[{}] {} — {}", result.score, result.title, result.path);
            println!("  {}", result.excerpt);
            let citation = result.source_uri.as_deref().unwrap_or(&result.path);
            let commit = result
                .last_commit
                .as_ref()
                .map(|commit| commit.hash.as_str())
                .unwrap_or("unavailable");
            println!(
                "  cite: {} | updated: {} | commit: {}",
                citation, result.updated_at, commit
            );
        }
    }
    Ok(())
}
