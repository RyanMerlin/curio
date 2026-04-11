use crate::{
    Result, config::Config, confluence::ConfluenceClient, resolve_managed_root_folder_id,
    resolve_or_create_scoped_child_page_id,
};
use anyhow::Context;
use chrono::Utc;
use scraper::{Html, Selector};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub async fn run_intake_create(
    config: &Config,
    dry_run: bool,
    url: &Option<String>,
    file: &Option<PathBuf>,
    folder: &Option<PathBuf>,
    subject_hint: &Option<String>,
    metadata_str: &Option<String>,
) -> Result<()> {
    println!("Running intake create command...");

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

    // Fetch managed root and Intake page IDs
    let root_folder_id = resolve_managed_root_folder_id(
        &client,
        space_key,
        root_folder_name,
        config.content_model.output_root_folder_id.as_deref(),
    )
    .await?;
    let intake_page_id = resolve_or_create_scoped_child_page_id(
        &client,
        space_key,
        &root_folder_id,
        "Intake",
        "<p>This page holds raw, unprocessed content that has been ingested.</p>",
    )
    .await?;

    let mut all_content: Vec<(String, String, String, Option<String>)> = Vec::new();
    // (content, source_id, content_type, subject_hint)

    if let Some(u) = url {
        println!("Ingesting from URL: {}", u);
        let content = fetch_url_content(u).await?;
        all_content.push((
            content,
            format!("url:{}", u),
            "web_page".to_string(),
            subject_hint.clone(),
        ));
    } else if let Some(f) = file {
        println!("Ingesting from file: {}", f.display());
        let content = read_file_content(f).await?;
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
                println!("  - Ingesting file: {}", path.display());
                let content = read_file_content(path).await?;
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

    for (content, source_id, content_type, item_subject_hint) in all_content {
        let effective_subject_hint = item_subject_hint.or_else(|| subject_hint.clone());
        process_single_content(
            &client,
            config,
            dry_run,
            &intake_page_id,
            &content,
            &source_id,
            &content_type,
            &effective_subject_hint,
            metadata_str,
        )
        .await?;
    }

    println!("Intake create command finished.");
    Ok(())
}

async fn fetch_url_content(url: &str) -> Result<String> {
    let response = reqwest::get(url)
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
    intake_page_id: &str,
    content: &str,
    source_id: &str,
    content_type: &str,
    subject_hint: &Option<String>,
    metadata_str: &Option<String>,
) -> Result<()> {
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
    println!("Checking for duplicates with CQL: {}", cql_dedupe);
    let existing_pages = client.execute_cql(&cql_dedupe).await?;

    if !existing_pages.is_empty() {
        println!(
            "Skipping ingestion: Duplicate content found with dedupe key '{}'. Existing page IDs: {:?}",
            dedupe_key,
            existing_pages
                .iter()
                .filter_map(|p| p["id"].as_str())
                .collect::<Vec<&str>>()
        );
        return Ok(());
    }

    // 2. Create Page
    let page_id = if dry_run {
        println!("(Dry run) Would create page: '{}' under Intake", page_title);
        "dry_run_page_id".to_string()
    } else {
        println!("Creating page: '{}' under Intake", page_title);
        client
            .create_or_update_page(
                &config.content_model.space_key,
                Some(intake_page_id),
                &page_title,
                "storage", // Assuming storage format for simplicity
                content,
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

    if !dry_run {
        println!("Setting curio_metadata for page {}", page_id);
        client
            .set_content_property(&page_id, "curio_metadata", curio_metadata)
            .await?;
    } else {
        println!(
            "(Dry run) Would set curio_metadata for page {}: {}",
            page_id,
            curio_metadata.to_string()
        );
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
        labels.push(format!("{}-subject-key-{}", label_namespace, subject_key));
    }

    if !dry_run {
        println!("Adding labels to page {}: {:?}", page_id, labels);
        client.add_labels(&page_id, labels).await?;
    } else {
        println!(
            "(Dry run) Would add labels to page {}: {:?}",
            page_id, labels
        );
    }

    println!("Successfully processed content from source: {}", source_id);
    Ok(())
}
