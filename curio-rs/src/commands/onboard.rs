use crate::{
    commands::sync::{ensure_curio_confluence_tree, parse_northstar_blueprint},
    config::{Config, ConnectionConfig, ContentModelConfig, RuntimeConfig},
    confluence::ConfluenceClient,
    harness::{HarnessPaths, run_checks},
    northstar::ensure_northstar_markdown,
};
use anyhow::{Context, Result, bail};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};

const CURIO_ENV_KEYS: &[&str] = &[
    "CURIO_CONFLUENCE_URL",
    "CURIO_CONFLUENCE_EMAIL",
    "CURIO_CONFLUENCE_TOKEN",
    "CURIO_SPACE_KEY",
    "CURIO_CONFLUENCE_PARENT_PAGE_ID",
    "CURIO_TEMP_DIR",
    "CURIO_AUDIT_DIR",
];

const CURIO_ROOT_TITLE: &str = "CURIO";
const CURIO_ROOT_STRUCTURE_PAGES: &[&str] = &["Intake", "Staged", "Review", "Published", "Config"];

const CURIO_CONFIG_STRUCTURE_PAGES: &[&str] = &["Northstar", "CURIO Readme", "Settings"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ValueSource {
    ProcessEnv,
    EnvFile,
    Default,
    Missing,
}

impl ValueSource {
    fn label(self) -> &'static str {
        match self {
            ValueSource::ProcessEnv => "env",
            ValueSource::EnvFile => "file",
            ValueSource::Default => "default",
            ValueSource::Missing => "missing",
        }
    }
}

#[derive(Debug, Clone)]
struct ResolvedValue {
    value: String,
    source: ValueSource,
}

#[derive(Debug, Clone)]
struct OnboardState {
    resolved: BTreeMap<String, ResolvedValue>,
    env_file_values: BTreeMap<String, String>,
    example_file_values: BTreeMap<String, String>,
    env_path: PathBuf,
    example_path: PathBuf,
}

pub async fn run_onboard(dry_run: bool, force_install: bool) -> Result<()> {
    let paths = HarnessPaths::discover()?;
    let mut critical_issues = 0usize;
    let mut warning_issues = 0usize;

    println!("Running Curio onboarding...");
    println!("repo_root :: {}", paths.repo_root.display());
    println!("crate_root :: {}", paths.crate_root.display());

    let harness_checks = run_checks(&paths, None)?;
    for check in harness_checks {
        let is_warning = check.label.ends_with(":launcher") || check.label == "claude_settings";
        let ok = check.ok;
        let status = if ok {
            "OK"
        } else if is_warning {
            warning_issues += 1;
            "WARN"
        } else {
            critical_issues += 1;
            "FAIL"
        };
        println!("[{}] {} :: {}", status, check.label, check.detail);
    }

    let mut state = load_onboard_state(&paths)?;
    report_env_alignment(&state, dry_run, &mut critical_issues)?;
    ensure_northstar_markdown(&paths.repo_root, dry_run)?;

    let config = build_onboard_config(&state);
    report_runtime_defaults(&config);
    let token = state
        .resolved
        .get("CURIO_CONFLUENCE_TOKEN")
        .map(|resolved| resolved.value.as_str())
        .unwrap_or("");
    validate_confluence(
        &config,
        token,
        &mut critical_issues,
        &mut warning_issues,
        dry_run,
    )
    .await?;

    if !dry_run {
        state = load_onboard_state(&paths)?;
    }

    if !dry_run {
        write_env_file(&state)?;
    } else {
        println!("(Dry run) Would sync {}", state.env_path.display());
    }

    let should_install = if dry_run {
        false
    } else if force_install {
        true
    } else {
        prompt_install_shim()?
    };

    if should_install {
        install_curio_shim(&paths, dry_run)?;
    } else {
        println!("[OK] shim_install :: skipped");
    }

    println!(
        "Onboarding complete with {} critical issue(s) and {} warning(s).",
        critical_issues, warning_issues
    );

    if critical_issues > 0 {
        bail!(
            "Curio onboarding found {} critical issue(s).",
            critical_issues
        );
    }

    Ok(())
}

