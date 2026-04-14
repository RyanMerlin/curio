use anyhow::{Context, Result};
use chrono::Utc;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::{
    config::Config,
    output::emit_json,
    wiki_fs::parse_wiki_page,
    wiki_index::append_log,
};

pub async fn run_sharpen(
    config: &Config,
    dry_run: bool,
    json: bool,
    prepare: bool,
    proposal_file: Option<PathBuf>,
    limit: u32,
) -> Result<()> {
    let wiki_dir = &config.wiki.wiki_dir;
    let published_dir = wiki_dir.join("published");
    let proposals_dir = wiki_dir.join("_config").join("sharpening-proposals");
    let legacy_dir = wiki_dir.join(".curio").join("sharpening-proposals");

    if let Some(proposal_path) = proposal_file {
        let raw = std::fs::read_to_string(&proposal_path)
            .with_context(|| format!("Failed to read proposal file {}", proposal_path.display()))?;
        let payload: serde_json::Value = serde_json::from_str(&raw)
            .context("Sharpen proposal file must be valid JSON")?;
        let wrapped = wrap_proposals_payload(payload);

        if dry_run {
            if json {
                let _ = emit_json("sharpen", true, &serde_json::json!({
                    "mode": "persist_proposals",
                    "dry_run": true,
                    "proposal_count": wrapped["proposals"].as_array().map(|items| items.len()).unwrap_or(0),
                }));
            } else {
                println!(
                    "Would persist {} sharpening proposal(s) into {}",
                    wrapped["proposals"].as_array().map(|items| items.len()).unwrap_or(0),
                    proposals_dir.display()
                );
            }
            return Ok(());
        }

        std::fs::create_dir_all(&proposals_dir)
            .with_context(|| format!("Failed to create {}", proposals_dir.display()))?;
        if legacy_dir.exists() {
            for entry in std::fs::read_dir(&legacy_dir).into_iter().flatten().filter_map(|entry| entry.ok()) {
                let src = entry.path();
                let dest = proposals_dir.join(entry.file_name());
                if src.extension().and_then(|ext| ext.to_str()) == Some("json") && !dest.exists() {
                    let _ = std::fs::copy(&src, &dest);
                }
            }
        }
        let filename = format!("{}.json", Utc::now().format("%Y%m%dT%H%M%SZ"));
        let dest_path = proposals_dir.join(filename);
        std::fs::write(&dest_path, serde_json::to_string_pretty(&wrapped)?)
            .with_context(|| format!("Failed to write {}", dest_path.display()))?;
        compact_proposal_store(&proposals_dir, 20)?;
        append_log(wiki_dir, &format!("sharpen: stored proposals at {}", dest_path.display()))?;

        if json {
            let _ = emit_json("sharpen", true, &serde_json::json!({
                "mode": "persist_proposals",
                "stored_at": dest_path,
                "proposal_count": wrapped["proposals"].as_array().map(|items| items.len()).unwrap_or(0),
            }));
        } else {
            println!(
                "Stored {} sharpening proposal(s) at {}",
                wrapped["proposals"].as_array().map(|items| items.len()).unwrap_or(0),
                dest_path.display()
            );
        }
        return Ok(());
    }

    let manifest = build_sharpen_manifest(&published_dir, wiki_dir, limit)?;
    if prepare || !dry_run {
        if json {
            println!("{}", serde_json::to_string_pretty(&manifest)?);
        } else {
            println!("{}", serde_json::to_string_pretty(&manifest)?);
        }
        return Ok(());
    }

    Ok(())
}

