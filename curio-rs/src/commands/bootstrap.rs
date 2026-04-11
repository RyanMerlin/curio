use crate::curio_docs::{
    AUDIT_TITLE, AuditEntry, CurioCorePages, README_TITLE, REGISTRY_TITLE, RegistryRecord,
    TEMPLATES_TITLE, append_audit_entry, build_audit_root_body, build_lifecycle_page,
    build_readme_body, build_registry_root_body, build_template_page, build_templates_root_body,
    ensure_registry_record, ensure_scoped_page,
};
use crate::output::emit_json;
use crate::{Result, config::Config, confluence::ConfluenceClient, harness::HarnessPaths};
use anyhow::Context;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use chrono::Utc;
use image::codecs::jpeg::JpegEncoder;
use image::imageops::FilterType;
use serde::Serialize;
use std::fs;

const HERO_IMAGE_PATH: &str = "docs/assets/Curio_curated_intelligence_operator.png";

#[derive(Debug, Serialize)]
struct BootstrapPageOutput {
    title: String,
    page_id: Option<String>,
    purpose: String,
}

#[derive(Debug, Serialize)]
struct BootstrapOutput {
    root_folder_id: String,
    readme_page_id: Option<String>,
    templates_page_id: Option<String>,
    registry_page_id: Option<String>,
    audit_page_id: Option<String>,
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
    let readme_body = build_readme_body(hero_image_html.as_deref());
    let root_page_specs = root_page_specs(readme_body);
    let template_page_specs = template_page_specs();

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
    let root_folder_id = crate::resolve_managed_root_folder_id(
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

    let actor_email = config.connection.confluence_email.clone();

    let mut ensured_pages = Vec::new();
    let mut core_ids = CurioCorePages {
        readme_page_id: String::new(),
        intake_page_id: String::new(),
        staged_page_id: String::new(),
        review_page_id: String::new(),
        published_page_id: String::new(),
        templates_page_id: String::new(),
        registry_page_id: String::new(),
        audit_page_id: String::new(),
    };

    for spec in root_page_specs {
        if !json_output {
            println!("Checking for sub-page: '{}' under managed root", spec.title);
        }

        let page_id = if dry_run {
            if !json_output {
                println!(
                    "(Dry run) Would ensure sub-page: '{}' under managed root",
                    spec.title
                );
            }
            None
        } else {
            let page_id =
                ensure_scoped_page(&client, space_key, &root_folder_id, spec.title, &spec.body)
                    .await?;
            if !json_output {
                println!("Ensured sub-page '{}' with ID: {}", spec.title, page_id);
            }
            Some(page_id)
        };

        if spec.title == README_TITLE {
            core_ids.readme_page_id = page_id.clone().unwrap_or_default();
        } else if spec.title == "Intake" {
            core_ids.intake_page_id = page_id.clone().unwrap_or_default();
        } else if spec.title == "Staged" {
            core_ids.staged_page_id = page_id.clone().unwrap_or_default();
        } else if spec.title == "Review" {
            core_ids.review_page_id = page_id.clone().unwrap_or_default();
        } else if spec.title == "Published" {
            core_ids.published_page_id = page_id.clone().unwrap_or_default();
        } else if spec.title == TEMPLATES_TITLE {
            core_ids.templates_page_id = page_id.clone().unwrap_or_default();
        } else if spec.title == REGISTRY_TITLE {
            core_ids.registry_page_id = page_id.clone().unwrap_or_default();
        } else if spec.title == AUDIT_TITLE {
            core_ids.audit_page_id = page_id.clone().unwrap_or_default();
        }

        ensured_pages.push(BootstrapPageOutput {
            title: spec.title.to_string(),
            page_id,
            purpose: spec.purpose.to_string(),
        });
    }

    if !dry_run {
        if let Some(legacy_overview) = client
            .get_page_by_title(space_key, Some(&root_folder_id), "Curio Overview")
            .await?
        {
            let legacy_id = legacy_overview["id"].as_str().unwrap_or_default();
            if !legacy_id.is_empty() && legacy_id != core_ids.readme_page_id {
                if !json_output {
                    println!("Retiring legacy Curio Overview page {}", legacy_id);
                }
                if let Err(err) = client.delete_page(legacy_id).await {
                    if !json_output {
                        println!(
                            "Warning: unable to retire legacy Curio Overview page {}: {}",
                            legacy_id, err
                        );
                    }
                }
            }
        }
    }

    let templates_root_id = if dry_run {
        None
    } else {
        Some(core_ids.templates_page_id.clone())
    };

    let mut template_outputs = Vec::new();
    if let Some(templates_root_id) = templates_root_id.as_deref() {
        for spec in template_page_specs {
            if !json_output {
                println!("Checking for template page: '{}'", spec.title);
            }
            let page_id = ensure_scoped_page(
                &client,
                space_key,
                templates_root_id,
                spec.title,
                &spec.body,
            )
            .await?;
            if !json_output {
                println!(
                    "Ensured template page '{}' with ID: {}",
                    spec.title, page_id
                );
            }
            template_outputs.push(BootstrapPageOutput {
                title: spec.title.to_string(),
                page_id: Some(page_id),
                purpose: spec.purpose.to_string(),
            });
        }
    } else if dry_run {
        for spec in template_page_specs {
            template_outputs.push(BootstrapPageOutput {
                title: spec.title.to_string(),
                page_id: None,
                purpose: spec.purpose.to_string(),
            });
        }
    }

    if !dry_run {
        let mut registry_targets = vec![
            (
                README_TITLE,
                core_ids.readme_page_id.as_str(),
                root_folder_id.as_str(),
                "landing page",
                "human landing page and operational start point",
            ),
            (
                "Intake",
                core_ids.intake_page_id.as_str(),
                root_folder_id.as_str(),
                "lane",
                "raw content capture lane",
            ),
            (
                "Staged",
                core_ids.staged_page_id.as_str(),
                root_folder_id.as_str(),
                "lane",
                "content ready for review",
            ),
            (
                "Review",
                core_ids.review_page_id.as_str(),
                root_folder_id.as_str(),
                "lane",
                "human arbitration lane",
            ),
            (
                "Published",
                core_ids.published_page_id.as_str(),
                root_folder_id.as_str(),
                "lane",
                "canonical published lane",
            ),
            (
                TEMPLATES_TITLE,
                core_ids.templates_page_id.as_str(),
                root_folder_id.as_str(),
                "section",
                "template playbook root",
            ),
            (
                REGISTRY_TITLE,
                core_ids.registry_page_id.as_str(),
                root_folder_id.as_str(),
                "section",
                "master index root",
            ),
            (
                AUDIT_TITLE,
                core_ids.audit_page_id.as_str(),
                root_folder_id.as_str(),
                "section",
                "append-only audit root",
            ),
        ];

        for (key, page_id, parent_id, item_type, summary) in registry_targets.drain(..) {
            if page_id.is_empty() {
                continue;
            }
            let record = RegistryRecord {
                key: key.to_string(),
                item_type: item_type.to_string(),
                title: key.to_string(),
                page_id: page_id.to_string(),
                parent_id: parent_id.to_string(),
                status: "structural".to_string(),
                source_id: "bootstrap".to_string(),
                summary: summary.to_string(),
                updated_at: Utc::now().to_rfc3339(),
            };
            let _ = ensure_registry_record(&client, space_key, &core_ids.registry_page_id, &record)
                .await?;
        }

        for spec in &template_outputs {
            if let Some(page_id) = spec.page_id.as_deref() {
                let record = RegistryRecord {
                    key: page_id.to_string(),
                    item_type: "template".to_string(),
                    title: spec.title.clone(),
                    page_id: page_id.to_string(),
                    parent_id: core_ids.templates_page_id.clone(),
                    status: "template".to_string(),
                    source_id: "bootstrap".to_string(),
                    summary: spec.purpose.clone(),
                    updated_at: Utc::now().to_rfc3339(),
                };
                let _ =
                    ensure_registry_record(&client, space_key, &core_ids.registry_page_id, &record)
                        .await?;
            }
        }

        let audit_entry = AuditEntry {
            actor: actor_email,
            command: "bootstrap".to_string(),
            subject: "Curio root structure".to_string(),
            action: "Ensured landing page, lifecycle pages, template playbook, registry, and audit roots".to_string(),
            rationale: "Keep Confluence as the indexed datastore with visible structure for humans and agents".to_string(),
            source: format!("root_folder_id={}", root_folder_id),
            result: "completed".to_string(),
            detail_lines: vec![
                format!("README page id: {}", core_ids.readme_page_id),
                format!("Intake page id: {}", core_ids.intake_page_id),
                format!("Staged page id: {}", core_ids.staged_page_id),
                format!("Review page id: {}", core_ids.review_page_id),
                format!("Published page id: {}", core_ids.published_page_id),
                format!("Templates page id: {}", core_ids.templates_page_id),
                format!("Registry page id: {}", core_ids.registry_page_id),
                format!("Audit page id: {}", core_ids.audit_page_id),
            ],
        };
        let _ =
            append_audit_entry(&client, space_key, &core_ids.audit_page_id, &audit_entry).await?;
    }

    let mut output_pages = ensured_pages;
    output_pages.extend(template_outputs);

    let landing_page_id = if core_ids.readme_page_id.is_empty() {
        None
    } else {
        Some(core_ids.readme_page_id.clone())
    };
    let templates_page_id = if core_ids.templates_page_id.is_empty() {
        None
    } else {
        Some(core_ids.templates_page_id.clone())
    };
    let registry_page_id = if core_ids.registry_page_id.is_empty() {
        None
    } else {
        Some(core_ids.registry_page_id.clone())
    };
    let audit_page_id = if core_ids.audit_page_id.is_empty() {
        None
    } else {
        Some(core_ids.audit_page_id.clone())
    };

    if json_output {
        emit_json(
            "bootstrap",
            true,
            BootstrapOutput {
                root_folder_id,
                readme_page_id: landing_page_id,
                templates_page_id,
                registry_page_id,
                audit_page_id,
                pages: output_pages,
            },
        )?;
    } else {
        println!("Bootstrap command finished.");
    }

    Ok(())
}

fn root_page_specs(readme_body: String) -> Vec<PageSpec> {
    vec![
        PageSpec {
            title: README_TITLE,
            purpose: "Plain-English starting point for non-technical users.",
            body: readme_body,
        },
        PageSpec {
            title: "Intake",
            purpose: "Raw capture lane for newly ingested material.",
            body: build_lifecycle_page(
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
            body: build_lifecycle_page(
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
            body: build_lifecycle_page(
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
            body: build_lifecycle_page(
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
            title: TEMPLATES_TITLE,
            purpose: "Reusable structural templates and page blueprints.",
            body: build_templates_root_body(),
        },
        PageSpec {
            title: REGISTRY_TITLE,
            purpose: "Canonical topic registry and routing inventory.",
            body: build_registry_root_body(),
        },
        PageSpec {
            title: AUDIT_TITLE,
            purpose: "Append-only audit log for Curio actions and decisions.",
            body: build_audit_root_body(),
        },
    ]
}

fn template_page_specs() -> Vec<PageSpec> {
    vec![
        PageSpec {
            title: "Template - Intake Page",
            purpose: "Copy this when you need a new intake-shaped capture page.",
            body: build_template_page(
                "Template - Intake Page",
                "Use this for a page that is capturing raw information before it has been fully analyzed.",
                "Sales, operations, and agents that need a landing spot for fresh material.",
                &[
                    "New notes and meeting captures.",
                    "Source snippets that still need classification.",
                    "Content that should be routed into Curio instead of being answered immediately.",
                ],
                &[
                    "Title",
                    "Source",
                    "Short summary",
                    "Attached artifacts or links",
                    "Next-step notes",
                ],
            ),
        },
        PageSpec {
            title: "Template - Staged Page",
            purpose: "Copy this when content is mostly ready and needs human approval.",
            body: build_template_page(
                "Template - Staged Page",
                "Use this for content that has already been cleaned up and is ready for a final decision.",
                "People who need a working page before publishing.",
                &[
                    "High-confidence drafts.",
                    "Content that is structurally complete.",
                    "Items waiting for a final review pass.",
                ],
                &[
                    "Approved summary",
                    "Open questions",
                    "Risks or exclusions",
                    "Publish-ready copy",
                ],
            ),
        },
        PageSpec {
            title: "Template - Review Page",
            purpose: "Copy this when a human needs to arbitrate a decision.",
            body: build_template_page(
                "Template - Review Page",
                "Use this for conflict, ambiguity, or policy decisions that should not be automated.",
                "Reviewers, managers, and subject-matter experts.",
                &[
                    "Competing interpretations.",
                    "Low-confidence outputs.",
                    "Items needing a business decision.",
                ],
                &[
                    "Problem statement",
                    "What Curio saw",
                    "Why the item was escalated",
                    "Decision requested",
                    "Outcome",
                ],
            ),
        },
        PageSpec {
            title: "Template - Published Page",
            purpose: "Copy this when you need a durable published answer.",
            body: build_template_page(
                "Template - Published Page",
                "Use this for final, approved content that other pages should reference.",
                "Anyone who needs a stable answer or deliverable.",
                &[
                    "Canonical answers.",
                    "Approved deliverables.",
                    "Final versions that should not change casually.",
                ],
                &[
                    "Executive summary",
                    "Final answer",
                    "Supporting context",
                    "Source lineage",
                    "Last reviewed date",
                ],
            ),
        },
        PageSpec {
            title: "Template - Registry Record",
            purpose: "Copy this when Curio needs a structured master-record page.",
            body: build_template_page(
                "Template - Registry Record",
                "Use this as the canonical record format for a page, artifact, or operational entity.",
                "Curio operators and administrators.",
                &[
                    "Any page that should be tracked as a record.",
                    "Any artifact that needs a current status and source trail.",
                ],
                &[
                    "Key",
                    "Title",
                    "Type",
                    "Status",
                    "Source",
                    "Current parent",
                    "Updated at",
                    "Summary",
                ],
            ),
        },
        PageSpec {
            title: "Template - Audit Entry",
            purpose: "Copy this when Curio needs an append-only event log page.",
            body: build_template_page(
                "Template - Audit Entry",
                "Use this as the durable history format for a Curio action.",
                "Operators, reviewers, and future Curio runs.",
                &[
                    "Any meaningful write action.",
                    "Any decision that should be explained later.",
                ],
                &[
                    "Timestamp",
                    "Command",
                    "Actor",
                    "Source used",
                    "Rationale",
                    "Result",
                    "Details",
                ],
            ),
        },
    ]
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
