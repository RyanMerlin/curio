use anyhow::{Context, Result};
use dirs;
use dotenvy;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::{env, fs};

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct Config {
    pub connection: ConnectionConfig,
    pub content_model: ContentModelConfig,
    pub runtime: RuntimeConfig,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct ConnectionConfig {
    pub confluence_url: String,
    pub confluence_email: String,
    // Confluence token will be read from environment variable
    // pub confluence_token: String,
    // Add other connection-related fields as needed (e.g., Jira, Slack)
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ContentModelConfig {
    pub space_key: String,
    pub root_folder_name: String,
    pub output_root_folder_id: Option<String>,
    pub label_namespace: String, // e.g., "curio"
                                 // Add other content model-related fields (e.g., templates)
}

impl Default for ContentModelConfig {
    fn default() -> Self {
        ContentModelConfig {
            space_key: String::default(),
            root_folder_name: String::default(),
            output_root_folder_id: None,
            label_namespace: "curio".to_string(), // Default label namespace
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct RuntimeConfig {
    pub temp_dir: Option<PathBuf>,
    pub log_level: Option<String>,
    // Add other runtime-related fields (e.g., retry policy, watcher intervals)
}

pub fn load_config(config_path: Option<&str>) -> Result<Config> {
    let mut config = Config::default();

    // 1. Load from default config files
    let mut candidates = Vec::new();

    // Current directory
    candidates.push(PathBuf::from(".curio.yaml"));
    candidates.push(PathBuf::from("curio.yaml"));

    // Home directory
    if let Some(home_dir) = dirs::home_dir() {
        candidates.push(home_dir.join(".curio.yaml"));
        candidates.push(home_dir.join("curio.yaml"));
    }

    // Explicitly provided config path
    if let Some(path_str) = config_path {
        candidates.insert(0, PathBuf::from(path_str)); // Prioritize explicit path
    }

    for candidate in candidates {
        if candidate.exists() {
            let raw = fs::read_to_string(&candidate)
                .with_context(|| format!("Failed to read config file: {}", candidate.display()))?;
            let loaded_config: Config = serde_yaml::from_str(&raw)
                .with_context(|| format!("Failed to parse config file: {}", candidate.display()))?;

            // Merge loaded config into default config (simple merge, last one wins)
            if loaded_config.connection.confluence_url != config.connection.confluence_url {
                config.connection.confluence_url = loaded_config.connection.confluence_url;
            }
            if loaded_config.connection.confluence_email != config.connection.confluence_email {
                config.connection.confluence_email = loaded_config.connection.confluence_email;
            }
            if loaded_config.content_model.space_key != config.content_model.space_key {
                config.content_model.space_key = loaded_config.content_model.space_key;
            }
            if loaded_config.content_model.root_folder_name != config.content_model.root_folder_name
            {
                config.content_model.root_folder_name =
                    loaded_config.content_model.root_folder_name;
            }
            if loaded_config.content_model.output_root_folder_id
                != config.content_model.output_root_folder_id
            {
                config.content_model.output_root_folder_id =
                    loaded_config.content_model.output_root_folder_id;
            }
            if loaded_config.runtime.temp_dir != config.runtime.temp_dir {
                config.runtime.temp_dir = loaded_config.runtime.temp_dir;
            }
            if loaded_config.runtime.log_level != config.runtime.log_level {
                config.runtime.log_level = loaded_config.runtime.log_level;
            }
            // This is a basic merge; a more robust solution would be recursive or field-specific.
        }
    }

    // 2. Load from environment variables (overrides file config)
    dotenvy::dotenv().ok(); // Load .env file

    if let Ok(url) = env::var("CURIO_CONFLUENCE_URL") {
        config.connection.confluence_url = url;
    }
    if let Ok(email) = env::var("CURIO_CONFLUENCE_EMAIL") {
        config.connection.confluence_email = email;
    }
    if let Ok(_token) = env::var("CURIO_CONFLUENCE_TOKEN") {
        // Token is read but not stored in config, it's passed directly to client.
        // Keeping it here with _ to avoid unused variable warning.
    }
    if let Ok(space_key) = env::var("CURIO_SPACE_KEY") {
        config.content_model.space_key = space_key;
    }
    if let Ok(root_name) = env::var("CURIO_ROOT_FOLDER_NAME") {
        config.content_model.root_folder_name = root_name;
    }
    if let Ok(output_root_folder_id) = env::var("CURIO_CONFLUENCE_OUTPUT_ROOT_FOLDER_ID") {
        let trimmed = output_root_folder_id.trim();
        if !trimmed.is_empty() {
            config.content_model.output_root_folder_id = Some(trimmed.to_string());
        }
    }
    if let Ok(temp_dir) = env::var("CURIO_TEMP_DIR") {
        if !temp_dir.trim().is_empty() {
            config.runtime.temp_dir = Some(PathBuf::from(temp_dir));
        }
    }
    if let Ok(log_level) = env::var("CURIO_LOG_LEVEL") {
        config.runtime.log_level = Some(log_level);
    }

    if config.runtime.temp_dir.is_none() {
        config.runtime.temp_dir = Some(default_temp_dir());
    }

    // Validate essential config
    if config.connection.confluence_url.is_empty() {
        anyhow::bail!(
            "Confluence URL is not configured. Set CURIO_CONFLUENCE_URL environment variable or in config file."
        );
    }
    if config.connection.confluence_email.is_empty() {
        anyhow::bail!(
            "Confluence email is not configured. Set CURIO_CONFLUENCE_EMAIL environment variable or in config file."
        );
    }
    if config.content_model.space_key.is_empty() {
        anyhow::bail!(
            "Confluence space key is not configured. Set CURIO_SPACE_KEY environment variable or in config file."
        );
    }
    if config.content_model.root_folder_name.is_empty()
        && config.content_model.output_root_folder_id.is_none()
    {
        anyhow::bail!(
            "Confluence output root folder is not configured. Set CURIO_CONFLUENCE_OUTPUT_ROOT_FOLDER_ID environment variable or in config file, or set CURIO_ROOT_FOLDER_NAME as a fallback."
        );
    }
    if config.content_model.label_namespace.is_empty() {
        anyhow::bail!(
            "Confluence label namespace is not configured. Set CURIO_LABEL_NAMESPACE environment variable or in config file, or use the default 'curio'."
        );
    }

    Ok(config)
}

fn default_temp_dir() -> PathBuf {
    env::temp_dir().join("curio")
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
        // Clear environment variables to ensure a clean test state
        unsafe {
            env::remove_var("CURIO_CONFLUENCE_URL");
            env::remove_var("CURIO_CONFLUENCE_EMAIL");
            env::remove_var("CURIO_SPACE_KEY");
            env::remove_var("CURIO_ROOT_FOLDER_NAME");
            env::remove_var("CURIO_CONFLUENCE_OUTPUT_ROOT_FOLDER_ID");
            env::remove_var("CURIO_TEMP_DIR");
        }

        unsafe {
            env::set_var("CURIO_CONFLUENCE_URL", "http://test.confluence.com");
            env::set_var("CURIO_CONFLUENCE_EMAIL", "test@example.com");
            env::set_var("CURIO_SPACE_KEY", "TEST");
            env::set_var("CURIO_CONFLUENCE_OUTPUT_ROOT_FOLDER_ID", "12345");
        }

        let config = load_config(None).unwrap();

        assert_eq!(
            config.connection.confluence_url,
            "http://test.confluence.com"
        );
        assert_eq!(config.connection.confluence_email, "test@example.com");
        assert_eq!(config.content_model.space_key, "TEST");
        assert_eq!(config.content_model.root_folder_name, "");
        assert_eq!(
            config.content_model.output_root_folder_id,
            Some("12345".to_string())
        );
        assert_eq!(config.runtime.temp_dir, Some(env::temp_dir().join("curio")));

        // Clean up environment variables
        unsafe {
            env::remove_var("CURIO_CONFLUENCE_URL");
            env::remove_var("CURIO_CONFLUENCE_EMAIL");
            env::remove_var("CURIO_SPACE_KEY");
            env::remove_var("CURIO_CONFLUENCE_OUTPUT_ROOT_FOLDER_ID");
            env::remove_var("CURIO_TEMP_DIR");
        }
    }

    #[test]
    fn test_load_config_with_folder_id_only() {
        let _guard = env_test_lock();
        unsafe {
            env::remove_var("CURIO_CONFLUENCE_URL");
            env::remove_var("CURIO_CONFLUENCE_EMAIL");
            env::remove_var("CURIO_SPACE_KEY");
            env::remove_var("CURIO_ROOT_FOLDER_NAME");
            env::remove_var("CURIO_CONFLUENCE_OUTPUT_ROOT_FOLDER_ID");
            env::remove_var("CURIO_TEMP_DIR");

            env::set_var("CURIO_CONFLUENCE_URL", "http://test.confluence.com");
            env::set_var("CURIO_CONFLUENCE_EMAIL", "test@example.com");
            env::set_var("CURIO_SPACE_KEY", "TEST");
            env::set_var("CURIO_CONFLUENCE_OUTPUT_ROOT_FOLDER_ID", "4241129474");
        }

        let config = load_config(None).unwrap();

        assert_eq!(config.content_model.root_folder_name, "");
        assert_eq!(
            config.content_model.output_root_folder_id,
            Some("4241129474".to_string())
        );

        unsafe {
            env::remove_var("CURIO_CONFLUENCE_URL");
            env::remove_var("CURIO_CONFLUENCE_EMAIL");
            env::remove_var("CURIO_SPACE_KEY");
            env::remove_var("CURIO_CONFLUENCE_OUTPUT_ROOT_FOLDER_ID");
            env::remove_var("CURIO_TEMP_DIR");
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
                        trimmed
                            .split_once('=')
                            .map(|(key, _)| key.trim().to_string())
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
