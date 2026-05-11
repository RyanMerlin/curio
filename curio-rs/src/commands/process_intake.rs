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
///   `curio process --slug <s> --category product-tree/example-server --status staged`
use anyhow::{Context, Result};
use chrono::Utc;
use std::path::{Path, PathBuf};

use crate::{
    PageStatus,
    config::Config,
    northstar::{load_taxonomy, taxonomy_path_exists},
    output::emit_json,
    overlap::find_peer_overlap,
    proposal::{
        ProposalDossier, ProposalKind, ProposalLane, ProposalRecord, ProposalScores,
        ProposalTaxonomyMutation, required_lane, save_proposal_record,
    },
    quality::assess_quality,
    reconcile::{ReconcileDecision, RoutingAnalysis},
    wiki_fs::generate_id,
    wiki_fs::{parse_wiki_page, update_frontmatter},
    wiki_index::{append_log, rebuild_index_md},
};

#[derive(Debug, Clone, serde::Serialize)]
struct HierarchyContextEntry {
    path: String,
    title: String,
    summary: String,
    /// Up to 5 peer leaf pages directly under this branch. Each carries
    /// title + short summary + keywords so the agent has real signal when
    /// judging hierarchy fit, not just branch labels.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    peer_pages: Vec<PeerPageSnippet>,
}

#[derive(Debug, Clone, serde::Serialize)]
struct PeerPageSnippet {
    path: String,
    title: String,
    summary: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    keywords: Vec<String>,
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
            let _ = emit_json(
                "process",
                true,
                &serde_json::json!({ "processed": 0, "message": msg }),
            );
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
        let decisions: Vec<(String, ReconcileDecision)> = serde_json::from_str(&raw).context(
            "Failed to parse route file — expected [{\"slug\": ..., \"decision\": {...}}, ...]",
        )?;

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
    let northstar_md = crate::northstar::read_northstar_markdown(wiki_dir).unwrap_or_default();
    let workspace_config_yaml =
        std::fs::read_to_string(crate::northstar::workspace_config_path(wiki_dir))
            .unwrap_or_default();

    // Load root index and recursively gather hierarchy context for efficient structure-first routing.
    let index_md = crate::wiki_index::read_index_md(wiki_dir).unwrap_or_default();
    let mut hierarchy_context = collect_hierarchy_context(wiki_dir)?;

    let pages: Vec<serde_json::Value> = intake_pages
        .iter()
        .map(|(slug, page)| {
            // Include reviewer feedback if a .feedback.md sidecar exists for this page
            let feedback = page.path.with_extension("feedback.md");
            let reviewer_feedback = if feedback.exists() {
                std::fs::read_to_string(&feedback).ok()
            } else {
                None
            };
            let mut entry = serde_json::json!({
                "slug": slug,
                "title": page.frontmatter.title,
                "source_url": page.frontmatter.source.origin_url,
                "content_hash": page.frontmatter.content_hash,
                "body_preview": page.body.chars().take(1000).collect::<String>(),
            });
            if let Some(fb) = reviewer_feedback {
                entry["reviewer_feedback"] = serde_json::json!(fb);
            }
            entry
        })
        .collect();

    // Apply byte budget — colleagues' KBs may grow large enough to blow
    // typical agent context windows. Default budget is read from
    // CURIO_MANIFEST_BUDGET_KB if set (KB), otherwise 64 KB.
    let budget_bytes = std::env::var("CURIO_MANIFEST_BUDGET_KB")
        .ok()
        .and_then(|v| v.trim().parse::<usize>().ok())
        .filter(|v| *v > 0)
        .unwrap_or(64)
        * 1024;
    // Approximate the static overhead of the rest of the manifest (taxonomy,
    // instructions, northstar, etc) so the budget guard accounts for it.
    let other_size = serde_json::to_string(&taxonomy)
        .map(|s| s.len())
        .unwrap_or(0)
        + northstar_md.len()
        + workspace_config_yaml.len()
        + index_md.len()
        + 4096; // fudge for instructions block + JSON envelope chars
    let (truncated, dropped_peer_pages) =
        enforce_manifest_budget(&mut hierarchy_context, &pages, other_size, budget_bytes);

