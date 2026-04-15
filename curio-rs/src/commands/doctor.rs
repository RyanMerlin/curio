//! `curio doctor [--scope <path>]` — KB structural health report.
//!
//! Scans wiki/published/ (or a sub-path) for structural issues.
//! Produces a JSON or text report. No LLM calls.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use walkdir::WalkDir;

use crate::{
    config::Config,
    freshness::freshness_score_from_str,
    output::emit_json,
    overlap::find_peer_overlap,
    quality::assess_quality,
    wiki_fs::parse_wiki_page,
};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FindingKind {
    LowQuality,
    HighOverlap,
    Stale,
    OrphanedXref,
    ThinBranch,
    MissingKeywords,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Warn,
    Error,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Finding {
    pub kind: FindingKind,
    pub severity: Severity,
    pub slug: String,
    pub path: String,
    pub detail: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overlap_peer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overlap_score: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freshness_score: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality_score: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DoctorReport {
    pub scope: String,
    pub pages_scanned: usize,
    pub findings: Vec<Finding>,
    pub summary: DoctorSummary,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DoctorSummary {
    pub errors: usize,
    pub warnings: usize,
    pub low_quality: usize,
    pub high_overlap: usize,
    pub stale: usize,
    pub orphaned_xrefs: usize,
    pub thin_branches: usize,
    pub missing_keywords: usize,
}

pub async fn run_doctor(
    config: &Config,
    _dry_run: bool, // doctor is always read-only; parameter accepted for interface uniformity
    json: bool,
    scope: Option<String>,
) -> Result<()> {
    let wiki_dir = &config.wiki.wiki_dir;
    let published_dir = wiki_dir.join("published");
    let overlap_threshold = config.heal.overlap_threshold() as f32;
    // Pages are "stale" when their freshness score falls below the half-life score (0.5).
    // By definition of exponential decay, a page aged stale_threshold_days has score ~0.5.
    let stale_threshold = 0.50f64;
    let min_body_words = config.heal.min_body_words() as usize;

    // Resolve scan root: full published dir or scoped subdir.
    let scan_root = if let Some(ref s) = scope {
        let candidate = published_dir.join(s);
        if !candidate.exists() {
            anyhow::bail!("Scope path not found: {}", candidate.display());
        }
        candidate
    } else {
        published_dir.clone()
    };

    let scope_label = scope.as_deref().unwrap_or("(all)");

    // Collect all published .md pages in scope (excluding index.md files).
    let pages: Vec<PathBuf> = WalkDir::new(&scan_root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_file()
                && e.path().extension().map_or(false, |x| x == "md")
                && e.path().file_name().map_or(false, |n| n != "index.md")
        })
        .map(|e| e.path().to_path_buf())
        .collect();

    let pages_scanned = pages.len();
    let mut findings: Vec<Finding> = Vec::new();

    // Collect all published slugs for xref validation.
    let all_slugs: std::collections::HashSet<String> = pages
        .iter()
        .filter_map(|p| p.file_stem().and_then(|s| s.to_str()).map(String::from))
        .collect();

    for path in &pages {
        let Ok(page) = parse_wiki_page(path) else { continue };
        let slug = path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();
        let path_str = path.display().to_string();

        // 1. Quality check
        let quality = assess_quality(&page.frontmatter.title, &page.body);
        if !quality.publishable {
            findings.push(Finding {
                kind: FindingKind::LowQuality,
                severity: Severity::Warn,
                slug: slug.clone(),
                path: path_str.clone(),
                detail: format!(
                    "quality flags: {:?}; info={:.2} usability={:.2}",
                    quality.flags, quality.information_quality, quality.usability
                ),
                overlap_peer: None,
                overlap_score: None,
                freshness_score: None,
                quality_score: Some((quality.information_quality + quality.usability) / 2.0),
            });
        }

        // 2. Freshness check — updated_at is always a String on Frontmatter
        let freshness = freshness_score_from_str(&page.frontmatter.updated_at).unwrap_or(1.0);
        if freshness < stale_threshold {
            findings.push(Finding {
                kind: FindingKind::Stale,
                severity: Severity::Warn,
                slug: slug.clone(),
                path: path_str.clone(),
                detail: format!(
                    "freshness {:.2} (last updated: {})",
                    freshness,
                    &page.frontmatter.updated_at
                ),
                overlap_peer: None,
                overlap_score: None,
                freshness_score: Some(freshness),
                quality_score: None,
            });
        }

        // 3. Missing keywords
        if page.frontmatter.keywords.is_empty() {
            findings.push(Finding {
                kind: FindingKind::MissingKeywords,
                severity: Severity::Warn,
                slug: slug.clone(),
                path: path_str.clone(),
                detail: "no keywords set".to_string(),
                overlap_peer: None,
                overlap_score: None,
                freshness_score: None,
                quality_score: None,
            });
        }

        // 4. Orphaned cross-refs
        for xref in &page.frontmatter.cross_refs {
            if !all_slugs.contains(xref.as_str()) {
                findings.push(Finding {
                    kind: FindingKind::OrphanedXref,
                    severity: Severity::Error,
                    slug: slug.clone(),
                    path: path_str.clone(),
                    detail: format!("broken cross_ref → {}", xref),
                    overlap_peer: None,
                    overlap_score: None,
                    freshness_score: None,
                    quality_score: None,
                });
            }
        }

        // 5. Overlap check — find_peer_overlap takes wiki_dir (not published_dir),
        //    exclude_slug to avoid self-match, returns Result<Vec<OverlapMatch>>
        //    OverlapMatch.path is a String (wiki_dir-relative), OverlapMatch.score is f32
        if let Ok(peers) = find_peer_overlap(
            wiki_dir,
            &page.frontmatter.category,
            &page.frontmatter.title,
            &page.body,
            Some(&slug),
        ) {
            for m in peers.iter().filter(|m| m.score >= overlap_threshold) {
                // Extract peer slug from path string (e.g. "published/cat/peer-slug.md")
                let peer_slug = std::path::Path::new(&m.path)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string();
                // Avoid duplicate pairs: only emit if slug < peer_slug (lexicographic).
                if slug < peer_slug {
                    findings.push(Finding {
                        kind: FindingKind::HighOverlap,
                        severity: Severity::Warn,
                        slug: slug.clone(),
                        path: path_str.clone(),
                        detail: format!("overlap {:.2} with {}", m.score, peer_slug),
                        overlap_peer: Some(peer_slug),
                        overlap_score: Some(m.score),
                        freshness_score: None,
                        quality_score: None,
                    });
                }
            }
        }
    }

    // 6. Thin branch check — directories with < min_body_words words in index.md
    for entry in WalkDir::new(&scan_root).min_depth(1).into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_dir() {
            continue;
        }
        let index = entry.path().join("index.md");
        if !index.exists() {
            continue;
        }
        if let Ok(idx_page) = parse_wiki_page(&index) {
            let word_count = idx_page.body.split_whitespace().count();
            if word_count < min_body_words {
                let branch_slug = entry
                    .path()
                    .strip_prefix(&published_dir)
                    .unwrap_or(entry.path())
                    .display()
                    .to_string()
                    .replace('\\', "/");
                findings.push(Finding {
                    kind: FindingKind::ThinBranch,
                    severity: Severity::Warn,
                    slug: branch_slug.clone(),
                    path: index.display().to_string(),
                    detail: format!("branch index has only {} words", word_count),
                    overlap_peer: None,
                    overlap_score: None,
                    freshness_score: None,
                    quality_score: None,
                });
            }
        }
    }

    // Build summary
    let summary = DoctorSummary {
        errors:           findings.iter().filter(|f| f.severity == Severity::Error).count(),
        warnings:         findings.iter().filter(|f| f.severity == Severity::Warn).count(),
        low_quality:      findings.iter().filter(|f| f.kind == FindingKind::LowQuality).count(),
        high_overlap:     findings.iter().filter(|f| f.kind == FindingKind::HighOverlap).count(),
        stale:            findings.iter().filter(|f| f.kind == FindingKind::Stale).count(),
        orphaned_xrefs:   findings.iter().filter(|f| f.kind == FindingKind::OrphanedXref).count(),
        thin_branches:    findings.iter().filter(|f| f.kind == FindingKind::ThinBranch).count(),
        missing_keywords: findings.iter().filter(|f| f.kind == FindingKind::MissingKeywords).count(),
    };

    let report = DoctorReport {
        scope: scope_label.to_string(),
        pages_scanned,
        findings,
        summary,
    };

    if json {
        emit_json("doctor", true, &report)?;
        return Ok(());
    }

    // Text output
    println!("KB Doctor — scope: {}", scope_label);
    println!("Pages scanned: {}", report.pages_scanned);
    println!();
    println!("Findings: {} errors, {} warnings", report.summary.errors, report.summary.warnings);
    println!("  low-quality:      {}", report.summary.low_quality);
    println!("  high-overlap:     {}", report.summary.high_overlap);
    println!("  stale (>8 mo):    {}", report.summary.stale);
    println!("  orphaned xrefs:   {}", report.summary.orphaned_xrefs);
    println!("  thin branches:    {}", report.summary.thin_branches);
    println!("  missing keywords: {}", report.summary.missing_keywords);

    if !report.findings.is_empty() {
        println!();
        for f in &report.findings {
            let icon = if f.severity == Severity::Error { "✖" } else { "⚠" };
            println!("  {} [{:?}] {} — {}", icon, f.kind, f.slug, f.detail);
        }
    }

    Ok(())
}
