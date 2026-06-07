use anyhow::Result;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::embeddings::{LocalCachedEmbeddings, cosine_similarity};
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

fn jaccard_overlap(
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
        .join(category.iter().collect::<PathBuf>());
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
        if let Some(slug) = exclude_slug
            && path.file_stem().and_then(|name| name.to_str()) == Some(slug)
        {
            continue;
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

fn vector_overlap(
    config: &Config,
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
        .join(category.iter().collect::<PathBuf>());
    if !peer_dir.exists() {
        return Ok(peers);
    }

    let cache_dir = config
        .overlap
        .embedding_cache_dir
        .clone()
        .unwrap_or_else(|| wiki_dir.join("_admin").join("embeddings"));
    let embeddings = LocalCachedEmbeddings::new(cache_dir);
    let target_key = format!("target:{}:{}", category.join("/"), title);
    let target_vector = embeddings.embed(&target_key, &format!("{} {}", title, body))?;

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
        if let Some(slug) = exclude_slug
            && path.file_stem().and_then(|name| name.to_str()) == Some(slug)
        {
            continue;
        }
        let page = match parse_wiki_page(path) {
            Ok(page) => page,
            Err(_) => continue,
        };
        let peer_key = path
            .strip_prefix(wiki_dir)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/");
        let peer_vector = embeddings.embed(
            &peer_key,
            &format!("{} {}", page.frontmatter.title, page.body),
        )?;
        let title_tokens = tokens(title);
        let peer_title_tokens = tokens(&page.frontmatter.title);
        let lexical_boost = jaccard(&title_tokens, &peer_title_tokens) * 0.15;
        let score = (cosine_similarity(&target_vector, &peer_vector) * 0.85) + lexical_boost;
        if score >= 0.40 {
            peers.push(OverlapMatch {
                path: peer_key,
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

pub fn find_peer_overlap(
    config: &Config,
    wiki_dir: &Path,
    category: &[String],
    title: &str,
    body: &str,
    exclude_slug: Option<&str>,
) -> Result<Vec<OverlapMatch>> {
    match config.overlap.provider() {
        "local-cached" => vector_overlap(config, wiki_dir, category, title, body, exclude_slug),
        _ => jaccard_overlap(wiki_dir, category, title, body, exclude_slug),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, OverlapConfig, WikiConfig};

    fn base_config(provider: &str, cache_dir: Option<PathBuf>) -> Config {
        Config {
            overlap: OverlapConfig {
                provider: Some(provider.to_string()),
                embedding_cache_dir: cache_dir,
            },
            wiki: WikiConfig {
                wiki_dir: PathBuf::from("wiki"),
                auto_commit: true,
                sync: Default::default(),
            },
            ..Default::default()
        }
    }

    fn write_page(path: &Path, title: &str, body: &str, category: &[&str]) {
        let raw = format!(
            "---\nid: {}\ntitle: \"{}\"\nstatus: published\nsource:\n  kind: web_page\n  id: src\n  origin_url: https://x\ncategory:\n{}\nkeywords: []\ncreated_at: \"2026-01-01T00:00:00Z\"\nupdated_at: \"2026-01-01T00:00:00Z\"\ncross_refs: []\ncontent_hash: \"h\"\n---\n\n{}\n",
            path.file_stem().and_then(|s| s.to_str()).unwrap_or("page"),
            title,
            category
                .iter()
                .map(|c| format!("  - {}\n", c))
                .collect::<String>(),
            body
        );
        std::fs::write(path, raw).unwrap();
    }

    #[test]
    fn local_cached_provider_writes_embedding_cache() {
        let tmp = tempfile::tempdir().unwrap();
        let wiki_dir = tmp.path().join("wiki");
        let published = wiki_dir.join("published").join("tree");
        std::fs::create_dir_all(&published).unwrap();
        write_page(
            &published.join("peer-one.md"),
            "Connection Failure",
            "The database connection failed after the driver rebooted.",
            &["tree"],
        );
        write_page(
            &published.join("peer-two.md"),
            "Driver Error",
            "The connector error appears after the service restarts.",
            &["tree"],
        );

        let config = base_config("local-cached", Some(tmp.path().join("cache")));
        let matches = find_peer_overlap(
            &config,
            &wiki_dir,
            &["tree".to_string()],
            "Connection Error",
            "The connection failed and the driver did not recover.",
            None,
        )
        .unwrap();

        assert!(
            !matches.is_empty(),
            "local-cached provider should score peers"
        );
        assert!(tmp.path().join("cache").exists());
    }

    #[test]
    fn jaccard_provider_still_defaults() {
        let tmp = tempfile::tempdir().unwrap();
        let wiki_dir = tmp.path().join("wiki");
        let published = wiki_dir.join("published").join("tree");
        std::fs::create_dir_all(&published).unwrap();
        write_page(
            &published.join("peer-one.md"),
            "Connection Failure",
            "Database connection failure guidance.",
            &["tree"],
        );

        let config = base_config("jaccard", None);
        let matches = find_peer_overlap(
            &config,
            &wiki_dir,
            &["tree".to_string()],
            "Connection Failure",
            "Database connection failure guidance.",
            None,
        )
        .unwrap();

        assert!(!matches.is_empty());
    }
}
