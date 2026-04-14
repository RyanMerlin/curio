use anyhow::{Context, Result};
use chrono::Utc;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::{
    config::Config,
    output::emit_json,
    wiki_fs::{content_hash, generate_id, slug_from_title, write_wiki_page},
    wiki_index::{append_log, load_registry, rebuild_index_md},
    Frontmatter, PageStatus, SourceRef, WikiPage,
};

pub async fn run_intake(
    config: &Config,
    dry_run: bool,
    json: bool,
    url: &Option<String>,
    file: &Option<PathBuf>,
    folder: &Option<PathBuf>,
    title: &Option<String>,
    subject_hint: &Option<String>,
    recursive: bool,
) -> Result<()> {
    let wiki_dir = &config.wiki.wiki_dir;

    let items = collect_items(config, url, file, folder, title, subject_hint, recursive).await?;

    if items.is_empty() {
        anyhow::bail!("No content to ingest — provide --url, --file, or --folder");
    }

    let mut ingested: Vec<String> = Vec::new();
    let mut skipped: Vec<String> = Vec::new();
    let registry = load_registry(wiki_dir)?;

    for item in items {
        let hash = content_hash(&item.text);

        // Dedup: same id means same source
        if registry.pages.iter().any(|e| e.id == item.id) {
            skipped.push(item.title.clone());
            continue;
        }

        if dry_run {
            ingested.push(item.title.clone());
            continue;
        }

        let slug = slug_from_title(&item.title);
        let filename = format!("{}.md", slug);
        let dest = wiki_dir.join("intake").join(&filename);

        let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let fm = Frontmatter {
            id: item.id.clone(),
            title: item.title.clone(),
            status: PageStatus::Intake,
            source: item.source_ref.clone(),
            category: vec![],
            keywords: vec![],
            created_at: now.clone(),
            updated_at: now,
            confidence: None,
            cross_refs: vec![],
            content_hash: hash,
            confluence_page_id: None,
            model_used: None,
        };

        let page = WikiPage { path: dest.clone(), frontmatter: fm.clone(), body: item.text.clone() };
        write_wiki_page(&dest, &page)?;
        ingested.push(item.title.clone());
    }

    if !dry_run && !ingested.is_empty() {
        rebuild_index_md(wiki_dir, &registry)?;
        append_log(wiki_dir, &format!("intake: {} items ingested", ingested.len()))?;

        if config.wiki.auto_commit {
            let repo_root = wiki_dir.parent().unwrap_or(wiki_dir);
            crate::git_ops::git_add(repo_root, wiki_dir)?;
            if crate::git_ops::git_has_staged(repo_root) {
                crate::git_ops::git_commit(
                    repo_root,
                    &format!("curio: intake {} item(s)", ingested.len()),
                )?;
            }
        }
    }

    if json {
        let _ = emit_json(
            "intake",
            true,
            &serde_json::json!({
                "ingested": ingested,
                "skipped": skipped,
                "dry_run": dry_run,
            }),
        );
    } else {
        for t in &ingested {
            println!("  ingested: {}", t);
        }
        for t in &skipped {
            println!("  skipped (duplicate): {}", t);
        }
        if !ingested.is_empty() || !skipped.is_empty() {
            println!(
                "{} item(s) ingested, {} skipped",
                ingested.len(),
                skipped.len()
            );
        }
    }
    Ok(())
}

// ─── Content collection ───────────────────────────────────────────────────

struct IntakeItem {
    id: String,
    title: String,
    text: String,
    source_ref: SourceRef,
}

async fn collect_items(
    config: &Config,
    url: &Option<String>,
    file: &Option<PathBuf>,
    folder: &Option<PathBuf>,
    title: &Option<String>,
    _subject_hint: &Option<String>,
    recursive: bool,
) -> Result<Vec<IntakeItem>> {
    let mut items = Vec::new();

    if let Some(url_str) = url {
        if is_confluence_url(url_str, &config.connection.confluence_url) {
            items.extend(collect_from_confluence(config, url_str, title, recursive).await?);
        } else {
            items.extend(collect_from_url(url_str, title).await?);
        }
    } else if let Some(file_path) = file {
        items.extend(collect_from_file(file_path, title)?);
    } else if let Some(folder_path) = folder {
        items.extend(collect_from_folder(folder_path)?);
    }

    Ok(items)
}

fn is_confluence_url(url: &str, confluence_base: &str) -> bool {
    !confluence_base.is_empty() && url.starts_with(confluence_base.trim_end_matches('/'))
}

