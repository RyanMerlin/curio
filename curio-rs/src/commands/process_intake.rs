/// Process command — agent-native routing for intake pages.
///
/// Flow:
///   1. `curio process --prepare [--limit=N|--all]`
///        → reads intake pages, outputs a routing manifest JSON to stdout, exits
///        → the agent (Claude / Gemini) reads the manifest, reasons over NORTHSTAR,
///          and produces a decisions JSON
///   2. `curio process --route-file decisions.json`
///        → applies the agent's routing decisions: git mv, frontmatter update, sidecar write
///
/// Manual single-page routing (no LLM needed):
///   `curio process --slug <s> --category product-tree/alteryx-server --status staged`
use anyhow::{Context, Result};
use chrono::Utc;
use std::path::{Path, PathBuf};

use crate::{
    config::Config,
    output::emit_json,
    reconcile::{ReconcileDecision, RoutingAnalysis},
    wiki_fs::{parse_wiki_page, update_frontmatter},
    wiki_index::{append_log, rebuild_index_md},
    PageStatus,
};

pub async fn run_process(
    config: &Config,
    dry_run: bool,
    json: bool,
    limit: u32,
    all: bool,
    prepare: bool,
    route_file: Option<PathBuf>,
    slug: Option<String>,
    category: Option<String>,
    status_arg: Option<String>,
    keywords: Option<String>,
    confidence: Option<f32>,
    summary: Option<String>,
) -> Result<()> {
    let wiki_dir = &config.wiki.wiki_dir;
    let intake_dir = wiki_dir.join("intake");

    // ── Single-page manual routing ─────────────────────────────────────────
    if let Some(slug_str) = slug {
        let src_path = intake_dir.join(format!("{}.md", slug_str));
        if !src_path.exists() {
            anyhow::bail!("Intake page not found: {}", src_path.display());
        }

        let cat_segments = category
            .as_deref()
            .context("Manual routing requires --category; Curio must not invent a published route")?
            .split('/')
            .map(|s| s.to_string())
            .collect::<Vec<_>>();
        let target_status = status_arg.as_deref().unwrap_or("staged");
        let kw = keywords
            .as_deref()
            .unwrap_or("")
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        let conf = confidence.unwrap_or(0.9);

        let decision = ReconcileDecision::manual(
            cat_segments,
            kw,
            conf,
            target_status.to_string(),
            summary.unwrap_or_default(),
        );

        apply_routing(config, &src_path, &slug_str, decision, None, dry_run)?;
        if !dry_run {
            finalize_indexes(config)?;
            commit_if_needed(config, 1)?;
        }
        return Ok(());
    }

    // ── Collect intake pages ───────────────────────────────────────────────
    let effective_limit = if all { u32::MAX } else { limit };
    let intake_pages = collect_intake_pages(&intake_dir, effective_limit)?;

    if intake_pages.is_empty() {
        let msg = "No intake pages to process";
        if json {
            let _ = emit_json("process", true, &serde_json::json!({ "processed": 0, "message": msg }));
        } else {
            println!("{}", msg);
        }
        return Ok(());
    }

    // ── Prepare mode: output manifest and exit ─────────────────────────────
    // The agent reads this manifest, makes routing decisions, then calls
    // `curio process --route-file <path>` to apply them.
    if prepare {
        return output_routing_manifest(config, &intake_pages, json);
    }

    // ── Apply route-file decisions ─────────────────────────────────────────
    if let Some(rf) = route_file {
        let raw = std::fs::read_to_string(&rf)
            .with_context(|| format!("Failed to read route file {}", rf.display()))?;
        let decisions: Vec<(String, ReconcileDecision)> = serde_json::from_str(&raw)
            .context("Failed to parse route file — expected [{\"slug\": ..., \"decision\": {...}}, ...]")?;

        return apply_decisions(config, &intake_pages, decisions, dry_run, json);
    }

    // ── No flags: emit manifest as the agent prompt ────────────────────────
    // This is the default agent-native path. Output the manifest so the agent
    // can reason over it and call back with --route-file.
    output_routing_manifest(config, &intake_pages, json)
}

