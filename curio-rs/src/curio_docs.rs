use crate::{BranchChildEntry, BranchIndex, Result, confluence::ConfluenceClient};
use chrono::Utc;
use scraper::Html;
use serde_json::{Value, json};
use std::fmt::Write as _;

pub const ADMIN_TITLE: &str = "Admin";
pub const README_TITLE: &str = "README";
pub const NORTHSTAR_TITLE: &str = "NORTHSTAR";
pub const TEMPLATES_TITLE: &str = "_templates";
pub const REGISTRY_TITLE: &str = "_registry";
pub const AUDIT_TITLE: &str = "_audit";
pub const EMOJI_TITLE_PUBLISHED: &str = "emoji-title-published";
pub const EMOJI_TITLE_DRAFT: &str = "emoji-title-draft";

pub struct CurioCorePages {
    pub readme_page_id: String,
    pub northstar_page_id: String,
    pub intake_page_id: String,
    pub staged_page_id: String,
    pub review_page_id: String,
    pub published_page_id: String,
    pub admin_page_id: String,
    pub templates_page_id: String,
    pub registry_page_id: String,
    pub audit_page_id: String,
}

#[derive(Debug, Clone)]
pub struct RegistryRecord {
    pub key: String,
    pub item_type: String,
    pub title: String,
    pub page_id: String,
    pub parent_id: String,
    pub status: String,
    pub source_id: String,
    pub summary: String,
    pub updated_at: String,
}