async fn collect_from_url(url: &str, title_hint: &Option<String>) -> Result<Vec<IntakeItem>> {
    let client = reqwest::Client::builder()
        .user_agent("curio/0.1")
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let html = client
        .get(url)
        .send()
        .await
        .with_context(|| format!("Failed to fetch {}", url))?
        .text()
        .await?;

    let text = extract_text_from_html(&html, "");
    let title = title_hint.clone().unwrap_or_else(|| {
        extract_title_from_html(&html).unwrap_or_else(|| url.to_string())
    });
    let id = generate_id(&format!("url:{}", url));

    Ok(vec![IntakeItem {
        id,
        title,
        text,
        source_ref: SourceRef {
            kind: "url".to_string(),
            id: format!("url:{}", url),
            origin_url: Some(url.to_string()),
            summary: None,
        },
    }])
}

async fn collect_from_confluence(
    config: &Config,
    url: &str,
    title_hint: &Option<String>,
    recursive: bool,
) -> Result<Vec<IntakeItem>> {
    config.connection.require_confluence()?;
    let token = std::env::var("CURIO_CONFLUENCE_TOKEN")
        .context("CURIO_CONFLUENCE_TOKEN not set")?;
    let client = crate::confluence::ConfluenceClient::new(
        config.connection.confluence_url.clone(),
        config.connection.confluence_email.clone(),
        token,
        None,
    )?;

    let page_id = extract_confluence_page_id(url)
        .ok_or_else(|| anyhow::anyhow!("Could not extract page ID from Confluence URL: {}", url))?;

    // Collect the root page + all descendants if recursive.
    // Also build a parent→children title map for hub-page body synthesis.
    let mut page_ids: Vec<String> = vec![page_id.clone()];
    let mut children_map: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    if recursive {
        let descendants = client.get_page_descendants_v2(&page_id).await?;
        for d in &descendants {
            if let Some(id) = d["id"].as_str() {
                page_ids.push(id.to_string());
                // Build parent→children title map using parentId from the API response
                if let Some(parent_id) = d["parentId"].as_str() {
                    let child_title = d["title"].as_str().unwrap_or("Untitled").to_string();
                    children_map.entry(parent_id.to_string()).or_default().push(child_title);
                }
            }
        }
        eprintln!("Fetching {} pages (root + {} descendants)...", page_ids.len(), page_ids.len() - 1);
    }

    let mut items = Vec::new();
    for (i, pid) in page_ids.iter().enumerate() {
        let page = match client.get_page_body(pid).await? {
            Some(p) => p,
            None => {
                eprintln!("  [{}/{}] page {} not found — skipping", i + 1, page_ids.len(), pid);
                continue;
            }
        };

        let title = if pid == &page_id {
            title_hint.clone().unwrap_or_else(|| page["title"].as_str().unwrap_or("Untitled").to_string())
        } else {
            page["title"].as_str().unwrap_or("Untitled").to_string()
        };

        let page_url = format!(
            "{}/wiki/spaces/{}/pages/{}",
            config.connection.confluence_url.trim_end_matches("/wiki"),
            config.content_model.space_key,
            pid
        );

        let html_body = page["body"]["storage"]["value"].as_str().unwrap_or("").to_string();
        let confluence_base = config.connection.confluence_url.trim_end_matches("/wiki").to_string();
        let mut text = extract_text_from_html(&html_body, &confluence_base);

        // Hub/index page synthesis: if the body is sparse (mostly navigation macros,
        // smart links, or children macros with no prose), synthesize a useful body
        // from known child page titles so the routing agent has real signal.
        let meaningful_chars = text.chars().filter(|c| c.is_alphanumeric()).count();
        if meaningful_chars < 80 {
            let child_titles = children_map.get(pid.as_str()).cloned().unwrap_or_default();
            if !child_titles.is_empty() {
                text = format!(
                    "Hub page: {title}\n\nThis page organizes the following sub-pages:\n\n{children}\n",
                    title = title,
                    children = child_titles.iter().map(|t| format!("- {}", t)).collect::<Vec<_>>().join("\n")
                );
            } else if !recursive {
                // For a non-recursive single-page intake of a hub, fetch direct children
                // so we can still produce a useful body.
                if let Ok(direct_children) = client.get_direct_children_v2(pid).await {
                    let child_titles: Vec<String> = direct_children
                        .iter()
                        .filter_map(|c| c["title"].as_str().map(|t| t.to_string()))
                        .collect();
                    if !child_titles.is_empty() {
                        text = format!(
                            "Hub page: {title}\n\nThis page organizes the following sub-pages:\n\n{children}\n",
                            title = title,
                            children = child_titles.iter().map(|t| format!("- {}", t)).collect::<Vec<_>>().join("\n")
                        );
                    } else {
                        text = format!("Hub page: {title}\n\n*This page serves as a section index with no prose body.*\n", title = title);
                    }
                }
            } else {
                text = format!("Hub page: {title}\n\n*This page serves as a section index with no prose body.*\n", title = title);
            }
            eprintln!("  [hub] synthesized body for sparse page: {}", title);
        }

        let id = generate_id(&format!("confluence-page:{}", pid));

        if recursive {
            eprintln!("  [{}/{}] {}", i + 1, page_ids.len(), title);
        }

        items.push(IntakeItem {
            id,
            title,
            text,
            source_ref: SourceRef {
                kind: "confluence_page".to_string(),
                id: format!("confluence-page:{}", pid),
                origin_url: Some(if pid == &page_id { url.to_string() } else { page_url }),
                summary: None,
            },
        });
    }

    Ok(items)
}

