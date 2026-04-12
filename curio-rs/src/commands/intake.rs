use crate::confluence::http_timeout_duration;
use crate::curio_docs::{
    ADMIN_TITLE, AUDIT_TITLE, AuditEntry, RegistryRecord, append_audit_entry, audit_bucket_path,
    build_admin_root_body, build_audit_root_body, build_registry_root_body, ensure_registry_record,
    ensure_scoped_page, ensure_scoped_structure_page, extract_page_text, infer_route_plan,
};
use crate::output::emit_json;
use crate::{
    ChangeProposal, Result, compact_change_proposal, config::Config, confluence::ConfluenceClient,
    generate_change_proposal_with_agent,
};
use anyhow::Context;
use chrono::Utc;
use scraper::{Html, Selector};
use serde::Serialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Serialize)]
struct IntakeOutput {
    source_items: usize,
    handled_items: usize,
    duplicate_skipped: usize,
    skipped_unavailable: usize,
    failed_items: usize,
    dry_run: bool,
}

pub async fn run_intake_create(
    config: &Config,
    dry_run: bool,
    json_output: bool,
    url: &Option<String>,
    file: &Option<PathBuf>,
    folder: &Option<PathBuf>,
    subject_hint: &Option<String>,
    metadata_str: &Option<String>,
) -> Result<()> {
    if !json_output {
        println!("Running intake create command...");
    }

    let auth_token = std::env::var("CURIO_CONFLUENCE_TOKEN")
        .context("CURIO_CONFLUENCE_TOKEN environment variable not set")?;

    let client = ConfluenceClient::new(
        config.connection.confluence_url.clone(),
        config.connection.confluence_email.clone(),
        auth_token,
        config.content_model.space_key.clone(),
        None,
    )?;

    let space_key = &config.content_model.space_key;
    let intake_page_id = ensure_scoped_structure_page(
        &client,
        space_key,
        "",
        "Intake",
        "<p>This page holds raw, unprocessed content that has been ingested.</p>",
    )
    .await?;
    let admin_page_id = ensure_scoped_structure_page(
        &client,
        space_key,
        "",
        ADMIN_TITLE,
        &build_admin_root_body(),
    )
    .await?;
    let registry_root_id = ensure_scoped_page(
        &client,
        space_key,
        &admin_page_id,
        "_registry",
        &build_registry_root_body(),
    )
    .await?;
    let audit_root_id = ensure_scoped_page(
        &client,
        space_key,
        &admin_page_id,
        AUDIT_TITLE,
        &build_audit_root_body(),
    )
    .await?;

    let mut all_content: Vec<(String, String, String, Option<String>)> = Vec::new();
    let mut source_items = 0usize;
    let mut skipped_unavailable = 0usize;
    // (content, source_id, content_type, subject_hint)

    if let Some(u) = url {
        if let Some(folder_id) = extract_confluence_folder_id(u) {
            if !json_output {
                println!("Ingesting from Confluence folder: {}", u);
            }

            let folder = client
                .get_folder_by_id_v2(&folder_id)
                .await?
                .context(format!("Confluence folder not found: {}", folder_id))?;
            let folder_title = folder["title"].as_str().unwrap_or(&folder_id).to_string();
            let descendants = client.get_folder_descendants_v2(&folder_id).await?;

            if !json_output {
                println!(
                    "Found {} descendant items under folder '{}'",
                    descendants.len(),
                    folder_title
                );
            }

            for descendant in descendants {
                let content_type = descendant["type"].as_str().unwrap_or_default();
                if content_type != "page" {
                    continue;
                }
                source_items += 1;

                let page_id = descendant["id"]
                    .as_str()
                    .context("Folder descendant page is missing an ID")?;
                let page = match client.get_page_by_id_with_body_v1(page_id).await {
                    Ok(Some(page)) => page,
                    Ok(None) => {
                        skipped_unavailable += 1;
                        if !json_output {
                            println!(
                                "  - Skipping inaccessible descendant page {} (not found)",
                                page_id
                            );
                        }
                        continue;
                    }
                    Err(err) => {
                        skipped_unavailable += 1;
                        if !json_output {
                            println!(
                                "  - Skipping descendant page {} because it could not be loaded: {}",
                                page_id, err
                            );
                        }
                        continue;
                    }
                };

                let page_title = page["title"]
                    .as_str()
                    .or_else(|| descendant["title"].as_str())
                    .unwrap_or("Untitled Confluence Page")
                    .to_string();
                let content = extract_page_text(&page);

                if !json_output {
                    println!("  - Queuing page from folder: {} ({})", page_title, page_id);
                }

                all_content.push((
                    content,
                    format!("confluence-folder:{}:page:{}", folder_id, page_id),
                    "confluence_page".to_string(),
                    Some(page_title),
                ));
            }
        } else if let Some(page_id) = extract_confluence_page_id_from_url(u) {
            if !json_output {
                println!("Ingesting from Confluence page tree: {}", u);
            }

            let root_page = client
                .get_page_by_id_with_body_v1(&page_id)
                .await?
                .context(format!("Confluence page not found: {}", page_id))?;
            let root_title = root_page["title"].as_str().unwrap_or(&page_id).to_string();
            let root_content = extract_page_text(&root_page);

            if !json_output {
                println!("  - Queuing page from tree: {} ({})", root_title, page_id);
            }
            source_items += 1;
            all_content.push((
                root_content,
                format!("confluence-page:{}:page:{}", page_id, page_id),
                "confluence_page".to_string(),
                Some(root_title.clone()),
            ));

            let descendants = client.get_page_descendants_v2(&page_id).await?;
            if !json_output {
                println!(
                    "Found {} descendant items under page '{}'",
                    descendants.len(),
                    root_title
                );
            }

            for descendant in descendants {
                let content_type = descendant["type"].as_str().unwrap_or_default();
                if content_type != "page" {
                    continue;
                }
                source_items += 1;

                let child_page_id = descendant["id"]
                    .as_str()
                    .context("Page descendant is missing an ID")?;
                let page = match client.get_page_by_id_with_body_v1(child_page_id).await {
                    Ok(Some(page)) => page,
                    Ok(None) => {
                        skipped_unavailable += 1;
                        if !json_output {
                            println!(
                                "  - Skipping inaccessible descendant page {} (not found)",
                                child_page_id
                            );
                        }
                        continue;
                    }
                    Err(err) => {
                        skipped_unavailable += 1;
                        if !json_output {
                            println!(
                                "  - Skipping descendant page {} because it could not be loaded: {}",
                                child_page_id, err
                            );
                        }
                        continue;
                    }
                };

                let page_title = page["title"]
                    .as_str()
                    .or_else(|| descendant["title"].as_str())
                    .unwrap_or("Untitled Confluence Page")
                    .to_string();
                let content = extract_page_text(&page);

                if !json_output {
                    println!(
                        "  - Queuing descendant page from tree: {} ({})",
                        page_title, child_page_id
                    );
                }

                all_content.push((
                    content,
                    format!("confluence-page:{}:page:{}", page_id, child_page_id),
                    "confluence_page".to_string(),
                    Some(page_title),
                ));
            }
        } else if let Some(space_key) = extract_confluence_space_key_from_url(u) {
            if !json_output {
                println!("Ingesting from Confluence space: {}", space_key);
            }
            let pages = client
                .execute_cql(&format!(
                    "space = \"{}\" AND type = page ORDER BY created ASC",
                    space_key
                ))
                .await?;
            source_items = pages.len();
            for page in pages {
                let page_id = page["id"].as_str().context("Space page is missing an ID")?;
                let loaded = match client.get_page_by_id_with_body_v1(page_id).await {
                    Ok(Some(page)) => page,
                    Ok(None) => {
                        skipped_unavailable += 1;
                        if !json_output {
                            println!(
                                "  - Skipping inaccessible space page {} (not found)",
                                page_id
                            );
                        }
                        continue;
                    }
                    Err(err) => {
                        skipped_unavailable += 1;
                        if !json_output {
                            println!(
                                "  - Skipping space page {} because it could not be loaded: {}",
                                page_id, err
                            );
                        }
                        continue;
                    }
                };

                let page_title = loaded["title"]
                    .as_str()
                    .or_else(|| page["title"].as_str())
                    .unwrap_or("Untitled Confluence Page")
                    .to_string();
                let content = extract_page_text(&loaded);

                if !json_output {
                    println!("  - Queuing page from space: {} ({})", page_title, page_id);
                }

                all_content.push((
                    content,
                    format!("confluence-space:{}:page:{}", space_key, page_id),
                    "confluence_page".to_string(),
                    Some(page_title),
                ));
            }
        } else {
            if !json_output {
                println!("Ingesting from URL: {}", u);
            }
            let content = fetch_url_content(u).await?;
            source_items += 1;
            all_content.push((
                content,
                format!("url:{}", u),
                "web_page".to_string(),
                subject_hint.clone(),
            ));
        }
    } else if let Some(f) = file {
        if !json_output {
            println!("Ingesting from file: {}", f.display());
        }
        let content = read_file_content(f).await?;
        source_items += 1;
        let filename = f
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown_file");
        let derived_subject_hint = f.file_stem().and_then(|name| name.to_str()).map(|name| {
            name.split_once("__")
                .map(|(_, subject)| subject.to_string())
                .unwrap_or_else(|| name.to_string())
        });
        all_content.push((
            content,
            format!("file:{}", f.display()),
            get_mime_type(filename),
            derived_subject_hint.or_else(|| subject_hint.clone()),
        ));
    } else if let Some(f) = folder {
        println!("Ingesting from folder: {}", f.display());
        for entry in WalkDir::new(f).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                let path = entry.path();
                if !json_output {
                    println!("  - Ingesting file: {}", path.display());
                }
                let content = read_file_content(path).await?;
                source_items += 1;
                let filename = path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("unknown_file");
                let derived_subject_hint =
                    path.file_stem().and_then(|name| name.to_str()).map(|name| {
                        name.split_once("__")
                            .map(|(_, subject)| subject.to_string())
                            .unwrap_or_else(|| name.to_string())
                    });
                all_content.push((
                    content,
                    format!("file:{}", path.display()),
                    get_mime_type(filename),
                    derived_subject_hint.or_else(|| subject_hint.clone()),
                ));
            }
        }
    } else {
        anyhow::bail!("No input source provided. Use --url, --file, or --folder.");
    }

    let mut handled_items = 0usize;
    let mut duplicate_skipped = 0usize;
    let mut failed_items = 0usize;

    for (content, source_id, content_type, item_subject_hint) in all_content {
        let effective_subject_hint = item_subject_hint.or_else(|| subject_hint.clone());
        match process_single_content(
            &client,
            config,
            dry_run,
            json_output,
            &intake_page_id,
            &registry_root_id,
            &audit_root_id,
            &content,
            &source_id,
            &content_type,
            &effective_subject_hint,
            metadata_str,
        )
        .await
        {
            Ok(true) => handled_items += 1,
            Ok(false) => duplicate_skipped += 1,
            Err(err) => {
                failed_items += 1;
                if !json_output {
                    eprintln!(
                        "Skipping source {} after intake write failure: {}",
                        source_id, err
                    );
                }
            }
        }
    }

    if json_output {
        emit_json(
            "intake-create",
            true,
            IntakeOutput {
                source_items,
                handled_items,
                duplicate_skipped,
                skipped_unavailable,
                failed_items,
                dry_run,
            },
        )?;
    } else {
        println!("Intake create command finished.");
    }
    Ok(())
}

