//! `curio doctor [--scope <path>]` — KB structural health report.
//!
//! Scans wiki/published/ (or a sub-path) for structural issues.
//! Produces a JSON or text report. No LLM calls.

use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use walkdir::WalkDir;

use crate::{
    Frontmatter, PageStatus, SourceRef, WikiPage,
    config::Config,
    freshness::freshness_score_from_str,
    output::emit_json,
    overlap::find_peer_overlap,
    quality::assess_quality,
    wiki_fs::parse_wiki_page,
    wiki_fs::{content_hash, generate_id, write_wiki_page},
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
    /// Per-KB infrastructure checks (config, NORTHSTAR, Confluence auth,
    /// git status). Run before content scanning so misconfigs surface
    /// even if the published tree is empty.
    #[serde(default)]
    pub infrastructure: Vec<InfraCheck>,
    pub findings: Vec<Finding>,
    pub summary: DoctorSummary,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InfraCheck {
    /// Stable, machine-friendly identifier (e.g. "kb.northstar_present").
    pub label: String,
    pub ok: bool,
    /// Human-readable detail (e.g. "wiki/NORTHSTAR.md (4321 bytes)").
    pub detail: String,
    /// Optional remediation hint shown to humans / colleagues.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fix_hint: Option<String>,
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
    emit_review: bool,
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
                && e.path().extension().is_some_and(|x| x == "md")
                && e.path().file_name().is_some_and(|n| n != "index.md")
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
        let Ok(page) = parse_wiki_page(path) else {
            continue;
        };
        let slug = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
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
                    freshness, page.frontmatter.updated_at
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
            config,
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
        errors: findings
            .iter()
            .filter(|f| f.severity == Severity::Error)
            .count(),
        warnings: findings
            .iter()
            .filter(|f| f.severity == Severity::Warn)
            .count(),
        low_quality: findings
            .iter()
            .filter(|f| f.kind == FindingKind::LowQuality)
            .count(),
        high_overlap: findings
            .iter()
            .filter(|f| f.kind == FindingKind::HighOverlap)
            .count(),
        stale: findings
            .iter()
            .filter(|f| f.kind == FindingKind::Stale)
            .count(),
        orphaned_xrefs: findings
            .iter()
            .filter(|f| f.kind == FindingKind::OrphanedXref)
            .count(),
        thin_branches: findings
            .iter()
            .filter(|f| f.kind == FindingKind::ThinBranch)
            .count(),
        missing_keywords: findings
            .iter()
            .filter(|f| f.kind == FindingKind::MissingKeywords)
            .count(),
    };

    // Run KB infrastructure checks before content scanning. Colleagues hit
    // these first when something's misconfigured — auth, NORTHSTAR.md
    // missing, .curio.yaml unparseable, git tree corrupted, etc.
    let infrastructure = run_infra_checks(config).await;

    let report = DoctorReport {
        scope: scope_label.to_string(),
        pages_scanned,
        infrastructure,
        findings,
        summary,
    };

    if emit_review {
        materialize_overlap_reviews(config, &report.findings)?;
    }

    if json {
        emit_json("doctor", true, &report)?;
        return Ok(());
    }

    // Text output
    println!("KB Doctor — scope: {}", scope_label);

    if !report.infrastructure.is_empty() {
        let infra_failed = report.infrastructure.iter().filter(|c| !c.ok).count();
        let infra_total = report.infrastructure.len();
        println!(
            "Infrastructure: {}/{} checks passed",
            infra_total - infra_failed,
            infra_total,
        );
        for c in &report.infrastructure {
            let icon = if c.ok { "✓" } else { "✖" };
            println!("  {} {} — {}", icon, c.label, c.detail);
            if !c.ok
                && let Some(hint) = &c.fix_hint
            {
                println!("      hint: {}", hint);
            }
        }
        println!();
    }

    println!("Pages scanned: {}", report.pages_scanned);
    println!();
    println!(
        "Findings: {} errors, {} warnings",
        report.summary.errors, report.summary.warnings
    );
    println!("  low-quality:      {}", report.summary.low_quality);
    println!("  high-overlap:     {}", report.summary.high_overlap);
    println!("  stale (>8 mo):    {}", report.summary.stale);
    println!("  orphaned xrefs:   {}", report.summary.orphaned_xrefs);
    println!("  thin branches:    {}", report.summary.thin_branches);
    println!("  missing keywords: {}", report.summary.missing_keywords);

    if !report.findings.is_empty() {
        println!();
        for f in &report.findings {
            let icon = if f.severity == Severity::Error {
                "✖"
            } else {
                "⚠"
            };
            println!("  {} [{:?}] {} — {}", icon, f.kind, f.slug, f.detail);
        }
    }

    Ok(())
}

