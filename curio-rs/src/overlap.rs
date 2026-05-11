use anyhow::Result;
use std::collections::HashSet;
use std::path::Path;

use crate::wiki_fs::parse_wiki_page;

#[derive(Debug, Clone)]
pub struct OverlapMatch {
    pub path: String,
    pub score: f32,
}

fn tokens(text: &str) -> HashSet<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|part| part.len() >= 4)
        .map(|part| part.to_string())
        .collect()
}

fn jaccard(a: &HashSet<String>, b: &HashSet<String>) -> f32 {
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    let intersection = a.intersection(b).count() as f32;
    let union = a.union(b).count() as f32;
    if union == 0.0 {
        0.0
    } else {
        intersection / union
    }
}

pub fn find_peer_overlap(
    wiki_dir: &Path,
    category: &[String],
    title: &str,
    body: &str,
    exclude_slug: Option<&str>,
) -> Result<Vec<OverlapMatch>> {
    let mut peers = Vec::new();
    if category.is_empty() {
        return Ok(peers);
    }
    let peer_dir = wiki_dir
        .join("published")
        .join(category.iter().collect::<std::path::PathBuf>());
    if !peer_dir.exists() {
        return Ok(peers);
    }

    let target_tokens = tokens(&format!("{} {}", title, body));
    for entry in walkdir::WalkDir::new(&peer_dir)
        .into_iter()
        .filter_map(|entry| entry.ok())
    {
        let path = entry.path();
        if !entry.file_type().is_file()
            || path.extension().and_then(|ext| ext.to_str()) != Some("md")
            || path.file_name().and_then(|name| name.to_str()) == Some("index.md")
        {
            continue;
        }
        if let Some(slug) = exclude_slug {
            if path.file_stem().and_then(|name| name.to_str()) == Some(slug) {
                continue;
            }
        }
        let page = match parse_wiki_page(path) {
            Ok(page) => page,
            Err(_) => continue,
        };
        let peer_tokens = tokens(&format!("{} {}", page.frontmatter.title, page.body));
        let title_score = jaccard(&tokens(title), &tokens(&page.frontmatter.title));
        let body_score = jaccard(&target_tokens, &peer_tokens);
        let score = (title_score * 0.45) + (body_score * 0.55);
        if score >= 0.45 {
            peers.push(OverlapMatch {
                path: path
                    .strip_prefix(wiki_dir)
                    .unwrap_or(path)
                    .to_string_lossy()
                    .replace('\\', "/"),
                score,
            });
        }
    }

    peers.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    Ok(peers)
}
