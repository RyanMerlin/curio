//! `curio heal` — confidence-gated AI heal loop.
//!
//! Two phases (mirroring `curio process`):
//!
//!   Phase 1 — `--prepare`:
//!     Scan the scope, compute quality/freshness/overlap signals, emit a
//!     JSON heal manifest.  Claude reads the manifest, uses external tools,
//!     and writes a `heal-routes.json` decision file.
//!
//!   Phase 2 — `--apply-file <path>`:
//!     Read the routes file.  For each action:
//!       - confidence >= threshold  → publish + mirror to review/auto-approved/
//!       - confidence <  threshold  → move to review/ as normal proposal

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::{
    config::Config,
    freshness::freshness_score_from_str,
    heal_types::{
        ExternalContext, HealManifest, ManifestPage, ManifestQuality, OverlapCandidate,
        StructuralIssue,
    },
    overlap::find_peer_overlap,
    quality::assess_quality,
    wiki_fs::parse_wiki_page,
};

// ── Phase 1: prepare ─────────────────────────────────────────────────────────

pub async fn run_heal_prepare(
    config: &Config,
    scope: Option<String>,
    out_file: Option<String>,
) -> Result<()> {
    let wiki_dir = &config.wiki.wiki_dir;
    let published_dir = wiki_dir.join("published");

    let scan_root = match &scope {
        Some(s) => {
            let p = published_dir.join(s);
            if !p.exists() {
                anyhow::bail!("Scope path does not exist: {}", p.display());
            }
            p
        }
        None => published_dir.clone(),
    };

    let scope_label = scope.as_deref().unwrap_or("(all)").to_string();
    let confidence_threshold = config.heal.confidence_threshold();

    // Collect all published pages in scope (exclude index.md)
    let page_paths: Vec<PathBuf> = WalkDir::new(&scan_root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_file()
                && e.path().extension().map_or(false, |x| x == "md")
                && e.path().file_name().map_or(false, |n| n != "index.md")
        })
        .map(|e| e.path().to_path_buf())
        .collect();

    let mut pages: Vec<ManifestPage> = Vec::new();

    for path in &page_paths {
        let Ok(page) = parse_wiki_page(path) else { continue };
        let slug = path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();

        let freshness = freshness_score_from_str(&page.frontmatter.updated_at).unwrap_or(1.0);
        let quality = assess_quality(&page.frontmatter.title, &page.body);

        let overlap_candidates: Vec<OverlapCandidate> = find_peer_overlap(
            wiki_dir,
            &page.frontmatter.category,
            &page.frontmatter.title,
            &page.body,
            Some(&slug),
        )
        .unwrap_or_default()
        .into_iter()
        .filter(|m| m.score >= 0.45)
        .filter_map(|m| {
            let peer_slug = Path::new(&m.path)
                .file_stem()?
                .to_str()?
                .to_string();
            let peer_path = wiki_dir.join(&m.path);
            let peer_title = parse_wiki_page(&peer_path)
                .map(|p| p.frontmatter.title.clone())
                .unwrap_or_else(|_| peer_slug.clone());
            Some(OverlapCandidate {
                slug: peer_slug,
                title: peer_title,
                score: m.score,
            })
        })
        .collect();

        pages.push(ManifestPage {
            slug,
            path: path.display().to_string(),
            title: page.frontmatter.title.clone(),
            body: page.body.clone(),
            category: page.frontmatter.category.clone(),
            keywords: page.frontmatter.keywords.clone(),
            source_url: page.frontmatter.source.origin_url.clone(),
            updated_at: page.frontmatter.updated_at.clone(),
            freshness_score: freshness,
            quality: ManifestQuality {
                information_quality: quality.information_quality,
                usability: quality.usability,
                publishable: quality.publishable,
                flags: quality.flags.clone(),
            },
            overlap_candidates,
        });
    }

    // Thin branch structural issues
    let mut structural_issues: Vec<StructuralIssue> = Vec::new();
    let min_body_words = config.heal.min_body_words() as usize;
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
                structural_issues.push(StructuralIssue {
                    kind: "thin_branch".to_string(),
                    slug: branch_slug,
                    path: index.display().to_string(),
                    detail: format!("{} words in branch index", word_count),
                });
            }
        }
    }

    let manifest = HealManifest {
        scope: scope_label.clone(),
        confidence_threshold,
        pages,
        structural_issues,
        external_context: ExternalContext {
            confluence_space_key: config.content_model.space_key.clone(),
            source_space_key: None,
        },
        apply_command: format!(
            "curio heal --apply-file /tmp/heal-routes.json{}",
            scope.as_deref().map(|s| format!(" --scope {}", s)).unwrap_or_default()
        ),
    };

    let manifest_json = serde_json::to_string_pretty(&manifest)
        .context("Failed to serialize manifest")?;

    if let Some(ref path) = out_file {
        std::fs::write(path, &manifest_json)
            .with_context(|| format!("Failed to write manifest to {}", path))?;
        eprintln!("Heal manifest written to {}", path);
        eprintln!("Pages in scope: {}", manifest.pages.len());
        eprintln!("Structural issues: {}", manifest.structural_issues.len());
        eprintln!();
        eprintln!("Next: have Claude read the manifest and produce a routes file.");
        eprintln!("Apply with: {}", manifest.apply_command);
    } else {
        println!("{}", manifest_json);
    }

    Ok(())
}

// ── Phase 2: apply (stub — implemented in Task B3) ───────────────────────────

pub async fn run_heal_apply(
    _config: &Config,
    _dry_run: bool,
    routes_file: &str,
    _scope: Option<String>,
) -> Result<()> {
    anyhow::bail!("run_heal_apply not yet implemented (routes_file={})", routes_file)
}
