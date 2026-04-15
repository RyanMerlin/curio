/// `curio feedback` — Read Confluence review signals and apply them to the wiki.
///
/// For each wiki/review/ page that has a `.sync-refs.json` sidecar (written by `curio sync`),
/// this command:
///   1. Reads labels and pinned-comment reactions from Confluence.
///   2. Determines the action: approve / reject / rewrite / capture.
///   3. In live mode, executes the action:
///      - approve  → mv review/ → staged/; update NORTHSTAR.md taxonomy if taxonomy_mutation
///      - reject   → rm wiki page + analysis sidecar; log in _config/log.md
///      - rewrite  → reset status=intake, mv review/ → intake/; append reviewer
///                   comments to <slug>.feedback.md
///      - capture  → append free-form comments to <slug>.feedback.md (no status change)
///   4. Writes a summary to stdout and appends to _config/log.md.
///
/// Signal precedence: labels (curio:approve / curio:reject / curio:rewrite) beat reactions
/// (👍 / 👎 / ❓) on the pinned comment.  Free-form comments (not the pinned comment,
/// no matching label/reaction) never trigger auto-action — they go into feedback.md only.
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::{
    config::Config,
    confluence::ConfluenceClient,
    northstar::{load_taxonomy, save_taxonomy},
    wiki_fs::parse_wiki_page,
};

const LABEL_APPROVE: &str = "curio:approve";
const LABEL_REJECT: &str = "curio:reject";
const LABEL_REWRITE: &str = "curio:rewrite";

#[derive(Debug, Clone, PartialEq)]
enum Action {
    Approve,
    Reject,
    Rewrite,
    /// Free-form reviewer comments only — no state change, just captured into feedback.md
    Capture,
    /// No signals at all
    NoSignal,
}

struct PageSignals {
    path: PathBuf,
    action: Action,
    reviewer_comments: Vec<String>,
}