fn collect_from_file(path: &Path, title_hint: &Option<String>) -> Result<Vec<IntakeItem>> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    let title = title_hint.clone().unwrap_or_else(|| {
        path.file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "Untitled".to_string())
    });
    let id = generate_id(&format!("file:{}", path.display()));

    Ok(vec![IntakeItem {
        id,
        title,
        text: content,
        source_ref: SourceRef {
            kind: "file".to_string(),
            id: format!("file:{}", path.display()),
            origin_url: None,
            summary: None,
        },
    }])
}

fn collect_from_folder(folder: &Path) -> Result<Vec<IntakeItem>> {
    let mut items = Vec::new();
    for entry in WalkDir::new(folder)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_file()
                && e.path().extension().map_or(false, |ext| {
                    matches!(ext.to_str(), Some("md") | Some("txt"))
                })
        })
    {
        items.extend(collect_from_file(entry.path(), &None)?);
    }
    Ok(items)
}

// ─── HTML → Markdown conversion ───────────────────────────────────────────
//
// Recursive top-down walk: each element is processed exactly once, so there
// are no duplicate lines from parent+child both being selected.

fn extract_text_from_html(html: &str, confluence_base: &str) -> String {
    // Make the base URL available to element_to_md via thread-local so we don't
    // have to thread it through every recursive call.
    CONFLUENCE_BASE.with(|cell| {
        *cell.borrow_mut() = confluence_base.trim_end_matches('/').to_string();
    });
    let doc = scraper::Html::parse_document(html);
    // Confluence storage format has content under <body>; plain HTML pages too.
    let body_sel = scraper::Selector::parse("body").unwrap();
    let root = doc.select(&body_sel).next()
        .map(|el| el.id())
        .and_then(|id| scraper::ElementRef::wrap(doc.tree.get(id).unwrap()))
        .unwrap_or_else(|| scraper::ElementRef::wrap(doc.tree.root()).unwrap());
    let md = element_to_md(root, 0);
    // Collapse runs of 3+ blank lines to 2
    let re_blank = regex::Regex::new(r"\n{3,}").unwrap();
    re_blank.replace_all(md.trim(), "\n\n").into_owned()
}

thread_local! {
    static CONFLUENCE_BASE: std::cell::RefCell<String> = std::cell::RefCell::new(String::new());
}

