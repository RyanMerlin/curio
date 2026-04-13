/// Markdown → Confluence storage format converter.
///
/// Vendored and simplified from confluence-cli (MIT License).
/// Source: https://github.com/rvben/confluence-cli/blob/main/src/markdown.rs
///
/// Supports:
///   :::confluence-info [title]\n ... \n:::   → <ac:structured-macro ac:name="info">
///   :::confluence-tip [title]\n ... \n:::    → <ac:structured-macro ac:name="tip">
///   :::confluence-note [title]\n ... \n:::   → <ac:structured-macro ac:name="note">
///   :::confluence-warning [title]\n ... \n:::→ <ac:structured-macro ac:name="warning">
///   :::confluence-expand [title]\n ... \n::: → <ac:structured-macro ac:name="expand">
///   ```confluence-storage\n...\n```          → raw Confluence XML passthrough
///
/// All other markdown is converted via pulldown_cmark.

use anyhow::Result;
use pulldown_cmark::{Options, Parser, html};
use regex::Regex;

/// Convert markdown with optional Confluence macro blocks to Confluence storage format HTML.
pub fn markdown_to_storage(markdown: &str) -> Result<String> {
    // Step 1: extract :::confluence-* fenced panel/expand blocks → placeholders
    let (normalized, macro_fragments) = replace_macro_blocks(markdown)?;

    // Step 2: extract ```confluence-storage raw passthrough blocks → placeholders
    let block_re = Regex::new(r"(?s)```confluence-storage\s*\n(.*?)\n```")?;
    let mut raw_fragments: Vec<String> = Vec::new();
    let normalized = block_re
        .replace_all(&normalized, |caps: &regex::Captures<'_>| {
            let idx = raw_fragments.len();
            raw_fragments.push(caps[1].to_string());
            format!("CURIO_RAW_PLACEHOLDER_{idx}")
        })
        .to_string();

    // Step 3: standard markdown → HTML
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_TASKLISTS);
    let parser = Parser::new_ext(&normalized, opts);
    let mut html = String::new();
    html::push_html(&mut html, parser);

    // Step 4: put raw fragments back
    for (idx, fragment) in raw_fragments.iter().enumerate() {
        html = html.replace(&format!("<p>CURIO_RAW_PLACEHOLDER_{idx}</p>"), fragment);
        html = html.replace(&format!("CURIO_RAW_PLACEHOLDER_{idx}"), fragment);
    }

    // Step 5: put macro fragments back
    for (idx, fragment) in macro_fragments.iter().enumerate() {
        html = html.replace(&format!("<p>CURIO_MACRO_PLACEHOLDER_{idx}</p>"), fragment);
        html = html.replace(&format!("CURIO_MACRO_PLACEHOLDER_{idx}"), fragment);
    }

    Ok(html.trim().to_string())
}

/// Replace `:::confluence-{info|tip|note|warning|expand}` fenced blocks with placeholders.
/// Returns (normalized_markdown, fragments_vec).
fn replace_macro_blocks(markdown: &str) -> Result<(String, Vec<String>)> {
    let mut fragments: Vec<String> = Vec::new();
    let mut output = String::with_capacity(markdown.len());
    let lines: Vec<&str> = markdown.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim_end();

        // Detect opening fence: `:::confluence-TYPE` optionally followed by a title
        let macro_type = ["info", "tip", "note", "warning", "expand"]
            .iter()
            .find_map(|&t| {
                let prefix = format!(":::confluence-{t}");
                if line.starts_with(&prefix) {
                    let title = line[prefix.len()..].trim();
                    Some((t, if title.is_empty() { None } else { Some(title.to_string()) }))
                } else {
                    None
                }
            });

        if let Some((macro_name, title)) = macro_type {
            // Collect body until closing `:::`
            i += 1;
            let mut body_lines: Vec<&str> = Vec::new();
            while i < lines.len() && lines[i].trim() != ":::" {
                body_lines.push(lines[i]);
                i += 1;
            }
            i += 1; // consume closing `:::`

            let body_md = body_lines.join("\n");
            let body_storage = markdown_to_storage(&body_md)?;

            // Confluence Cloud's Fabric editor rejects <ac:structured-macro> tags.
            // Render as an HTML panel using a table with a left border colour strip.
            let icon = match macro_name {
                "info"    => "ℹ️",
                "tip"     => "💡",
                "note"    => "⚠️",
                "warning" => "🚨",
                _         => "▶",
            };
            let title_html = match title {
                Some(t) => format!("<strong>{icon} {}</strong><br/>", escape_xml(&t)),
                None    => format!("<strong>{icon}</strong><br/>"),
            };
            let fragment = format!(
                "<blockquote>{title_html}{body_storage}</blockquote>",
                title_html = title_html,
                body_storage = body_storage,
            );
            let idx = fragments.len();
            fragments.push(fragment);
            output.push_str(&format!("CURIO_MACRO_PLACEHOLDER_{idx}\n"));
        } else {
            output.push_str(lines[i]);
            output.push('\n');
            i += 1;
        }
    }

    if !markdown.ends_with('\n') && output.ends_with('\n') {
        output.pop();
    }

    Ok((output, fragments))
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
