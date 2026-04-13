use anyhow::{Context, Result};
use chrono::Utc;
use std::path::{Path, PathBuf};

use crate::{
    config::Config,
    output::emit_json,
    reconcile::{ReconcileDecision, RoutingAnalysis, route_with_llm},
    wiki_fs::{parse_wiki_page, update_frontmatter},
    wiki_index::{
        append_log, load_registry, rebuild_index_md, save_registry,
        update_entry_path, update_entry_status,
    },
    PageStatus,
};

pub async fn run_process(
    config: &Config,
    dry_run: bool,
    json: bool,
    limit: u32,
    _auto_mode: bool,
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
            .unwrap_or("topic-tree")
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
        let conf = confidence.unwrap_or(0.8);

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

    // ── Batch routing ──────────────────────────────────────────────────────
    let intake_pages = collect_intake_pages(&intake_dir, limit)?;

    if intake_pages.is_empty() {
        let msg = "No intake pages to process";
        if json {
            let _ = emit_json("process", true, &serde_json::json!({ "processed": 0, "message": msg }));
        } else {
            println!("{}", msg);
        }
        return Ok(());
    }

    // ── Pre-compute decisions (file-based or LLM) ─────────────────────────
    let decisions: Vec<(String, ReconcileDecision, Option<RoutingAnalysis>)> = if let Some(rf) = route_file {
        // Manual route file — no LLM, no sidecar
        let raw = std::fs::read_to_string(&rf)
            .with_context(|| format!("Failed to read route file {}", rf.display()))?;
        let pairs: Vec<(String, ReconcileDecision)> = serde_json::from_str(&raw)
            .context("Failed to parse route file")?;
        pairs.into_iter().map(|(s, d)| (s, d, None)).collect()
    } else {
        // LLM routing — one call per page, concurrent
        let api_key = config.llm.require_api_key()?;
        let model = config.llm.effective_model();

        // Load NORTHSTAR trees for routing context
        let ns_path = wiki_dir.join("_schema/northstar.md");
        let trees = if ns_path.exists() {
            let md = std::fs::read_to_string(&ns_path).unwrap_or_default();
            crate::commands::sync::parse_northstar_blueprint(&md)
        } else {
            vec![]
        };

        if trees.is_empty() {
            eprintln!("Warning: no NORTHSTAR blueprint found — routing context will be minimal. Run `curio init` to seed _schema/northstar.md.");
        }

        // Spawn concurrent LLM calls
        let mut handles = Vec::new();
        for (slug, page) in intake_pages.iter() {
            let api_key = api_key.clone();
            let model = model.clone();
            let slug = slug.clone();
            let title = page.frontmatter.title.clone();
            let body = page.body.clone();
            let source_url = page.frontmatter.source.origin_url.clone();
            let content_hash = page.frontmatter.content_hash.clone();
            let trees = trees.clone();

            let handle = tokio::spawn(async move {
                let result = route_with_llm(
                    &api_key,
                    &model,
                    &title,
                    &body,
                    source_url.as_deref(),
                    &content_hash,
                    &trees,
                )
                .await;
                (slug, result)
            });
            handles.push(handle);
        }

        let mut decisions = Vec::new();
        for handle in handles {
            let (slug, result) = handle.await.context("LLM routing task panicked")?;
            match result {
                Ok((decision, analysis)) => {
                    decisions.push((slug, decision, Some(analysis)));
                }
                Err(e) => {
                    eprintln!("  warning: LLM routing failed for '{}': {} — routing to review", slug, e);
                    // Fallback: route to review with error flag
                    let fallback = ReconcileDecision {
                        category: vec!["topic-tree".to_string()],
                        keywords: vec![],
                        confidence: 0.0,
                        status: "review".to_string(),
                        summary: format!("LLM routing failed: {}", e),
                        cross_refs: vec![],
                        review_reason: Some(format!("LLM error: {}", e)),
                        merge_target: None,
                        model_used: "error-fallback".to_string(),
                    };
                    decisions.push((slug, fallback, None));
                }
            }
        }
        decisions
    };

    let mut processed = Vec::new();
    let mut errors: Vec<String> = Vec::new();

    for (slug, page) in &intake_pages {
        let (decision, analysis) = decisions
            .iter()
            .find(|(s, _, _)| s == slug)
            .map(|(_, d, a)| (d.clone(), a.clone()))
            .unwrap_or_else(|| {
                // Shouldn't happen but safe fallback
                (ReconcileDecision::manual(
                    vec!["topic-tree".to_string()],
                    vec![],
                    0.5,
                    "review".to_string(),
                    String::new(),
                ), None)
            });

        let src_path = intake_dir.join(format!("{}.md", slug));
        if dry_run {
            let conf_pct = (decision.confidence * 100.0) as u32;
            processed.push(serde_json::json!({
                "slug": slug,
                "title": &page.frontmatter.title,
                "would_route_to": &decision.status,
                "category": &decision.category,
                "confidence": conf_pct,
                "rationale": analysis.as_ref().map(|a| &a.routing.rationale),
            }));
        } else {
            match apply_routing(config, &src_path, slug, decision.clone(), analysis.as_ref(), false) {
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
            let to = item["routed_to"]
                .as_str()
                .or_else(|| item["would_route_to"].as_str())
                .unwrap_or("?");
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
    let cat_path: PathBuf = decision.category.iter().collect();
    let dest_dir = wiki_dir.join(target_status).join(&cat_path);
    let dest_path = dest_dir.join(format!("{}.md", slug));

    std::fs::create_dir_all(&dest_dir)?;

    let mut page = parse_wiki_page(src_path)?;
    let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    page.frontmatter.status = if target_status == "staged" {
        PageStatus::Staged
    } else {
        PageStatus::Review
    };
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

    // Move analysis sidecar alongside the content file
    let analysis_src = src_path.with_extension("analysis.json");
    if analysis_src.exists() {
        let analysis_dest = dest_path.with_extension("analysis.json");
        let rel_asrc = analysis_src.strip_prefix(repo_root).unwrap_or(&analysis_src);
        let rel_adest = analysis_dest.strip_prefix(repo_root).unwrap_or(&analysis_dest);
        crate::git_ops::git_mv(repo_root, rel_asrc, rel_adest)?;
    }

    let new_rel = dest_path
        .strip_prefix(wiki_dir)
        .unwrap_or(&dest_path)
        .to_string_lossy()
        .replace('\\', "/");
    update_entry_path(wiki_dir, &page.frontmatter.id, &new_rel)?;
    update_entry_status(wiki_dir, &page.frontmatter.id, &page.frontmatter.status)?;

    Ok(())
}

/// Write a routing analysis sidecar file alongside `content_path`.
/// The sidecar has the same stem with `.analysis.json` extension.
/// e.g. `intake/my-page.md` → `intake/my-page.analysis.json`
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
    let registry = load_registry(wiki_dir)?;
    save_registry(wiki_dir, &registry)?;
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
        crate::git_ops::git_commit(
            repo_root,
            &format!("curio: process {} intake item(s)", count),
        )?;
    }
    Ok(())
}
