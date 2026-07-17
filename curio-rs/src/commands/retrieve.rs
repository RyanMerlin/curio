use anyhow::Result;

use crate::{
    config::Config,
    output::emit_json,
    retrieval::{RetrieveRequest, fetch_published, retrieve_published},
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

/// Thin CLI adapter for deterministic published-page fetch by stable retrieve id.
pub async fn run_fetch(config: &Config, _dry_run: bool, json: bool, id: String) -> Result<()> {
    let response = fetch_published(&config.wiki.wiki_dir, &id)?;

    if json {
        emit_json("fetch", true, response)?;
    } else {
        println!("{}", response.title);
        println!("path: {}", response.path);
        println!("id: {}", response.id);
        println!("category: {}", response.category);
        println!("updated: {}", response.updated_at);
        let citation = response.source_uri.as_deref().unwrap_or(&response.path);
        let commit = response
            .last_commit
            .as_ref()
            .map(|commit| commit.hash.as_str())
            .unwrap_or("unavailable");
        println!(
            "cite: {} | authority: {} | commit: {}",
            citation, response.authority, commit
        );
        println!();
        print!("{}", response.body);
        if !response.body.ends_with('\n') {
            println!();
        }
    }
    Ok(())
}
