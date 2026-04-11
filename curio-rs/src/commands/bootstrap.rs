use crate::output::emit_json;
use crate::{
    Result, config::Config, confluence::ConfluenceClient, harness::HarnessPaths,
    resolve_managed_root_folder_id, resolve_or_create_scoped_child_page_id,
};
use anyhow::Context;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use image::codecs::jpeg::JpegEncoder;
use image::imageops::FilterType;
use serde::Serialize;
use std::fmt::Write as _;
use std::fs;

const HERO_IMAGE_PATH: &str = "docs/assets/Curio_curated_intelligence_operator.png";
const OVERVIEW_TITLE: &str = "Curio Overview";

#[derive(Debug, Serialize)]
struct BootstrapPageOutput {
    title: String,
    page_id: Option<String>,
    purpose: String,
}

#[derive(Debug, Serialize)]
struct BootstrapOutput {
    root_folder_id: String,
    overview_page_id: Option<String>,
    pages: Vec<BootstrapPageOutput>,
}

struct PageSpec {
    title: &'static str,
    purpose: &'static str,
    body: String,
}

pub async fn run_bootstrap(config: &Config, dry_run: bool, json_output: bool) -> Result<()> {
    if !json_output {
        println!("Running bootstrap command...");
    }

    let paths = HarnessPaths::discover()?;
    let hero_image_html = build_hero_image_html(&paths);

    let auth_token = std::env::var("CURIO_CONFLUENCE_TOKEN")
        .context("CURIO_CONFLUENCE_TOKEN environment variable not set")?;

    let client = ConfluenceClient::new(
        config.connection.confluence_url.clone(),
        config.connection.confluence_email.clone(),
        auth_token,
        config.content_model.output_root_folder_id.clone(),
    )?;

    let space_key = &config.content_model.space_key;
    let root_folder_name = &config.content_model.root_folder_name;
    let root_folder_id = resolve_managed_root_folder_id(
        &client,
        space_key,
        root_folder_name,
        config.content_model.output_root_folder_id.as_deref(),
        json_output,
    )
    .await?;

    if !json_output {
        println!(
            "Using managed write root folder ID {} for space '{}'",
            root_folder_id, space_key
        );
    }

    let page_specs = lifecycle_page_specs();
    let mut ensured_pages = Vec::new();

    for spec in page_specs {
        if !json_output {
            println!("Checking for sub-page: '{}' under managed root", spec.title);
        }

        if dry_run {
            if !json_output {
                println!(
                    "(Dry run) Would ensure sub-page: '{}' under managed root",
                    spec.title
                );
            }
            ensured_pages.push(BootstrapPageOutput {
                title: spec.title.to_string(),
                page_id: None,
                purpose: spec.purpose.to_string(),
            });
            continue;
        }

        let page_id = resolve_or_create_scoped_child_page_id(
            &client,
            space_key,
            &root_folder_id,
            spec.title,
            &spec.body,
        )
        .await?;

        if !json_output {
            println!("Ensured sub-page '{}' with ID: {}", spec.title, page_id);
        }

        ensured_pages.push(BootstrapPageOutput {
            title: spec.title.to_string(),
            page_id: Some(page_id),
            purpose: spec.purpose.to_string(),
        });
    }

    let overview_body = render_overview_body(hero_image_html.as_deref());
    let overview_page_id = if dry_run {
        if !json_output {
            println!("(Dry run) Would ensure overview page: '{}'", OVERVIEW_TITLE);
        }
        None
    } else {
        let page_id = resolve_or_create_scoped_child_page_id(
            &client,
            space_key,
            &root_folder_id,
            OVERVIEW_TITLE,
            &overview_body,
        )
        .await?;

        if !json_output {
            println!(
                "Ensured overview page '{}' with ID: {}",
                OVERVIEW_TITLE, page_id
            );
        }

        Some(page_id)
    };

    let mut output_pages = ensured_pages;
    output_pages.push(BootstrapPageOutput {
        title: OVERVIEW_TITLE.to_string(),
        page_id: overview_page_id.clone(),
        purpose: "Curio landing page, visual identity, and navigation hub.".to_string(),
    });

    if json_output {
        emit_json(
            "bootstrap",
            true,
            BootstrapOutput {
                root_folder_id,
                overview_page_id,
                pages: output_pages,
            },
        )?;
    } else {
        println!("Bootstrap command finished.");
    }

    Ok(())
}