// ─── Manifest output ──────────────────────────────────────────────────────

fn output_routing_manifest(
    config: &Config,
    intake_pages: &[(String, crate::WikiPage)],
    json: bool,
) -> Result<()> {
    let wiki_dir = &config.wiki.wiki_dir;

    // Load NORTHSTAR for routing context
    let ns_path = wiki_dir.join("_config/northstar.md");
    let northstar_md = if ns_path.exists() {
        std::fs::read_to_string(&ns_path).unwrap_or_default()
    } else {
        String::new()
    };

    // Load root index for quick orientation
    let index_md = crate::wiki_index::read_index_md(wiki_dir).unwrap_or_default();

    let pages: Vec<serde_json::Value> = intake_pages
        .iter()
        .map(|(slug, page)| {
            serde_json::json!({
                "slug": slug,
                "title": page.frontmatter.title,
                "source_url": page.frontmatter.source.origin_url,
                "content_hash": page.frontmatter.content_hash,
                "body_preview": page.body.chars().take(1000).collect::<String>(),
            })
        })
        .collect();

    let manifest = serde_json::json!({
        "action": "route_intake_pages",
        "page_count": pages.len(),
        "northstar_context": northstar_md,
        "index_summary": index_md,
        "pages": pages,
        "instructions": {
            "task": "Route each page to exactly one wiki subtree.",
                "output_format": "JSON array of routing decisions. Each element: {\"slug\": \"...\", \"category\": [\"tree\", \"subtree\"], \"confidence\": 0.0-1.0, \"status\": \"staged|review\", \"keywords\": [...], \"summary\": \"max 200 chars\", \"rationale\": \"...\", \"alternatives_considered\": [{\"path\": [...], \"score\": 0.0, \"ruled_out_because\": \"...\"}], \"review_reason\": null, \"proposed_new_subtree\": null, \"proposal_rationale\": null}",
            "confidence_rule": "confidence >= 0.75 and a valid existing subtree fit → staged. Otherwise → review.",
            "new_subtree_rule": "If no existing subtree fits confidently, do not force publication. Route to review, provide the closest category path you can justify, and fill proposed_new_subtree plus proposal_rationale.",
            "apply_command": "curio process --route-file <path-to-decisions.json>"
        }
    });

    if json {
        println!("{}", serde_json::to_string_pretty(&manifest)?);
    } else {
        println!("{}", serde_json::to_string_pretty(&manifest)?);
    }

    Ok(())
}

// ─── Apply decisions ──────────────────────────────────────────────────────