fn element_to_md(el: scraper::ElementRef<'_>, depth: usize) -> String {
    let tag = el.value().name().to_lowercase();
    match tag.as_str() {
        // Skip non-content tags entirely
        "script" | "style" | "head" | "meta" | "link" | "noscript" => String::new(),

        // Headings
        "h1" => format!("# {}\n\n", inline_text(el)),
        "h2" => format!("## {}\n\n", inline_text(el)),
        "h3" => format!("### {}\n\n", inline_text(el)),
        "h4" => format!("#### {}\n\n", inline_text(el)),
        "h5" | "h6" => format!("##### {}\n\n", inline_text(el)),

        // Paragraphs
        "p" => {
            let t = inline_children(el, depth);
            if t.trim().is_empty() { String::new() } else { format!("{}\n\n", t.trim()) }
        }

        // Lists
        "ul" | "ol" => {
            let mut out = String::new();
            let mut idx = 1usize;
            for child in el.children().filter_map(scraper::ElementRef::wrap) {
                if child.value().name() == "li" {
                    let bullet = if tag == "ol" {
                        format!("{}{}. ", "   ".repeat(depth), idx)
                    } else {
                        format!("{}- ", "   ".repeat(depth))
                    };
                    let content = inline_children(child, depth + 1).trim().replace('\n', " ");
                    if !content.is_empty() {
                        out.push_str(&bullet);
                        out.push_str(&content);
                        out.push('\n');
                        // Recurse into nested lists inside this li
                        for sub in child.children().filter_map(scraper::ElementRef::wrap) {
                            match sub.value().name() {
                                "ul" | "ol" => out.push_str(&element_to_md(sub, depth + 1)),
                                _ => {}
                            }
                        }
                    }
                    idx += 1;
                }
            }
            if out.is_empty() { String::new() } else { format!("{}\n", out) }
        }
        "li" => String::new(), // handled by parent ul/ol

        // Code blocks
        "pre" | "code" if tag == "pre" => {
            let code: String = el.text().collect();
            if code.trim().is_empty() { String::new() }
            else { format!("```\n{}\n```\n\n", code.trim_end()) }
        }
        "code" => format!("`{}`", el.text().collect::<String>().trim()),

        // Inline formatting — return inline, parent adds paragraph breaks
        "strong" | "b" => format!("**{}**", inline_text(el)),
        "em" | "i" => format!("*{}*", inline_text(el)),

        // Links
        "a" => {
            let href = el.value().attr("href").unwrap_or("#");
            let text = inline_text(el);
            if text.trim().is_empty() || text.trim() == href {
                format!("<{}>", href)
            } else {
                format!("[{}]({})", text.trim(), href)
            }
        }

        // Horizontal rules / breaks
        "hr" => "---\n\n".to_string(),
        "br" => "\n".to_string(),

        // Tables — simple: header row from <th>, body rows from <td>
        "table" => {
            let mut rows: Vec<Vec<String>> = Vec::new();
            let tr_sel = scraper::Selector::parse("tr").unwrap();
            for tr in el.select(&tr_sel) {
                let cells: Vec<String> = tr.children()
                    .filter_map(scraper::ElementRef::wrap)
                    .filter(|c| matches!(c.value().name(), "td" | "th"))
                    .map(|c| {
                        // children_to_md (not element_to_md) so we process cell contents
                        // without hitting the "td" => String::new() short-circuit.
                        // Collapse newlines to spaces and escape pipe chars for GFM table syntax.
                        children_to_md(c, depth + 1)
                            .replace('\n', " ")
                            .replace('|', "\\|")
                            .trim()
                            .to_string()
                    })
                    .collect();
                if !cells.is_empty() {
                    rows.push(cells);
                }
            }
            if rows.is_empty() { return String::new(); }
            let cols = rows.iter().map(|r| r.len()).max().unwrap_or(1);
            let separator = format!("| {} |", vec!["---"; cols].join(" | "));
            let mut out = String::new();
            for (i, row) in rows.iter().enumerate() {
                let padded: Vec<String> = (0..cols)
                    .map(|j| row.get(j).cloned().unwrap_or_default())
                    .collect();
                out.push_str(&format!("| {} |\n", padded.join(" | ")));
                if i == 0 {
                    out.push_str(&separator);
                    out.push('\n');
                }
            }
            format!("{}\n", out)
        }
        "tr" | "td" | "th" | "thead" | "tbody" => String::new(), // handled by table

        // Block-level containers — recurse into children
        "div" | "section" | "article" | "main" | "body"
        | "html" | "span" | "figure" | "figcaption"
        | "header" | "footer" | "nav" | "aside" => children_to_md(el, depth),

        // Confluence internal page links: <ac:link><ri:page ri:content-title="X" ri:space-key="Y"/></ac:link>
        // Render as a proper markdown link using the Confluence base URL.
        "ac:link" => {
            let ri_page_sel = scraper::Selector::parse("ri\\:page").unwrap();
            // Display text can be in ac:link-title or ac:link-body (new ADF format)
            let link_title_sel = scraper::Selector::parse("ac\\:link-title").unwrap();
            let link_body_sel  = scraper::Selector::parse("ac\\:link-body").unwrap();

            let display = el.select(&link_title_sel).next()
                .or_else(|| el.select(&link_body_sel).next())
                .map(|lt| lt.text().collect::<String>())
                .filter(|t| !t.trim().is_empty());

            if let Some(ri_el) = el.select(&ri_page_sel).next() {
                // Namespace prefixes are stripped from attribute names by HTML parsers:
                // "ri:content-title" → "content-title", "ri:space-key" → "space-key"
                let content_title = ri_el.value().attr("ri:content-title")
                    .or_else(|| ri_el.value().attr("content-title"))
                    .unwrap_or("");
                let space_key = ri_el.value().attr("ri:space-key")
                    .or_else(|| ri_el.value().attr("space-key"))
                    .unwrap_or("");
                let text = display.as_deref().unwrap_or(content_title).trim().to_string();
                if !content_title.is_empty() {
                    let base = CONFLUENCE_BASE.with(|c| c.borrow().clone());
                    if !base.is_empty() && !space_key.is_empty() {
                        // Encode the title for URL use
                        let encoded = content_title.replace(' ', "+");
                        let url = format!("{}/wiki/spaces/{}/pages?title={}", base, space_key, encoded);
                        return format!("[{}]({})", text, url);
                    } else if !base.is_empty() {
                        // No space key — link to search
                        let encoded = content_title.replace(' ', "+");
                        let url = format!("{}/wiki/search?text={}", base, encoded);
                        return format!("[{}]({})", text, url);
                    }
                    return format!("[{}]", text);
                }
            }

            // Fallback: display text only
            if let Some(text) = display {
                return format!("[{}]", text.trim());
            }
            children_to_md(el, depth)
        }
        "ri:page" => {
            let ct = el.value().attr("ri:content-title")
                .or_else(|| el.value().attr("content-title"));
            if let Some(ct) = ct {
                return ct.to_string();
            }
            String::new()
        }
        "ri:attachment" | "ri:space" | "ri:user" => String::new(),

        // Confluence macros: extract text from rich/plain body
        // ac:parameter is a macro metadata node (e.g. the "title" param of a note macro).
        // It must be suppressed here — it is NOT body content and renders as duplicate text
        // if allowed to fall through to the default arm.
        "ac:parameter" => String::new(),

        // ADF (Atlassian Document Format) extension elements.
        // ac:adf-extension contains ac:adf-content (real content) AND ac:adf-fallback
        // (a legacy HTML duplicate for old clients). We render only ac:adf-content.
        "ac:adf-fallback" => String::new(),
        "ac:adf-attribute" => String::new(), // metadata, not body text
        "ac:adf-extension" => {
            let content_sel = scraper::Selector::parse("ac\\:adf-content").unwrap();
            el.select(&content_sel)
                .map(|c| children_to_md(c, depth))
                .collect::<String>()
        }
        "ac:adf-node" => {
            // Panel nodes (note/warning/info/tip) — render their children as a blockquote callout.
            let panel_type = el.value().attr("type").unwrap_or("");
            let content = children_to_md(el, depth);
            if content.trim().is_empty() { return String::new(); }
            match panel_type {
                "panel" => {
                    // panel-type attribute is in a child ac:adf-attribute — read it from child text
                    // or just use a generic blockquote (panel-type already suppressed by ac:adf-attribute rule).
                    let indented = content.trim().lines()
                        .map(|l| format!("> {}", l))
                        .collect::<Vec<_>>()
                        .join("\n");
                    format!("{}\n\n", indented)
                }
                _ => content,
            }
        }
        "ac:adf-content" => children_to_md(el, depth),

        "ac:structured-macro" => {
            // info/note/warning → blockquote-style callout
            // HTML parsers strip namespace prefixes from attribute names, so
            // "ac:name" is stored as just "name" in the DOM.
            let macro_name = el.value().attr("ac:name")
                .or_else(|| el.value().attr("name"))
                .unwrap_or("");
            // children/pagetree macros signal a section-index page.
            // Return empty — the hub synthesis at the page level already handles this.
            if macro_name == "children" || macro_name == "pagetree" {
                return String::new();
            }
            let body_sel = scraper::Selector::parse("ac\\:rich-text-body, ac\\:plain-text-body").unwrap();
            let body = el.select(&body_sel)
                .map(|b| element_to_md(b, depth))
                .collect::<String>();
            if body.trim().is_empty() { return String::new(); }
            match macro_name {
                "info" | "note" | "warning" | "tip" => {
                    let label = match macro_name {
                        "warning" => "⚠️ Warning",
                        "note"    => "📝 Note",
                        "tip"     => "💡 Tip",
                        _         => "ℹ️ Info",
                    };
                    // Indent each line of the body with "> "
                    let indented = body.trim().lines()
                        .map(|l| format!("> {}", l))
                        .collect::<Vec<_>>()
                        .join("\n");
                    format!("> **{}**\n>\n{}\n\n", label, indented)
                }
                "code" => {
                    // Try both namespaced and non-namespaced attribute selectors
                    let lang = el.select(&scraper::Selector::parse("ac\\:parameter[name=language]").unwrap())
                        .next()
                        .or_else(|| el.select(&scraper::Selector::parse("ac\\:parameter[ac\\:name=language]").unwrap()).next())
                        .map(|p| p.text().collect::<String>())
                        .unwrap_or_default();
                    format!("```{}\n{}\n```\n\n", lang, body.trim())
                }
                _ => format!("{}\n", body.trim()),
            }
        }
        "ac:rich-text-body" | "ac:plain-text-body" => children_to_md(el, depth),
        "ac:task-list" => children_to_md(el, depth),
        "ac:task" => {
            let status_sel = scraper::Selector::parse("ac\\:task-status").unwrap();
            let body_sel   = scraper::Selector::parse("ac\\:task-body").unwrap();
            let done = el.select(&status_sel).next()
                .map(|s| s.text().collect::<String>().trim().to_lowercase() == "complete")
                .unwrap_or(false);
            let body = el.select(&body_sel).next()
                .map(|b| inline_text(b))
                .unwrap_or_default();
            format!("- [{}] {}\n", if done { "x" } else { " " }, body.trim())
        }
        "ac:task-status" | "ac:task-body" => String::new(), // handled by ac:task

        // Default: recurse
        _ => children_to_md(el, depth),
    }
}