fn lifecycle_page_specs() -> Vec<PageSpec> {
    vec![
        PageSpec {
            title: "Intake",
            purpose: "Raw capture lane for newly ingested material.",
            body: render_lifecycle_page(
                "Intake",
                "Raw content enters here first. Curio uses Intake as the controlled entry point for new sources, unsorted notes, web captures, and file imports.",
                "Purpose",
                "Use this page for content that has just been captured and has not yet been confidently classified.",
                &[
                    "Incoming web captures, files, notes, and folders.",
                    "Source metadata, hashes, and dedupe markers.",
                    "Drafts that still need Curio to classify, route, or enrich them.",
                ],
                &[
                    "Final answers, polished narratives, or canonical published content.",
                    "Items that are already vetted and ready for human review.",
                ],
                &[
                    "Curio stamps intake items with `curio-status-intake` and keeps source provenance intact.",
                    "The page should read like a controlled intake queue, not a scratchpad.",
                    "If a source has a suggested subject, Curio uses it to name the page consistently.",
                ],
            ),
        },
        PageSpec {
            title: "Staged",
            purpose: "High-confidence content prepared for human review or publish.",
            body: render_lifecycle_page(
                "Staged",
                "Staged is the ready room. Content here has cleared the first pass, is conflict-free, and is waiting for a human or a higher-confidence workflow to approve the final move.",
                "Purpose",
                "Use this page for work that has been normalized and is close to publishable, but still benefits from review or final alignment.",
                &[
                    "Pages that passed automated analysis with high confidence.",
                    "Content that is structurally complete and easy to approve.",
                    "Material that should be reviewed before it becomes canonical.",
                ],
                &[
                    "Raw intake pages that still need triage.",
                    "Ambiguous or disputed content that belongs in Review.",
                ],
                &[
                    "Curio stamps staged items with `curio-status-staged`.",
                    "The page should feel calm, current, and ready for a decision.",
                    "Use Staged as the handoff point between automation and human approval.",
                ],
            ),
        },
        PageSpec {
            title: "Review",
            purpose: "Human arbitration lane for conflict, ambiguity, or risk.",
            body: render_lifecycle_page(
                "Review",
                "Review is where Curio stops and asks for judgment. This is the place for conflicts, low-confidence outputs, policy questions, or anything that should not move forward automatically.",
                "Purpose",
                "Use this page when Curio needs a human to resolve uncertainty, confirm intent, or decide between competing options.",
                &[
                    "Low-confidence analysis results.",
                    "Semantic collisions or duplicate detection concerns.",
                    "Content that needs a business decision or editorial correction.",
                ],
                &[
                    "Finished published content.",
                    "Mechanical intake artifacts that do not need human attention.",
                ],
                &[
                    "Curio stamps review items with `curio-status-review_required` and records the reason for escalation.",
                    "Treat this page like a decision queue, not a dumping ground.",
                    "A clear review outcome should move the item onward or close the loop with a documented rejection.",
                ],
            ),
        },
        PageSpec {
            title: "Published",
            purpose: "Canonical output surface for approved Curio content.",
            body: render_lifecycle_page(
                "Published",
                "Published is the source of truth. Once content lands here, it should read as intentional, stable, and reusable by people and agents alike.",
                "Purpose",
                "Use this page for finalized content that has passed approval and is ready to be treated as the canonical version.",
                &[
                    "Gold pages and finalized deliverables.",
                    "Outputs that downstream agents should prefer when searching for answers.",
                    "Approved content that should remain stable until a deliberate update is made.",
                ],
                &[
                    "Drafts, scraps, or unresolved discussions.",
                    "Content that still depends on active review or correction.",
                ],
                &[
                    "Curio stamps published content with `curio-status-published`.",
                    "Treat this area as the durable record, not a working area.",
                    "If a published item changes, the change should be traceable and intentional.",
                ],
            ),
        },
        PageSpec {
            title: "_templates",
            purpose: "Reusable structural templates and page blueprints.",
            body: render_lifecycle_page(
                "_templates",
                "Templates are the shape library for Curio. This area holds reusable structures that help agents generate consistent pages without reinventing the layout each time.",
                "Purpose",
                "Use this page for stable structural templates, boilerplate layouts, and form-factors that should be copied into new content.",
                &[
                    "Page blueprints and content scaffolds.",
                    "Reusable headings, sections, and lifecycle layouts.",
                    "Standard language that keeps the workspace consistent.",
                ],
                &[
                    "Live work items and active intake pages.",
                    "Published content that is meant to be the final answer.",
                ],
                &[
                    "Templates should be obvious, conservative, and easy for an agent to reuse safely.",
                    "Keep the content descriptive enough that a new agent knows when to start from it.",
                ],
            ),
        },
        PageSpec {
            title: "_registry",
            purpose: "Canonical topic registry and routing inventory.",
            body: render_lifecycle_page(
                "_registry",
                "The registry is Curio's map of what exists and where the canonical version lives. It keeps the workspace navigable and helps agents route requests to the right published target.",
                "Purpose",
                "Use this page for indexes, topic maps, canonical references, and pointers to published assets.",
                &[
                    "Subject-to-page mappings.",
                    "Registry records for published deliverables.",
                    "Operational references that help agents avoid duplicating work.",
                ],
                &[
                    "Narrative content that belongs in Published.",
                    "Working draft content that belongs in Intake, Staged, or Review.",
                ],
                &[
                    "A good registry entry should explain what the topic is, why it exists, and where the current authoritative page lives.",
                    "Registry content should be stable and easy to scan.",
                ],
            ),
        },
    ]
}

