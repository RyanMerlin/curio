use crate::{Frontmatter, WikiPage};
use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::path::Path;

// ─── Parsing ─────────────────────────────────────────────────────────────

/// Read a markdown file and parse its YAML frontmatter.
///
/// Expected format:
/// ```text
/// ---
/// id: "abc"
/// title: "My Page"
/// ...
/// ---
///
/// Markdown body here.
/// ```
pub fn parse_wiki_page(path: &Path) -> Result<WikiPage> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    let (frontmatter, body) = split_frontmatter(&content)
        .with_context(|| format!("Failed to parse frontmatter in {}", path.display()))?;
    Ok(WikiPage {
        path: path.to_path_buf(),
        frontmatter,
        body,
    })
}

/// Split a raw file string into (Frontmatter, body).
fn split_frontmatter(content: &str) -> Result<(Frontmatter, String)> {
    let rest = content
        .strip_prefix("---")
        .ok_or_else(|| anyhow::anyhow!("File does not start with YAML frontmatter (---)"))?;

    // Find the closing ---
    let close = rest
        .find("\n---")
        .ok_or_else(|| anyhow::anyhow!("No closing --- found in frontmatter"))?;

    let yaml_str = &rest[..close];
    let body = rest[close + 4..].trim_start_matches('\n').to_string();

    let frontmatter: Frontmatter =
        serde_yaml::from_str(yaml_str).context("Failed to parse YAML frontmatter")?;

    Ok((frontmatter, body))
}

// ─── Writing ─────────────────────────────────────────────────────────────

/// Write a wiki page to disk, serialising frontmatter as YAML.
pub fn write_wiki_page(path: &Path, page: &WikiPage) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory {}", parent.display()))?;
    }
    let yaml =
        serde_yaml::to_string(&page.frontmatter).context("Failed to serialise frontmatter")?;
    let content = format!("---\n{}---\n\n{}", yaml, page.body);
    std::fs::write(path, content).with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(())
}

/// Update only the frontmatter of an existing wiki page on disk.
/// Re-reads the file to preserve the body, then rewrites with new frontmatter.
pub fn update_frontmatter(path: &Path, frontmatter: &Frontmatter) -> Result<()> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    let (_, body) = split_frontmatter(&content)
        .with_context(|| format!("Failed to parse frontmatter in {}", path.display()))?;
    let yaml = serde_yaml::to_string(frontmatter).context("Failed to serialise frontmatter")?;
    let new_content = format!("---\n{}---\n\n{}", yaml, body);
    std::fs::write(path, new_content)
        .with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(())
}

// ─── Sync helpers ────────────────────────────────────────────────────────

/// Return the markdown body of a file, stripping YAML frontmatter if present.
pub fn strip_frontmatter(content: &str) -> &str {
    let Some(rest) = content.strip_prefix("---") else {
        return content;
    };
    if let Some(end) = rest.find("\n---") {
        return rest[end + 4..].trim_start_matches('\n');
    }
    content
}

// ─── Slug & ID helpers ───────────────────────────────────────────────────

/// Convert a title to a filesystem-safe slug.
///
/// ```
/// use curio::wiki_fs::slug_from_title;
/// assert_eq!(slug_from_title("My Page Title!"), "my-page-title");
/// ```
pub fn slug_from_title(title: &str) -> String {
    title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Generate a stable 16-character hex ID from an input string (SHA256 prefix).
pub fn generate_id(input: &str) -> String {
    let digest = Sha256::digest(input.as_bytes());
    digest
        .iter()
        .take(8)
        .map(|b| format!("{:02x}", b))
        .collect::<String>()
}

/// Compute a full SHA256 content hash in the form `sha256:<hex>`.
pub fn content_hash(content: &str) -> String {
    let digest = Sha256::digest(content.as_bytes());
    format!(
        "sha256:{}",
        digest
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>()
    )
}

/// Return the first line of `body` that contains substantive text, truncated to `max_chars`.
///
/// Skips lines that are pure markdown decoration (blockquote markers with only bold/emoji,
/// heading-only lines, horizontal rules, etc.) so that callout bodies like
/// `> **ℹ️ Info**\n> Actual content` surface the actual content as the summary.
pub fn first_line_summary(body: &str, max_chars: usize) -> String {
    for raw in body.lines() {
        // Strip leading blockquote markers and whitespace
        let line = raw.trim().trim_start_matches('>').trim();
        // Strip leading heading markers
        let line = line.trim_start_matches('#').trim();
        // Skip empty lines
        if line.is_empty() {
            continue;
        }
        // Skip lines that are purely markdown decoration:
        // - horizontal rules: ---, ***, ___
        // - lines consisting only of bold/italic markers + emoji + punctuation (no letters/digits)
        if line.chars().all(|c| matches!(c, '-' | '*' | '_' | ' ')) {
            continue;
        }
        // Strip bold/italic markers (**text** or *text*) to get plain text for the check
        let stripped: String = line
            .replace("**", "")
            .replace('*', "")
            .replace("__", "")
            .replace('_', "");
        let plain = stripped.trim();
        if looks_like_callout_label(plain) {
            continue;
        }
        // Skip if no ASCII letters or digits remain (pure emoji/punctuation decoration line)
        if !plain.chars().any(|c| c.is_alphanumeric() && c.is_ascii()) {
            continue;
        }
        // This line has real content — use it
        let summary = if line.len() <= max_chars {
            line.to_string()
        } else {
            format!("{}…", &line[..max_chars])
        };
        return summary;
    }
    String::new()
}

fn looks_like_callout_label(text: &str) -> bool {
    let normalized = text
        .trim_matches(|c: char| !c.is_alphanumeric())
        .to_ascii_lowercase();
    matches!(
        normalized.as_str(),
        "info" | "warning" | "note" | "tip" | "important" | "caution"
    )
}

// ─── Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slug_basic() {
        assert_eq!(slug_from_title("My Page Title!"), "my-page-title");
    }

    #[test]
    fn slug_spaces_and_specials() {
        assert_eq!(slug_from_title("  Hello   World  "), "hello-world");
    }

    #[test]
    fn content_hash_format() {
        let h = content_hash("hello");
        assert!(h.starts_with("sha256:"));
        assert_eq!(h.len(), 7 + 64); // "sha256:" + 64 hex chars
    }

    #[test]
    fn strip_no_frontmatter() {
        let s = "# Hello\nworld";
        assert_eq!(strip_frontmatter(s), s);
    }

    #[test]
    fn strip_with_frontmatter() {
        let s = "---\nid: abc\n---\n\n# Hello\nworld";
        assert_eq!(strip_frontmatter(s), "# Hello\nworld");
    }

    #[test]
    fn first_line_summary_heading() {
        let s = "# My Page\n\nSome body.";
        assert_eq!(first_line_summary(s, 100), "My Page");
    }
}
