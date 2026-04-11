use crate::{Result, confluence::ConfluenceClient};
use chrono::Utc;
use std::fmt::Write as _;

pub const README_TITLE: &str = "README";
pub const TEMPLATES_TITLE: &str = "_templates";
pub const REGISTRY_TITLE: &str = "_registry";
pub const AUDIT_TITLE: &str = "_audit";

pub struct CurioCorePages {
    pub readme_page_id: String,
    pub intake_page_id: String,
    pub staged_page_id: String,
    pub review_page_id: String,
    pub published_page_id: String,
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

pub async fn ensure_scoped_page(
    client: &ConfluenceClient,
    space_key: &str,
    parent_id: &str,
    title: &str,
    body: &str,
) -> Result<String> {
    let page_id =
        crate::resolve_or_create_scoped_child_page_id(client, space_key, parent_id, title, body)
            .await?;
    Ok(page_id)
}

pub async fn ensure_registry_record(
    client: &ConfluenceClient,
    space_key: &str,
    registry_root_id: &str,
    record: &RegistryRecord,
) -> Result<String> {
    let body = build_registry_record_body(record);
    let title = format!("Record - {}", record.key);
    client
        .create_or_update_page(space_key, Some(registry_root_id), &title, "storage", &body)
        .await
}

pub async fn append_audit_entry(
    client: &ConfluenceClient,
    space_key: &str,
    audit_root_id: &str,
    entry: &AuditEntry,
) -> Result<String> {
    let stamp = Utc::now().format("%Y%m%dT%H%M%S%.3fZ").to_string();
    let title = format!("Audit - {} - {}", stamp, entry.command);
    let body = build_audit_entry_body(entry, &stamp);
    client
        .create_or_update_page(space_key, Some(audit_root_id), &title, "storage", &body)
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
            "_templates holds reusable page patterns.",
            "_registry is the authoritative catalog of Curio records.",
            "_audit is the append-only history of Curio actions.",
        ],
    );

    append_section(
        &mut html,
        "What To Do First",
        Some("If you are new here, use the simplest lane that fits the work."),
        &[
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

pub fn build_templates_root_body() -> String {
    let mut html = String::new();
    html.push_str("<h1>_templates</h1>");
    html.push_str("<p>This area holds Curio's reusable page patterns. If Curio creates something repeatedly, the shape should live here first.</p>");
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
    html.push_str("<p>The registry is Curio's master index. Every meaningful Curio page should be represented here as a record page.</p>");
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
    write_row(&mut html, "Summary", &record.summary);
    write_row(&mut html, "Updated At", &record.updated_at);
    html.push_str("</table>");
    html
}

pub fn build_audit_root_body() -> String {
    let mut html = String::new();
    html.push_str("<h1>_audit</h1>");
    html.push_str("<p>The audit log is append-only. It records what Curio did, what it used, and why it made the decision.</p>");
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
