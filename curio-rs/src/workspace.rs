/// Workspace management — locate and manage named KB stores.
///
/// Resolution order for `--workspace` / `--kb-dir`:
///   1. `--kb-dir <path>` — explicit path, used directly
///   2. `--workspace <name>` — looked up in `curio.workspaces.toml`
///   3. Neither given — error; suggest `curio init-kb` or pass `--kb-dir`
///
/// `curio.workspaces.toml` is found by:
///   1. `CURIO_HARNESS_DIR` env var (explicit harness root)
///   2. Walk up from CWD until a file named `curio.workspaces.toml` is found
///   3. `~/.curio/workspaces.toml` (global fallback)
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub name: String,
    pub path: String, // stored as string to preserve ~/ notation
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl Workspace {
    /// Resolve `~` and return an absolute PathBuf.
    pub fn resolved_path(&self) -> PathBuf {
        expand_tilde(&self.path)
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct WorkspaceFile {
    #[serde(default)]
    pub workspaces: Vec<Workspace>,
}

/// Find the path of `curio.workspaces.toml`, or `None` if not found anywhere.
pub fn find_workspace_file() -> Option<PathBuf> {
    // 1. CURIO_HARNESS_DIR env var
    if let Ok(harness) = std::env::var("CURIO_HARNESS_DIR") {
        let p = PathBuf::from(&harness).join("curio.workspaces.toml");
        if p.exists() {
            return Some(p);
        }
    }

    // 2. Walk up from CWD
    if let Ok(cwd) = std::env::current_dir() {
        let mut dir = cwd.as_path();
        loop {
            let candidate = dir.join("curio.workspaces.toml");
            if candidate.exists() {
                return Some(candidate);
            }
            match dir.parent() {
                Some(parent) => dir = parent,
                None => break,
            }
        }
    }

    // 3. ~/.curio/workspaces.toml
    if let Some(home) = dirs::home_dir() {
        let p = home.join(".curio").join("workspaces.toml");
        if p.exists() {
            return Some(p);
        }
    }

    None
}

/// Default path where the workspace file is created when none exists.
/// Uses CURIO_HARNESS_DIR if set, otherwise CWD.
pub fn default_workspace_file_path() -> PathBuf {
    if let Ok(harness) = std::env::var("CURIO_HARNESS_DIR") {
        return PathBuf::from(harness).join("curio.workspaces.toml");
    }
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("curio.workspaces.toml")
}

pub fn load_workspaces() -> Result<Vec<Workspace>> {
    match find_workspace_file() {
        None => Ok(vec![]),
        Some(path) => {
            let raw = std::fs::read_to_string(&path)
                .with_context(|| format!("Failed to read {}", path.display()))?;
            let file: WorkspaceFile = toml::from_str(&raw)
                .with_context(|| format!("Failed to parse {}", path.display()))?;
            Ok(file.workspaces)
        }
    }
}

pub fn save_workspaces(workspaces: &[Workspace]) -> Result<()> {
    let path = find_workspace_file().unwrap_or_else(default_workspace_file_path);
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let file = WorkspaceFile {
        workspaces: workspaces.to_vec(),
    };
    let content = toml::to_string_pretty(&file)
        .context("Failed to serialize workspaces")?;
    std::fs::write(&path, content)
        .with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(())
}

/// Add or update a workspace entry. Returns the path of the workspace file written.
pub fn upsert_workspace(name: &str, path: &Path, description: Option<&str>) -> Result<PathBuf> {
    let mut workspaces = load_workspaces().unwrap_or_default();
    let path_str = path.to_string_lossy().to_string();
    if let Some(existing) = workspaces.iter_mut().find(|w| w.name == name) {
        existing.path = path_str;
        existing.description = description.map(|s| s.to_string());
    } else {
        workspaces.push(Workspace {
            name: name.to_string(),
            path: path_str,
            description: description.map(|s| s.to_string()),
        });
    }
    save_workspaces(&workspaces)?;
    Ok(find_workspace_file().unwrap_or_else(default_workspace_file_path))
}

/// Remove a workspace entry by name. Returns true if it was found and removed.
pub fn remove_workspace(name: &str) -> Result<bool> {
    let mut workspaces = load_workspaces().unwrap_or_default();
    let before = workspaces.len();
    workspaces.retain(|w| w.name != name);
    if workspaces.len() < before {
        save_workspaces(&workspaces)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Resolve a KB directory from CLI flags.
///
/// Returns `Err` with a helpful message if neither `--kb-dir` nor `--workspace` was given.
pub fn resolve_kb_dir(
    kb_dir: Option<&PathBuf>,
    workspace_name: Option<&str>,
) -> Result<PathBuf> {
    // Explicit path wins
    if let Some(dir) = kb_dir {
        let abs = if dir.is_absolute() {
            dir.clone()
        } else {
            std::env::current_dir()?.join(dir)
        };
        return Ok(abs);
    }

    // Named workspace
    if let Some(name) = workspace_name {
        let workspaces = load_workspaces()?;
        let ws = workspaces
            .iter()
            .find(|w| w.name == name)
            .with_context(|| {
                format!(
                    "Workspace '{}' not found. Run `curio workspace list` to see available workspaces.",
                    name
                )
            })?;
        return Ok(ws.resolved_path());
    }

    // Neither given — error with actionable message
    anyhow::bail!(
        "No KB store specified.\n\n\
        Options:\n  \
          --kb-dir <path>      Use a KB at an explicit path\n  \
          --workspace <name>   Use a named workspace from curio.workspaces.toml\n\n\
        To create a new KB:\n  \
          curio init-kb --path ~/curio-kb --name default\n\n\
        To register an existing KB:\n  \
          curio workspace add --name <name> --path <path>"
    )
}

/// Expand a leading `~/` to the user's home directory.
pub fn expand_tilde(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest);
        }
    }
    PathBuf::from(path)
}
