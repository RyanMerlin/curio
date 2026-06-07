use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct LocalCachedEmbeddings {
    pub dimensions: usize,
    pub cache_dir: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
struct CachedEmbedding {
    source_hash: String,
    vector: Vec<f32>,
}

impl LocalCachedEmbeddings {
    pub fn new(cache_dir: impl Into<PathBuf>) -> Self {
        Self {
            dimensions: 128,
            cache_dir: cache_dir.into(),
        }
    }

    pub fn embed(&self, cache_key: &str, text: &str) -> Result<Vec<f32>> {
        fs::create_dir_all(&self.cache_dir).with_context(|| {
            format!(
                "Failed to create embedding cache dir: {}",
                self.cache_dir.display()
            )
        })?;

        let source_hash = hash_text(text);
        let cache_path = self
            .cache_dir
            .join(format!("{}.json", hash_text(cache_key)));
        if let Ok(raw) = fs::read_to_string(&cache_path)
            && let Ok(cached) = serde_json::from_str::<CachedEmbedding>(&raw)
            && cached.source_hash == source_hash
            && cached.vector.len() == self.dimensions
        {
            return Ok(cached.vector);
        }

        let vector = self.vectorize(text);
        let cached = CachedEmbedding {
            source_hash,
            vector: vector.clone(),
        };
        fs::write(&cache_path, serde_json::to_string_pretty(&cached)?).with_context(|| {
            format!(
                "Failed to write embedding cache entry: {}",
                cache_path.display()
            )
        })?;
        Ok(vector)
    }

    fn vectorize(&self, text: &str) -> Vec<f32> {
        let mut vector = vec![0.0f32; self.dimensions];
        let normalized = normalize_text(text);
        let tokens = tokenize(&normalized);
        for token in &tokens {
            add_feature(&mut vector, self.dimensions, token, 1.0);
            for gram in character_ngrams(token, 3) {
                add_feature(&mut vector, self.dimensions, &gram, 0.35);
            }
        }
        let total = vector.iter().map(|v| v * v).sum::<f32>().sqrt();
        if total > 0.0 {
            for value in &mut vector {
                *value /= total;
            }
        }
        vector
    }
}

pub fn cosine_similarity(left: &[f32], right: &[f32]) -> f32 {
    let len = left.len().min(right.len());
    if len == 0 {
        return 0.0;
    }
    let mut dot = 0.0f32;
    let mut left_mag = 0.0f32;
    let mut right_mag = 0.0f32;
    for i in 0..len {
        dot += left[i] * right[i];
        left_mag += left[i] * left[i];
        right_mag += right[i] * right[i];
    }
    if left_mag == 0.0 || right_mag == 0.0 {
        0.0
    } else {
        (dot / (left_mag.sqrt() * right_mag.sqrt())).clamp(0.0, 1.0)
    }
}

fn normalize_text(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { ' ' })
        .collect::<String>()
}

fn tokenize(text: &str) -> Vec<String> {
    text.split_whitespace()
        .filter(|part| part.len() >= 3)
        .map(normalize_token)
        .filter(|part| !part.is_empty())
        .collect()
}

fn normalize_token(token: &str) -> String {
    let trimmed = token.trim().to_string();
    if trimmed.ends_with("ies") && trimmed.len() > 4 {
        return format!("{}y", &trimmed[..trimmed.len() - 3]);
    }
    if trimmed.ends_with("ing") && trimmed.len() > 5 {
        return trimmed[..trimmed.len() - 3].to_string();
    }
    if trimmed.ends_with('s') && trimmed.len() > 4 {
        return trimmed[..trimmed.len() - 1].to_string();
    }
    trimmed
}

fn character_ngrams(token: &str, n: usize) -> Vec<String> {
    if token.len() <= n {
        return vec![token.to_string()];
    }
    token
        .chars()
        .collect::<Vec<_>>()
        .windows(n)
        .map(|window| window.iter().collect::<String>())
        .collect()
}

fn add_feature(vector: &mut [f32], dimensions: usize, feature: &str, weight: f32) {
    let mut hasher = Sha256::new();
    hasher.update(feature.as_bytes());
    let hash = hasher.finalize();
    let idx = u64::from_le_bytes(hash[0..8].try_into().unwrap_or([0; 8])) as usize % dimensions;
    vector[idx] += weight;
}

fn hash_text(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{:02x}", byte))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_round_trip_preserves_vector() {
        let tmp = tempfile::tempdir().unwrap();
        let cache = LocalCachedEmbeddings::new(tmp.path());
        let first = cache.embed("key", "Database connection error").unwrap();
        let second = cache.embed("key", "Database connection error").unwrap();
        assert_eq!(first, second);
        assert!(
            tmp.path()
                .join(format!("{}.json", hash_text("key")))
                .exists()
        );
    }

    #[test]
    fn cosine_similarity_is_bounded() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        assert_eq!(cosine_similarity(&a, &b), 0.0);
        let c = vec![1.0, 1.0, 0.0];
        let sim = cosine_similarity(&a, &c);
        assert!(sim > 0.0 && sim <= 1.0);
    }
}