fn apply_decisions(
    config: &Config,
    intake_pages: &[(String, crate::WikiPage)],
    decisions: Vec<(String, ReconcileDecision)>,
    dry_run: bool,
    json: bool,
) -> Result<()> {
    let wiki_dir = &config.wiki.wiki_dir;
    let intake_dir = wiki_dir.join("intake");
    let mut processed = Vec::new();
    let mut errors: Vec<String> = Vec::new();

    for (slug, page) in intake_pages {
        let decision = match decisions.iter().find(|(s, _)| s == slug) {
            Some((_, d)) => d.clone(),
            None => {
                eprintln!("  warning: no decision for '{}' — skipping", slug);
                continue;
            }
        };

        let src_path = intake_dir.join(format!("{}.md", slug));
        if dry_run {
            let conf_pct = (decision.confidence * 100.0) as u32;
            processed.push(serde_json::json!({
                "slug": slug,
                "title": &page.frontmatter.title,
                "would_route_to": &decision.status,
                "category": &decision.category,
                "confidence": conf_pct,
            }));
        } else {
            // Build a RoutingAnalysis sidecar from the decision
            let analysis = build_analysis_from_decision(&decision, &page.frontmatter.title, &page.body, page.frontmatter.source.origin_url.as_deref(), &page.frontmatter.content_hash);

            match apply_routing(config, &src_path, slug, decision.clone(), Some(&analysis), false) {
                Ok(()) => {
                    let conf_pct = (decision.confidence * 100.0) as u32;
                    processed.push(serde_json::json!({
                        "slug": slug,
                        "title": &page.frontmatter.title,
                        "routed_to": &decision.status,
                        "category": &decision.category,
                        "confidence": conf_pct,
                    }));
                }
                Err(e) => errors.push(format!("{}: {}", slug, e)),
            }
        }
    }

    if !dry_run && !processed.is_empty() {
        finalize_indexes(config)?;
        commit_if_needed(config, processed.len())?;
    }

    if json {
        let _ = emit_json(
            "process",
            true,
            &serde_json::json!({ "processed": processed, "errors": errors, "dry_run": dry_run }),
        );
    } else {
        for item in &processed {
            let title = item["title"].as_str().unwrap_or("?");
            let to = item["routed_to"].as_str().or_else(|| item["would_route_to"].as_str()).unwrap_or("?");
            let cat = item["category"]
                .as_array()
                .map(|a| a.iter().map(|v| v.as_str().unwrap_or("")).collect::<Vec<_>>().join("/"))
                .unwrap_or_default();
            let conf = item["confidence"].as_u64().map(|c| format!(" [{c}%]")).unwrap_or_default();
            println!("  {} → {}/{}{}", title, to, cat, conf);
        }
        for e in &errors {
            eprintln!("  error: {}", e);
        }
        println!(
            "{} processed, {} errors{}",
            processed.len(),
            errors.len(),
            if dry_run { " (dry run)" } else { "" }
        );
    }
    Ok(())
}

// ─── Helpers ──────────────────────────────────────────────────────────────

fn collect_intake_pages(intake_dir: &Path, limit: u32) -> Result<Vec<(String, crate::WikiPage)>> {
    if !intake_dir.exists() {
        return Ok(vec![]);
    }
    let mut pages = Vec::new();
    for entry in std::fs::read_dir(intake_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().map_or(false, |e| e == "md") {
            if let Ok(page) = parse_wiki_page(&path) {
                let slug = path
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();
                pages.push((slug, page));
            }
        }
        if pages.len() >= limit as usize {
            break;
        }
    }
    Ok(pages)
}

fn apply_routing(
    config: &Config,
    src_path: &Path,
    slug: &str,
    decision: ReconcileDecision,
    analysis: Option<&RoutingAnalysis>,
    _dry_run: bool,
) -> Result<()> {
    let wiki_dir = &config.wiki.wiki_dir;
    let target_status = if decision.status == "staged" { "staged" } else { "review" };
    if decision.category.is_empty() {
        anyhow::bail!("Routing decision for '{}' is missing a category path", slug);
    }
    let cat_path: PathBuf = decision.category.iter().collect();
    let dest_dir = wiki_dir.join(target_status).join(&cat_path);
    let dest_path = dest_dir.join(format!("{}.md", slug));

    std::fs::create_dir_all(&dest_dir)?;

    let mut page = parse_wiki_page(src_path)?;
    let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    page.frontmatter.status = if target_status == "staged" { PageStatus::Staged } else { PageStatus::Review };
    page.frontmatter.category = decision.category.clone();
    page.frontmatter.keywords = decision.keywords.clone();
    page.frontmatter.confidence = Some(decision.confidence);
    page.frontmatter.updated_at = now;
    page.frontmatter.model_used = Some(decision.model_used.clone());

    update_frontmatter(src_path, &page.frontmatter)?;

    // Write analysis sidecar before git mv
    if let Some(a) = analysis {
        write_analysis_sidecar(src_path, a)?;
    }

    let repo_root = wiki_dir.parent().unwrap_or(wiki_dir);
    let rel_src = src_path.strip_prefix(repo_root).unwrap_or(src_path);
    let rel_dest = dest_path.strip_prefix(repo_root).unwrap_or(&dest_path);
    crate::git_ops::git_mv(repo_root, rel_src, rel_dest)?;

    // Move analysis sidecar alongside content
    let analysis_src = src_path.with_extension("analysis.json");
    if analysis_src.exists() {
        let analysis_dest = dest_path.with_extension("analysis.json");
        let rel_asrc = analysis_src.strip_prefix(repo_root).unwrap_or(&analysis_src);
        let rel_adest = analysis_dest.strip_prefix(repo_root).unwrap_or(&analysis_dest);
        crate::git_ops::git_mv(repo_root, rel_asrc, rel_adest)?;
    }

    Ok(())
}

