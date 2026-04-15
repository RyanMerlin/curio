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
        ExternalContext, HealAction, HealKind, HealManifest, HealRoutesFile, ManifestPage,
        ManifestQuality, OverlapCandidate, StructuralIssue,
    },
    overlap::find_peer_overlap,
    quality::assess_quality,
    wiki_fs::parse_wiki_page,
    wiki_index::append_log,
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
        let Ok(page) = parse_wiki_page(path) else {
            continue;
        };
        let slug = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();

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
            let peer_slug = Path::new(&m.path).file_stem()?.to_str()?.to_string();
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
    for entry in WalkDir::new(&scan_root)
        .min_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
    {
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
            scope
                .as_deref()
                .map(|s| format!(" --scope {}", s))
                .unwrap_or_default()
        ),
    };

    let manifest_json =
        serde_json::to_string_pretty(&manifest).context("Failed to serialize manifest")?;

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

// ── Phase 2: apply ───────────────────────────────────────────────────────────

pub async fn run_heal_apply(
    config: &Config,
    dry_run: bool,
    routes_file: &str,
    _scope: Option<String>,
) -> Result<()> {
    let wiki_dir = &config.wiki.wiki_dir;
    let threshold = config.heal.confidence_threshold();

    let raw = std::fs::read_to_string(routes_file)
        .with_context(|| format!("Cannot read routes file: {}", routes_file))?;
    let routes: HealRoutesFile = serde_json::from_str(&raw)
        .with_context(|| format!("Cannot parse routes file: {}", routes_file))?;

    let timestamp = chrono::Utc::now().to_rfc3339();
    let mut auto_approved = 0usize;
    let mut routed_to_review = 0usize;
    let mut no_action = 0usize;
    let mut log_lines: Vec<String> = Vec::new();

    for action in &routes.actions {
        match action.kind {
            HealKind::NoAction => {
                no_action += 1;
                if dry_run {
                    println!("[dry-run] NO_ACTION  {}", action.slug);
                }
                continue;
            }
            _ => {
                let src_path = find_published_page(wiki_dir, &action.slug)?;

                if action.confidence >= threshold {
                    apply_auto_approve(
                        wiki_dir,
                        action,
                        &src_path,
                        &timestamp,
                        dry_run,
                        &mut log_lines,
                    )?;
                    auto_approved += 1;
                } else {
                    apply_to_review(wiki_dir, action, &timestamp, dry_run, &mut log_lines)?;
                    routed_to_review += 1;
                }
            }
        }
    }

    if !dry_run {
        for line in &log_lines {
            append_log(wiki_dir, line)?;
        }
    }

    if dry_run {
        println!(
            "[dry-run] auto-approve: {} | to-review: {} | no-action: {}",
            auto_approved, routed_to_review, no_action
        );
    } else {
        println!("Heal apply complete:");
        println!("  auto-approved (published): {}", auto_approved);
        println!("  routed to review:          {}", routed_to_review);
        println!("  no action:                 {}", no_action);
    }

    Ok(())
}

fn find_published_page(wiki_dir: &Path, slug: &str) -> Result<PathBuf> {
    // Search published first, then review and staged.
    for lane in &["published", "review", "staged"] {
        let lane_dir = wiki_dir.join(lane);
        if !lane_dir.exists() {
            continue;
        }
        for entry in WalkDir::new(&lane_dir).into_iter().filter_map(|e| e.ok()) {
            if entry.path().extension().map_or(false, |e| e == "md") {
                if entry.path().file_stem().map_or(false, |s| s == slug) {
                    return Ok(entry.path().to_path_buf());
                }
            }
        }
    }
    anyhow::bail!("Page not found for slug: {}", slug)
}

fn apply_auto_approve(
    wiki_dir: &Path,
    action: &HealAction,
    src_path: &Path,
    timestamp: &str,
    dry_run: bool,
    log_lines: &mut Vec<String>,
) -> Result<()> {
    let slug = &action.slug;

    if dry_run {
        println!(
            "[dry-run] AUTO-APPROVE  {} ({:?}, confidence {:.2})",
            slug, action.kind, action.confidence
        );
        return Ok(());
    }

    let auto_dir = wiki_dir.join("review").join("auto-approved");
    std::fs::create_dir_all(&auto_dir)?;

    match action.kind {
        HealKind::Archive => {
            // Move page to review/auto-approved/ with status=archived
            let dest = auto_dir.join(src_path.file_name().unwrap());
            let raw = std::fs::read_to_string(src_path)?;
            // Replace status in frontmatter
            let updated = raw
                .replacen("status: published", "status: archived", 1)
                .replacen("status: staged", "status: archived", 1)
                .replacen("status: review", "status: archived", 1);
            std::fs::write(&dest, updated)?;
            std::fs::remove_file(src_path)?;
        }
        HealKind::Rewrite | HealKind::Merge => {
            let new_content = action.new_content.as_deref().ok_or_else(|| {
                anyhow::anyhow!("Rewrite/Merge action for {} missing new_content", slug)
            })?;

            // Inject auto_healed frontmatter fields before closing ---
            let final_content =
                inject_auto_heal_frontmatter(new_content, timestamp, action.confidence);
            std::fs::write(src_path, &final_content)?;

            // Delete merge source pages
            if action.kind == HealKind::Merge {
                for merge_slug in &action.merge_sources {
                    if let Ok(mp) = find_published_page(wiki_dir, merge_slug) {
                        std::fs::remove_file(&mp).ok();
                        for ext in &["analysis.json", "sync-refs.json"] {
                            let sidecar = mp.with_extension(ext);
                            if sidecar.exists() {
                                std::fs::remove_file(&sidecar).ok();
                            }
                        }
                    }
                }
            }

            write_auto_approve_record(&auto_dir, action, timestamp)?;
        }
        HealKind::UpdateMetadata | HealKind::FixStructure => {
            if let Some(content) = &action.new_content {
                std::fs::write(src_path, content)?;
            }
            write_auto_approve_record(&auto_dir, action, timestamp)?;
        }
        HealKind::NoAction => {}
    }

    log_lines.push(format!(
        "[{}] heal auto-approve {:?} {} (conf {:.2}) — {}",
        timestamp, action.kind, slug, action.confidence, action.rationale
    ));

    Ok(())
}

