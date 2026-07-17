use anyhow::Result;

use crate::{
    acl::AccessContext,
    config::Config,
    output::emit_json,
    retrieval::{RetrieveRequest, fetch_published},
};

/// Thin CLI adapter for deterministic published-page retrieval.
pub async fn run_retrieve(
    config: &Config,
    _dry_run: bool,
    json: bool,
    query: String,
    category: Option<String>,
    limit: u32,
    principals: Vec<String>,
) -> Result<()> {
    let access = (!principals.is_empty()).then(|| AccessContext::new(principals));
    let response = crate::retrieval::retrieve_published_with_access(
        &config.wiki.wiki_dir,
        &RetrieveRequest {
            query,
            category,
            limit: limit as usize,
        },
        access.as_ref(),
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
pub async fn run_fetch(
    config: &Config,
    _dry_run: bool,
    json: bool,
    id: String,
    principals: Vec<String>,
) -> Result<()> {
    let access = (!principals.is_empty()).then(|| AccessContext::new(principals));
    let response = if let Some(access) = access.as_ref() {
        crate::retrieval::fetch_published_with_access(&config.wiki.wiki_dir, &id, Some(access))?
    } else {
        fetch_published(&config.wiki.wiki_dir, &id)?
    };

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