pub async fn run_feedback(config: &Config, dry_run: bool) -> Result<()> {
    let wiki_dir = &config.wiki.wiki_dir;
    let review_dir = wiki_dir.join("review");

    if !review_dir.exists() {
        println!("No wiki/review/ directory found — nothing to do.");
        return Ok(());
    }

    config.connection.require_confluence()?;
    let token =
        std::env::var("CURIO_CONFLUENCE_TOKEN").context("CURIO_CONFLUENCE_TOKEN not set")?;
    let client = ConfluenceClient::new(
        config.connection.confluence_url.clone(),
        config.connection.confluence_email.clone(),
        token,
        None,
    )?;

    // ── Collect all review pages that have a sync-refs sidecar ──────────────
    let mut candidates: Vec<(PathBuf, serde_json::Value)> = Vec::new();
    for entry in WalkDir::new(&review_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path().to_path_buf();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let fname = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if !fname.ends_with(".sync-refs.json") {
            continue;
        }
        let raw = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        let refs: serde_json::Value = serde_json::from_str(&raw)
            .with_context(|| format!("Failed to parse {}", path.display()))?;
        // The wiki page lives next to the sidecar with the .md extension
        let md_path = path.with_extension("").with_extension("md");
        if md_path.exists() {
            candidates.push((md_path, refs));
        }
    }

    if candidates.is_empty() {
        println!("No sync-refs sidecars found — run `curio sync` first.");
        return Ok(());
    }

    println!(
        "Checking {} review page(s) for Confluence signals…",
        candidates.len()
    );

    let mut signals: Vec<PageSignals> = Vec::new();

    for (md_path, refs) in &candidates {
        let review_page_id = match refs["confluence_review_page_id"].as_str() {
            Some(id) => id.to_string(),
            None => continue,
        };
        let pinned_comment_id = refs["pinned_comment_id"].as_str().map(|s| s.to_string());

        // ── Fetch labels ────────────────────────────────────────────────────
        let labels: Vec<String> = client
            .get_page_labels_v2(&review_page_id)
            .await
            .unwrap_or_default();

        // ── Label-driven action (takes precedence) ──────────────────────────
        let label_action: Option<Action> = if labels.iter().any(|l| l == LABEL_APPROVE) {
            Some(Action::Approve)
        } else if labels.iter().any(|l| l == LABEL_REJECT) {
            Some(Action::Reject)
        } else if labels.iter().any(|l| l == LABEL_REWRITE) {
            Some(Action::Rewrite)
        } else {
            None
        };

        // ── Reaction-driven action (only consulted when no label) ───────────
        let reaction_action: Option<Action> = if label_action.is_none() {
            if let Some(ref comment_id) = pinned_comment_id {
                let reactions: Vec<serde_json::Value> = client
                    .get_comment_reactions(comment_id)
                    .await
                    .unwrap_or_default();
                // Each reaction object has an "emoji" field with "value" or "shortName"
                let emojis: Vec<String> = reactions
                    .iter()
                    .filter_map(|r| {
                        r["emoji"]["value"]
                            .as_str()
                            .or_else(|| r["emoji"]["shortName"].as_str())
                            .map(|s: &str| s.to_string())
                    })
                    .collect();
                if emojis.iter().any(|e: &String| {
                    e.contains('\u{1F44D}') || e.contains("+1") || e.contains("thumbsup")
                }) {
                    Some(Action::Approve)
                } else if emojis.iter().any(|e: &String| {
                    e.contains('\u{1F44E}') || e.contains("-1") || e.contains("thumbsdown")
                }) {
                    Some(Action::Reject)
                } else if emojis
                    .iter()
                    .any(|e: &String| e.contains('\u{2753}') || e.contains("question"))
                {
                    Some(Action::Rewrite)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        // ── Collect free-form comments (non-pinned footer + inline) ─────────
        let footer_comments: Vec<serde_json::Value> = client
            .get_page_footer_comments(&review_page_id)
            .await
            .unwrap_or_default();
        let inline_comments: Vec<serde_json::Value> = client
            .get_page_inline_comments(&review_page_id)
            .await
            .unwrap_or_default();
        let reviewer_comments: Vec<String> = footer_comments
            .iter()
            .chain(inline_comments.iter())
            .filter(|c| {
                pinned_comment_id
                    .as_deref()
                    .map(|pid| c["id"].as_str() != Some(pid))
                    .unwrap_or(true)
            })
            .filter_map(|c| {
                let body = c["body"]["storage"]["value"]
                    .as_str()
                    .or_else(|| c["body"]["value"].as_str())
                    .unwrap_or("");
                if body.is_empty() {
                    return None;
                }
                Some(strip_html_tags(body))
            })
            .filter(|s: &String| !s.trim().is_empty())
            .collect();

        let action = label_action
            .or(reaction_action)
            .unwrap_or(if reviewer_comments.is_empty() {
                Action::NoSignal
            } else {
                Action::Capture
            });

        signals.push(PageSignals {
            path: md_path.clone(),
            action,
            reviewer_comments,
        });
    }

    // ── Report and execute ───────────────────────────────────────────────────
    let mut log_lines: Vec<String> = Vec::new();
    let timestamp = chrono::Utc::now().to_rfc3339();
    let mut approve_count = 0usize;
    let mut reject_count = 0usize;
    let mut rewrite_count = 0usize;
    let mut capture_count = 0usize;
    let mut no_signal_count = 0usize;

    for sig in &signals {
        let slug = sig
            .path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        match &sig.action {
            Action::NoSignal => {
                no_signal_count += 1;
                // Suppress noise for auto-approved pages — no signal is expected.
                let is_auto_approved = sig.path.to_string_lossy().contains("auto-approved");
                if !is_auto_approved {
                    // Future: could log these to a "pending human review" list.
                }
            }
            Action::Capture => {
                capture_count += 1;
                println!(
                    "  CAPTURE  {}: {} comment(s) → feedback.md",
                    slug,
                    sig.reviewer_comments.len()
                );
                if !dry_run {
                    append_feedback_md(&sig.path, &sig.reviewer_comments)?;
                    log_lines.push(format!(
                        "[{}] capture {} — {} reviewer comment(s) captured",
                        timestamp,
                        slug,
                        sig.reviewer_comments.len()
                    ));
                }
            }
            Action::Approve => {
                approve_count += 1;
                println!("  APPROVE  {}", slug);
                if !dry_run {
                    apply_approve(config, &sig.path, wiki_dir, &mut log_lines, &timestamp)?;
                }
            }
            Action::Reject => {
                reject_count += 1;
                println!("  REJECT   {}", slug);
                if !dry_run {
                    apply_reject(&sig.path, &mut log_lines, &timestamp)?;
                }
            }
            Action::Rewrite => {
                rewrite_count += 1;
                println!("  REWRITE  {}", slug);
                if !dry_run {
                    apply_rewrite(
                        &sig.path,
                        wiki_dir,
                        &sig.reviewer_comments,
                        &mut log_lines,
                        &timestamp,
                    )?;
                }
            }
        }
    }

    println!();
    if dry_run {
        println!(
            "[dry-run] {} approve / {} reject / {} rewrite / {} capture / {} no-signal",
            approve_count, reject_count, rewrite_count, capture_count, no_signal_count
        );
    } else {
        println!(
            "Done: {} approved / {} rejected / {} rewritten / {} captured / {} no-signal",
            approve_count, reject_count, rewrite_count, capture_count, no_signal_count
        );
        if !log_lines.is_empty() {
            append_audit_log(wiki_dir, &log_lines)?;
        }
    }

    Ok(())
}

// ── Action helpers ───────────────────────────────────────────────────────────

fn apply_approve(
    _config: &Config,
    md_path: &Path,
    wiki_dir: &Path,
    log_lines: &mut Vec<String>,
    timestamp: &str,
) -> Result<()> {
    let page = parse_wiki_page(md_path)?;
    let category_path = if page.frontmatter.category.is_empty() {
        "uncategorised".to_string()
    } else {
        page.frontmatter.category.join("/")
    };

    let dest_dir = wiki_dir.join("staged").join(&category_path);
    std::fs::create_dir_all(&dest_dir)
        .with_context(|| format!("Failed to create {}", dest_dir.display()))?;

    let filename = md_path.file_name().unwrap();
    let dest = dest_dir.join(filename);

    // Update frontmatter status → staged
    let raw = std::fs::read_to_string(md_path)?;
    let updated = raw.replacen("status: review", "status: staged", 1);
    std::fs::write(&dest, &updated)
        .with_context(|| format!("Failed to write {}", dest.display()))?;
    std::fs::remove_file(md_path)
        .with_context(|| format!("Failed to remove {}", md_path.display()))?;

    // Move analysis sidecar if present
    let analysis = md_path.with_extension("analysis.json");
    if analysis.exists() {
        let dest_analysis = dest_dir.join(analysis.file_name().unwrap());
        std::fs::rename(&analysis, &dest_analysis).ok();

        // Check for taxonomy_mutation in the analysis and update NORTHSTAR.md YAML block
        maybe_update_northstar(wiki_dir, &dest_analysis, log_lines, timestamp);
    }

    // Remove sync-refs sidecar (specific to the review page)
    let refs_path = md_path.with_extension("sync-refs.json");
    if refs_path.exists() {
        std::fs::remove_file(&refs_path).ok();
    }

    log_lines.push(format!(
        "[{}] approve {} → staged/{}",
        timestamp, page.frontmatter.title, category_path
    ));
    Ok(())
}

fn apply_reject(md_path: &Path, log_lines: &mut Vec<String>, timestamp: &str) -> Result<()> {
    let page = parse_wiki_page(md_path)?;
    std::fs::remove_file(md_path)?;
    for ext in &["analysis.json", "sync-refs.json", "feedback.md"] {
        let sidecar = md_path.with_extension(ext);
        if sidecar.exists() {
            std::fs::remove_file(&sidecar).ok();
        }
    }
    log_lines.push(format!(
        "[{}] reject {} — removed",
        timestamp, page.frontmatter.title
    ));
    Ok(())
}

fn apply_rewrite(
    md_path: &Path,
    wiki_dir: &Path,
    reviewer_comments: &[String],
    log_lines: &mut Vec<String>,
    timestamp: &str,
) -> Result<()> {
    let page = parse_wiki_page(md_path)?;

    let dest_dir = wiki_dir.join("intake");
    std::fs::create_dir_all(&dest_dir)?;
    let filename = md_path.file_name().unwrap();
    let dest = dest_dir.join(filename);

    let raw = std::fs::read_to_string(md_path)?;
    let updated = raw.replacen("status: review", "status: intake", 1);
    std::fs::write(&dest, &updated)?;
    std::fs::remove_file(md_path)?;

    for ext in &["analysis.json", "sync-refs.json"] {
        let sidecar = md_path.with_extension(ext);
        if sidecar.exists() {
            std::fs::remove_file(&sidecar).ok();
        }
    }

    if !reviewer_comments.is_empty() {
        append_feedback_md(&dest, reviewer_comments)?;
    }

    log_lines.push(format!(
        "[{}] rewrite {} — moved back to intake",
        timestamp, page.frontmatter.title
    ));
    Ok(())
}

fn append_feedback_md(md_path: &Path, comments: &[String]) -> Result<()> {
    let feedback_path = md_path.with_extension("feedback.md");
    let timestamp = chrono::Utc::now().to_rfc3339();
    let mut content = if feedback_path.exists() {
        std::fs::read_to_string(&feedback_path)?
    } else {
        "# Reviewer Feedback\n\n".to_string()
    };
    content.push_str(&format!("\n## Comments ({})\n\n", timestamp));
    for comment in comments {
        content.push_str(&format!("- {}\n", comment.trim()));
    }
    std::fs::write(&feedback_path, content)?;
    Ok(())
}

fn maybe_update_northstar(
    wiki_dir: &Path,
    analysis_path: &Path,
    log_lines: &mut Vec<String>,
    timestamp: &str,
) {
    let Ok(raw) = std::fs::read_to_string(analysis_path) else {
        return;
    };
    let Ok(analysis): Result<serde_json::Value, _> = serde_json::from_str(&raw) else {
        return;
    };
    let Some(mutation) = analysis.get("taxonomy_mutation") else {
        return;
    };
    let Some(path_arr) = mutation
        .get("proposed_new_subtree")
        .and_then(|v| v.as_array())
    else {
        return;
    };
    let path_segments: Vec<String> = path_arr
        .iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect();
    if path_segments.is_empty() {
        return;
    }

    let new_slug = match path_segments.last() {
        Some(s) => s.clone(),
        None => return,
    };
    let new_title = mutation["node_description"]
        .as_str()
        .unwrap_or(&new_slug)
        .to_string();

    // Load the live taxonomy from NORTHSTAR.md
    let Ok(mut taxonomy) = load_taxonomy(wiki_dir) else {
        return;
    };

    // Skip if slug already exists anywhere in the tree
    fn slug_exists(nodes: &[crate::northstar::TaxonomyNode], slug: &str) -> bool {
        nodes
            .iter()
            .any(|n| n.slug == slug || slug_exists(&n.children, slug))
    }
    if slug_exists(&taxonomy.nodes, &new_slug) {
        return;
    }

    // Navigate to the parent node and insert the new child
    let parent_segments = &path_segments[..path_segments.len().saturating_sub(1)];
    fn find_node_mut<'a>(
        nodes: &'a mut Vec<crate::northstar::TaxonomyNode>,
        path: &[String],
    ) -> Option<&'a mut Vec<crate::northstar::TaxonomyNode>> {
        if path.is_empty() {
            return Some(nodes);
        }
        let target = &path[0];
        let idx = nodes.iter().position(|n| &n.slug == target)?;
        find_node_mut(&mut nodes[idx].children, &path[1..])
    }

    if let Some(children) = find_node_mut(&mut taxonomy.nodes, parent_segments) {
        children.push(crate::northstar::TaxonomyNode {
            title: new_title.clone(),
            slug: new_slug.clone(),
            description_markdown: new_title,
            icon: None,
            children: vec![],
        });
        if save_taxonomy(wiki_dir, &taxonomy).is_ok() {
            log_lines.push(format!(
                "[{}] NORTHSTAR.md: added {} under {}",
                timestamp,
                new_slug,
                parent_segments.join("/")
            ));
        }
    }
}

fn append_audit_log(wiki_dir: &Path, lines: &[String]) -> Result<()> {
    let log_path = wiki_dir.join("_config/log.md");
    let mut content = if log_path.exists() {
        std::fs::read_to_string(&log_path)?
    } else {
        "# Curio Audit Log\n\n".to_string()
    };
    for line in lines {
        content.push_str(line);
        content.push('\n');
    }
    std::fs::write(&log_path, content)?;
    Ok(())
}

/// Minimal HTML tag stripper for storage-format comment bodies.
fn strip_html_tags(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}