fn apply_to_review(
    wiki_dir: &Path,
    action: &HealAction,
    timestamp: &str,
    dry_run: bool,
    log_lines: &mut Vec<String>,
) -> Result<()> {
    let slug = &action.slug;

    if dry_run {
        println!(
            "[dry-run] TO-REVIEW  {} ({:?}, confidence {:.2})",
            slug, action.kind, action.confidence
        );
        return Ok(());
    }

    let review_dir = wiki_dir.join("review");
    std::fs::create_dir_all(&review_dir)?;

    let proposal = serde_json::json!({
        "slug": slug,
        "kind": format!("{:?}", action.kind),
        "confidence": action.confidence,
        "rationale": action.rationale,
        "sources_consulted": action.sources_consulted,
        "new_content": action.new_content,
        "merge_sources": action.merge_sources,
        "created_at": timestamp,
        "status": "review",
    });
    let proposal_path = review_dir.join(format!("{}.heal-proposal.json", slug));
    std::fs::write(&proposal_path, serde_json::to_string_pretty(&proposal)?)?;

    log_lines.push(format!(
        "[{}] heal route-to-review {:?} {} (conf {:.2}) — {}",
        timestamp, action.kind, slug, action.confidence, action.rationale
    ));

    Ok(())
}

fn write_auto_approve_record(auto_dir: &Path, action: &HealAction, timestamp: &str) -> Result<()> {
    let record = serde_json::json!({
        "slug": action.slug,
        "kind": format!("{:?}", action.kind),
        "confidence": action.confidence,
        "rationale": action.rationale,
        "sources_consulted": action.sources_consulted,
        "merge_sources": action.merge_sources,
        "approved_at": timestamp,
        "status": "auto-approved",
    });
    let record_path = auto_dir.join(format!("{}.decision.json", action.slug));
    std::fs::write(&record_path, serde_json::to_string_pretty(&record)?)?;

    // Human-readable .md companion for Confluence sync
    let sources_md = if action.sources_consulted.is_empty() {
        "_none_".to_string()
    } else {
        action
            .sources_consulted
            .iter()
            .map(|s| format!("- {}", s))
            .collect::<Vec<_>>()
            .join("\n")
    };
    let md = format!(
        "---\ntitle: \"Auto-Approved: {slug}\"\nstatus: auto-approved\nauto_healed_at: \"{timestamp}\"\nauto_healed_confidence: {confidence}\n---\n\n\
# Auto-Approved: {slug}\n\n**Action:** {kind:?}  \n**Confidence:** {confidence:.2}  \n**Approved at:** {timestamp}  \n\n\
## Rationale\n\n{rationale}\n\n## Sources Consulted\n\n{sources}\n",
        slug = action.slug,
        timestamp = timestamp,
        confidence = action.confidence,
        kind = action.kind,
        rationale = action.rationale,
        sources = sources_md,
    );
    let md_path = auto_dir.join(format!("{}.md", action.slug));
    std::fs::write(&md_path, md)?;

    Ok(())
}

fn inject_auto_heal_frontmatter(content: &str, timestamp: &str, confidence: f64) -> String {
    let note = format!(
        "auto_healed_at: \"{}\"\nauto_healed_confidence: {}\n",
        timestamp, confidence
    );
    // Insert before the closing --- of the frontmatter block.
    // The frontmatter ends at the first "\n---" after the opening "---\n".
    if content.starts_with("---") {
        if let Some(pos) = content[3..].find("\n---") {
            let close_pos = 3 + pos; // position of the \n before ---
            let (front, rest) = content.split_at(close_pos);
            return format!("{}\n{}{}", front, note, rest);
        }
    }
    // No frontmatter — prepend minimal block
    format!("---\n{}\n---\n\n{}", note, content)
}
