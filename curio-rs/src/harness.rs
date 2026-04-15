use crate::cli::AgentProvider;
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct HarnessPaths {
    pub repo_root: PathBuf,
    pub crate_root: PathBuf,
    pub docs_dir: PathBuf,
    pub skills_dir: PathBuf,
    pub agents_skills_dir: PathBuf,
    pub plugins_dir: PathBuf,
    pub providers_dir: PathBuf,
    pub marketplace_path: PathBuf,
    pub codex_plugin_manifest_path: PathBuf,
    pub codex_entrypoint: PathBuf,
    pub claude_entrypoint: PathBuf,
    pub gemini_entrypoint: PathBuf,
    pub claude_settings_path: PathBuf,
}

impl HarnessPaths {
    pub fn discover() -> Result<Self> {
        if let Some(repo_root) = repo_root_override() {
            return Self::discover_from(&repo_root);
        }

        let cwd = std::env::current_dir().context("Failed to resolve current working directory")?;
        Self::discover_from(&cwd)
    }

    pub fn discover_from(start: &Path) -> Result<Self> {
        let repo_root = find_repo_root(start)?;
        let crate_root = repo_root.join("curio-rs");

        Ok(Self {
            docs_dir: repo_root.join("docs"),
            skills_dir: repo_root.join("skills"),
            agents_skills_dir: repo_root.join(".agents").join("skills"),
            plugins_dir: repo_root.join("plugins"),
            providers_dir: repo_root.join("providers"),
            marketplace_path: repo_root
                .join(".agents")
                .join("plugins")
                .join("marketplace.json"),
            codex_plugin_manifest_path: repo_root.join(".codex-plugin").join("plugin.json"),
            codex_entrypoint: repo_root.join("AGENTS.md"),
            claude_entrypoint: repo_root.join("CLAUDE.md"),
            gemini_entrypoint: repo_root.join("GEMINI.md"),
            claude_settings_path: repo_root.join(".claude").join("settings.local.json"),
            repo_root,
            crate_root,
        })
    }

    pub fn entrypoint_for(&self, provider: AgentProvider) -> &Path {
        match provider {
            AgentProvider::Codex => &self.codex_entrypoint,
            AgentProvider::Claude => &self.claude_entrypoint,
            AgentProvider::Gemini => &self.gemini_entrypoint,
        }
    }
}

fn repo_root_override() -> Option<PathBuf> {
    let value = std::env::var_os("CURIO_REPO_ROOT")?;
    if value.is_empty() {
        return None;
    }

    Some(PathBuf::from(value))
}

fn find_repo_root(start: &Path) -> Result<PathBuf> {
    let mut current = if start.is_file() {
        start
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| start.to_path_buf())
    } else {
        start.to_path_buf()
    };

    loop {
        if current.join("curio-rs").is_dir() {
            return Ok(current);
        }

        let looks_like_crate_root = current.file_name().and_then(|name| name.to_str())
            == Some("curio-rs")
            && current.join("Cargo.toml").is_file();
        if looks_like_crate_root {
            return current
                .parent()
                .map(Path::to_path_buf)
                .context("Could not resolve the Curio repo root from curio-rs");
        }

        if !current.pop() {
            break;
        }
    }

    bail!(
        "Could not find the Curio repo root. Run this command from the Curio repository root or set CURIO_REPO_ROOT."
    )
}

#[derive(Debug, Clone, Serialize)]
pub struct SkillInfo {
    pub name: String,
    pub path: PathBuf,
}

pub fn discover_skills(paths: &HarnessPaths) -> Result<Vec<SkillInfo>> {
    let mut skills = discover_skill_dirs(&paths.skills_dir)?;
    let marketplace = load_marketplace(paths)?;

    for plugin in marketplace
        .plugins
        .into_iter()
        .filter(|plugin| plugin.enabled)
    {
        let plugin_skills_dir = paths.repo_root.join(&plugin.path).join("skills");
        let plugin_skills = discover_skill_dirs(&plugin_skills_dir)?;
        skills.extend(plugin_skills);
    }

    skills.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(skills)
}