fn prompt_install_shim() -> Result<bool> {
    if !io::stdin().is_terminal() {
        println!("[OK] shim_install :: non-interactive session, defaulting to yes");
        return Ok(true);
    }

    print!("Install Curio shim to cargo bin so `curio` works from any terminal? [Y/n] ");
    io::stdout()
        .flush()
        .context("Failed to flush install prompt")?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .context("Failed to read install prompt response")?;

    let answer = input.trim().to_ascii_lowercase();
    Ok(answer.is_empty() || matches!(answer.as_str(), "y" | "yes"))
}

fn cargo_bin_dir() -> Result<PathBuf> {
    if let Ok(cargo_home) = env::var("CARGO_HOME") {
        return Ok(PathBuf::from(cargo_home).join("bin"));
    }

    let home = dirs::home_dir().context("Could not resolve home directory for cargo bin path")?;
    Ok(home.join(".cargo").join("bin"))
}

fn install_curio_shim(paths: &HarnessPaths, dry_run: bool) -> Result<()> {
    let cargo_bin = cargo_bin_dir()?;
    let shim_path = if cfg!(windows) {
        cargo_bin.join("curio.cmd")
    } else {
        cargo_bin.join("curio")
    };

    let manifest_path = paths.crate_root.join("Cargo.toml");
    let manifest_path = manifest_path.canonicalize().unwrap_or(manifest_path);
    let manifest_path = manifest_path.display().to_string();

    let desired_content = if cfg!(windows) {
        format!(
            "@echo off\r\nset \"CURIO_REPO_ROOT={}\"\r\ncargo run --manifest-path \"{}\" -- %*\r\n",
            paths.repo_root.display(),
            manifest_path
        )
    } else {
        format!(
            "#!/usr/bin/env sh\nexport CURIO_REPO_ROOT=\"{}\"\nexec cargo run --manifest-path \"{}\" -- \"$@\"\n",
            paths.repo_root.display(),
            manifest_path
        )
    };

    if dry_run {
        println!("[Dry run] Would install shim at {}", shim_path.display());
        return Ok(());
    }

    fs::create_dir_all(&cargo_bin).with_context(|| {
        format!(
            "Failed to create cargo bin directory {}",
            cargo_bin.display()
        )
    })?;

    let current_content = fs::read_to_string(&shim_path).unwrap_or_default();
    if current_content != desired_content {
        fs::write(&shim_path, desired_content)
            .with_context(|| format!("Failed to write shim {}", shim_path.display()))?;
        println!("[OK] shim_install :: updated {}", shim_path.display());
    } else {
        println!("[OK] shim_install :: {}", shim_path.display());
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&shim_path)
            .with_context(|| format!("Failed to read metadata for {}", shim_path.display()))?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&shim_path, perms)
            .with_context(|| format!("Failed to set executable bit on {}", shim_path.display()))?;
    }

    if !path_contains(&cargo_bin)? {
        println!(
            "[WARN] shim_install :: {} is not currently on PATH",
            cargo_bin.display()
        );
        println!(
            "  hint :: add {} to PATH or open a new shell if your environment refreshes PATH on startup",
            cargo_bin.display()
        );
    }

    Ok(())
}

fn path_contains(target: &Path) -> Result<bool> {
    let Some(path_var) = env::var_os("PATH") else {
        return Ok(false);
    };

    let normalized_target = normalize_path(target);
    for entry in env::split_paths(&path_var) {
        if normalize_path(&entry) == normalized_target {
            return Ok(true);
        }
    }

    Ok(false)
}

fn normalize_path(path: &Path) -> PathBuf {
    let text = path.to_string_lossy().replace('/', "\\");
    PathBuf::from(text.trim_end_matches('\\'))
}

