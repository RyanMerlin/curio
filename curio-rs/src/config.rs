use anyhow::{Context, Result};
use dirs;
use dotenvy;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::{env, fs};

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct Config {
    #[serde(default)]
    pub connection: ConnectionConfig,
    #[serde(default)]
    pub content_model: ContentModelConfig,
    #[serde(default)]
    pub runtime: RuntimeConfig,
    #[serde(default)]
    pub wiki: WikiConfig,
}

/// Confluence connection settings — only required when sync or Confluence-source intake is used.
#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct ConnectionConfig {
    pub confluence_url: String,
    pub confluence_email: String,
}

impl ConnectionConfig {
    /// Validate that Confluence credentials are present.
    /// Call this in commands that need Confluence (sync, intake-from-confluence).
    pub fn require_confluence(&self) -> Result<()> {
        if self.confluence_url.is_empty() {
            anyhow::bail!(
                "Confluence URL is not configured. Set CURIO_CONFLUENCE_URL or add to .curio.yaml."
            );
        }
        if self.confluence_email.is_empty() {
            anyhow::bail!(
                "Confluence email is not configured. Set CURIO_CONFLUENCE_EMAIL or add to .curio.yaml."
            );
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ContentModelConfig {
    pub space_key: String,
    #[serde(default = "ContentModelConfig::default_label_namespace")]
    pub label_namespace: String,
}

impl ContentModelConfig {
    fn default_label_namespace() -> String { "curio".to_string() }
}

impl Default for ContentModelConfig {
    fn default() -> Self {
        ContentModelConfig {
            space_key: String::default(),
            label_namespace: "curio".to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct RuntimeConfig {
    pub temp_dir: Option<PathBuf>,
    pub log_level: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WikiConfig {
    /// Root directory of the wiki, relative to repo root.  Default: `wiki`.
    pub wiki_dir: PathBuf,
    /// If true, pipeline commands commit after every mutation. Default: `true`.
    pub auto_commit: bool,
    /// Confluence sync settings.
    pub sync: SyncConfig,
}

impl Default for WikiConfig {
    fn default() -> Self {
        WikiConfig {
            wiki_dir: PathBuf::from("wiki"),
            auto_commit: true,
            sync: SyncConfig::default(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct SyncConfig {
    pub enabled: bool,
    /// Numeric ID of the Confluence parent page to publish under.
    pub confluence_parent_page_id: Option<String>,
}

pub fn load_config(config_path: Option<&str>) -> Result<Config> {
    let mut config = Config::default();
    let config_root = repo_root_override()
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    // 1. Load from config files (first match wins)
    let mut candidates = Vec::new();
    candidates.push(config_root.join(".curio.yaml"));
    candidates.push(config_root.join("curio.yaml"));
    if let Some(home_dir) = dirs::home_dir() {
        candidates.push(home_dir.join(".curio.yaml"));
        candidates.push(home_dir.join("curio.yaml"));
    }
    if let Some(path_str) = config_path {
        candidates.insert(0, PathBuf::from(path_str));
    }

    for candidate in candidates {
        if candidate.exists() {
            let raw = fs::read_to_string(&candidate)
                .with_context(|| format!("Failed to read config file: {}", candidate.display()))?;
            let loaded: Config = serde_yaml::from_str(&raw)
                .with_context(|| format!("Failed to parse config file: {}", candidate.display()))?;

            // Merge (non-default values override defaults)
            if !loaded.connection.confluence_url.is_empty() {
                config.connection.confluence_url = loaded.connection.confluence_url;
            }
            if !loaded.connection.confluence_email.is_empty() {
                config.connection.confluence_email = loaded.connection.confluence_email;
            }
            if !loaded.content_model.space_key.is_empty() {
                config.content_model.space_key = loaded.content_model.space_key;
            }
            if loaded.content_model.label_namespace != ContentModelConfig::default().label_namespace {
                config.content_model.label_namespace = loaded.content_model.label_namespace;
            }
            if loaded.runtime.temp_dir.is_some() {
                config.runtime.temp_dir = loaded.runtime.temp_dir;
            }
            if loaded.runtime.log_level.is_some() {
                config.runtime.log_level = loaded.runtime.log_level;
            }
            if loaded.wiki.wiki_dir != WikiConfig::default().wiki_dir {
                config.wiki.wiki_dir = loaded.wiki.wiki_dir;
            }
            if !loaded.wiki.auto_commit {
                config.wiki.auto_commit = false;
            }
            if loaded.wiki.sync.enabled {
                config.wiki.sync.enabled = true;
            }
            if loaded.wiki.sync.confluence_parent_page_id.is_some() {
                config.wiki.sync.confluence_parent_page_id =
                    loaded.wiki.sync.confluence_parent_page_id;
            }
            break; // First match wins
        }
    }

    // 2. Load .env file
    let env_path = config_root.join(".env");
    if env_path.is_file() {
        dotenvy::from_path(&env_path)
            .with_context(|| format!("Failed to load env file: {}", env_path.display()))?;
    } else {
        dotenvy::dotenv().ok();
    }

    // 3. Override with environment variables
    if let Ok(url) = env::var("CURIO_CONFLUENCE_URL") {
        config.connection.confluence_url = url;
    }
    if let Ok(email) = env::var("CURIO_CONFLUENCE_EMAIL") {
        config.connection.confluence_email = email;
    }
    // CURIO_CONFLUENCE_TOKEN is read directly by ConfluenceClient, not stored here.
    if let Ok(space_key) = env::var("CURIO_SPACE_KEY") {
        config.content_model.space_key = space_key;
    }
    if let Ok(temp_dir) = env::var("CURIO_TEMP_DIR") {
        if !temp_dir.trim().is_empty() {
            config.runtime.temp_dir = Some(PathBuf::from(temp_dir));
        }
    }
    if let Ok(log_level) = env::var("CURIO_LOG_LEVEL") {
        config.runtime.log_level = Some(log_level);
    }
    if let Ok(wiki_dir) = env::var("CURIO_WIKI_DIR") {
        if !wiki_dir.trim().is_empty() {
            config.wiki.wiki_dir = PathBuf::from(wiki_dir);
        }
    }

    if config.runtime.temp_dir.is_none() {
        config.runtime.temp_dir = Some(default_temp_dir());
    }

    // wiki_dir defaults to <repo_root>/wiki if not absolute
    if config.wiki.wiki_dir.is_relative() {
        config.wiki.wiki_dir = config_root.join(&config.wiki.wiki_dir);
    }

    Ok(config)
}

fn default_temp_dir() -> PathBuf {
    env::temp_dir().join("curio")
}

fn repo_root_override() -> Option<PathBuf> {
    let value = env::var_os("CURIO_REPO_ROOT")?;
    if value.is_empty() { None } else { Some(PathBuf::from(value)) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    fn env_test_lock() -> std::sync::MutexGuard<'static, ()> {
        static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        ENV_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .expect("env test lock poisoned")
    }

    #[test]
    fn test_load_config_from_env() {
        let _guard = env_test_lock();
        unsafe {
            env::remove_var("CURIO_CONFLUENCE_URL");
            env::remove_var("CURIO_CONFLUENCE_EMAIL");
            env::remove_var("CURIO_SPACE_KEY");
            env::remove_var("CURIO_TEMP_DIR");
            env::remove_var("CURIO_WIKI_DIR");
        }

        unsafe {
            env::set_var("CURIO_CONFLUENCE_URL", "http://test.confluence.com");
            env::set_var("CURIO_CONFLUENCE_EMAIL", "test@example.com");
            env::set_var("CURIO_SPACE_KEY", "TEST");
        }

        let config = load_config(None).unwrap();

        assert_eq!(config.connection.confluence_url, "http://test.confluence.com");
        assert_eq!(config.connection.confluence_email, "test@example.com");
        assert_eq!(config.content_model.space_key, "TEST");
        assert_eq!(config.runtime.temp_dir, Some(env::temp_dir().join("curio")));

        unsafe {
            env::remove_var("CURIO_CONFLUENCE_URL");
            env::remove_var("CURIO_CONFLUENCE_EMAIL");
            env::remove_var("CURIO_SPACE_KEY");
            env::remove_var("CURIO_TEMP_DIR");
        }
    }

    #[test]
    fn test_load_config_works_without_confluence() {
        let _guard = env_test_lock();
        unsafe {
            env::remove_var("CURIO_CONFLUENCE_URL");
            env::remove_var("CURIO_CONFLUENCE_EMAIL");
            env::remove_var("CURIO_SPACE_KEY");
        }

        // Should not bail even with no Confluence config
        let result = load_config(None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_require_confluence_fails_without_creds() {
        let conn = ConnectionConfig::default();
        assert!(conn.require_confluence().is_err());
    }

    #[test]
    fn test_require_confluence_passes_with_creds() {
        let conn = ConnectionConfig {
            confluence_url: "https://test.atlassian.net".to_string(),
            confluence_email: "user@test.com".to_string(),
        };
        assert!(conn.require_confluence().is_ok());
    }

    #[test]
    fn test_env_files_have_matching_keys() {
        fn read_keys(path: std::path::PathBuf) -> std::collections::BTreeSet<String> {
            std::fs::read_to_string(&path)
                .unwrap_or_else(|err| panic!("failed to read {}: {}", path.display(), err))
                .lines()
                .filter_map(|line| {
                    let trimmed = line.trim();
                    if trimmed.is_empty() || trimmed.starts_with('#') {
                        None
                    } else {
                        trimmed.split_once('=').map(|(key, _)| key.trim().to_string())
                    }
                })
                .collect()
        }

        let repo_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("crate should have a parent repo directory")
            .to_path_buf();
        let env_keys = read_keys(repo_root.join(".env"));
        let example_keys = read_keys(repo_root.join(".env.example"));
        assert_eq!(env_keys, example_keys);
    }
}
