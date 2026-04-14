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
    northstar::{load_taxonomy, taxonomy_path_exists},
    output::emit_json,
    overlap::find_peer_overlap,
    proposal::{
        required_lane, save_proposal_record, ProposalDossier, ProposalKind, ProposalLane,
        ProposalRecord, ProposalScores, ProposalTaxonomyMutation,
    },
    quality::assess_quality,
    reconcile::{ReconcileDecision, RoutingAnalysis},
    wiki_fs::{parse_wiki_page, update_frontmatter},
    wiki_index::{append_log, rebuild_index_md},
    PageStatus,
};

#[derive(Debug, Clone, serde::Serialize)]
struct HierarchyContextEntry {
    path: String,
    title: String,
    summary: String,
}

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

    let taxonomy = load_taxonomy(wiki_dir)?;
    let northstar_md = std::fs::read_to_string(wiki_dir.join("_config/northstar.md")).unwrap_or_default();

    // Load root index and recursively gather hierarchy context for efficient structure-first routing.
    let index_md = crate::wiki_index::read_index_md(wiki_dir).unwrap_or_default();
    let hierarchy_context = collect_hierarchy_context(wiki_dir)?;

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
        "taxonomy": taxonomy,
        "northstar_context": northstar_md,
        "index_summary": index_md,
        "hierarchy_context": hierarchy_context,
        "pages": pages,
        "instructions": {
            "task": "Turn each intake page into a hierarchy-first proposal with a defensible full path, overlap assessment, and review/staged lane.",
                "output_format": "JSON array of routing decisions. Each element: {\"slug\": \"...\", \"category\": [\"tree\", \"subtree\", \"optional-deeper-node\"], \"confidence\": 0.0-1.0, \"status\": \"staged|review\", \"keywords\": [...], \"summary\": \"max 200 chars\", \"rationale\": \"...\", \"alternatives_considered\": [{\"path\": [...], \"score\": 0.0, \"ruled_out_because\": \"...\"}], \"review_reason\": null, \"proposed_new_subtree\": null, \"proposal_rationale\": null, \"merge_target\": null}",
            "confidence_rule": "confidence >= 0.75 and a valid existing subtree fit is necessary but not sufficient for staged. Weak or low-usability content should still go to review.",
            "quality_rule": "Assess information quality and human usability separately from routing confidence. Low-signal, placeholder-like, or weak-content pages must go to review even if the route is obvious.",
            "hierarchy_rule": "Hierarchy is the primary design goal. Use the taxonomy plus recursive branch index context to find the best full path. Do not stop at the first acceptable shallow match if the information clearly implies a deeper structure.",
            "recursive_index_rule": "Use hierarchy_context and branch indexes religiously. Keep traversing likely branch paths and nearby peers until you believe you have found the relevant surrounding structure and overlaps.",
            "new_subtree_rule": "If no existing subtree fits confidently, do not force publication. Route to review, provide the closest category path you can justify, and fill proposed_new_subtree plus proposal_rationale.",
            "overlap_rule": "If the material appears semantically duplicative with likely peers, prefer review with merge or consolidation guidance instead of staged.",
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

            match apply_routing(config, &src_path, slug, decision.clone(), Some(analysis), false) {
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

fn collect_hierarchy_context(wiki_dir: &Path) -> Result<Vec<HierarchyContextEntry>> {
    let mut entries = Vec::new();
    let published_dir = wiki_dir.join("published");
    if !published_dir.exists() {
        return Ok(entries);
    }
    for entry in walkdir::WalkDir::new(&published_dir).into_iter().filter_map(|entry| entry.ok()) {
        let path = entry.path();
        if !entry.file_type().is_file()
            || path.file_name().and_then(|name| name.to_str()) != Some("index.md")
        {
            continue;
        }
        let raw = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        let rel = path.strip_prefix(wiki_dir).unwrap_or(path).to_string_lossy().replace('\\', "/");
        let title = raw
            .lines()
            .find(|line| line.starts_with('#'))
            .map(|line| line.trim_start_matches('#').trim().to_string())
            .unwrap_or_else(|| rel.clone());
        let summary = raw
            .lines()
            .filter(|line| !line.trim().is_empty())
            .skip(1)
            .take(6)
            .collect::<Vec<_>>()
            .join(" ");
        entries.push(HierarchyContextEntry { path: rel, title, summary });
    }
    entries.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(entries)
}

fn apply_routing(
    config: &Config,
    src_path: &Path,
    slug: &str,
    decision: ReconcileDecision,
    analysis: Option<RoutingAnalysis>,
    _dry_run: bool,
) -> Result<()> {
    let wiki_dir = &config.wiki.wiki_dir;
    let mut decision = decision;
    if decision.category.is_empty() {
        anyhow::bail!("Routing decision for '{}' is missing a category path", slug);
    }
    let taxonomy = load_taxonomy(wiki_dir)?;
    let mut analysis = analysis;
    let mut page = parse_wiki_page(src_path)?;
    let quality = assess_quality(&page.frontmatter.title, &page.body);
    let overlap_candidates = find_peer_overlap(wiki_dir, &decision.category, &page.frontmatter.title, &page.body, Some(slug))?;
    let overlap_risk = overlap_candidates.first().map(|candidate| candidate.score).unwrap_or(0.0);
    let mut target_status = if decision.status == "staged" { "staged" } else { "review" };
    let taxonomy_has_path = taxonomy_path_exists(&taxonomy, &decision.category);
    if !taxonomy_has_path && decision.proposed_new_subtree.is_none() {
        anyhow::bail!(
            "Routing decision for '{}' uses invalid taxonomy path '{}'. Attach a new-node proposal or move it to review.",
            slug,
            decision.category.join("/")
        );
    }
    let required_lane = required_lane(
        decision.confidence,
        quality.information_quality,
        if taxonomy_has_path { decision.confidence } else { 0.5 },
        overlap_risk,
        !taxonomy_has_path,
        decision.review_reason.is_some(),
    );
    if required_lane == ProposalLane::Review {
        target_status = "review";
        decision.status = "review".to_string();
        let fallback_reason = if overlap_risk >= 0.7 {
            format!(
                "High semantic overlap ({:.0}%) with existing peer content — requires review for merge or consolidation",
                overlap_risk * 100.0
            )
        } else if !taxonomy_has_path {
            "Proposed taxonomy path does not exist yet — requires review with a taxonomy mutation".to_string()
        } else {
            format!(
                "Low information quality ({:.0}%) or usability ({:.0}%) — requires review before publication",
                quality.information_quality * 100.0,
                quality.usability * 100.0
            )
        };
        if decision.review_reason.is_none() {
            decision.review_reason = Some(fallback_reason.clone());
        }
        if let Some(ref mut sidecar) = analysis {
            sidecar.routing.review_reason = Some(
                sidecar.routing.review_reason.clone().unwrap_or(fallback_reason)
            );
            sidecar.routing.information_quality = Some(quality.information_quality);
            sidecar.routing.usability = Some(quality.usability);
            sidecar.routing.flags.extend(quality.flags.clone());
            sidecar.routing.flags.extend(overlap_candidates.iter().take(3).map(|candidate| format!("overlap:{}", candidate.path)));
        }
    } else if let Some(ref mut sidecar) = analysis {
        sidecar.routing.information_quality = Some(quality.information_quality);
        sidecar.routing.usability = Some(quality.usability);
        sidecar.routing.flags.extend(quality.flags.clone());
        sidecar.routing.flags.extend(overlap_candidates.iter().take(3).map(|candidate| format!("overlap:{}", candidate.path)));
    }
    let cat_path: PathBuf = decision.category.iter().collect();
    let dest_dir = wiki_dir.join(target_status).join(&cat_path);
    let dest_path = dest_dir.join(format!("{}.md", slug));

    std::fs::create_dir_all(&dest_dir)?;

    let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    page.frontmatter.status = if target_status == "staged" { PageStatus::Staged } else { PageStatus::Review };
    page.frontmatter.category = decision.category.clone();
    page.frontmatter.keywords = decision.keywords.clone();
    page.frontmatter.confidence = Some(decision.confidence);
    page.frontmatter.updated_at = now;
    page.frontmatter.model_used = Some(decision.model_used.clone());

    update_frontmatter(src_path, &page.frontmatter)?;

    let proposal = build_proposal_record(
        slug,
        &page.frontmatter.title,
        &page.body,
        &decision,
        target_status,
        quality.information_quality,
        quality.usability,
        overlap_risk,
        overlap_candidates.iter().map(|candidate| candidate.path.clone()).collect(),
        &page.frontmatter.source,
    );

    // Write analysis sidecar before git mv
    if let Some(ref a) = analysis {
        write_analysis_sidecar(src_path, a)?;
    }
    save_proposal_record(src_path, &proposal)?;

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
    let proposal_src = crate::proposal::proposal_sidecar_path(src_path);
    if proposal_src.exists() {
        let proposal_dest = crate::proposal::proposal_sidecar_path(&dest_path);
        let rel_psrc = proposal_src.strip_prefix(repo_root).unwrap_or(&proposal_src);
        let rel_pdest = proposal_dest.strip_prefix(repo_root).unwrap_or(&proposal_dest);
        crate::git_ops::git_mv(repo_root, rel_psrc, rel_pdest)?;
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
            information_quality: None,
            usability: None,
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

fn build_proposal_record(
    slug: &str,
    title: &str,
    body: &str,
    decision: &ReconcileDecision,
    target_status: &str,
    information_quality: f32,
    usability: f32,
    overlap_risk: f32,
    overlap_candidates: Vec<String>,
    source: &crate::SourceRef,
) -> ProposalRecord {
    let lane = if target_status == "staged" {
        ProposalLane::Staged
    } else {
        ProposalLane::Review
    };
    let kind = if decision.proposed_new_subtree.is_some() {
        ProposalKind::TaxonomyChange
    } else if decision.merge_target.is_some() || overlap_risk >= 0.7 {
        ProposalKind::Merge
    } else {
        ProposalKind::NewPage
    };
    let taxonomy_mutation = decision.proposed_new_subtree.as_ref().map(|slug_value| ProposalTaxonomyMutation {
        proposed_parent_path: decision.category.clone(),
        proposed_node_title: slug_value.replace('-', " "),
        proposed_node_slug: slug_value.clone(),
        node_description: decision.proposal_rationale.clone().unwrap_or_default(),
        rationale: decision.proposal_rationale.clone().unwrap_or_default(),
        rejected_nearby_nodes: vec![],
    });
    ProposalRecord {
        schema_version: 1,
        proposal_id: format!("proposal-{}", slug),
        generated_at: Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        lane,
        kind,
        subject_slug: slug.to_string(),
        title: title.to_string(),
        target_path: decision.category.clone(),
        summary: decision.summary.clone(),
        body_markdown: body.to_string(),
        recommended_action: if target_status == "staged" {
            "stage for approval".to_string()
        } else {
            "review before further action".to_string()
        },
        scores: ProposalScores {
            route_confidence: decision.confidence,
            quality_confidence: information_quality,
            hierarchy_fit_confidence: if taxonomy_mutation.is_some() { 0.5 } else { decision.confidence },
            overlap_risk,
            evidence_completeness: 0.7,
            usability,
            freshness_confidence: 1.0,
        },
        review_reason: decision.review_reason.clone(),
        merge_target: decision.merge_target.clone(),
        taxonomy_mutation,
        dossier: ProposalDossier {
            source_ids: vec![source.id.clone()],
            source_locations: source.origin_url.clone().into_iter().collect(),
            fetched_artifacts: vec![source.kind.clone()],
            compared_pages: overlap_candidates.clone(),
            alternatives_considered: decision
                .category
                .iter()
                .take(1)
                .map(|segment| format!("Primary route candidate: {}", segment))
                .collect(),
            unresolved_questions: decision.review_reason.clone().into_iter().collect(),
            overlap_candidates,
            rationale: decision.proposal_rationale.clone().unwrap_or_default(),
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