async fn validate_structure_pages(
    client: &ConfluenceClient,
    config: &Config,
    warning_issues: &mut usize,
) -> Result<()> {
    let space_key = config.content_model.space_key.as_str();
    let root_page = client
        .get_page_by_title(space_key, None, CURIO_ROOT_TITLE)
        .await?;
    let Some(root_page) = root_page else {
        *warning_issues += 1;
        println!(
            "[WARN] structure_page :: {} root page is missing",
            CURIO_ROOT_TITLE
        );
        return Ok(());
    };
    let root_page_id = root_page["id"].as_str().unwrap_or_default();
    println!("[OK] structure_page :: {}", CURIO_ROOT_TITLE);

    for page_name in CURIO_ROOT_STRUCTURE_PAGES {
        let page = client
            .get_page_by_title(space_key, Some(root_page_id), page_name)
            .await?;
        if page.is_some() {
            println!("[OK] structure_page :: {}/{}", CURIO_ROOT_TITLE, page_name);
        } else {
            *warning_issues += 1;
            println!(
                "[WARN] structure_page :: {}/{} is missing",
                CURIO_ROOT_TITLE, page_name
            );
        }
    }

    let config_page = client
        .get_page_by_title(space_key, Some(root_page_id), "Config")
        .await?;
    let Some(config_page) = config_page else {
        *warning_issues += 1;
        println!("[WARN] structure_page :: Config branch is missing");
        return Ok(());
    };

    let config_page_id = config_page["id"].as_str().unwrap_or_default();
    for page_name in CURIO_CONFIG_STRUCTURE_PAGES {
        let page = client
            .get_page_by_title(space_key, Some(config_page_id), page_name)
            .await?;
        if page.is_some() {
            println!("[OK] structure_page :: Config/{}", page_name);
        } else {
            *warning_issues += 1;
            println!("[WARN] structure_page :: Config/{} is missing", page_name);
        }
    }

    let published_page = client
        .get_page_by_title(space_key, Some(root_page_id), "Published")
        .await?;
    let Some(published_page) = published_page else {
        *warning_issues += 1;
        println!("[WARN] structure_page :: Published branch is missing");
        return Ok(());
    };
    let published_page_id = published_page["id"].as_str().unwrap_or_default();
    let northstar_path = config.wiki.wiki_dir.join("_config").join("northstar.md");
    let expected_tree_pages = std::fs::read_to_string(&northstar_path)
        .ok()
        .map(|md| {
            parse_northstar_blueprint(&md)
                .into_iter()
                .map(|tree| tree.title)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    for page_name in expected_tree_pages {
        let page = client
            .get_page_by_title(space_key, Some(published_page_id), &page_name)
            .await?;
        if page.is_some() {
            println!("[OK] structure_page :: Published/{}", page_name);
        } else {
            *warning_issues += 1;
            println!(
                "[WARN] structure_page :: Published/{} is missing",
                page_name
            );
        }
    }

    Ok(())
}

fn load_onboard_state(paths: &HarnessPaths) -> Result<OnboardState> {
    let env_path = paths.repo_root.join(".env");
    let example_path = paths.repo_root.join(".env.example");
    let env_file_values = read_env_file(&env_path)?;
    let example_file_values = read_env_file(&example_path)?;

    let resolved = CURIO_ENV_KEYS
        .iter()
        .map(|key| {
            let (value, source) = resolve_env_value(key, &env_file_values);
            ((*key).to_string(), ResolvedValue { value, source })
        })
        .collect();

    Ok(OnboardState {
        resolved,
        env_file_values,
        example_file_values,
        env_path,
        example_path,
    })
}

fn resolve_env_value(
    key: &str,
    env_file_values: &BTreeMap<String, String>,
) -> (String, ValueSource) {
    if let Ok(value) = env::var(key) {
        if !value.trim().is_empty() {
            return (value, ValueSource::ProcessEnv);
        }
    }

    if let Some(value) = env_file_values.get(key) {
        if !value.trim().is_empty() {
            return (value.clone(), ValueSource::EnvFile);
        }
    }

    match key {
        "CURIO_SPACE_KEY" => ("CURIO".to_string(), ValueSource::Default),
        "CURIO_CONFLUENCE_PARENT_PAGE_ID" => (String::new(), ValueSource::Default),
        "CURIO_TEMP_DIR" => (String::new(), ValueSource::Default),
        "CURIO_AUDIT_DIR" => (
            "${REPO_ROOT}/wiki/_config".to_string(),
            ValueSource::Default,
        ),
        _ => (String::new(), ValueSource::Missing),
    }
}

fn read_env_file(path: &Path) -> Result<BTreeMap<String, String>> {
    if !path.is_file() {
        return Ok(BTreeMap::new());
    }

    let raw =
        fs::read_to_string(path).with_context(|| format!("Failed to read {}", path.display()))?;

    let mut values = BTreeMap::new();
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = trimmed.split_once('=') {
            values.insert(key.trim().to_string(), strip_quotes(value.trim()));
        }
    }

    Ok(values)
}

fn strip_quotes(value: &str) -> String {
    let bytes = value.as_bytes();
    if bytes.len() >= 2
        && ((bytes[0] == b'"' && bytes[bytes.len() - 1] == b'"')
            || (bytes[0] == b'\'' && bytes[bytes.len() - 1] == b'\''))
    {
        value[1..value.len() - 1].to_string()
    } else {
        value.to_string()
    }
}

fn report_env_alignment(
    state: &OnboardState,
    dry_run: bool,
    critical_issues: &mut usize,
) -> Result<()> {
    if state.env_path.is_file() {
        println!("[OK] env_file :: {}", state.env_path.display());
    } else {
        *critical_issues += 1;
        println!("[FAIL] env_file :: {} is missing", state.env_path.display());
        println!("  hint :: create the file or run `curio onboard` with `--dry-run` first");
    }

    if state.example_path.is_file() {
        println!("[OK] env_example :: {}", state.example_path.display());
    } else {
        *critical_issues += 1;
        println!(
            "[FAIL] env_example :: {} is missing",
            state.example_path.display()
        );
        println!("  hint :: restore the tracked example file from the repository");
    }

    let env_keys: BTreeSet<_> = state.env_file_values.keys().cloned().collect();
    let example_keys: BTreeSet<_> = state.example_file_values.keys().cloned().collect();

    if env_keys == example_keys {
        println!(
            "[OK] env_key_parity :: {} key(s) match between .env and .env.example",
            env_keys.len()
        );
    } else {
        *critical_issues += 1;
        let missing_from_env: Vec<_> = example_keys.difference(&env_keys).cloned().collect();
        let extra_in_env: Vec<_> = env_keys.difference(&example_keys).cloned().collect();
        println!("[FAIL] env_key_parity :: .env and .env.example are out of sync");
        if !missing_from_env.is_empty() {
            println!("  missing_from_.env :: {}", missing_from_env.join(", "));
        }
        if !extra_in_env.is_empty() {
            println!("  extra_in_.env :: {}", extra_in_env.join(", "));
        }
    }

    for key in CURIO_ENV_KEYS {
        if let Some(resolved) = state.resolved.get(*key) {
            let rendered = render_value(key, &resolved.value);
            let status = match resolved.source {
                ValueSource::ProcessEnv | ValueSource::EnvFile | ValueSource::Default => "OK",
                ValueSource::Missing => "WARN",
            };
            println!(
                "[{}] {} :: {} :: {}",
                status,
                key,
                resolved.source.label(),
                rendered
            );
        }
    }

    if dry_run {
        println!("(Dry run) Would synchronize {}", state.env_path.display());
    }

    Ok(())
}

fn render_value(key: &str, value: &str) -> String {
    if value.is_empty() {
        return "<empty>".to_string();
    }

    if key.contains("TOKEN") || key.contains("PASSWORD") || key.contains("SECRET") {
        let visible = value.chars().take(4).collect::<String>();
        return format!("{}*** ({} chars)", visible, value.chars().count());
    }

    value.to_string()
}

fn build_onboard_config(state: &OnboardState) -> Config {
    let get = |key: &str| {
        state
            .resolved
            .get(key)
            .map(|resolved| resolved.value.clone())
            .unwrap_or_default()
    };
    let temp_dir = {
        let raw = get("CURIO_TEMP_DIR");
        if raw.trim().is_empty() {
            Some(std::env::temp_dir().join("curio"))
        } else {
            Some(PathBuf::from(raw))
        }
    };

    Config {
        connection: ConnectionConfig {
            confluence_url: get("CURIO_CONFLUENCE_URL"),
            confluence_email: get("CURIO_CONFLUENCE_EMAIL"),
        },
        content_model: ContentModelConfig {
            space_key: get("CURIO_SPACE_KEY"),
            label_namespace: "curio".to_string(),
        },
        runtime: RuntimeConfig {
            temp_dir,
            log_level: None,
        },
        wiki: crate::config::WikiConfig {
            sync: crate::config::SyncConfig {
                enabled: true,
                confluence_parent_page_id: {
                    let raw = get("CURIO_CONFLUENCE_PARENT_PAGE_ID");
                    if raw.trim().is_empty() {
                        None
                    } else {
                        Some(raw)
                    }
                },
            },
            ..Default::default()
        },
        llm: Default::default(),
        heal: Default::default(),
        slack: Default::default(),
    }
}

fn report_runtime_defaults(config: &Config) {
    if let Some(temp_dir) = &config.runtime.temp_dir {
        println!("[OK] temp_dir :: {}", temp_dir.display());
    }
}

async fn validate_confluence(
    config: &Config,
    auth_token: &str,
    critical_issues: &mut usize,
    warning_issues: &mut usize,
    dry_run: bool,
) -> Result<()> {
    let missing = missing_core_fields(config, auth_token);
    if !missing.is_empty() {
        *critical_issues += missing.len();
        println!(
            "[FAIL] confluence_core :: missing required value(s): {}",
            missing.join(", ")
        );
        println!("  hint :: run `curio onboard` after setting the missing environment variables");
        return Ok(());
    }

    let client = ConfluenceClient::new(
        config.connection.confluence_url.clone(),
        config.connection.confluence_email.clone(),
        auth_token.to_string(),
        None,
    )?;

    match client.get_current_user().await {
        Ok(user) => {
            let display_name = user["displayName"].as_str().unwrap_or("unknown");
            println!("[OK] confluence_auth :: authenticated as {}", display_name);
        }
        Err(err) => {
            *critical_issues += 1;
            println!("[FAIL] confluence_auth :: {}", err);
            println!(
                "  hint :: confirm CURIO_CONFLUENCE_URL, CURIO_CONFLUENCE_EMAIL, and CURIO_CONFLUENCE_TOKEN"
            );
            return Ok(());
        }
    }

    if !dry_run {
        let tree = ensure_curio_confluence_tree(
            config,
            &client,
            config.wiki.sync.confluence_parent_page_id.clone(),
            true,
        )
        .await?;
        println!(
            "[OK] confluence_root :: {} ({})",
            CURIO_ROOT_TITLE, tree.root_id
        );
    }

    validate_structure_pages(&client, config, warning_issues).await?;

    Ok(())
}

fn missing_core_fields(config: &Config, auth_token: &str) -> Vec<&'static str> {
    let mut missing = Vec::new();
    if config.connection.confluence_url.trim().is_empty() {
        missing.push("CURIO_CONFLUENCE_URL");
    }
    if config.connection.confluence_email.trim().is_empty() {
        missing.push("CURIO_CONFLUENCE_EMAIL");
    }
    if auth_token.trim().is_empty() {
        missing.push("CURIO_CONFLUENCE_TOKEN");
    }
    if config.content_model.space_key.trim().is_empty() {
        missing.push("CURIO_SPACE_KEY");
    }
    missing
}

fn write_env_file(state: &OnboardState) -> Result<()> {
    let mut lines = Vec::new();
    lines.push("# Curio harness config".to_string());
    for key in CURIO_ENV_KEYS {
        let value = if *key == "CURIO_TEMP_DIR" {
            String::new()
        } else {
            state
                .resolved
                .get(*key)
                .map(|resolved| resolved.value.clone())
                .unwrap_or_default()
        };
        lines.push(format!("{}={}", key, value));
    }

    let new_content = lines.join("\n") + "\n";
    let current_content = fs::read_to_string(&state.env_path).unwrap_or_default();
    if current_content == new_content {
        println!("[OK] env_sync :: {}", state.env_path.display());
        return Ok(());
    }

    fs::write(&state.env_path, new_content)
        .with_context(|| format!("Failed to write {}", state.env_path.display()))?;
    println!("[OK] env_sync :: updated {}", state.env_path.display());
    Ok(())
}