fn materialize_overlap_reviews(config: &Config, findings: &[Finding]) -> Result<()> {
    let wiki_dir = &config.wiki.wiki_dir;
    let review_root = wiki_dir.join("review").join("doctor").join("high-overlap");
    std::fs::create_dir_all(&review_root)?;
    let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    for finding in findings
        .iter()
        .filter(|f| f.kind == FindingKind::HighOverlap)
    {
        let peer = finding.overlap_peer.as_deref().unwrap_or("peer");
        let slug = sanitize_segment(&finding.slug);
        let peer_slug = sanitize_segment(peer);
        let filename = format!("{}--{}.md", slug, peer_slug);
        let path = review_root.join(filename);
        let title = format!("Doctor overlap review: {} vs {}", finding.slug, peer);
        let body = format!(
            "# {}\n\n- Source page: `{}`\n- Overlap peer: `{}`\n- Overlap score: {:.2}\n- Recommendation: merge, consolidate, or cross-link after editorial review.\n\nCurio surfaced this item from `curio doctor` so it can be handled as review work instead of terminal output.",
            title,
            finding.slug,
            peer,
            finding.overlap_score.unwrap_or(0.0)
        );
        let page = WikiPage {
            path: path.clone(),
            frontmatter: Frontmatter {
                id: generate_id(&format!("doctor-overlap:{}:{}", finding.slug, peer)),
                title: title.clone(),
                status: PageStatus::Review,
                source: SourceRef {
                    kind: "doctor_overlap".to_string(),
                    id: format!("doctor:high_overlap:{}:{}", finding.slug, peer),
                    origin_url: None,
                    summary: Some(finding.detail.clone()),
                    acl: None,
                },
                category: vec!["doctor".to_string(), "high-overlap".to_string()],
                keywords: vec!["doctor".to_string(), "overlap".to_string()],
                created_at: now.clone(),
                updated_at: now.clone(),
                confidence: Some(finding.overlap_score.unwrap_or(0.0)),
                cross_refs: finding
                    .overlap_peer
                    .as_ref()
                    .map(|peer| vec![peer.clone()])
                    .unwrap_or_default(),
                content_hash: content_hash(&body),
                confluence_page_id: None,
                model_used: Some("doctor".to_string()),
                auto_healed_at: None,
                auto_healed_confidence: None,
                intake_request_id: None,
            },
            body,
        };
        write_wiki_page(&path, &page)?;
    }

    Ok(())
}

fn sanitize_segment(s: &str) -> String {
    s.chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' {
                ch
            } else {
                '-'
            }
        })
        .collect()
}

