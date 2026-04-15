use anyhow::{Context, Result};
use dirs;
use dotenvy;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
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
    #[serde(default)]
    pub llm: LlmConfig,
    #[serde(default)]
    pub heal: HealConfig,
}

/// LLM inference settings — required for `curio process` (routing) and `curio query`.
///
/// **Recommended:** Set `OPENAI_API_KEY` in your environment (via OAuth token exchange,
/// your org's SSO-issued key, or a shared team secret). Do not commit API keys to .curio.yaml.
///
/// The `api_key` field in config is supported for compatibility but discouraged for
/// personal/org use — prefer environment variables managed via your auth flow.
#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct LlmConfig {
    /// OpenAI API key. Prefer OPENAI_API_KEY env var over setting this directly in config.
    /// See: https://platform.openai.com/docs/guides/authentication
    #[serde(default)]
    pub api_key: String,
    /// Model to use for routing/analysis. Default: gpt-4o.
    #[serde(default = "LlmConfig::default_model")]
    pub model: String,
}

impl LlmConfig {
    fn default_model() -> String {
        "gpt-4o".to_string()
    }

    pub fn effective_api_key(&self) -> String {
        if !self.api_key.is_empty() {
            return self.api_key.clone();
        }
        std::env::var("OPENAI_API_KEY").unwrap_or_default()
    }

    pub fn effective_model(&self) -> String {
        if self.model.is_empty() {
            return Self::default_model();
        }
        self.model.clone()
    }