#[derive(Debug, Clone)]
pub struct AuditEntry {
    pub actor: String,
    pub command: String,
    pub subject: String,
    pub action: String,
    pub rationale: String,
    pub source: String,
    pub result: String,
    pub detail_lines: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct RoutePlan {
    pub target_path: Vec<String>,
    pub registry_path: Vec<String>,
    pub rationale: String,
    pub validation_requirements: Vec<String>,
    pub sibling_context: Vec<String>,
    pub confidence: f32,
    pub snapshot_hash: Option<String>,
}

pub async fn ensure_scoped_page(
    client: &ConfluenceClient,
    space_key: &str,
    parent_id: &str,
    title: &str,
    body: &str,
) -> Result<String> {
    if parent_id.is_empty() {
        let page_id = client
            .create_or_update_page(space_key, None, title, "storage", body)
            .await?;
        return Ok(page_id);
    }
    let page_id =
        crate::resolve_or_create_scoped_child_page_id(client, space_key, parent_id, title, body)
            .await?;
    Ok(page_id)
}

pub async fn ensure_scoped_structure_page(
    client: &ConfluenceClient,
    space_key: &str,
    parent_id: &str,
    title: &str,
    body: &str,
) -> Result<String> {
    let page_id = ensure_scoped_page(client, space_key, parent_id, title, body).await?;
    apply_title_emoji(client, &page_id, title).await?;
    Ok(page_id)
}

pub async fn ensure_tree_page(
    client: &ConfluenceClient,
    space_key: &str,
    root_parent_id: &str,
    root_label: &str,
    branch_path: &[String],
    title: &str,
    body: &str,
) -> Result<String> {
    let mut parent_id = root_parent_id.to_string();
    for segment in branch_path {
        parent_id = ensure_scoped_page(
            client,
            space_key,
            &parent_id,
            segment,
            &build_tree_branch_body(root_label, branch_path, segment),
        )
        .await?;
    }

    ensure_scoped_page(client, space_key, &parent_id, title, body).await
}

pub async fn ensure_registry_record(
    client: &ConfluenceClient,
    space_key: &str,
    registry_root_id: &str,
    registry_path: &[String],
    record: &RegistryRecord,
) -> Result<String> {
    let body = build_registry_record_body_with_path(record, registry_path);
    let title = format!("Record - {}", record.key);
    ensure_tree_page(
        client,
        space_key,
        registry_root_id,
        REGISTRY_TITLE,
        registry_path,
        &title,
        &body,
    )
    .await
}

pub async fn apply_title_emoji(
    client: &ConfluenceClient,
    page_id: &str,
    title: &str,
) -> Result<()> {
    let (published_emoji, draft_emoji) = title_emoji_values(title);
    if let Some(value) = published_emoji {
        client
            .set_content_property(page_id, EMOJI_TITLE_PUBLISHED, json!(value))
            .await?;
    }
    if let Some(value) = draft_emoji {
        client
            .set_content_property(page_id, EMOJI_TITLE_DRAFT, json!(value))
            .await?;
    }
    Ok(())
}

pub async fn append_audit_entry(
    client: &ConfluenceClient,
    space_key: &str,
    audit_root_id: &str,
    bucket_path: &[String],
    entry: &AuditEntry,
) -> Result<String> {
    let stamp = Utc::now().format("%Y%m%dT%H%M%S%.3fZ").to_string();
    let title = format!("Audit - {} - {}", stamp, entry.command);
    let body = build_audit_entry_body(entry, &stamp);
    ensure_tree_page(
        client,
        space_key,
        audit_root_id,
        AUDIT_TITLE,
        bucket_path,
        &title,
        &body,
    )
    .await
}

pub fn build_readme_body(hero_image_html: Option<&str>) -> String {
    let mut html = String::new();
    html.push_str("<h1>README</h1>");
    html.push_str("<p><strong>Start here.</strong> This is the human landing page for Curio.</p>");
    html.push_str("<p>Curio turns Confluence into a working intelligence system. It captures raw material, stages it for review, preserves published answers, and keeps the workspace understandable for both people and agents.</p>");

    if let Some(hero_image_html) = hero_image_html {
        html.push_str("<p>");
        html.push_str(hero_image_html);
        html.push_str("</p>");
    }

    append_section(
        &mut html,
        "How To Use Curio",
        Some("The lane names tell you where work belongs."),
        &[
            "README is the guided starting point.",
            "Intake is for raw captures and incoming material.",
            "Staged is for content that is almost ready.",
            "Review is for uncertain or disputed work.",
            "Published is for the current approved answer.",
            "Admin holds the machine-managed branch for templates, registry, and audit.",
            "_templates holds reusable page patterns under Admin.",
            "_registry is the authoritative catalog of Curio records under Admin.",
            "_audit is the append-only history of Curio actions under Admin.",
        ],
    );

    append_section(
        &mut html,
        "What To Do First",
        Some("If you are new here, use the simplest lane that fits the work."),
        &[
            "If you want the operating contract, start in NORTHSTAR.",
            "If the content is new, put it in Intake.",
            "If you need a template, start in _templates and use `Template - Intake Page` unless another lane fits better.",
            "If you need to know where something lives, check _registry.",
            "If you want to see how Curio decided something, check _audit.",
        ],
    );

    append_section(
        &mut html,
        "Operating Style",
        Some("Curio should feel professional, fun, and easy to trust."),
        &[
            "Keep page names clear and stable.",
            "Keep provenance visible.",
            "Prefer structured content over freeform notes.",
            "Make the next action obvious to a non-technical user.",
        ],
    );

    html
}

pub fn build_lifecycle_page(
    title: &str,
    intro: &str,
    purpose_heading: &str,
    purpose: &str,
    use_for: &[&str],
    avoid: &[&str],
    notes: &[&str],
) -> String {
    let mut html = String::new();
    let _ = write!(&mut html, "<h1>{}</h1>", title);
    let _ = write!(&mut html, "<p>{}</p>", intro);
    append_section(&mut html, purpose_heading, Some(purpose), &[]);
    append_section(&mut html, "Use This For", None, use_for);
    append_section(&mut html, "Do Not Use This For", None, avoid);
    append_section(&mut html, "Operating Notes", None, notes);
    html
}

pub fn extract_page_text(page_json: &Value) -> String {
    let storage = page_json["body"]["storage"]["value"]
        .as_str()
        .unwrap_or_default();
    if !storage.trim().is_empty() {
        let fragment = Html::parse_fragment(storage);
        let text = fragment.root_element().text().collect::<Vec<_>>().join(" ");
        return text.split_whitespace().collect::<Vec<_>>().join(" ");
    }

    let adf_value = page_json["body"]["atlas_doc_format"]["value"]
        .as_str()
        .unwrap_or_default();
    if adf_value.trim().is_empty() {
        return String::new();
    }

    let Ok(adf_json) = serde_json::from_str::<Value>(adf_value) else {
        return adf_value.to_string();
    };

    let mut output = String::new();
    collect_adf_text(&adf_json, &mut output);
    output.trim().to_string()
}

fn collect_adf_text(node: &Value, output: &mut String) {
    match node {
        Value::Object(map) => match map.get("type").and_then(Value::as_str) {
            Some("text") => {
                if let Some(text) = map.get("text").and_then(Value::as_str) {
                    output.push_str(text);
                }
            }
            Some("hardBreak") => {
                output.push('\n');
            }
            Some("paragraph") | Some("heading") | Some("blockquote") | Some("bulletList")
            | Some("orderedList") | Some("listItem") | Some("table") | Some("tableRow")
            | Some("tableCell") | Some("panel") | Some("expand") | Some("codeBlock")
            | Some("doc") => {
                if let Some(children) = map.get("content").and_then(Value::as_array) {
                    for child in children {
                        collect_adf_text(child, output);
                    }
                    if matches!(
                        map.get("type").and_then(Value::as_str),
                        Some("paragraph")
                            | Some("heading")
                            | Some("blockquote")
                            | Some("listItem")
                            | Some("tableCell")
                            | Some("panel")
                            | Some("expand")
                            | Some("codeBlock")
                    ) {
                        output.push('\n');
                    }
                }
            }
            _ => {
                if let Some(children) = map.get("content").and_then(Value::as_array) {
                    for child in children {
                        collect_adf_text(child, output);
                    }
                }
            }
        },
        Value::Array(items) => {
            for item in items {
                collect_adf_text(item, output);
            }
        }
        _ => {}
    }
}

pub fn build_admin_root_body() -> String {
    let mut html = String::new();
    html.push_str("<h1>Admin</h1>");
    html.push_str("<p>This is Curio's machine-managed branch. It holds the pages that keep the workspace organized, indexed, and auditable.</p>");
    append_section(
        &mut html,
        "What Lives Here",
        Some("Admin is for operational structure, not working content."),
        &[
            "_templates for reusable page shapes.",
            "_registry for canonical records and routing indexes.",
            "_audit for append-only action history.",
        ],
    );
    append_section(
        &mut html,
        "How To Use It",
        Some(
            "Most users should not create content directly under Admin unless Curio or an administrator is maintaining the workspace.",
        ),
        &[
            "Let Curio manage the structure here.",
            "Use the registry and audit pages to inspect system state.",
            "Use templates when creating new content patterns.",
        ],
    );
    html
}

pub fn build_templates_root_body() -> String {
    let mut html = String::new();
    html.push_str("<h1>_templates</h1>");
    html.push_str("<p>This area holds Curio's reusable page patterns. It lives under Admin because it is machine-managed structure, not user-facing work.</p>");
    append_section(
        &mut html,
        "What Goes Here",
        Some("Templates are safe starting points, not live work."),
        &[
            "Human-friendly page scaffolds.",
            "Example bodies for intake and review content.",
            "Registry and audit record formats.",
            "Reusable phrasing for sales, operations, and publishing pages.",
        ],
    );
    append_section(
        &mut html,
        "Default Template",
        Some("Curio should favor the intake template unless a different lane is clearly better."),
        &[
            "Use `Template - Intake Page` for new raw captures, notes, and source material.",
            "Use `Template - Staged Page` for near-final work that needs approval.",
            "Use `Template - Review Page` for disputes, ambiguity, or policy questions.",
            "Use `Template - Published Page` for canonical answers and deliverables.",
        ],
    );
    append_section(
        &mut html,
        "How To Use It",
        Some("Pick the template that matches the job, then fill in the live data."),
        &[
            "Use the template pages as starting points when creating new content.",
            "Do not treat templates as source-of-truth pages.",
            "If a pattern becomes common, add it here instead of copying it by hand.",
        ],
    );
    html
}

pub fn build_published_branch_page(
    title: &str,
    what_it_is: &str,
    what_it_is_not: &str,
    metadata: &[&str],
) -> String {
    let mut html = String::new();
    let _ = write!(&mut html, "<h1>{}</h1>", title);
    html.push_str("<p>This page is part of the Published tree defined by NORTHSTAR.</p>");
    append_section(&mut html, "What It Is", Some(what_it_is), &[]);
    append_section(&mut html, "What It Is Not", Some(what_it_is_not), &[]);
    append_section(
        &mut html,
        "Useful Metadata",
        Some("Keep these fields visible so the branch stays useful to humans and agents."),
        metadata,
    );
    append_section(
        &mut html,
        "Related Navigation",
        Some("Use these pages to move from the charter into the operating tree."),
        &["NORTHSTAR", "README", "_registry", "_audit"],
    );
    html
}

/// Renders a human-readable routing table for a branch page.
pub fn build_branch_index_body(
    branch_title: &str,
    branch_path: &[String],
    root_label: &str,
    children: &[BranchChildEntry],
    total_descendants: u32,
) -> String {
    let mut html = String::new();
    let _ = write!(&mut html, "<h1>{}</h1>", branch_title);
    let breadcrumb = if branch_path.is_empty() {
        root_label.to_string()
    } else {
        format!("{} &gt; {}", root_label, branch_path.join(" &gt; "))
    };
    let _ = write!(
        &mut html,
        "<p>Branch in <strong>{}</strong> tree. Path: {}.</p>",
        root_label, breadcrumb
    );
    let _ = write!(
        &mut html,
        "<p>{} direct children. {} total descendants.</p>",
        children.len(),
        total_descendants
    );

    if children.is_empty() {
        html.push_str("<p><em>No children yet.</em></p>");
    } else {
        html.push_str("<h2>Child Index</h2>");
        html.push_str("<table><tr><th>Page</th><th>Type</th><th>Summary</th><th>Children</th><th>Status</th></tr>");
        for child in children {
            let link = format!(
                "<ac:link><ri:content-entity ri:content-id=\"{}\"/><ac:plain-text-link-body><![CDATA[{}]]></ac:plain-text-link-body></ac:link>",
                child.page_id, child.title
            );
            let _ = write!(
                &mut html,
                "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
                link, child.child_type, child.summary, child.child_count, child.status
            );
        }
        html.push_str("</table>");
    }

    html.push_str("<h2>Navigation</h2>");
    let _ = write!(
        &mut html,
        "<ul><li>Tree root: <strong>{}</strong></li></ul>",
        root_label
    );
    html
}

/// Updates the branch index (content property + page body) for a branch page.
/// Call this after any operation that creates, moves, or removes a child under the branch.
///
/// Write safety: all writes delegate to `set_content_property` and `update_page_body_by_id`,
/// which both call `assert_within_write_root` on every page_id before writing. Any attempt to
/// write outside the configured write root will fail with an error -- no stray writes possible.
pub async fn update_branch_index(
    client: &ConfluenceClient,
    branch_page_id: &str,
    branch_title: &str,
    branch_path: &[String],
    root_label: &str,
    parent_page_id: Option<&str>,
) -> Result<()> {
    let raw_children = client.get_direct_children_v2(branch_page_id).await?;

    let now = Utc::now().to_rfc3339();
    let mut child_entries: Vec<BranchChildEntry> = Vec::new();

    for child in &raw_children {
        let child_page_id = child["id"].as_str().unwrap_or_default().to_string();
        let child_title = child["title"].as_str().unwrap_or_default().to_string();

        // Try to read the child's branch index to get child_count
        let (child_type, child_count) =
            if let Ok(Some(branch_prop)) = client.get_content_property(&child_page_id, "curio_branch_index").await {
                let count = branch_prop["value"]["children"]
                    .as_array()
                    .map(|a| a.len() as u32)
                    .unwrap_or(0);
                ("branch".to_string(), count)
            } else {
                ("leaf".to_string(), 0)
            };

        // Extract summary and status from curio_metadata (single read)
        let (summary, status) = if let Ok(Some(meta_prop)) = client.get_content_property(&child_page_id, "curio_metadata").await {
            let meta = &meta_prop["value"];
            let s = meta["agent_analysis"]["summary"]
                .as_str()
                .or_else(|| meta["subject_key"].as_str())
                .unwrap_or(&child_title)
                .chars()
                .take(200)
                .collect::<String>();
            let st = meta["status"].as_str().unwrap_or("structural").to_string();
            (s, st)
        } else {
            (child_title.chars().take(200).collect::<String>(), "structural".to_string())
        };

        child_entries.push(BranchChildEntry {
            page_id: child_page_id,
            title: child_title,
            child_type,
            summary,
            child_count,
            status,
            labels: vec![],
            updated_at: now.clone(),
        });
    }

    // Read existing index version for monotonic increment
    let existing_version = client
        .get_content_property(branch_page_id, "curio_branch_index")
        .await
        .ok()
        .flatten()
        .and_then(|p| p["value"]["index_version"].as_u64())
        .unwrap_or(0) as u32;

    let total_descendants = child_entries.iter().map(|c| 1 + c.child_count).sum::<u32>();

    let branch_index = BranchIndex {
        branch_page_id: branch_page_id.to_string(),
        branch_title: branch_title.to_string(),
        branch_path: branch_path.to_vec(),
        parent_page_id: parent_page_id.map(|s| s.to_string()),
        children: child_entries.clone(),
        total_descendants,
        index_updated_at: now.clone(),
        index_version: existing_version + 1,
    };

    let index_value = serde_json::to_value(&branch_index)
        .map_err(|e| anyhow::anyhow!("Failed to serialize BranchIndex: {}", e))?;
    client
        .set_content_property(branch_page_id, "curio_branch_index", index_value)
        .await?;

    let body = build_branch_index_body(branch_title, branch_path, root_label, &child_entries, total_descendants);
    client
        .update_page_body_by_id(branch_page_id, "storage", &body)
        .await?;

    Ok(())
}

pub fn build_tree_branch_body(root_label: &str, path: &[String], segment: &str) -> String {
    let mut html = String::new();
    let _ = write!(
        &mut html,
        "<h1>{}</h1><p>This branch lives under <strong>{}</strong>.</p>",
        segment, root_label
    );
    append_section(
        &mut html,
        "Path",
        Some("This page is part of the Curio index tree."),
        &path.iter().map(|item| item.as_str()).collect::<Vec<_>>(),
    );
    append_section(
        &mut html,
        "Usage",
        Some("Curio creates child pages under this branch as routing and content evolve."),
        &[
            "Keep child names stable when possible.",
            "Use this branch to expose discoverable child pages and canonical records.",
        ],
    );
    html
}

pub fn infer_route_plan(title: Option<&str>, _content: &str, hints: Option<&Value>) -> RoutePlan {
    let hints = hints.and_then(|value| value.as_object());
    let title_text = title.unwrap_or("").trim();

    let explicit_target = hints
        .and_then(|obj| obj.get("target_path"))
        .and_then(|value| value.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str())
                .map(sanitize_segment)
                .filter(|item| !item.is_empty())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    if !explicit_target.is_empty() {
        return RoutePlan {
            target_path: explicit_target.clone(),
            registry_path: explicit_target,
            rationale: "Explicit target path supplied in metadata.".to_string(),
            validation_requirements: vec![
                "sibling_alignment".to_string(),
                "pre_change_snapshot".to_string(),
            ],
            sibling_context: vec!["explicit_target_path".to_string()],
            confidence: 0.98,
            snapshot_hash: None,
        };
    }

    let account = first_hint(hints, &["account", "client", "customer"]);
    let use_case = first_hint(hints, &["use_case", "usecase"]);
    let product = first_hint(hints, &["product"]);
    let audience = first_hint(hints, &["audience", "persona"]);
    let topic = first_hint(hints, &["topic", "subject", "theme"]);
    let has_topic = topic.is_some();
    let fallback = sanitize_segment(title_text);
    let topic_branch = topic
        .clone()
        .or_else(|| strip_date_suffix(&fallback))
        .filter(|value| !value.is_empty())
        .map(|value| shorten_branch_label(&value, 4))
        .unwrap_or_else(|| {
            if fallback.is_empty() {
                "General".to_string()
            } else {
                shorten_branch_label(&fallback, 4)
            }
        });

    let mut path = Vec::new();
    let basis;
    let confidence;

    if let Some(account) = account {
        basis = "account";
        confidence = 0.9;
        path.push("By Account".to_string());
        path.push(shorten_branch_label(&account, 4));
        if let Some(use_case) = use_case {
            path.push(shorten_branch_label(&use_case, 4));
        } else {
            path.push(topic_branch.clone());
        }
    } else if let Some(product) = product {
        basis = "product";
        confidence = 0.86;
        path.push("By Product".to_string());
        path.push(shorten_branch_label(&product, 4));
        path.push(topic_branch.clone());
    } else if let Some(audience) = audience {
        basis = "audience";
        confidence = 0.82;
        path.push("By Audience".to_string());
        path.push(shorten_branch_label(&audience, 4));
        path.push(topic_branch.clone());
    } else if let Some(use_case) = use_case {
        basis = "use_case";
        confidence = 0.8;
        path.push("By Use Case".to_string());
        path.push(shorten_branch_label(&use_case, 4));
        path.push(topic_branch.clone());
    } else {
        basis = "topic";
        confidence = if has_topic { 0.72 } else { 0.58 };
        path.push("By Topic".to_string());
        path.push(topic_branch.clone());
    }

    RoutePlan {
        target_path: path.clone(),
        registry_path: path,
        rationale: format!("Route inferred from {} metadata/title.", basis),
        validation_requirements: vec![
            "sibling_alignment".to_string(),
            "pre_change_snapshot".to_string(),
        ],
        sibling_context: vec![
            format!("basis={}", basis),
            format!("confidence={:.2}", confidence),
        ],
        confidence,
        snapshot_hash: None,
    }
}

pub fn audit_bucket_path(stamp: &str) -> Vec<String> {
    if stamp.len() >= 6 {
        vec![stamp[0..4].to_string(), stamp[4..6].to_string()]
    } else {
        vec!["unknown".to_string(), "unknown".to_string()]
    }
}

fn first_hint(hints: Option<&serde_json::Map<String, Value>>, keys: &[&str]) -> Option<String> {
    let hints = hints?;
    for key in keys {
        if let Some(value) = hints.get(*key) {
            if let Some(text) = value.as_str() {
                let cleaned = sanitize_segment(text);
                if !cleaned.is_empty() {
                    return Some(cleaned);
                }
            }
        }
    }
    None
}

fn sanitize_segment(input: &str) -> String {
    let mut cleaned = input
        .replace(['/', '\\', ':', '|', '[', ']', '{', '}', '(', ')'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    cleaned = cleaned.trim().trim_matches('-').trim().to_string();
    cleaned
}

fn shorten_branch_label(input: &str, max_words: usize) -> String {
    let sanitized = sanitize_segment(input);
    if sanitized.is_empty() {
        return "General".to_string();
    }
    sanitized
        .split_whitespace()
        .take(max_words)
        .collect::<Vec<_>>()
        .join(" ")
}

fn strip_date_suffix(input: &str) -> Option<String> {
    let tokens: Vec<&str> = input.split_whitespace().collect();
    let filtered: Vec<&str> = tokens
        .into_iter()
        .filter(|token| {
            let compact = token.trim_matches(|c: char| !c.is_alphanumeric());
            !(compact.len() >= 6 && compact.chars().all(|c| c.is_ascii_digit()))
        })
        .collect();
    let joined = filtered.join(" ");
    if joined.is_empty() {
        None
    } else {
        Some(joined)
    }
}

pub fn build_template_page(
    title: &str,
    purpose: &str,
    audience: &str,
    when_to_use: &[&str],
    example_bullets: &[&str],
) -> String {
    let mut html = String::new();
    let _ = write!(&mut html, "<h1>{}</h1>", title);
    let _ = write!(&mut html, "<p>{}</p>", purpose);
    append_section(&mut html, "Audience", Some(audience), &[]);
    append_section(&mut html, "When To Use", None, when_to_use);
    append_section(&mut html, "Example Fields", None, example_bullets);
    html
}

pub fn build_registry_root_body() -> String {
    let mut html = String::new();
    html.push_str("<h1>_registry</h1>");
    html.push_str("<p>The registry is Curio's master index. It lives under Admin because the index itself is machine-managed structure.</p>");
    append_section(
        &mut html,
        "Registry Contract",
        Some(
            "The registry is hybrid: Curio seeds it and keeps it updated, while humans can inspect it at any time.",
        ),
        &[
            "Every core page gets a record.",
            "Every created content page should get a record.",
            "Every record should show the current status and source.",
            "Records should be easy to scan and easy to audit.",
        ],
    );
    append_section(
        &mut html,
        "Navigation Branches",
        Some("The registry is organized as a tree of branch pages, not a single flat list."),
        &[
            "Index - Templates",
            "Index - Published",
            "Index - By Account",
            "Index - By Product",
            "Index - By Audience",
            "Index - By Use Case",
            "Index - By Topic",
        ],
    );
    append_section(
        &mut html,
        "How To Read Records",
        Some("Each record page is the source of truth for one Curio artifact."),
        &[
            "Page ID",
            "Title",
            "Type",
            "Status",
            "Source",
            "Current parent",
            "Latest update time",
        ],
    );
    html
}

pub fn build_registry_record_body(record: &RegistryRecord) -> String {
    build_registry_record_body_with_path(record, &[])
}

pub fn build_registry_record_body_with_path(
    record: &RegistryRecord,
    registry_path: &[String],
) -> String {
    let mut html = String::new();
    let _ = write!(&mut html, "<h1>Record - {}</h1>", record.key);
    html.push_str("<p>This is a machine-maintained record page in the Curio registry.</p>");
    html.push_str("<table>");
    write_row(&mut html, "Key", &record.key);
    write_row(&mut html, "Title", &record.title);
    write_row(&mut html, "Type", &record.item_type);
    write_row(&mut html, "Status", &record.status);
    write_row(&mut html, "Page ID", &record.page_id);
    write_row(&mut html, "Source", &record.source_id);
    write_row(&mut html, "Current Parent", &record.parent_id);
    write_row(&mut html, "Registry Path", &registry_path.join(" / "));
    write_row(&mut html, "Summary", &record.summary);
    write_row(&mut html, "Updated At", &record.updated_at);
    html.push_str("</table>");
    html
}

pub fn build_audit_root_body() -> String {
    let mut html = String::new();
    html.push_str("<h1>_audit</h1>");
    html.push_str("<p>The audit log is append-only. It lives under Admin because the history is operational state, not user-facing content.</p>");
    append_section(
        &mut html,
        "What Gets Logged",
        Some("Log the interesting parts of the action, not just the result."),
        &[
            "Timestamp",
            "Command",
            "Subject",
            "Source inputs",
            "Decision or rationale",
            "Affected pages",
            "Dry-run or live-run outcome",
        ],
    );
    html
}

pub fn build_audit_entry_body(entry: &AuditEntry, stamp: &str) -> String {
    let mut html = String::new();
    let _ = write!(&mut html, "<h1>Audit - {}</h1>", stamp);
    html.push_str(
        "<p>This entry records a single Curio action and is intended to remain immutable.</p>",
    );
    html.push_str("<table>");
    write_row(&mut html, "Actor", &entry.actor);
    write_row(&mut html, "Command", &entry.command);
    write_row(&mut html, "Subject", &entry.subject);
    write_row(&mut html, "Action", &entry.action);
    write_row(&mut html, "Source", &entry.source);
    write_row(&mut html, "Rationale", &entry.rationale);
    write_row(&mut html, "Result", &entry.result);
    html.push_str("</table>");
    if !entry.detail_lines.is_empty() {
        let details: Vec<&str> = entry.detail_lines.iter().map(|s| s.as_str()).collect();
        append_section(&mut html, "Details", None, &details);
    }
    html
}

fn append_section(body: &mut String, heading: &str, intro: Option<&str>, items: &[&str]) {
    let _ = write!(body, "<h2>{}</h2>", heading);
    if let Some(intro) = intro {
        let _ = write!(body, "<p>{}</p>", intro);
    }
    if !items.is_empty() {
        body.push_str("<ul>");
        for item in items {
            let _ = write!(body, "<li>{}</li>", item);
        }
        body.push_str("</ul>");
    }
}

fn write_row(body: &mut String, key: &str, value: &str) {
    let _ = write!(body, "<tr><th>{}</th><td>{}</td></tr>", key, value);
}

fn title_emoji_values(title: &str) -> (Option<&'static str>, Option<&'static str>) {
    match title {
        README_TITLE => (Some("atlassian-info"), None),
        NORTHSTAR_TITLE => (Some("1f9ed"), None),
        "Intake" => (Some("1f4e5"), None),
        "Staged" => (Some("atlassian-logo_projects"), None),
        "Review" => (Some("atlassian-logo_opsgenie"), None),
        "Published" => (Some("atlassian-check_mark"), None),
        ADMIN_TITLE => (Some("2699"), Some("2699")),
        TEMPLATES_TITLE => (Some("1f4d1"), None),
        REGISTRY_TITLE => (Some("atlassian-logo_admin"), None),
        AUDIT_TITLE => (Some("1f4dc"), None),
        _ => (None, None),
    }
}

// --- Summary helpers ---

/// Extracts a short summary from Confluence storage HTML.
/// Strips all tags, collapses whitespace into a single string, and returns None if the
/// total extracted text is shorter than 20 chars. Truncates to `max_chars`.
pub fn extract_summary_from_html(html: &str, max_chars: usize) -> Option<String> {
    let fragment = Html::parse_fragment(html);
    let raw = fragment
        .root_element()
        .text()
        .collect::<Vec<_>>()
        .join(" ");
    let cleaned = raw.split_whitespace().collect::<Vec<_>>().join(" ");
    if cleaned.len() < 20 {
        return None;
    }
    Some(truncate_summary(&cleaned, max_chars))
}

/// Extracts a short summary from plain text.
/// Finds the first line of >= 20 non-whitespace chars and truncates to `max_chars`.
pub fn extract_summary_from_text(text: &str, max_chars: usize) -> Option<String> {
    let line = text
        .lines()
        .map(|l| l.trim())
        .find(|l| l.chars().count() >= 20)?;
    Some(truncate_summary(line, max_chars))
}

fn truncate_summary(s: &str, max_chars: usize) -> String {
    let char_count = s.chars().count();
    if char_count <= max_chars {
        return s.to_string();
    }
    // Collect to max_chars chars
    let cut: String = s.chars().take(max_chars).collect();
    // Trim to last word boundary
    let trimmed = match cut.rfind(' ') {
        Some(i) => &cut[..i],
        None => &cut,
    };
    format!("{}…", trimmed)
}

// --- ADF helper primitives ---

fn rc_heading(level: u8, text: &str) -> serde_json::Value {
    json!({
        "type": "heading",
        "attrs": { "level": level },
        "content": [{ "type": "text", "text": text }]
    })
}

fn rc_paragraph(text: &str) -> serde_json::Value {
    json!({
        "type": "paragraph",
        "content": [{ "type": "text", "text": text }]
    })
}

fn rc_paragraph_with_link(label: &str, link_text: &str, href: &str) -> serde_json::Value {
    json!({
        "type": "paragraph",
        "content": [
            { "type": "text", "text": label },
            {
                "type": "text",
                "text": link_text,
                "marks": [{ "type": "link", "attrs": { "href": href } }]
            }
        ]
    })
}

fn rc_table(rows: &[(&str, &str)]) -> serde_json::Value {
    let table_rows: Vec<serde_json::Value> = rows
        .iter()
        .map(|(k, v)| {
            json!({
                "type": "tableRow",
                "content": [
                    { "type": "tableCell", "attrs": {}, "content": [rc_paragraph(k)] },
                    { "type": "tableCell", "attrs": {}, "content": [rc_paragraph(v)] }
                ]
            })
        })
        .collect();
    json!({
        "type": "table",
        "attrs": { "isNumberColumnEnabled": false, "layout": "default" },
        "content": table_rows
    })
}

fn rc_bullet_list(items: &[&str]) -> serde_json::Value {
    let list_items: Vec<serde_json::Value> = items
        .iter()
        .map(|item| {
            json!({
                "type": "listItem",
                "content": [rc_paragraph(item)]
            })
        })
        .collect();
    json!({ "type": "bulletList", "content": list_items })
}

// --- Public body builders ---

/// Builds an ADF reference card body for Confluence page and URL sources.
/// No body duplication — Curio stores a reference card with link + summary + routing metadata.
pub fn build_reference_card_adf(
    title: &str,
    source_id: &str,
    origin_url: Option<&str>,
    summary: Option<&str>,
    lane: &str,
    ingested_at: &str,
    metadata_rows: &[(&str, &str)],
    commands: &[&str],
) -> String {
    let mut content: Vec<serde_json::Value> = vec![
        rc_heading(1, title),
        rc_paragraph(&format!("Curio lane: {} | Source: {} | {}", lane, source_id, ingested_at)),
        rc_heading(2, "Source"),
    ];

    match origin_url {
        Some(url) => content.push(rc_paragraph_with_link("Origin: ", title, url)),
        None => content.push(rc_paragraph(&format!("Origin: {}", source_id))),
    }

    content.push(rc_heading(2, "Summary"));
    content.push(rc_paragraph(summary.unwrap_or("No summary available.")));

    if !metadata_rows.is_empty() {
        content.push(rc_heading(2, "Metadata"));
        content.push(rc_table(metadata_rows));
    }

    if !commands.is_empty() {
        content.push(rc_heading(2, "Commands"));
        content.push(rc_bullet_list(commands));
    }

    let adf = json!({ "type": "doc", "version": 1, "content": content });
    serde_json::to_string(&adf).unwrap_or_else(|_| {
        json!({
            "type": "doc", "version": 1,
            "content": [rc_heading(1, title), rc_paragraph(source_id)]
        })
        .to_string()
    })
}

/// Builds an ADF body for file capture sources.
/// Produces: title heading, status line, file metadata table, content body (truncated at 500 lines).
pub fn build_capture_intake_adf(
    title: &str,
    source_id: &str,
    lane: &str,
    ingested_at: &str,
    content: &str,
    mime: &str,
) -> String {
    const MAX_LINES: usize = 500;
    let lines: Vec<&str> = content.lines().collect();
    let truncated = lines.len() > MAX_LINES;
    let display_lines = &lines[..lines.len().min(MAX_LINES)];

    let mut body_nodes: Vec<serde_json::Value> = display_lines
        .iter()
        .filter(|l| !l.trim().is_empty())
        .map(|l| rc_paragraph(l))
        .collect();

    if truncated {
        body_nodes.push(rc_paragraph(&format!(
            "— Content truncated at {} lines ({} total). Open the source file for the full content.",
            MAX_LINES,
            lines.len()
        )));
    }

    let metadata = rc_table(&[
        ("Source", source_id),
        ("Type", mime),
        ("Ingested", ingested_at),
        ("Lines", &lines.len().to_string()),
    ]);

    let mut content_nodes: Vec<serde_json::Value> = vec![
        rc_heading(1, title),
        rc_paragraph(&format!("Curio lane: {} | Captured file content.", lane)),
        rc_heading(2, "File Metadata"),
        metadata,
        rc_heading(2, "Content"),
    ];
    content_nodes.extend(body_nodes);

    let adf = json!({ "type": "doc", "version": 1, "content": content_nodes });
    serde_json::to_string(&adf).unwrap_or_else(|_| {
        json!({
            "type": "doc", "version": 1,
            "content": [rc_heading(1, title), rc_paragraph(source_id)]
        })
        .to_string()
    })
}

#[cfg(test)]
mod body_builder_tests {
    use super::*;

    #[test]
    fn extract_summary_from_html_takes_first_paragraph() {
        let html = "<p>Hello world this is a long enough paragraph.</p><p>Second paragraph.</p>";
        let result = extract_summary_from_html(html, 300);
        assert!(result.is_some());
        let s = result.unwrap();
        assert!(s.contains("Hello world"));
    }

    #[test]
    fn extract_summary_from_html_ignores_short_fragments() {
        let html = "<p>Hi</p><p>This one is long enough to be a summary candidate yes.</p>";
        let result = extract_summary_from_html(html, 300);
        assert!(result.is_some());
        assert!(result.unwrap().contains("This one is long enough"));
    }

    #[test]
    fn extract_summary_from_html_returns_none_when_empty() {
        let result = extract_summary_from_html("<p></p><p>   </p>", 300);
        assert!(result.is_none());
    }

    #[test]
    fn extract_summary_from_html_truncates_at_max_chars() {
        let long = "a".repeat(500);
        let html = format!("<p>{}</p>", long);
        let result = extract_summary_from_html(&html, 100);
        assert!(result.is_some());
        let s = result.unwrap();
        assert!(s.len() <= 105); // max_chars + "…" (multi-byte)
    }

    #[test]
    fn extract_summary_from_text_takes_first_long_line() {
        let text = "short\n\nThis line is definitely long enough to qualify as a summary candidate.\n\nAnother line.";
        let result = extract_summary_from_text(text, 300);
        assert!(result.is_some());
        assert!(result.unwrap().contains("This line is definitely"));
    }

    #[test]
    fn extract_summary_from_text_returns_none_when_no_long_lines() {
        let text = "hi\nok\nyes";
        assert!(extract_summary_from_text(text, 300).is_none());
    }

    #[test]
    fn reference_card_adf_is_valid_json() {
        let body = build_reference_card_adf(
            "Test Page",
            "confluence-page:1:page:2",
            Some("https://example.com/wiki/pages/2"),
            Some("A useful summary of the page."),
            "Intake",
            "2026-04-11T10:00:00Z",
            &[("subject_key", "test"), ("content_type", "confluence_page")],
            &[],
        );
        let parsed: serde_json::Value = serde_json::from_str(&body).expect("must be valid JSON");
        assert_eq!(parsed["type"], "doc");
        assert_eq!(parsed["version"], 1);
    }

    #[test]
    fn reference_card_adf_includes_source_link_and_summary() {
        let body = build_reference_card_adf(
            "My Page",
            "url:https://example.com",
            Some("https://example.com"),
            Some("Great summary here."),
            "Staged",
            "2026-04-11T10:00:00Z",
            &[],
            &["curio review approve <id>"],
        );
        assert!(body.contains("https://example.com"));
        assert!(body.contains("Great summary here."));
        assert!(body.contains("curio review approve"));
    }

    #[test]
    fn capture_intake_adf_is_valid_json() {
        let body = build_capture_intake_adf(
            "My Notes",
            "file:/tmp/notes.txt",
            "Intake",
            "2026-04-11T10:00:00Z",
            "Line one\nLine two\nLine three",
            "text/plain",
        );
        let parsed: serde_json::Value = serde_json::from_str(&body).expect("must be valid JSON");
        assert_eq!(parsed["type"], "doc");
    }

    #[test]
    fn capture_intake_adf_truncates_long_content() {
        let long_content = (0..600).map(|i| format!("line {}", i)).collect::<Vec<_>>().join("\n");
        let body = build_capture_intake_adf(
            "Big File",
            "file:/tmp/big.txt",
            "Intake",
            "2026-04-11",
            &long_content,
            "text/plain",
        );
        assert!(body.contains("truncated"));
    }

    #[test]
    fn truncate_summary_respects_word_boundary() {
        let s = "hello world foo bar baz";
        // max_chars=11 → cut="hello world", rfind(' ')=5 → "hello…"
        let result = truncate_summary(s, 11);
        assert!(result.starts_with("hello"));
        assert!(result.ends_with('…'));
    }

    #[test]
    fn truncate_summary_handles_unicode() {
        // "café" is 4 chars but 5 bytes — truncating at 3 chars should not panic
        let s = "café is a nice place to sit and work";
        let result = truncate_summary(s, 3);
        assert!(!result.is_empty());
    }

    #[test]
    fn reference_card_adf_with_no_origin_url() {
        let body = build_reference_card_adf(
            "Unknown Source",
            "file:/tmp/unknown",
            None,
            None,
            "Intake",
            "2026-04-11",
            &[],
            &[],
        );
        assert!(body.contains("file:/tmp/unknown"));
        let parsed: serde_json::Value = serde_json::from_str(&body).expect("valid JSON");
        assert_eq!(parsed["type"], "doc");
    }
}