async fn fetch_url_content(url: &str) -> Result<String> {
    let client = reqwest::Client::builder()
        .timeout(http_timeout_duration())
        .connect_timeout(std::time::Duration::from_secs(15))
        .build()
        .context("Failed to build URL fetch client")?;

    let response = client
        .get(url)
        .send()
        .await
        .context(format!("Failed to fetch URL: {}", url))?;
    let status = response.status();
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("text/plain")
        .to_string(); // Convert to owned String

    let text = response
        .text()
        .await
        .context("Failed to read response body from URL")?;

    if status.is_success() {
        // Simple HTML parsing for now
        if content_type.contains("text/html") {
            let document = Html::parse_document(&text);
            let selector = Selector::parse("body").unwrap(); // Extract text from body
            if let Some(body) = document.select(&selector).next() {
                Ok(body.text().collect::<Vec<_>>().join(" "))
            } else {
                Ok(text) // Fallback to full HTML if body not found
            }
        } else {
            Ok(text)
        }
    } else {
        anyhow::bail!("Failed to fetch URL {} with status: {}", url, status);
    }
}

async fn read_file_content(path: &Path) -> Result<String> {
    let content = std::fs::read_to_string(path)
        .context(format!("Failed to read file: {}", path.display()))?;
    Ok(content
        .strip_prefix('\u{feff}')
        .unwrap_or(&content)
        .to_string())
}