fn render_overview_body(hero_image_html: Option<&str>) -> String {
    let mut html = String::new();
    html.push_str("<h1>Curio Overview</h1>");
    html.push_str("<p><strong>Curated Intelligence Operator</strong> for Confluence-first knowledge operations.</p>");
    html.push_str("<p>Curio is the operating layer that turns Confluence into a structured agent workspace. It captures raw material, stages it for review, routes exceptions to humans, and preserves the published result as a reusable source of truth.</p>");

    if let Some(hero_image_html) = hero_image_html {
        html.push_str("<p>");
        html.push_str(hero_image_html);
        html.push_str("</p>");
    } else {
        html.push_str("<p><em>Hero artwork is managed from docs/assets/Curio_curated_intelligence_operator.png.</em></p>");
    }

    html.push_str("<h2>How This Space Works</h2>");
    html.push_str("<ul>");
    html.push_str("<li><strong>Intake</strong> is the raw capture lane for new material.</li>");
    html.push_str("<li><strong>Staged</strong> is the high-confidence handoff zone before final approval.</li>");
    html.push_str("<li><strong>Review</strong> holds conflict, ambiguity, and items that need human judgment.</li>");
    html.push_str(
        "<li><strong>Published</strong> is the canonical output surface for approved content.</li>",
    );
    html.push_str(
        "<li><strong>_templates</strong> holds reusable blueprints and page scaffolds.</li>",
    );
    html.push_str("<li><strong>_registry</strong> tracks canonical topics and where the authoritative page lives.</li>");
    html.push_str("</ul>");

    html.push_str("<h2>Operating Principles</h2>");
    html.push_str("<ol>");
    html.push_str("<li>Write only inside the managed Curio root.</li>");
    html.push_str("<li>Keep source provenance intact as content moves through the lifecycle.</li>");
    html.push_str("<li>Prefer clear page names, stable labels, and visible structure over implicit conventions.</li>");
    html.push_str("<li>Use the registry and published pages as the authoritative references for downstream agent work.</li>");
    html.push_str("</ol>");

    html.push_str("<h2>Agent Notes</h2>");
    html.push_str("<p>When agents land in this space, they should quickly understand what each lane is for, where to place new material, and where to look for the current answer. The documentation layer is part of the operating system, not an afterthought.</p>");

    html
}

fn render_lifecycle_page(
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

fn build_hero_image_html(paths: &HarnessPaths) -> Option<String> {
    let hero_path = paths.repo_root.join(HERO_IMAGE_PATH);
    let raw = fs::read(&hero_path).ok()?;

    let decoded = image::load_from_memory(&raw).ok()?;
    let resized = if decoded.width() > 1400 {
        decoded.resize(1400, u32::MAX, FilterType::Lanczos3)
    } else {
        decoded
    };

    let mut encoded = Vec::new();
    let mut encoder = JpegEncoder::new_with_quality(&mut encoded, 84);
    if encoder.encode_image(&resized).is_err() {
        return None;
    }

    let data = STANDARD.encode(encoded);
    Some(format!(
        "<img src=\"data:image/jpeg;base64,{}\" alt=\"Curio hero artwork\" style=\"max-width:100%;height:auto;border-radius:16px;display:block;margin:16px 0;\" />",
        data
    ))
}