/// Per-KB infrastructure checks: config / NORTHSTAR / git / Confluence auth.
///
/// Returns one `InfraCheck` per probe. Checks are independent — a failure
/// in one does not short-circuit the rest, so the colleague sees every
/// problem on the same `curio doctor` run.
async fn run_infra_checks(config: &Config) -> Vec<InfraCheck> {
    let mut checks = Vec::new();
    let wiki_dir = &config.wiki.wiki_dir;
    // Two valid layouts: (a) KB root contains a `wiki/` subdir with the
    // content, with `.curio.yaml` at the root; (b) the wiki dir IS the
    // KB root (wiki.wiki_dir = "." in config), with `.curio.yaml`
    // alongside intake/, staged/, .... Resolve kb_dir as whichever of
    // wiki_dir or wiki_dir.parent() actually has the config file.
    let candidate_a = wiki_dir.clone();
    let candidate_b = wiki_dir.parent().unwrap_or(wiki_dir).to_path_buf();
    let kb_dir = if candidate_a.join(".curio.yaml").exists()
        || candidate_a.join("curio.yaml").exists()
    {
        candidate_a.clone()
    } else if candidate_b.join(".curio.yaml").exists() || candidate_b.join("curio.yaml").exists() {
        candidate_b
    } else {
        // Neither has .curio.yaml — keep the conventional (parent) guess
        // so the missing-config check reports a sensible path.
        wiki_dir.parent().unwrap_or(wiki_dir).to_path_buf()
    };

    // 1. .curio.yaml present & parses
    let curio_yaml = [kb_dir.join(".curio.yaml"), kb_dir.join("curio.yaml")]
        .iter()
        .find(|p| p.exists())
        .cloned();
    match &curio_yaml {
        Some(path) => {
            match std::fs::read_to_string(path)
                .map_err(|e| e.to_string())
                .and_then(|raw| {
                    serde_yaml::from_str::<crate::config::Config>(&raw)
                        .map(|_| ())
                        .map_err(|e| e.to_string())
                }) {
                Ok(()) => checks.push(InfraCheck {
                    label: "kb.config".into(),
                    ok: true,
                    detail: format!("{} parses cleanly", path.display()),
                    fix_hint: None,
                }),
                Err(err) => checks.push(InfraCheck {
                    label: "kb.config".into(),
                    ok: false,
                    detail: format!("{} failed to parse: {}", path.display(), err),
                    fix_hint: Some("Check YAML syntax. See curio init-kb output for the canonical template.".into()),
                }),
            }
        }
        None => checks.push(InfraCheck {
            label: "kb.config".into(),
            ok: false,
            detail: format!("no .curio.yaml found at {}", kb_dir.display()),
            fix_hint: Some("Run `curio init-kb --path <dir>` to scaffold one, or copy curio.yaml from another KB.".into()),
        }),
    }

    // 2. NORTHSTAR.md present (charter file). May live at wiki/NORTHSTAR.md
    //    or kb_dir/NORTHSTAR.md depending on operator convention.
    let northstar_candidates = [wiki_dir.join("NORTHSTAR.md"), kb_dir.join("NORTHSTAR.md")];
    let northstar = northstar_candidates.iter().find(|p| p.exists()).cloned();
    match northstar {
        Some(path) => {
            let bytes = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
            checks.push(InfraCheck {
                label: "kb.northstar".into(),
                ok: bytes > 0,
                detail: format!("{} ({} bytes)", path.display(), bytes),
                fix_hint: if bytes == 0 {
                    Some("NORTHSTAR.md is empty — author the KB charter and routing rules.".into())
                } else {
                    None
                },
            });
        }
        None => checks.push(InfraCheck {
            label: "kb.northstar".into(),
            ok: false,
            detail: format!(
                "NORTHSTAR.md not found in {} or {}",
                wiki_dir.display(),
                kb_dir.display()
            ),
            fix_hint: Some(
                "Author wiki/NORTHSTAR.md — it is required for routing decisions.".into(),
            ),
        }),
    }

    // 3. git status — is the KB a git repo? Is the working tree readable?
    match crate::git_ops::git_status_porcelain(&kb_dir) {
        Ok(out) => {
            let dirty_lines = out.lines().count();
            checks.push(InfraCheck {
                label: "kb.git".into(),
                ok: true,
                detail: if dirty_lines == 0 {
                    "git working tree clean".into()
                } else {
                    format!("git working tree has {} uncommitted change(s)", dirty_lines)
                },
                fix_hint: None,
            });
        }
        Err(err) => checks.push(InfraCheck {
            label: "kb.git".into(),
            ok: false,
            detail: format!("git status failed: {}", err),
            fix_hint: Some(format!(
                "Initialize the KB as a git repo: cd {} && git init -b main",
                kb_dir.display()
            )),
        }),
    }

    // 4. Confluence connection: URL + email + token resolution
    let url_set = !config.connection.confluence_url.is_empty();
    let email_set = !config.connection.confluence_email.is_empty();
    let token_env = config.connection.token_env_name().to_string();
    let token_resolved = config.connection.resolve_token();
    let space_key_set = !config.content_model.space_key.is_empty();

    checks.push(InfraCheck {
        label: "kb.confluence.url".into(),
        ok: url_set,
        detail: if url_set {
            config.connection.confluence_url.clone()
        } else {
            "connection.confluence_url is empty".into()
        },
        fix_hint: if url_set {
            None
        } else {
            Some("Set connection.confluence_url in .curio.yaml (e.g. https://yourorg.atlassian.net/wiki).".into())
        },
    });
    checks.push(InfraCheck {
        label: "kb.confluence.email".into(),
        ok: email_set,
        detail: if email_set {
            config.connection.confluence_email.clone()
        } else {
            "connection.confluence_email is empty".into()
        },
        fix_hint: if email_set {
            None
        } else {
            Some("Set connection.confluence_email in .curio.yaml (the Atlassian account email matching the API token).".into())
        },
    });
    checks.push(InfraCheck {
        label: "kb.confluence.token".into(),
        ok: token_resolved.is_ok(),
        detail: match &token_resolved {
            Ok(_) => format!("token resolved from env var {}", token_env),
            Err(err) => err.to_string(),
        },
        fix_hint: if token_resolved.is_err() {
            Some(format!(
                "Set the env var {} to your Confluence API token (e.g. in deploy/local/.env).",
                token_env
            ))
        } else {
            None
        },
    });
    checks.push(InfraCheck {
        label: "kb.confluence.space_key".into(),
        ok: space_key_set,
        detail: if space_key_set {
            config.content_model.space_key.clone()
        } else {
            "content_model.space_key is empty".into()
        },
        fix_hint: if space_key_set {
            None
        } else {
            Some("Set content_model.space_key in .curio.yaml to your Confluence space key.".into())
        },
    });

    // 5. Confluence auth probe — actually call the API.
    if url_set && email_set {
        if let Ok(token) = token_resolved {
            match crate::confluence::ConfluenceClient::new(
                config.connection.confluence_url.clone(),
                config.connection.confluence_email.clone(),
                token,
                None,
            ) {
                Ok(client) => match client.get_current_user().await {
                    Ok(user) => {
                        let display = user["displayName"].as_str().unwrap_or("unknown");
                        let email = user["email"].as_str().unwrap_or("");
                        checks.push(InfraCheck {
                            label: "kb.confluence.auth".into(),
                            ok: true,
                            detail: format!("authenticated as {} ({})", display, email),
                            fix_hint: None,
                        });
                    }
                    Err(err) => {
                        let msg = err.to_string();
                        let likely_email_mismatch =
                            msg.contains("403") || msg.contains("FORBIDDEN");
                        checks.push(InfraCheck {
                            label: "kb.confluence.auth".into(),
                            ok: false,
                            detail: format!("auth probe failed: {}", msg),
                            fix_hint: Some(if likely_email_mismatch {
                                "403 usually means the email/token mismatch or the account lacks Confluence access.".into()
                            } else {
                                "Verify CURIO_CONFLUENCE_URL, the email matches the token, and the token is current.".into()
                            }),
                        });
                    }
                },
                Err(err) => checks.push(InfraCheck {
                    label: "kb.confluence.auth".into(),
                    ok: false,
                    detail: format!("could not build Confluence client: {}", err),
                    fix_hint: None,
                }),
            }
        }
    } else {
        checks.push(InfraCheck {
            label: "kb.confluence.auth".into(),
            ok: false,
            detail: "skipped — url, email, or token not set".into(),
            fix_hint: Some("Fix the prior kb.confluence.* checks first.".into()),
        });
    }

    checks
}