fn get_mime_type(filename: &str) -> String {
    // Basic mime type detection based on extension
    if filename.ends_with(".html") || filename.ends_with(".htm") {
        "text/html".to_string()
    } else if filename.ends_with(".md") {
        "text/markdown".to_string()
    } else if filename.ends_with(".json") {
        "application/json".to_string()
    } else if filename.ends_with(".txt") {
        "text/plain".to_string()
    } else {
        "application/octet-stream".to_string()
    }
}

async fn process_single_content(
    client: &ConfluenceClient,
    config: &Config,
    dry_run: bool,
    json_output: bool,
    intake_page_id: &str,
    registry_root_id: &str,
    audit_root_id: &str,
    content: &str,
    source_id: &str,
    content_type: &str,
    subject_hint: &Option<String>,
    metadata_str: &Option<String>,
) -> Result<bool> {
    let source_hash = format!("{:x}", Sha256::digest(content.as_bytes()));
    let dedupe_key = format!("{}:{}", content_type, &source_hash);

    let current_datetime = Utc::now().format("%Y%m%d%H%M%S").to_string();
    let subject_key = subject_hint
        .as_ref()
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            // Basic heuristic: first few words of content or source ID
            content
                .split_whitespace()
                .take(5)
                .collect::<Vec<&str>>()
                .join(" ")
        });

    let page_title = if !subject_key.is_empty() {
        subject_key.clone()
    } else {
        format!(
            "[{}] {} - {}",
            content_type.to_uppercase(),
            source_id,
            current_datetime
        )
    };

    let label_namespace = &config.content_model.label_namespace;

    // 1. Check for Duplicates
    let cql_dedupe = format!(
        "label = \"{}::dedupe-key::{}\"",
        label_namespace, dedupe_key
    );
    if !json_output {
        println!("Checking for duplicates with CQL: {}", cql_dedupe);
    }
    let existing_pages = client.execute_cql(&cql_dedupe).await?;

    if !existing_pages.is_empty() {
        if !json_output {
            println!(
                "Skipping ingestion: Duplicate content found with dedupe key '{}'. Existing page IDs: {:?}",
                dedupe_key,
                existing_pages
                    .iter()
                    .filter_map(|p| p["id"].as_str())
                    .collect::<Vec<&str>>()
            );
        }
        return Ok(false);
    }

    // 2. Create Page
    let page_id = if dry_run {
        if !json_output {
            println!("(Dry run) Would create page: '{}' under Intake", page_title);
        }
        "dry_run_page_id".to_string()
    } else {
        if !json_output {
            println!("Creating page: '{}' under Intake", page_title);
        }
        create_scoped_intake_page(
            client,
            &config.content_model.space_key,
            intake_page_id,
            &page_title,
            content,
            source_id,
            json_output,
        )
        .await?
    };

    // 3. Set Metadata
    let mut curio_metadata = json!({
        "curio_version": "1.0",
        "source_id": source_id,
        "source_hash": source_hash,
        "dedupe_key": dedupe_key,
        "subject_key": subject_key,
        "status": "intake",
        "ingested_at": Utc::now().to_rfc3339(),
        "source_metadata": {
            "type": content_type,
        }
    });

    if let Some(meta_str) = metadata_str {
        let user_metadata: serde_json::Value =
            serde_json::from_str(meta_str).context("Failed to parse --metadata JSON string")?;
        // Merge user provided metadata, user metadata will overwrite default fields
        if let Some(obj) = curio_metadata.as_object_mut() {
            if let Some(user_obj) = user_metadata.as_object() {
                for (k, v) in user_obj {
                    obj.insert(k.clone(), v.clone());
                }
            }
        }
    }

    let route_hint_value = curio_metadata.clone();
    let route_plan = infer_route_plan(Some(&page_title), content, Some(&route_hint_value));
    let mut proposal: ChangeProposal = generate_change_proposal_with_agent(
        Some(&page_title),
        content,
        &[source_id.to_string()],
        Some(&route_hint_value),
    )
    .await?;
    proposal.target_path = route_plan.target_path.clone();
    proposal.registry_path = route_plan.registry_path.clone();
    proposal.rationale = Some(route_plan.rationale.clone());
    proposal.validation_requirements = route_plan.validation_requirements.clone();
    proposal.sibling_context = route_plan.sibling_context.clone();
    proposal.model_used = Some("curio-route-plan-v1".to_string());
    curio_metadata["change_proposal"] = compact_change_proposal(&proposal);

    if !dry_run {
        if !json_output {
            println!("Setting curio_metadata for page {}", page_id);
        }
        client
            .set_content_property(&page_id, "curio_metadata", curio_metadata)
            .await?;
    } else {
        if !json_output {
            println!(
                "(Dry run) Would set curio_metadata for page {}: {}",
                page_id,
                curio_metadata.to_string()
            );
        }
    }

    // 4. Add Labels
    let mut labels = vec![
        format!("{}-status-intake", label_namespace),
        format!(
            "{}-type-{}",
            label_namespace,
            content_type.replace("/", "-").replace(".", "-")
        ), // Normalize type for labels
        format!("{}-dedupe-key-{}", label_namespace, dedupe_key),
    ];
    if !subject_key.is_empty() {
        let sanitized_subject_key = sanitize_label_segment(&subject_key);
        if !sanitized_subject_key.is_empty() {
            labels.push(format!(
                "{}-subject-key-{}",
                label_namespace, sanitized_subject_key
            ));
        }
    }

    if !dry_run {
        if !json_output {
            println!("Adding labels to page {}: {:?}", page_id, labels);
        }
        client.add_labels(&page_id, labels).await?;

        let registry_record = RegistryRecord {
            key: page_id.clone(),
            item_type: content_type.to_string(),
            title: page_title.clone(),
            page_id: page_id.clone(),
            parent_id: intake_page_id.to_string(),
            status: "intake".to_string(),
            source_id: source_id.to_string(),
            summary: format!(
                "Created from {} and routed to Intake with subject '{}'",
                source_id, subject_key
            ),
            updated_at: Utc::now().to_rfc3339(),
        };
        ensure_registry_record(
            client,
            &config.content_model.space_key,
            registry_root_id,
            &proposal.registry_path,
            &registry_record,
        )
        .await?;

        let audit_entry = AuditEntry {
            actor: config.connection.confluence_email.clone(),
            command: "intake-create".to_string(),
            subject: page_title.clone(),
            action: "Created intake page and indexed it in Curio".to_string(),
            rationale: format!("Capture source {} for Curio processing", source_id),
            source: source_id.to_string(),
            result: "intake".to_string(),
            detail_lines: vec![
                format!("Page ID: {}", page_id),
                format!("Content type: {}", content_type),
                format!("Subject key: {}", subject_key),
                format!("Dedupe key: {}", dedupe_key),
            ],
        };
        let audit_stamp = Utc::now().format("%Y%m%dT%H%M%S%.3fZ").to_string();
        append_audit_entry(
            client,
            &config.content_model.space_key,
            audit_root_id,
            &audit_bucket_path(&audit_stamp),
            &audit_entry,
        )
        .await?;
    } else {
        if !json_output {
            println!(
                "(Dry run) Would add labels to page {}: {:?}",
                page_id, labels
            );
        }
    }

    if !json_output {
        println!("Successfully processed content from source: {}", source_id);
    }
    Ok(true)
}

