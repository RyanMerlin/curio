use anyhow::Result;

use crate::{
    config::Config,
    output::emit_json,
    wiki_index::{
        append_log, rebuild_colocated_indexes, reindex_from_filesystem, save_registry,
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

    save_registry(wiki_dir, &index)?;

    // Load NORTHSTAR blueprint for rich hierarchical indexes
    let ns_path = wiki_dir.join("_schema/northstar.md");
    let trees = if ns_path.exists() {
        let md = std::fs::read_to_string(&ns_path).unwrap_or_default();
        crate::commands::sync::parse_northstar_blueprint(&md)
    } else {
        eprintln!("Warning: _schema/northstar.md not found — indexes will have minimal descriptions.");
        vec![]
    };

    rebuild_colocated_indexes(wiki_dir, &index, &trees)?;
    append_log(wiki_dir, &format!("reindex: rebuilt hierarchical indexes from {} pages", count))?;

    if json {
        let _ = emit_json("reindex", true, &serde_json::json!({ "pages_indexed": count }));
    } else {
        println!("Reindexed {} pages → co-located index.md files rebuilt", count);
    }
    Ok(())
}
