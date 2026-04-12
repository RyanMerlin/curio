use anyhow::Result;

use crate::{
    config::Config,
    output::emit_json,
    wiki_index::{append_log, rebuild_index_md, reindex_from_filesystem, save_registry},
};

pub async fn run_reindex(config: &Config, dry_run: bool, json: bool) -> Result<()> {
    let wiki_dir = &config.wiki.wiki_dir;

    if !wiki_dir.exists() {
        anyhow::bail!(
            "Wiki directory not found: {}. Run `curio init` first.",
            wiki_dir.display()
        );
    }

    let index = reindex_from_filesystem(wiki_dir)?;
    let count = index.pages.len();

    if dry_run {
        if json {
            let _ = emit_json("reindex", true, &serde_json::json!({ "pages_found": count, "dry_run": true }));
        } else {
            println!("Would reindex {} pages (dry run)", count);
        }
        return Ok(());
    }

    save_registry(wiki_dir, &index)?;
    rebuild_index_md(wiki_dir, &index)?;
    append_log(wiki_dir, &format!("reindex: rebuilt index from {} pages", count))?;

    if json {
        let _ = emit_json("reindex", true, &serde_json::json!({ "pages_indexed": count }));
    } else {
        println!("Reindexed {} pages in {}", count, wiki_dir.display());
    }
    Ok(())
}