async fn create_scoped_intake_page(
    client: &ConfluenceClient,
    space_key: &str,
    intake_page_id: &str,
    page_title: &str,
    content: &str,
    source_id: &str,
    json_output: bool,
) -> Result<String> {
    let body = build_safe_intake_body(page_title, source_id, content);
    match client
        .create_or_update_page(
            space_key,
            Some(intake_page_id),
            page_title,
            "atlas_doc_format",
            &body,
        )
        .await
    {
        Ok(page_id) => Ok(page_id),
        Err(err) => {
            let err_text = err.to_string();
            if err_text.contains("same TITLE in this space") {
                let unique_title = build_unique_intake_title(page_title, source_id);
                if !json_output {
                    println!(
                        "Title collision detected for '{}'; retrying with unique title '{}'",
                        page_title, unique_title
                    );
                }
                match client
                    .create_or_update_page(
                        space_key,
                        Some(intake_page_id),
                        &unique_title,
                        "atlas_doc_format",
                        &build_safe_intake_body(&unique_title, source_id, content),
                    )
                    .await
                {
                    Ok(page_id) => return Ok(page_id),
                    Err(unique_err) => {
                        let fallback_body =
                            build_safe_intake_body(&unique_title, source_id, content);
                        if !json_output {
                            println!(
                                "Primary and unique-title writes failed for '{}'; retrying with sanitized fallback body: {}",
                                unique_title, unique_err
                            );
                        }
                        return client
                            .create_or_update_page(
                                space_key,
                                Some(intake_page_id),
                                &unique_title,
                                "atlas_doc_format",
                                &fallback_body,
                            )
                            .await
                            .with_context(|| {
                                format!(
                                    "Failed to create intake page '{}' from source {} even after unique-title fallback and body sanitization",
                                    page_title, source_id
                                )
                            });
                    }
                }
            }
            let fallback_body = build_safe_intake_body(page_title, source_id, content);
            if !json_output {
                println!(
                    "Primary page write failed for '{}'; retrying with sanitized fallback body: {}",
                    page_title, err
                );
            }
            client
                .create_or_update_page(
                    space_key,
                    Some(intake_page_id),
                    page_title,
                    "atlas_doc_format",
                    &fallback_body,
                )
                .await
                .with_context(|| {
                    format!(
                        "Failed to create intake page '{}' from source {} even after fallback sanitization",
                        page_title, source_id
                    )
                })
        }
    }
}