fn discover_skill_dirs(root: &Path) -> Result<Vec<SkillInfo>> {
    if !root.is_dir() {
        return Ok(Vec::new());
    }

    let mut skills = Vec::new();
    for entry in fs::read_dir(root)
        .with_context(|| format!("Failed to read skills directory: {}", root.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() && path.join("SKILL.md").is_file() {
            skills.push(SkillInfo {
                name: entry.file_name().to_string_lossy().to_string(),
                path,
            });
        }
    }

    skills.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(skills)
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MarketplacePlugin {
    pub name: String,
    pub path: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_enabled() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize)]
pub struct MarketplaceCatalog {
    #[serde(default)]
    pub plugins: Vec<MarketplacePlugin>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ProviderProfile {
    #[serde(default)]
    pub command: Option<String>,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    #[serde(default)]
    pub bootstrap_summary: String,
}

pub fn load_marketplace(paths: &HarnessPaths) -> Result<MarketplaceCatalog> {
    let raw = fs::read_to_string(&paths.marketplace_path).with_context(|| {
        format!(
            "Failed to read marketplace catalog: {}",
            paths.marketplace_path.display()
        )
    })?;
    let mut catalog: MarketplaceCatalog = serde_json::from_str(&raw).with_context(|| {
        format!(
            "Failed to parse marketplace catalog: {}",
            paths.marketplace_path.display()
        )
    })?;
    catalog
        .plugins
        .sort_by(|left, right| left.name.cmp(&right.name));
    validate_marketplace(paths, &catalog)?;
    Ok(catalog)
}

fn validate_marketplace(paths: &HarnessPaths, catalog: &MarketplaceCatalog) -> Result<()> {
    for plugin in catalog.plugins.iter().filter(|plugin| plugin.enabled) {
        let plugin_path = paths.repo_root.join(&plugin.path);
        if !plugin_path.is_dir() {
            bail!(
                "Enabled plugin '{}' points to a missing directory: {}",
                plugin.name,
                plugin_path.display()
            );
        }
    }

    Ok(())
}

pub fn load_provider_profile(
    paths: &HarnessPaths,
    provider: AgentProvider,
) -> Result<(ProviderProfile, PathBuf)> {
    let profile_path = paths
        .providers_dir
        .join(format!("{}.json", provider.as_str()));
    let raw = fs::read_to_string(&profile_path).with_context(|| {
        format!(
            "Failed to read provider profile for {}: {}",
            provider.as_str(),
            profile_path.display()
        )
    })?;
    let profile: ProviderProfile = serde_json::from_str(&raw).with_context(|| {
        format!(
            "Failed to parse provider profile for {}: {}",
            provider.as_str(),
            profile_path.display()
        )
    })?;
    Ok((profile, profile_path))
}

#[derive(Debug, Clone)]
pub struct ProviderLaunchPlan {
    pub provider: AgentProvider,
    pub command: String,
    pub args: Vec<String>,
    pub env: BTreeMap<String, String>,
    pub bootstrap_summary: String,
    pub profile_path: PathBuf,
    pub entrypoint_path: PathBuf,
    pub repo_root: PathBuf,
}

impl ProviderLaunchPlan {
    pub fn command_line(&self) -> String {
        let mut parts = vec![self.command.clone()];
        parts.extend(self.args.iter().cloned());
        parts.join(" ")
    }
}

pub fn build_launch_plan(
    provider: AgentProvider,
    paths: &HarnessPaths,
) -> Result<ProviderLaunchPlan> {
    let (profile, profile_path) = load_provider_profile(paths, provider)?;
    let command = resolve_provider_command(provider, profile.command.as_deref())?;
    let mut args = profile.args;
    args.extend(resolve_provider_args(provider));
    let entrypoint_path = paths.entrypoint_for(provider).to_path_buf();

    let mut env = BTreeMap::new();
    env.insert(
        "CURIO_HARNESS_DIR".to_string(),
        paths.repo_root.display().to_string(),
    );
    env.insert("CURIO_PROVIDER".to_string(), provider.as_str().to_string());
    env.insert(
        "CURIO_REPO_ROOT".to_string(),
        paths.repo_root.display().to_string(),
    );
    env.insert(
        "CURIO_CRATE_ROOT".to_string(),
        paths.crate_root.display().to_string(),
    );
    env.insert(
        "CURIO_DOCS_DIR".to_string(),
        paths.docs_dir.display().to_string(),
    );
    env.insert(
        "CURIO_SKILLS_DIR".to_string(),
        paths.skills_dir.display().to_string(),
    );
    env.insert(
        "CURIO_AGENTS_SKILLS_DIR".to_string(),
        paths.agents_skills_dir.display().to_string(),
    );
    env.insert(
        "CURIO_PLUGINS_DIR".to_string(),
        paths.plugins_dir.display().to_string(),
    );
    env.insert(
        "CURIO_MARKETPLACE_PATH".to_string(),
        paths.marketplace_path.display().to_string(),
    );
    env.insert(
        "CURIO_ENTRYPOINT".to_string(),
        entrypoint_path.display().to_string(),
    );
    env.insert(
        "CURIO_CODEX_PLUGIN_MANIFEST".to_string(),
        paths.codex_plugin_manifest_path.display().to_string(),
    );
    env.insert(
        "CURIO_PROVIDER_PROFILE".to_string(),
        profile_path.display().to_string(),
    );
    env.insert(
        "CURIO_BOOTSTRAP_SUMMARY".to_string(),
        profile.bootstrap_summary.clone(),
    );
    // Expose wiki directory so launched agents can discover it without parsing .curio.yaml
    if let Ok(wiki_dir) = std::env::var("CURIO_WIKI_DIR") {
        if !wiki_dir.is_empty() {
            env.insert("CURIO_WIKI_DIR".to_string(), wiki_dir);
        }
    } else {
        // Default: wiki/ relative to repo root
        env.insert(
            "CURIO_WIKI_DIR".to_string(),
            paths.repo_root.join("wiki").display().to_string(),
        );
    }
    for (key, value) in profile.env {
        env.insert(key, value);
    }

    Ok(ProviderLaunchPlan {
        provider,
        command,
        args,
        env,
        bootstrap_summary: profile.bootstrap_summary,
        profile_path,
        entrypoint_path,
        repo_root: paths.repo_root.clone(),
    })
}

fn resolve_provider_command(
    provider: AgentProvider,
    profile_command: Option<&str>,
) -> Result<String> {
    let env_key = match provider {
        AgentProvider::Codex => "CURIO_CODEX_CMD",
        AgentProvider::Claude => "CURIO_CLAUDE_CMD",
        AgentProvider::Gemini => "CURIO_GEMINI_CMD",
    };

    if let Ok(cmd) = std::env::var(env_key) {
        let trimmed = cmd.trim();
        if !trimmed.is_empty() {
            return Ok(trimmed.to_string());
        }
    }

    if let Some(command) = profile_command
        .map(str::trim)
        .filter(|command| !command.is_empty())
    {
        return Ok(command.to_string());
    }

    let candidates: &[&str] = match provider {
        AgentProvider::Codex => &["codex"],
        AgentProvider::Claude => &["claude"],
        AgentProvider::Gemini => &["gemini", "adk"],
    };

    for candidate in candidates {
        if command_exists(candidate) {
            return Ok((*candidate).to_string());
        }
    }

    let setup_hint = match provider {
        AgentProvider::Codex => "Install the Codex CLI or set CURIO_CODEX_CMD.",
        AgentProvider::Claude => "Install the Claude CLI or set CURIO_CLAUDE_CMD.",
        AgentProvider::Gemini => {
            "Install a Gemini-compatible launcher or set CURIO_GEMINI_CMD. Curio's Gemini harness expects a launcher that honors the CURIO_* environment contract and GEMINI.md entrypoint."
        }
    };
    bail!(
        "No launcher detected for {}. {}",
        provider.as_str(),
        setup_hint
    )
}

fn resolve_provider_args(provider: AgentProvider) -> Vec<String> {
    let env_key = match provider {
        AgentProvider::Codex => "CURIO_CODEX_ARGS",
        AgentProvider::Claude => "CURIO_CLAUDE_ARGS",
        AgentProvider::Gemini => "CURIO_GEMINI_ARGS",
    };

    std::env::var(env_key)
        .ok()
        .map(|value| {
            value
                .split_whitespace()
                .map(|segment| segment.to_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn command_exists(command: &str) -> bool {
    #[cfg(target_os = "windows")]
    {
        Command::new("where")
            .arg(command)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    #[cfg(not(target_os = "windows"))]
    {
        Command::new("sh")
            .arg("-lc")
            .arg(format!("command -v {}", command))
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CheckResult {
    pub label: String,
    pub ok: bool,
    pub detail: String,
}

pub fn run_checks(
    paths: &HarnessPaths,
    provider: Option<AgentProvider>,
) -> Result<Vec<CheckResult>> {
    let mut results = Vec::new();

    results.push(path_check("repo_root", &paths.repo_root, true));
    results.push(path_check("crate_root", &paths.crate_root, true));
    results.push(path_check("docs_dir", &paths.docs_dir, true));
    results.push(path_check("skills_dir", &paths.skills_dir, true));
    results.push(path_check(
        "agents_skills_dir",
        &paths.agents_skills_dir,
        true,
    ));
    results.push(path_check("plugins_dir", &paths.plugins_dir, true));
    results.push(path_check("providers_dir", &paths.providers_dir, true));
    results.push(path_check("marketplace", &paths.marketplace_path, false));
    results.push(path_check(
        "codex_plugin_manifest",
        &paths.codex_plugin_manifest_path,
        false,
    ));
    results.push(path_check(
        "codex_entrypoint",
        &paths.codex_entrypoint,
        false,
    ));
    results.push(path_check(
        "claude_entrypoint",
        &paths.claude_entrypoint,
        false,
    ));
    results.push(path_check(
        "gemini_entrypoint",
        &paths.gemini_entrypoint,
        false,
    ));
    results.push(path_check(
        "claude_settings",
        &paths.claude_settings_path,
        false,
    ));

    match load_marketplace(paths) {
        Ok(catalog) => results.push(CheckResult {
            label: "marketplace_parse".to_string(),
            ok: true,
            detail: format!("Loaded {} plugin(s)", catalog.plugins.len()),
        }),
        Err(err) => results.push(CheckResult {
            label: "marketplace_parse".to_string(),
            ok: false,
            detail: err.to_string(),
        }),
    }
    results.extend(skill_mirror_checks(paths));
    results.extend(plugin_catalog_checks(paths));

    if let Some(provider) = provider {
        results.extend(provider_checks(paths, provider));
    } else {
        for provider in [
            AgentProvider::Codex,
            AgentProvider::Claude,
            AgentProvider::Gemini,
        ] {
            results.extend(provider_checks(paths, provider));
        }
    }

    Ok(results)
}

fn provider_checks(paths: &HarnessPaths, provider: AgentProvider) -> Vec<CheckResult> {
    let mut results = Vec::new();
    let entrypoint = paths.entrypoint_for(provider);
    let profile_path = paths
        .providers_dir
        .join(format!("{}.json", provider.as_str()));

    results.push(CheckResult {
        label: format!("provider:{}:entrypoint", provider.as_str()),
        ok: entrypoint.is_file(),
        detail: entrypoint.display().to_string(),
    });
    results.push(CheckResult {
        label: format!("provider:{}:profile", provider.as_str()),
        ok: profile_path.is_file(),
        detail: profile_path.display().to_string(),
    });

    match build_launch_plan(provider, paths) {
        Ok(plan) => results.push(CheckResult {
            label: format!("provider:{}:launcher", provider.as_str()),
            ok: true,
            detail: plan.command_line(),
        }),
        Err(err) => results.push(CheckResult {
            label: format!("provider:{}:launcher", provider.as_str()),
            ok: false,
            detail: err.to_string(),
        }),
    }

    results
}

fn skill_mirror_checks(paths: &HarnessPaths) -> Vec<CheckResult> {
    let mut results = Vec::new();
    let authored_skills = match discover_skill_dirs(&paths.skills_dir) {
        Ok(skills) => skills,
        Err(err) => {
            results.push(CheckResult {
                label: "skills:authored".to_string(),
                ok: false,
                detail: err.to_string(),
            });
            return results;
        }
    };

    for skill in authored_skills {
        let mirrored = paths.agents_skills_dir.join(&skill.name).join("SKILL.md");
        let authored_path = skill.path.join("SKILL.md");
        match (
            fs::read_to_string(&authored_path),
            fs::read_to_string(&mirrored),
        ) {
            (Ok(authored), Ok(mirror)) => results.push(CheckResult {
                label: format!("skill-mirror:{}", skill.name),
                ok: normalize_text(&authored) == normalize_text(&mirror),
                detail: mirrored.display().to_string(),
            }),
            (Ok(_), Err(err)) => results.push(CheckResult {
                label: format!("skill-mirror:{}", skill.name),
                ok: false,
                detail: format!("Missing compatibility mirror {} ({})", mirrored.display(), err),
            }),
            (Err(err), _) => results.push(CheckResult {
                label: format!("skill-mirror:{}", skill.name),
                ok: false,
                detail: format!("Failed to read authored skill {} ({})", authored_path.display(), err),
            }),
        }
    }

    results
}

fn normalize_text(text: &str) -> String {
    text.replace("\r\n", "\n").trim().to_string()
}

fn plugin_catalog_checks(paths: &HarnessPaths) -> Vec<CheckResult> {
    let mut results = Vec::new();
    let catalog = match load_marketplace(paths) {
        Ok(catalog) => catalog,
        Err(err) => {
            results.push(CheckResult {
                label: "plugins:catalog".to_string(),
                ok: false,
                detail: err.to_string(),
            });
            return results;
        }
    };

    for plugin in catalog.plugins.into_iter().filter(|plugin| plugin.enabled) {
        let plugin_path = paths.repo_root.join(&plugin.path);
        let skills_path = plugin_path.join("skills");
        results.push(CheckResult {
            label: format!("plugin:{}:path", plugin.name),
            ok: plugin_path.is_dir(),
            detail: plugin_path.display().to_string(),
        });

        let detail = if skills_path.is_dir() {
            match discover_skill_dirs(&skills_path) {
                Ok(skills) => format!("{} skill(s) in {}", skills.len(), skills_path.display()),
                Err(err) => err.to_string(),
            }
        } else {
            format!("No plugin-local skills under {}", skills_path.display())
        };
        let ok = if skills_path.is_dir() {
            discover_skill_dirs(&skills_path).is_ok()
        } else {
            true
        };
        results.push(CheckResult {
            label: format!("plugin:{}:skills", plugin.name),
            ok,
            detail,
        });
    }

    results
}

fn path_check(label: &str, path: &Path, expect_dir: bool) -> CheckResult {
    let ok = if expect_dir {
        path.is_dir()
    } else {
        path.is_file()
    };
    CheckResult {
        label: label.to_string(),
        ok,
        detail: path.display().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discovers_repo_root_from_crate_root() {
        let root = HarnessPaths::discover_from(Path::new("..")).unwrap();
        assert!(root.crate_root.ends_with("curio-rs"));
        assert_eq!(root.repo_root.join("curio-rs"), root.crate_root);
    }

    #[test]
    fn discovers_skills_from_repo_root() {
        let paths = HarnessPaths::discover_from(Path::new("..")).unwrap();
        let skills = discover_skills(&paths).unwrap();
        assert!(
            skills
                .iter()
                .any(|skill| skill.name == "curio-workspace-bootstrap")
        );
    }

    #[test]
    fn loads_marketplace_catalog() {
        let paths = HarnessPaths::discover_from(Path::new("..")).unwrap();
        let catalog = load_marketplace(&paths).unwrap();
        assert!(
            catalog
                .plugins
                .iter()
                .any(|plugin| plugin.name == "curio-core-harness")
        );
    }

    #[test]
    fn loads_provider_profile() {
        let paths = HarnessPaths::discover_from(Path::new("..")).unwrap();
        let (profile, path) = load_provider_profile(&paths, AgentProvider::Gemini).unwrap();
        assert!(path.ends_with("gemini.json"));
        assert!(!profile.bootstrap_summary.is_empty());
    }

    #[test]
    fn authored_skills_match_compatibility_mirrors() {
        let paths = HarnessPaths::discover_from(Path::new("..")).unwrap();
        let checks = skill_mirror_checks(&paths);
        assert!(!checks.is_empty());
        assert!(checks.iter().all(|check| check.ok), "{checks:#?}");
    }

    #[test]
    fn enabled_plugins_have_valid_paths() {
        let paths = HarnessPaths::discover_from(Path::new("..")).unwrap();
        let checks = plugin_catalog_checks(&paths);
        assert!(!checks.is_empty());
        assert!(checks.iter().all(|check| check.ok), "{checks:#?}");
    }
}