    pub fn require_api_key(&self) -> anyhow::Result<String> {
        let key = self.effective_api_key();
        if key.is_empty() {
            anyhow::bail!(
                "OpenAI API key not configured. Set OPENAI_API_KEY env var or add to .curio.yaml under llm.api_key."
            );
        }
        Ok(key)
    }
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

/// Self-healing configuration, read from `wiki/_config/settings.yaml`.
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct HealConfig {
    pub confidence_threshold: Option<f64>,
    pub show_auto_heal_callout: Option<bool>,
    pub auto_heal_label: Option<String>,
    pub max_pages_per_run: Option<u32>,
    pub stale_threshold_days: Option<u32>,
    pub overlap_threshold: Option<f64>,
    pub external_search_enabled: Option<bool>,
    pub min_body_words: Option<u32>,
}

impl HealConfig {
    pub fn confidence_threshold(&self) -> f64 {
        self.confidence_threshold.unwrap_or(0.85)
    }
    pub fn show_auto_heal_callout(&self) -> bool {
        self.show_auto_heal_callout.unwrap_or(true)
    }
    pub fn auto_heal_label(&self) -> &str {
        self.auto_heal_label.as_deref().unwrap_or("curio:auto-healed")
    }
    pub fn max_pages_per_run(&self) -> u32 {
        self.max_pages_per_run.unwrap_or(20)
    }
    pub fn stale_threshold_days(&self) -> u32 {
        self.stale_threshold_days.unwrap_or(240)
    }
    pub fn overlap_threshold(&self) -> f64 {
        self.overlap_threshold.unwrap_or(0.60)
    }
    pub fn external_search_enabled(&self) -> bool {
        self.external_search_enabled.unwrap_or(true)
    }
    pub fn min_body_words(&self) -> u32 {
        self.min_body_words.unwrap_or(50)
    }
}

/// Minimal struct for deserializing `wiki/_config/settings.yaml`.
/// Only used internally during config loading.
#[derive(Debug, Deserialize, Default)]
pub(crate) struct WikiSettingsFile {
    #[serde(default)]
    pub heal: Option<HealConfig>,
}

/// Load config, optionally targeting a specific KB directory.
///
/// `kb_dir` is the root of a KB store (the directory containing `wiki/` and `NORTHSTAR.md`).
/// When provided it is used as `config_root` — the place where `.curio.yaml` and `.env` are
/// looked for first. When `None`, the old behaviour applies: CWD / `CURIO_REPO_ROOT`.
///
/// Credential resolution order (3-tier):
///   1. KB `<kb_dir>/.env` — per-KB secrets, git-ignored in the KB repo
///   2. Shell environment variables — CI/CD, current session
///   3. Harness `<CURIO_HARNESS_DIR>/.env` — user-level fallback (not KB-specific)
pub fn load_config(config_path: Option<&str>, kb_dir: Option<&std::path::Path>) -> Result<Config> {
    let mut config = Config::default();
    let config_root: PathBuf = if let Some(dir) = kb_dir {
        dir.to_path_buf()
    } else {
        repo_root_override()
            .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
    };

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

    // 2. Load .env files — KB first, then harness fallback.
    //    dotenvy does NOT override already-set env vars, so KB values win over harness values.
    //    Shell env vars are already present and take precedence over both .env files.
    load_env_tier(&config_root);
    if let Ok(harness_dir) = env::var("CURIO_HARNESS_DIR") {
        let harness_path = PathBuf::from(&harness_dir);
        if harness_path != config_root {
            load_env_tier(&harness_path);
        }
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
    if let Ok(parent_page_id) = env::var("CURIO_CONFLUENCE_PARENT_PAGE_ID") {
        if !parent_page_id.trim().is_empty() {
            config.wiki.sync.confluence_parent_page_id = Some(parent_page_id);
        }
    }
    // OPENAI_API_KEY is read by LlmConfig::effective_api_key() at call time — no caching needed.

    if config.runtime.temp_dir.is_none() {
        config.runtime.temp_dir = Some(default_temp_dir());
    }

    // wiki_dir defaults to <config_root>/wiki if not absolute
    if config.wiki.wiki_dir.is_relative() {
        config.wiki.wiki_dir = config_root.join(&config.wiki.wiki_dir);
    }

    // Load wiki/_config/settings.yaml for heal settings (and any future wiki-level config).
    // This file is synced to Confluence, making it the user-visible settings surface.
    let wiki_settings_path = config.wiki.wiki_dir.join("_config/settings.yaml");
    if wiki_settings_path.exists() {
        if let Ok(raw) = fs::read_to_string(&wiki_settings_path) {
            match serde_yaml::from_str::<WikiSettingsFile>(&raw) {
                Ok(ws) => {
                    if let Some(heal) = ws.heal {
                        config.heal = heal;
                    }
                }
                Err(e) => {
                    eprintln!("curio: warning: failed to parse {}: {e}", wiki_settings_path.display());
                }
            }
        }
    }

    Ok(config)
}

/// Load a .env file from `dir` without overriding already-set env vars.
fn load_env_tier(dir: &Path) {
    let env_path = dir.join(".env");
    if env_path.is_file() {
        dotenvy::from_path(&env_path).ok();
    }
}

fn default_temp_dir() -> PathBuf {
    env::temp_dir().join("curio")
}

fn repo_root_override() -> Option<PathBuf> {
    let value = env::var_os("CURIO_REPO_ROOT")?;
    if value.is_empty() { None } else { Some(PathBuf::from(value)) }
}

pub fn repo_root() -> PathBuf {
    repo_root_override()
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
}

pub fn upsert_repo_env_var(key: &str, value: &str) -> Result<()> {
    let env_path = repo_root().join(".env");
    let mut lines: Vec<String> = if env_path.exists() {
        fs::read_to_string(&env_path)
            .with_context(|| format!("Failed to read env file: {}", env_path.display()))?
            .lines()
            .map(|line| line.to_string())
            .collect()
    } else {
        vec!["# Curio harness config".to_string()]
    };

    let mut replaced = false;
    for line in &mut lines {
        if let Some((existing_key, _)) = line.split_once('=') {
            if existing_key.trim() == key {
                *line = format!("{}={}", key, value);
                replaced = true;
                break;
            }
        }
    }

    if !replaced {
        if !lines.is_empty() && !lines.last().map(|l| l.is_empty()).unwrap_or(false) {
            lines.push(String::new());
        }
        lines.push(format!("{}={}", key, value));
    }

    let content = lines.join("\n") + "\n";
    fs::write(&env_path, content)
        .with_context(|| format!("Failed to write env file: {}", env_path.display()))?;
    Ok(())
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

    fn clear_test_env() {
        for key in [
            "CURIO_CONFLUENCE_URL",
            "CURIO_CONFLUENCE_EMAIL",
            "CURIO_SPACE_KEY",
            "CURIO_TEMP_DIR",
            "CURIO_WIKI_DIR",
            "CURIO_HARNESS_DIR",
            "CURIO_REPO_ROOT",
        ] {
            unsafe {
                env::remove_var(key);
            }
        }
    }

    #[test]
    fn test_load_config_from_env() {
        let _guard = env_test_lock();
        clear_test_env();

        unsafe {
            env::set_var("CURIO_CONFLUENCE_URL", "http://test.confluence.com");
            env::set_var("CURIO_CONFLUENCE_EMAIL", "test@example.com");
            env::set_var("CURIO_SPACE_KEY", "TEST");
        }

        let config = load_config(None, None).unwrap();

        assert_eq!(config.connection.confluence_url, "http://test.confluence.com");
        assert_eq!(config.connection.confluence_email, "test@example.com");
        assert_eq!(config.content_model.space_key, "TEST");
        assert_eq!(config.runtime.temp_dir, Some(env::temp_dir().join("curio")));

        clear_test_env();
    }

    #[test]
    fn test_load_config_works_without_confluence() {
        let _guard = env_test_lock();
        clear_test_env();

        // Should not bail even with no Confluence config
        let result = load_config(None, None);
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
    fn test_heal_config_defaults() {
        let h = HealConfig::default();
        assert_eq!(h.confidence_threshold(), 0.85);
        assert!(h.show_auto_heal_callout());
        assert_eq!(h.auto_heal_label(), "curio:auto-healed");
        assert_eq!(h.max_pages_per_run(), 20);
        assert_eq!(h.stale_threshold_days(), 240);
        assert!((h.overlap_threshold() - 0.60).abs() < f64::EPSILON);
        assert!(h.external_search_enabled());
        assert_eq!(h.min_body_words(), 50);
    }

    #[test]
    fn test_heal_config_override() {
        let h = HealConfig {
            confidence_threshold: Some(0.7),
            show_auto_heal_callout: Some(false),
            auto_heal_label: Some("custom:label".to_string()),
            max_pages_per_run: Some(5),
            stale_threshold_days: Some(120),
            overlap_threshold: Some(0.80),
            external_search_enabled: Some(false),
            min_body_words: Some(100),
        };
        assert_eq!(h.confidence_threshold(), 0.7);
        assert!(!h.show_auto_heal_callout());
        assert_eq!(h.auto_heal_label(), "custom:label");
        assert_eq!(h.max_pages_per_run(), 5);
        assert_eq!(h.stale_threshold_days(), 120);
        assert!((h.overlap_threshold() - 0.80).abs() < f64::EPSILON);
        assert!(!h.external_search_enabled());
        assert_eq!(h.min_body_words(), 100);
    }

    #[test]
    fn test_settings_yaml_is_loaded() {
        // Verify the actual wiki/_config/settings.yaml can be parsed.
        let settings_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("wiki/_config/settings.yaml");
        if settings_path.exists() {
            let raw = std::fs::read_to_string(&settings_path).unwrap();
            let ws: WikiSettingsFile = serde_yaml::from_str(&raw).expect("settings.yaml should parse cleanly");
            if let Some(heal) = ws.heal {
                // Confirm defaults are in-range
                assert!(heal.confidence_threshold() >= 0.0 && heal.confidence_threshold() <= 1.0);
            }
        }
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