fn build_safe_intake_body(page_title: &str, source_id: &str, content: &str) -> String {
    let excerpt_lines = content.lines().take(120).collect::<Vec<_>>();
    let excerpt_blocks = excerpt_lines
        .into_iter()
        .map(adf_text_line)
        .collect::<Vec<_>>();

    let adf = json!({
        "type": "doc",
        "version": 1,
        "content": [
            adf_heading(1, page_title),
            adf_paragraph_text(&format!("Source: {}", source_id)),
            adf_paragraph_text("Curio stored a sanitized intake copy because the original page body could not be written cleanly."),
            adf_heading(2, "Captured Text"),
            adf_paragraph_from_lines(excerpt_blocks),
        ]
    });

    serde_json::to_string(&adf).unwrap_or_else(|_| {
        json!({
            "type": "doc",
            "version": 1,
            "content": [
                adf_heading(1, page_title),
                adf_paragraph_text(&format!("Source: {}", source_id))
            ]
        })
        .to_string()
    })
}

fn adf_heading(level: u8, text: &str) -> serde_json::Value {
    json!({
        "type": "heading",
        "attrs": { "level": level },
        "content": [adf_text(text)]
    })
}

fn adf_paragraph_text(text: &str) -> serde_json::Value {
    json!({
        "type": "paragraph",
        "content": [adf_text(text)]
    })
}