    // T2-C: group intake pages by intake_request_id so the agent sees
    // each `curio intake` invocation as a UNIT and can decide whether
    // to merge N sources into 1 proposal, split 1 source into N, or
    // produce 1:1 proposals. Pages without a request_id (older intake
    // runs predating T2-C) fall into the "_legacy" group, treated as
    // independent 1:1 sources.
    let mut requests_map: std::collections::BTreeMap<String, Vec<&str>> =
        std::collections::BTreeMap::new();
    for (slug, page) in intake_pages {
        let key = page
            .frontmatter
            .intake_request_id
            .clone()
            .unwrap_or_else(|| "_legacy".to_string());
        requests_map.entry(key).or_default().push(slug.as_str());
    }
    let intake_requests: Vec<serde_json::Value> = requests_map
        .into_iter()
        .map(|(rid, slugs)| {
            serde_json::json!({
                "request_id": rid,
                "page_count": slugs.len(),
                "page_slugs": slugs,
            })
        })
        .collect();

    let manifest = serde_json::json!({
        "schema_version": 2,
        "action": "route_intake_pages",
        "page_count": pages.len(),
        "manifest_budget_bytes": budget_bytes,
        "truncated": truncated,
        "dropped_peer_pages": dropped_peer_pages,
        "taxonomy": taxonomy,
        "northstar_context": northstar_md,
        "workspace_config_yaml": workspace_config_yaml,
        "index_summary": index_md,
        "hierarchy_context": hierarchy_context,
        "intake_requests": intake_requests,
        "pages": pages,
        "instructions": {
            "task": "Turn each intake page into a hierarchy-first proposal with a defensible FULL PATH, overlap assessment, and review/staged lane. You are the information architect — not a page router.",
            "output_format": "JSON array of routing decisions. Each element: {\"slug\": \"...\", \"category\": [\"tree\", \"subtree\", \"optional-deeper-node\", \"optional-leaf-node\"], \"confidence\": 0.0-1.0, \"status\": \"staged|review\", \"keywords\": [...], \"summary\": \"max 200 chars\", \"rationale\": \"...\", \"alternatives_considered\": [{\"path\": [...], \"score\": 0.0, \"ruled_out_because\": \"...\"}], \"review_reason\": null, \"proposed_new_subtree\": null, \"proposal_rationale\": null, \"merge_target\": null}",
            "confidence_rule": "confidence >= 0.75 and a valid existing subtree fit is necessary but not sufficient for staged. Weak or low-usability content should still go to review.",
            "quality_rule": "Assess information quality and human usability separately from routing confidence. Low-signal, placeholder-like, hub-only, or weak-content pages must go to review even if the route is obvious.",
            "hierarchy_rule": "Hierarchy is the PRIMARY design goal. Use the taxonomy plus recursive branch index context to propose the DEEPEST defensible path, not the first acceptable shallow match. A 1-component category (tree only) is almost never correct for leaf content — it means you stopped too early.",
            "depth_rule": "Default to paths with 3+ components for technical, operational, troubleshooting, version-specific, scenario-specific, or procedural content. Only use a 1- or 2-component path when the content is explicitly broad, cross-cutting, or designed as a branch landing page. If you choose a shallow path for technical content, you MUST explicitly rule out deeper alternatives in alternatives_considered.",
            "recursive_index_rule": "Traverse hierarchy_context exhaustively. Read every index.md summary in the relevant branch neighborhood. Do not propose a path until you have looked one level deeper than you think is needed.",
            "new_subtree_rule": "If the ideal path does not yet exist in the taxonomy, route to review, use the closest justified path, and fill proposed_new_subtree (the new node slug) plus proposal_rationale (why this node is necessary and where it belongs). Do NOT force content into a wrong existing node just to avoid a new-subtree proposal.",
            "overlap_rule": "If the material is semantically duplicative with a likely peer, set merge_target to the peer path, route to review, and explain in review_reason what a merge would produce. Do not publish a duplicate.",
            "hub_page_rule": "Hub/index pages (those whose body is mainly child-page lists) should be routed as branch-node proposals, not leaf pages. Their category should point to the branch they organize. Mark them review unless the branch is already well-defined.",
            "body_rewrite_rule": "Each routing decision MAY include `proposed_body_markdown` (an agent-authored rewrite of the page body) and `decision_section_markdown` (a structured pre-amble with rationale, scores, route, alternatives, recommended action). Use these to ship a curated knowledge object instead of a raw capture. Set `body_rewrite_kind` to one of `none` | `light_edit` | `full_synthesis` so the dossier records what you did. Default expectation: full_synthesis for staged proposals; light_edit or none for review proposals where the body shape is still up for debate.",
            "multi_source_rule": "The `intake_requests` block groups intake pages by the single `curio intake` invocation that produced them. An intake REQUEST is the editorial unit, not a single page. For each request with >1 source, decide explicitly: (a) merge multiple sources into ONE proposal (set `merge_into_slug` on the secondary pages, pointing at the primary slug; the primary keeps a normal decision), (b) split one source into N proposals (only when the source clearly contains separable knowledge — produce N normal decisions, mark the original as merged-into-first), or (c) keep 1:1 routing when sources are independent. Sources sharing a request_id are very likely related; default toward merge when the topic family overlaps.",
            "decision_section_template": "Suggested structure (markdown): a `## Curation Decision` heading followed by a bullet list of route, scores (route/quality/hierarchy_fit/overlap_risk/usability), recommended action, top alternatives considered, and any merge target or taxonomy mutation. Place ABOVE the proposed body so reviewers see editorial rationale before content.",
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
    let mut merged_into_count: usize = 0;

    // T2-C pre-pass: process merge_into_slug directives first. Each
    // matched pair appends the secondary's body to the primary intake
    // page under a heading and removes the secondary. The primary then
    // continues through normal routing carrying the consolidated content.
    if !dry_run {
        let mut merge_directives: Vec<(String, String)> = Vec::new();
        for (slug, decision) in &decisions {
            if let Some(target) = decision.merge_into_slug.as_deref() {
                if target == slug {
                    eprintln!("  warning: '{}' cannot merge into itself — ignoring", slug);
                    continue;
                }
                merge_directives.push((slug.clone(), target.to_string()));
            }
        }
        for (secondary_slug, primary_slug) in &merge_directives {
            let primary_path = intake_dir.join(format!("{}.md", primary_slug));
            let secondary_path = intake_dir.join(format!("{}.md", secondary_slug));
            if !primary_path.exists() {
                errors.push(format!(
                    "{}: merge target '{}' not in intake/",
                    secondary_slug, primary_slug
                ));
                continue;
            }
            if !secondary_path.exists() {
                errors.push(format!(
                    "{}: secondary not in intake/ (already moved?)",
                    secondary_slug
                ));
                continue;
            }
            // Append secondary body to primary under a clear heading.
            let primary = match crate::wiki_fs::parse_wiki_page(&primary_path) {
                Ok(p) => p,
                Err(e) => {
                    errors.push(format!("{}: parse primary failed: {}", secondary_slug, e));
                    continue;
                }
            };
            let secondary = match crate::wiki_fs::parse_wiki_page(&secondary_path) {
                Ok(p) => p,
                Err(e) => {
                    errors.push(format!("{}: parse secondary failed: {}", secondary_slug, e));
                    continue;
                }
            };
            let merged_body = format!(
                "{}\n\n## Merged source: {} ({})\n\n{}",
                primary.body.trim_end(),
                secondary.frontmatter.title,
                secondary_slug,
                secondary.body.trim_start()
            );
            let mut updated_primary = primary.clone();
            updated_primary.body = merged_body;
            if let Err(e) = crate::wiki_fs::write_wiki_page(&primary_path, &updated_primary) {
                errors.push(format!(
                    "{}: write merged primary failed: {}",
                    secondary_slug, e
                ));
                continue;
            }
            // Track merged-source provenance for the eventual dossier.
            let provenance_path = primary_path.with_extension("md.merged-sources.json");
            let mut prov: serde_json::Value = if provenance_path.exists() {
                std::fs::read_to_string(&provenance_path)
                    .ok()
                    .and_then(|s| serde_json::from_str(&s).ok())
                    .unwrap_or(serde_json::json!({"merged_sources": []}))
            } else {
                serde_json::json!({"merged_sources": []})
            };
            if let Some(arr) = prov["merged_sources"].as_array_mut() {
                arr.push(serde_json::json!({
                    "slug": secondary_slug,
                    "title": secondary.frontmatter.title,
                    "source_id": secondary.frontmatter.source.id,
                    "source_url": secondary.frontmatter.source.origin_url,
                    "request_id": secondary.frontmatter.intake_request_id,
                }));
            }
            let _ = std::fs::write(
                &provenance_path,
                serde_json::to_string_pretty(&prov).unwrap_or_default(),
            );
            // Remove the secondary from intake (and from disk).
            if let Err(e) = std::fs::remove_file(&secondary_path) {
                errors.push(format!(
                    "{}: remove secondary failed: {}",
                    secondary_slug, e
                ));
                continue;
            }
            processed.push(serde_json::json!({
                "slug": secondary_slug,
                "title": &secondary.frontmatter.title,
                "merged_into": primary_slug,
            }));
            merged_into_count += 1;
        }
    }

    for (slug, page) in intake_pages {
        let decision = match decisions.iter().find(|(s, _)| s == slug) {
            Some((_, d)) => d.clone(),
            None => {
                eprintln!("  warning: no decision for '{}' — skipping", slug);
                continue;
            }
        };

        // Skip secondaries — they've already been folded into their primary.
        if decision.merge_into_slug.is_some() {
            continue;
        }

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
            let analysis = build_analysis_from_decision(
                &decision,
                &page.frontmatter.title,
                &page.body,
                page.frontmatter.source.origin_url.as_deref(),
                &page.frontmatter.content_hash,
                &config.products,
            );

            match apply_routing(
                config,
                &src_path,
                slug,
                decision.clone(),
                Some(analysis),
                false,
            ) {
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
            if let Some(target) = item["merged_into"].as_str() {
                println!("  {} ▸ merged into '{}'", title, target);
                continue;
            }
            let to = item["routed_to"]
                .as_str()
                .or_else(|| item["would_route_to"].as_str())
                .unwrap_or("?");
            let cat = item["category"]
                .as_array()
                .map(|a| {
                    a.iter()
                        .map(|v| v.as_str().unwrap_or(""))
                        .collect::<Vec<_>>()
                        .join("/")
                })
                .unwrap_or_default();
            let conf = item["confidence"]
                .as_u64()
                .map(|c| format!(" [{c}%]"))
                .unwrap_or_default();
            println!("  {} → {}/{}{}", title, to, cat, conf);
        }
        for e in &errors {
            eprintln!("  error: {}", e);
        }
        let merge_note = if merged_into_count > 0 {
            format!(" ({merged_into_count} merged into primary)")
        } else {
            String::new()
        };
        println!(
            "{} processed{}, {} errors{}",
            processed.len(),
            merge_note,
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

/// Maximum number of peer leaf pages surfaced per branch in the manifest.
/// Picked so the agent gets enough signal to judge hierarchy fit but the
/// manifest stays under typical context budgets even for wide branches.
const PEERS_PER_BRANCH: usize = 5;
/// Default cap on peer-page summary length (in chars). Trimmed body lines.
const PEER_SUMMARY_CHARS: usize = 240;

fn collect_hierarchy_context(wiki_dir: &Path) -> Result<Vec<HierarchyContextEntry>> {
    let mut entries = Vec::new();
    let published_dir = wiki_dir.join("published");
    if !published_dir.exists() {
        return Ok(entries);
    }
    for entry in walkdir::WalkDir::new(&published_dir)
        .into_iter()
        .filter_map(|entry| entry.ok())
    {
        let path = entry.path();
        if !entry.file_type().is_file()
            || path.file_name().and_then(|name| name.to_str()) != Some("index.md")
        {
            continue;
        }
        let raw = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        let rel = path
            .strip_prefix(wiki_dir)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/");
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

        let branch_dir = path.parent().unwrap_or(path);
        let peer_pages = collect_peer_pages(wiki_dir, branch_dir);

        entries.push(HierarchyContextEntry {
            path: rel,
            title,
            summary,
            peer_pages,
        });
    }
    entries.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(entries)
}

/// Collect up to `PEERS_PER_BRANCH` leaf .md files in `branch_dir` (NOT
/// recursive — descendants are covered by their own index.md entries).
/// Each peer carries title + a short body summary + keywords, all from
/// the parsed frontmatter.
fn collect_peer_pages(wiki_dir: &Path, branch_dir: &Path) -> Vec<PeerPageSnippet> {
    let mut peers = Vec::new();
    let read_dir = match std::fs::read_dir(branch_dir) {
        Ok(d) => d,
        Err(_) => return peers,
    };
    for entry in read_dir.flatten() {
        if peers.len() >= PEERS_PER_BRANCH {
            break;
        }
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let fname = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n,
            None => continue,
        };
        // Skip non-md and the branch index itself.
        if !fname.ends_with(".md") || fname == "index.md" {
            continue;
        }
        // Best-effort parse — non-frontmatter pages are skipped silently.
        let page = match crate::wiki_fs::parse_wiki_page(&path) {
            Ok(p) => p,
            Err(_) => continue,
        };
        let summary: String = page
            .body
            .chars()
            .take(PEER_SUMMARY_CHARS)
            .collect::<String>()
            .replace(['\n', '\r'], " ");
        let rel = path
            .strip_prefix(wiki_dir)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        peers.push(PeerPageSnippet {
            path: rel,
            title: page.frontmatter.title,
            summary,
            keywords: page.frontmatter.keywords,
        });
    }
    peers
}

/// Apply a hard byte budget to the manifest by progressively dropping
/// peer_pages from the largest branches first. Sets `truncated=true` and
/// records how many peer entries were removed in `dropped_peer_pages`.
/// Returns (truncated, dropped_count).
fn enforce_manifest_budget(
    hierarchy: &mut [HierarchyContextEntry],
    pages: &[serde_json::Value],
    other_size: usize,
    budget_bytes: usize,
) -> (bool, usize) {
    fn estimate_size(hierarchy: &[HierarchyContextEntry]) -> usize {
        serde_json::to_string(hierarchy)
            .map(|s| s.len())
            .unwrap_or(0)
    }
    let pages_size = serde_json::to_string(pages).map(|s| s.len()).unwrap_or(0);
    let mut current = estimate_size(hierarchy) + pages_size + other_size;
    if current <= budget_bytes {
        return (false, 0);
    }

    let mut dropped = 0usize;
    // Pop one peer at a time from the entry currently carrying the most.
    loop {
        if current <= budget_bytes {
            return (true, dropped);
        }
        // Find branch with the most peers; tie-break by largest serialized size.
        let target = hierarchy
            .iter_mut()
            .filter(|e| !e.peer_pages.is_empty())
            .max_by_key(|e| e.peer_pages.len());
        let Some(target) = target else {
            return (true, dropped);
        };
        target.peer_pages.pop();
        dropped += 1;
        current = estimate_size(hierarchy) + pages_size + other_size;
    }
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

    // T2-A: when the agent supplies a rewrite, the agent-authored body IS
    // the proposed page. Score quality and overlap against THAT body, not
    // the raw intake — otherwise a clean rewrite of a thin capture would be
    // unfairly demoted to review by the original body's quality flags.
    let scoring_body: String = match decision.proposed_body_markdown.as_deref() {
        Some(b) if !b.trim().is_empty() => b.to_string(),
        _ => page.body.clone(),
    };
    let quality = assess_quality(&page.frontmatter.title, &scoring_body);
    let overlap_candidates = find_peer_overlap(
        wiki_dir,
        &decision.category,
        &page.frontmatter.title,
        &scoring_body,
        Some(slug),
    )?;
    let overlap_risk = overlap_candidates
        .first()
        .map(|candidate| candidate.score)
        .unwrap_or(0.0);
    let mut target_status = if decision.status == "staged" {
        "staged"
    } else {
        "review"
    };
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
        if taxonomy_has_path {
            decision.confidence
        } else {
            0.5
        },
        overlap_risk,
        !taxonomy_has_path,
        decision.review_reason.is_some(),
    );
    if required_lane == ProposalLane::Review {
        target_status = "review";
        decision.status = "review".to_string();

        // Auto-populate merge_target from the top overlap candidate when the agent
        // didn't provide one explicitly. This gives reviewers an actionable pointer.
        if overlap_risk >= 0.7 && decision.merge_target.is_none() {
            if let Some(top_candidate) = overlap_candidates.first() {
                decision.merge_target = Some(top_candidate.path.clone());
            }
        }

        let fallback_reason = if overlap_risk >= 0.7 {
            format!(
                "High semantic overlap ({:.0}%) with existing peer content at '{}' — requires review for merge or consolidation",
                overlap_risk * 100.0,
                decision.merge_target.as_deref().unwrap_or("unknown peer")
            )
        } else if !taxonomy_has_path {
            "Proposed taxonomy path does not exist yet — requires review with a taxonomy mutation"
                .to_string()
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
                sidecar
                    .routing
                    .review_reason
                    .clone()
                    .unwrap_or(fallback_reason),
            );
            sidecar.routing.information_quality = Some(quality.information_quality);
            sidecar.routing.usability = Some(quality.usability);
            sidecar.routing.flags.extend(quality.flags.clone());
            sidecar.routing.flags.extend(
                overlap_candidates
                    .iter()
                    .take(3)
                    .map(|candidate| format!("overlap:{}", candidate.path)),
            );
        }
    } else if let Some(ref mut sidecar) = analysis {
        sidecar.routing.information_quality = Some(quality.information_quality);
        sidecar.routing.usability = Some(quality.usability);
        sidecar.routing.flags.extend(quality.flags.clone());
        sidecar.routing.flags.extend(
            overlap_candidates
                .iter()
                .take(3)
                .map(|candidate| format!("overlap:{}", candidate.path)),
        );
    }

    // Shallow-route guard: if a leaf page was routed to a single-component category
    // (tree-level only) and the content signals technical/operational depth, demote to
    // review so a human can verify the path is intentional, not premature.
    if target_status == "staged" && decision.category.len() == 1 {
        let body_lower = page.body.to_lowercase();
        let title_lower = page.frontmatter.title.to_lowercase();
        let technical_signals = [
            "troubleshoot",
            "install",
            "configur",
            "version",
            "upgrade",
            "migrat",
            "error",
            "debug",
            "procedure",
            "step-by-step",
            "how to",
            "howto",
        ];
        let is_technical = technical_signals
            .iter()
            .any(|sig| title_lower.contains(sig) || body_lower.contains(sig));
        if is_technical {
            target_status = "review";
            decision.status = "review".to_string();
            let shallow_reason = format!(
                "Shallow-route warning: content appears technical/operational but was routed to a top-level category '{}' with no subtree. \
                Verify this is the correct final depth or propose a deeper path.",
                decision.category[0]
            );
            if decision.review_reason.is_none() {
                decision.review_reason = Some(shallow_reason.clone());
            }
            if let Some(ref mut sidecar) = analysis {
                sidecar
                    .routing
                    .flags
                    .push("shallow_route_warning".to_string());
                if sidecar.routing.review_reason.is_none() {
                    sidecar.routing.review_reason = Some(shallow_reason);
                }
            }
        }
    }

    let cat_path: PathBuf = decision.category.iter().collect();
    let dest_dir = wiki_dir.join(target_status).join(&cat_path);
    let dest_path = dest_dir.join(format!("{}.md", slug));

    std::fs::create_dir_all(&dest_dir)?;

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

    // Phase 2 page-body rewriting (T2-A): if the agent supplied a rewritten
    // body and/or a structured decision section in the route file, compose
    // the new page body and rewrite the file. Backwards-compatible: when
    // neither is set, only the frontmatter is updated, preserving older
    // route-file behavior.
    let body_rewrite_kind = decision.body_rewrite_kind.clone().unwrap_or_else(|| {
        if decision.proposed_body_markdown.is_some() {
            "full_synthesis".to_string()
        } else {
            "none".to_string()
        }
    });
    let decision_section_present = decision
        .decision_section_markdown
        .as_deref()
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false);
    let effective_body = if decision.proposed_body_markdown.is_some() || decision_section_present {
        let prelude = decision
            .decision_section_markdown
            .as_deref()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| format!("{}\n\n", s))
            .unwrap_or_default();
        let body = decision
            .proposed_body_markdown
            .as_deref()
            .unwrap_or(&page.body)
            .to_string();
        format!("{}{}", prelude, body)
    } else {
        page.body.clone()
    };

    if decision.proposed_body_markdown.is_some() || decision_section_present {
        // Atomically rewrite frontmatter + body together.
        page.body = effective_body.clone();
        crate::wiki_fs::write_wiki_page(src_path, &page)?;
    } else {
        update_frontmatter(src_path, &page.frontmatter)?;
    }

    let mut proposal = build_proposal_record(
        slug,
        &page.frontmatter.title,
        &effective_body,
        &decision,
        target_status,
        quality.information_quality,
        quality.usability,
        overlap_risk,
        overlap_candidates
            .iter()
            .map(|candidate| candidate.path.clone())
            .collect(),
        &page.frontmatter.source,
    );
    // Record what the agent did to the body, for audit and later
    // sharpening signal.
    proposal.dossier.body_rewrite_kind = Some(body_rewrite_kind);
    proposal.dossier.decision_section_present = decision_section_present;

    // T2-C: if a merged-sources provenance sidecar exists alongside the
    // intake page (left by the merge_into_slug pre-pass), fold its source
    // ids / urls into the dossier so the proposal records ALL contributing
    // sources, not just the primary's. Then remove the sidecar — it's
    // ephemeral provenance for this single apply.
    let merge_provenance_path = src_path.with_extension("md.merged-sources.json");
    if merge_provenance_path.exists() {
        if let Ok(raw) = std::fs::read_to_string(&merge_provenance_path) {
            if let Ok(prov) = serde_json::from_str::<serde_json::Value>(&raw) {
                if let Some(arr) = prov["merged_sources"].as_array() {
                    for src in arr {
                        if let Some(id) = src["source_id"].as_str() {
                            proposal.dossier.source_ids.push(id.to_string());
                        }
                        if let Some(url) = src["source_url"].as_str() {
                            proposal.dossier.source_locations.push(url.to_string());
                        }
                    }
                    if !arr.is_empty() {
                        proposal.kind = crate::proposal::ProposalKind::Consolidation;
                        let merged_titles: Vec<String> = arr
                            .iter()
                            .filter_map(|s| s["title"].as_str().map(String::from))
                            .collect();
                        if !merged_titles.is_empty() {
                            proposal.dossier.alternatives_considered.insert(
                                0,
                                format!(
                                    "Consolidated {} merged source(s) per agent decision: {}",
                                    merged_titles.len(),
                                    merged_titles.join("; ")
                                ),
                            );
                        }
                    }
                }
            }
        }
        let _ = std::fs::remove_file(&merge_provenance_path);
    }

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
        let rel_asrc = analysis_src
            .strip_prefix(repo_root)
            .unwrap_or(&analysis_src);
        let rel_adest = analysis_dest
            .strip_prefix(repo_root)
            .unwrap_or(&analysis_dest);
        crate::git_ops::git_mv(repo_root, rel_asrc, rel_adest)?;
    }
    let proposal_src = crate::proposal::proposal_sidecar_path(src_path);
    if proposal_src.exists() {
        let proposal_dest = crate::proposal::proposal_sidecar_path(&dest_path);
        let rel_psrc = proposal_src
            .strip_prefix(repo_root)
            .unwrap_or(&proposal_src);
        let rel_pdest = proposal_dest
            .strip_prefix(repo_root)
            .unwrap_or(&proposal_dest);
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
    products: &[crate::config::ProductDefinition],
) -> RoutingAnalysis {
    use crate::reconcile::{AnalysisInputs, AnalysisRouting, AnalysisSignals};

    let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let content_preview: String = body.chars().take(500).collect();
    let title_tokens = title.split_whitespace().map(|w| w.to_lowercase()).collect();
    let keywords_extracted = crate::reconcile::extract_keywords(&format!("{} {}", title, body), 8);
    let pre_signal = crate::reconcile::heuristic_pre_signal(title, body, products);

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
    let taxonomy_mutation =
        decision
            .proposed_new_subtree
            .as_ref()
            .map(|slug_value| ProposalTaxonomyMutation {
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
        } else if decision.merge_target.is_some() {
            format!(
                "review for merge into '{}'",
                decision.merge_target.as_deref().unwrap_or("target page")
            )
        } else if decision.proposed_new_subtree.is_some() {
            "review taxonomy mutation proposal before publication".to_string()
        } else {
            "review before further action".to_string()
        },
        scores: ProposalScores {
            route_confidence: decision.confidence,
            quality_confidence: information_quality,
            hierarchy_fit_confidence: if taxonomy_mutation.is_some() {
                0.5
            } else {
                decision.confidence
            },
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
            body_rewrite_kind: decision.body_rewrite_kind.clone(),
            decision_section_present: decision
                .decision_section_markdown
                .as_deref()
                .map(|s| !s.trim().is_empty())
                .unwrap_or(false),
        },
    }
}

/// Write a routing analysis sidecar alongside `content_path`.
pub fn write_analysis_sidecar(content_path: &Path, analysis: &RoutingAnalysis) -> Result<()> {
    let sidecar_path = content_path.with_extension("analysis.json");
    let json =
        serde_json::to_string_pretty(analysis).context("Failed to serialize routing analysis")?;
    std::fs::write(&sidecar_path, json).with_context(|| {
        format!(
            "Failed to write analysis sidecar: {}",
            sidecar_path.display()
        )
    })?;

    // Stage the sidecar for git
    let repo_root = content_path
        .ancestors()
        .find(|p| p.join(".git").exists())
        .unwrap_or(content_path.parent().unwrap_or(content_path));
    let rel = sidecar_path
        .strip_prefix(repo_root)
        .unwrap_or(&sidecar_path);
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
    let batch_stamp = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let batch_id = generate_id(&format!("process:{}:{}", batch_stamp, wiki_dir.display()));
    append_log(
        wiki_dir,
        &format!(
            "process batch {} at {}: intake items routed",
            batch_id, batch_stamp
        ),
    )?;
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

#[cfg(test)]
mod manifest_tests {
    use super::*;

    fn write_page(dir: &Path, slug: &str, title: &str, body: &str, kw: &[&str]) {
        let kw_yaml: String = kw.iter().map(|k| format!("  - {k}\n")).collect();
        let raw = format!(
            "---\nid: {slug}\ntitle: \"{title}\"\nstatus: published\nsource:\n  kind: web_page\n  id: src\n  origin_url: https://x\ncategory: []\nkeywords:\n{kw_yaml}created_at: \"2026-01-01T00:00:00Z\"\nupdated_at: \"2026-01-01T00:00:00Z\"\ncross_refs: []\ncontent_hash: \"h\"\n---\n\n{body}\n"
        );
        std::fs::write(dir.join(format!("{slug}.md")), raw).unwrap();
    }

    #[test]
    fn collect_peer_pages_caps_at_five_and_skips_index() {
        let tmp = tempfile::tempdir().unwrap();
        let wiki_dir = tmp.path().join("wiki");
        let branch = wiki_dir.join("published").join("ops");
        std::fs::create_dir_all(&branch).unwrap();
        std::fs::write(branch.join("index.md"), "# Ops\n\nbranch index\n").unwrap();
        for i in 0..7 {
            write_page(
                &branch,
                &format!("p-{i}"),
                &format!("Page {i}"),
                &format!("Body {i} with content"),
                &["one", "two"],
            );
        }
        let peers = collect_peer_pages(&wiki_dir, &branch);
        assert_eq!(
            peers.len(),
            PEERS_PER_BRANCH,
            "must cap at PEERS_PER_BRANCH"
        );
        for p in &peers {
            assert!(
                !p.path.ends_with("index.md"),
                "branch index must not be a peer"
            );
            assert!(!p.title.is_empty());
            assert!(!p.summary.is_empty());
        }
    }

    #[test]
    fn enforce_manifest_budget_drops_peers_until_under_budget() {
        // 5 branches × 5 fat peers each — way over a tiny budget.
        let mut hierarchy: Vec<HierarchyContextEntry> = (0..5)
            .map(|i| HierarchyContextEntry {
                path: format!("published/branch-{i}/index.md"),
                title: format!("Branch {i}"),
                summary: "branch summary".into(),
                peer_pages: (0..5)
                    .map(|j| PeerPageSnippet {
                        path: format!("published/branch-{i}/page-{j}.md"),
                        title: format!("Page {j}"),
                        summary: "x".repeat(500), // big payload
                        keywords: vec!["a".into(), "b".into()],
                    })
                    .collect(),
            })
            .collect();
        let pages = vec![serde_json::json!({"slug": "intake-1", "title": "T"})];
        // Tiny budget forces aggressive trimming.
        let (truncated, dropped) = enforce_manifest_budget(&mut hierarchy, &pages, 0, 2_000);
        assert!(truncated, "must report truncation");
        assert!(dropped > 0, "must drop at least one peer entry");
        // Sanity: total peers across all branches should now be smaller than start.
        let remaining: usize = hierarchy.iter().map(|e| e.peer_pages.len()).sum();
        assert!(
            remaining < 25,
            "must drop at least one peer (started at 25)"
        );
    }

    #[test]
    fn enforce_manifest_budget_no_op_when_under_budget() {
        let mut hierarchy = vec![HierarchyContextEntry {
            path: "p".into(),
            title: "t".into(),
            summary: "s".into(),
            peer_pages: vec![PeerPageSnippet {
                path: "p1".into(),
                title: "t".into(),
                summary: "s".into(),
                keywords: vec![],
            }],
        }];
        let pages: Vec<serde_json::Value> = vec![];
        let (truncated, dropped) = enforce_manifest_budget(&mut hierarchy, &pages, 0, 1_000_000);
        assert!(!truncated);
        assert_eq!(dropped, 0);
        assert_eq!(hierarchy[0].peer_pages.len(), 1);
    }
}
