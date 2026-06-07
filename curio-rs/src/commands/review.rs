use anyhow::Result;

use crate::{config::Config, output::emit_json, wiki_index::load_registry};

pub async fn run_review(config: &Config, _dry_run: bool, json: bool, lane: &str) -> Result<()> {
    let wiki_dir = &config.wiki.wiki_dir;
    let registry = load_registry(wiki_dir)?;

    let pages: Vec<_> = registry
        .pages
        .iter()
        .filter(|e| match lane {
            "review" => e.status == "review",
            "staged" => e.status == "staged",
            _ => e.status == "review" || e.status == "staged",
        })
        .collect();

    if json {
        let _ = emit_json("review", true, serde_json::json!({ "pages": pages }));
    } else {
        if pages.is_empty() {
            println!("No items in {}", lane);
        } else {
            let width = pages.iter().map(|p| p.path.len()).max().unwrap_or(30);
            println!("{:<width$}  {:<10}  TITLE", "PATH", "STATUS", width = width);
            println!("{}", "-".repeat(width + 40));
            for p in &pages {
                println!(
                    "{:<width$}  {:<10}  {}",
                    p.path,
                    p.status,
                    p.title,
                    width = width
                );
            }
        }
    }
    Ok(())
}