/// Build a RoutingAnalysis sidecar from a ReconcileDecision (for --route-file path).
fn build_analysis_from_decision(
    decision: &ReconcileDecision,
    title: &str,
    body: &str,
    source_url: Option<&str>,
    content_hash: &str,
) -> RoutingAnalysis {
    use crate::reconcile::{AnalysisInputs, AnalysisRouting, AnalysisSignals};

    let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let content_preview: String = body.chars().take(500).collect();
    let title_tokens = title.split_whitespace().map(|w| w.to_lowercase()).collect();
    let keywords_extracted = crate::reconcile::extract_keywords(&format!("{} {}", title, body), 8);
    let pre_signal = crate::reconcile::heuristic_pre_signal(title, body);

    RoutingAnalysis {
        schema_version: 1,
        analyzed_at: now,
        model: decision.model_used.clone(),
        inputs: AnalysisInputs {
            title: title.to_string(),
            source_url: source_url.map(|s| s.to_string()),
            content_hash: content_hash.to_string(),
            content_preview,
        },
        routing: AnalysisRouting {
            decision: decision.category.clone(),
            confidence: decision.confidence,
            rationale: String::new(), // populated by agent via --route-file if provided
            alternatives_considered: vec![],
            flags: vec![],
            review_reason: decision.review_reason.clone(),
            proposed_new_subtree: decision.proposed_new_subtree.clone(),
            proposal_rationale: decision.proposal_rationale.clone(),
        },
        signals: AnalysisSignals {
            heuristic_pre_signal: pre_signal,
            title_tokens,
            keywords_extracted,
        },
    }
}

/// Write a routing analysis sidecar alongside `content_path`.
pub fn write_analysis_sidecar(content_path: &Path, analysis: &RoutingAnalysis) -> Result<()> {
    let sidecar_path = content_path.with_extension("analysis.json");
    let json = serde_json::to_string_pretty(analysis)
        .context("Failed to serialize routing analysis")?;
    std::fs::write(&sidecar_path, json)
        .with_context(|| format!("Failed to write analysis sidecar: {}", sidecar_path.display()))?;

    // Stage the sidecar for git
    let repo_root = content_path
        .ancestors()
        .find(|p| p.join(".git").exists())
        .unwrap_or(content_path.parent().unwrap_or(content_path));
    let rel = sidecar_path.strip_prefix(repo_root).unwrap_or(&sidecar_path);
    let _ = std::process::Command::new("git")
        .args(["add", &rel.to_string_lossy()])
        .current_dir(repo_root)
        .output();

    Ok(())
}

fn finalize_indexes(config: &Config) -> Result<()> {
    let wiki_dir = &config.wiki.wiki_dir;
    let registry = crate::WikiIndex::default();
    rebuild_index_md(wiki_dir, &registry)?;
    append_log(wiki_dir, "process: intake items routed")?;
    Ok(())
}

fn commit_if_needed(config: &Config, count: usize) -> Result<()> {
    if !config.wiki.auto_commit {
        return Ok(());
    }
    let wiki_dir = &config.wiki.wiki_dir;
    let repo_root = wiki_dir.parent().unwrap_or(wiki_dir);
    crate::git_ops::git_add(repo_root, wiki_dir)?;
    if crate::git_ops::git_has_staged(repo_root) {
        crate::git_ops::git_commit(repo_root, &format!("curio: process {} intake item(s)", count))?;
    }
    Ok(())
}