fn build_sharpen_manifest(published_dir: &Path, wiki_dir: &Path, limit: u32) -> Result<serde_json::Value> {
    let northstar_path = wiki_dir.join("_config").join("northstar.md");
    let northstar_md = std::fs::read_to_string(&northstar_path).unwrap_or_default();

    let mut pages = Vec::new();
    let mut by_title: HashMap<String, Vec<String>> = HashMap::new();
    let mut by_hash: HashMap<String, Vec<String>> = HashMap::new();
    let mut oversized = Vec::new();

    if published_dir.exists() {
        for entry in walkdir::WalkDir::new(published_dir)
            .into_iter()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().is_file())
        {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
                continue;
            }
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            if name == "index.md" {
                continue;
            }

            let page = parse_wiki_page(path)
                .with_context(|| format!("Failed to parse {}", path.display()))?;
            let rel = path
                .strip_prefix(wiki_dir)
                .unwrap_or(path)
                .to_string_lossy()
                .replace('\\', "/");
            let word_count = page.body.split_whitespace().count();
            let source_url = page.frontmatter.source.origin_url.clone();

            by_title
                .entry(page.frontmatter.title.clone())
                .or_default()
                .push(rel.clone());
            by_hash
                .entry(page.frontmatter.content_hash.clone())
                .or_default()
                .push(rel.clone());

            if word_count > 1800 {
                oversized.push(serde_json::json!({
                    "path": rel,
                    "title": page.frontmatter.title,
                    "word_count": word_count,
                    "suggested_action": "review_for_split",
                }));
            }

            pages.push(serde_json::json!({
                "path": rel,
                "title": page.frontmatter.title,
                "category": page.frontmatter.category,
                "updated_at": page.frontmatter.updated_at,
                "word_count": word_count,
                "source_url": source_url,
                "confidence": page.frontmatter.confidence,
            }));
        }
    }

    pages.sort_by(|a, b| {
        a["title"]
            .as_str()
            .unwrap_or("")
            .cmp(b["title"].as_str().unwrap_or(""))
    });

    let duplicate_titles: Vec<_> = by_title
        .into_iter()
        .filter(|(_, paths)| paths.len() > 1)
        .map(|(title, paths)| serde_json::json!({
            "title": title,
            "paths": paths,
            "suggested_action": "review_for_consolidation_or_rename",
        }))
        .collect();

    let duplicate_hashes: Vec<_> = by_hash
        .into_iter()
        .filter(|(_, paths)| paths.len() > 1)
        .map(|(content_hash, paths)| serde_json::json!({
            "content_hash": content_hash,
            "paths": paths,
            "suggested_action": "review_for_merge",
        }))
        .collect();

    Ok(serde_json::json!({
        "action": "sharpen_review",
        "generated_at": Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        "northstar_context": northstar_md,
        "page_count": pages.len(),
        "pages": pages.into_iter().take(limit as usize).collect::<Vec<_>>(),
        "deterministic_candidates": {
            "duplicate_titles": duplicate_titles,
            "duplicate_hashes": duplicate_hashes,
            "oversized_pages": oversized,
        },
        "instructions": {
            "task": "Review the published corpus and propose merge, split, retitle, reroute, archive, or new-subtree actions. This is proposal-only; do not mutate content.",
            "output_format": {
                "proposals": [{
                    "type": "merge|split|retitle|reroute|archive|new_subtree",
                    "affected_paths": ["wiki/published/..."],
                    "recommended_action": "short directive",
                    "rationale": "why this improves signal density",
                    "evidence": ["source url or page path"],
                    "confidence": 0.0,
                    "expected_signal_gain": "short statement"
                }]
            },
            "persist_command": "curio sharpen --proposal-file <path-to-proposals.json>"
        }
    }))
}

fn wrap_proposals_payload(payload: serde_json::Value) -> serde_json::Value {
    if payload.get("proposals").is_some() {
        serde_json::json!({
            "generated_at": Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            "proposals": payload["proposals"].clone(),
        })
    } else if payload.is_array() {
        serde_json::json!({
            "generated_at": Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            "proposals": payload,
        })
    } else {
        serde_json::json!({
            "generated_at": Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            "proposals": [payload],
        })
    }
}

fn compact_proposal_store(dir: &Path, keep: usize) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }

    let mut files: Vec<_> = std::fs::read_dir(dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().and_then(|ext| ext.to_str()) == Some("json"))
        .collect();
    files.sort_by_key(|entry| entry.file_name());

    if files.len() <= keep {
        return Ok(());
    }

    let remove_count = files.len().saturating_sub(keep);
    for entry in files.into_iter().take(remove_count) {
        let path = entry.path();
        std::fs::remove_file(&path)
            .with_context(|| format!("Failed to remove old sharpening proposal {}", path.display()))?;
    }

    Ok(())
}
