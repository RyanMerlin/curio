use anyhow::Result;

use crate::{
    config::Config,
    northstar::sync_taxonomy_from_markdown,
    output::emit_json,
    wiki_index::{
        append_log, rebuild_colocated_indexes, reindex_from_filesystem,
    },
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

    let _ = sync_taxonomy_from_markdown(wiki_dir);
    let trees = crate::northstar::load_taxonomy(wiki_dir)
        .map(|taxonomy| taxonomy.nodes.iter().map(crate::commands::sync::tree_node_from_taxonomy).collect())
        .unwrap_or_else(|_| {
            eprintln!("Warning: _config/northstar.json not available — indexes will have minimal descriptions.");
            vec![]
        });

    rebuild_colocated_indexes(wiki_dir, &index, &trees)?;
    append_log(wiki_dir, &format!("reindex: rebuilt hierarchical indexes from {} pages", count))?;

    if json {
        let _ = emit_json("reindex", true, &serde_json::json!({ "pages_indexed": count }));
    } else {
        println!("Reindexed {} pages → co-located index.md files rebuilt", count);
    }
    Ok(())
}