fn adf_paragraph_from_lines(lines: Vec<serde_json::Value>) -> serde_json::Value {
    let mut content = Vec::new();
    for (index, line) in lines.into_iter().enumerate() {
        if index > 0 {
            content.push(json!({ "type": "hardBreak" }));
        }
        content.push(line);
    }
    json!({
        "type": "paragraph",
        "content": content
    })
}

fn adf_text(text: &str) -> serde_json::Value {
    json!({
        "type": "text",
        "text": text
    })
}

fn adf_text_line(text: &str) -> serde_json::Value {
    adf_text(text)
}

fn build_unique_intake_title(page_title: &str, source_id: &str) -> String {
    let source_tail = source_id
        .rsplit(':')
        .next()
        .filter(|value| !value.is_empty())
        .unwrap_or("source");
    format!("{} [{}]", page_title, source_tail)
}

fn sanitize_label_segment(input: &str) -> String {
    let mut cleaned = input
        .replace(['/', '\\', ':', '|', '[', ']', '{', '}', '(', ')'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    cleaned = cleaned.trim().trim_matches('-').trim().to_string();
    if cleaned.len() > 120 {
        cleaned.truncate(120);
    }
    cleaned
}

fn extract_confluence_folder_id(url: &str) -> Option<String> {
    let marker = "/folder/";
    let start = url.find(marker)? + marker.len();
    let tail = &url[start..];
    let folder_id: String = tail.chars().take_while(|c| c.is_ascii_digit()).collect();

    if folder_id.is_empty() {
        None
    } else {
        Some(folder_id)
    }
}

fn extract_confluence_space_key_from_url(url: &str) -> Option<String> {
    let marker = "/spaces/";
    let start = url.find(marker)? + marker.len();
    let tail = &url[start..];
    let space_key: String = tail
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
        .collect();

    if space_key.is_empty() {
        return None;
    }

    if url.contains("/overview") {
        Some(space_key)
    } else {
        None
    }
}

fn extract_confluence_page_id_from_url(url: &str) -> Option<String> {
    if let Some(marker) = url.find("/pages/") {
        let tail = &url[(marker + "/pages/".len())..];
        let page_id: String = tail.chars().take_while(|c| c.is_ascii_digit()).collect();
        if !page_id.is_empty() {
            return Some(page_id);
        }
    }

    if let Some(marker) = url.find("homepageId=") {
        let tail = &url[(marker + "homepageId=".len())..];
        let page_id: String = tail.chars().take_while(|c| c.is_ascii_digit()).collect();
        if !page_id.is_empty() {
            return Some(page_id);
        }
    }

    None
}