/// Collect inline text from an element's children without adding block structure.
fn inline_text(el: scraper::ElementRef<'_>) -> String {
    el.text().collect::<String>()
}

/// Collect inline children, handling inline tags like <a>, <strong>, <em>, <code>.
fn inline_children(el: scraper::ElementRef<'_>, depth: usize) -> String {
    let mut out = String::new();
    for child in el.children() {
        match child.value() {
            scraper::node::Node::Text(t) => {
                let s = t.trim_matches('\n');
                if !s.is_empty() { out.push_str(s); }
            }
            scraper::node::Node::Element(_) => {
                if let Some(child_el) = scraper::ElementRef::wrap(child) {
                    let tag = child_el.value().name();
                    // Recurse into inline elements (HTML + Confluence inline link elements).
                    match tag {
                        "a" | "strong" | "b" | "em" | "i" | "code" | "span"
                        | "br" | "sup" | "sub" | "u" | "s" | "del"
                        // Confluence inline elements that should produce linked output:
                        | "ac:link" | "ac:link-title" | "ac:link-body"
                        | "ri:page" | "ri:attachment" | "ri:user" | "ri:space" => {
                            out.push_str(&element_to_md(child_el, depth));
                        }
                        // Other elements inside inline context: just grab text
                        _ => {
                            let t = child_el.text().collect::<String>();
                            out.push_str(t.trim());
                        }
                    }
                }
            }
            _ => {}
        }
    }
    out
}

/// Recurse into all children and concatenate their markdown output.
fn children_to_md(el: scraper::ElementRef<'_>, depth: usize) -> String {
    let mut out = String::new();
    for child in el.children() {
        match child.value() {
            scraper::node::Node::Text(t) => {
                let s = t.trim_matches('\n').trim();
                if !s.is_empty() { out.push_str(s); out.push(' '); }
            }
            scraper::node::Node::Element(_) => {
                if let Some(child_el) = scraper::ElementRef::wrap(child) {
                    out.push_str(&element_to_md(child_el, depth));
                }
            }
            _ => {}
        }
    }
    out
}

fn extract_title_from_html(html: &str) -> Option<String> {
    let doc = scraper::Html::parse_document(html);
    let sel = scraper::Selector::parse("title, h1").ok()?;
    doc.select(&sel)
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
}

fn extract_confluence_page_id(url: &str) -> Option<String> {
    if let Some(idx) = url.find("/pages/") {
        let rest = &url[idx + 7..];
        let id: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
        if !id.is_empty() {
            return Some(id);
        }
    }
    if let Some(idx) = url.find("pageId=") {
        let rest = &url[idx + 7..];
        let id: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
        if !id.is_empty() {
            return Some(id);
        }
    }
    None
}
