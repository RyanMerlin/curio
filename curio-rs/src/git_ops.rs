use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

/// Run `git add <path>` in the given repo root.
pub fn git_add(repo_root: &Path, path: &Path) -> Result<()> {
    let status = Command::new("git")
        .args(["add", "--"])
        .arg(path)
        .current_dir(repo_root)
        .status()
        .context("Failed to spawn git add")?;

    if !status.success() {
        anyhow::bail!("git add failed (exit {})", status);
    }
    Ok(())
}

/// Run `git mv <from> <to>` in the given repo root.
///
/// Creates the target's parent directory first if it does not exist.
pub fn git_mv(repo_root: &Path, from: &Path, to: &Path) -> Result<()> {
    // Ensure target parent exists (git mv won't create missing directories)
    if let Some(parent) = to.parent() {
        let abs_parent = if parent.is_absolute() {
            parent.to_path_buf()
        } else {
            repo_root.join(parent)
        };
        std::fs::create_dir_all(&abs_parent)
            .with_context(|| format!("Failed to create directory {}", abs_parent.display()))?;
    }

    let status = Command::new("git")
        .args(["mv", "--"])
        .arg(from)
        .arg(to)
        .current_dir(repo_root)
        .status()
        .context("Failed to spawn git mv")?;

    if !status.success() {
        anyhow::bail!("git mv failed (exit {})", status);
    }
    Ok(())
}

/// Run `git commit -m <message>` in the given repo root.
pub fn git_commit(repo_root: &Path, message: &str) -> Result<()> {
    let status = Command::new("git")
        .args(["commit", "-m", message])
        .current_dir(repo_root)
        .status()
        .context("Failed to spawn git commit")?;

    if !status.success() {
        anyhow::bail!("git commit failed (exit {})", status);
    }
    Ok(())
}

/// Returns `true` if there are staged changes ready to commit.
pub fn git_has_staged(repo_root: &Path) -> bool {
    Command::new("git")
        .args(["diff", "--cached", "--quiet"])
        .current_dir(repo_root)
        .status()
        .map(|s| !s.success())
        .unwrap_or(false)
}

/// Stage all changes under `path` and commit with `message` if `auto_commit` is true.
/// Does nothing if there is nothing to commit.
pub fn stage_and_commit(repo_root: &Path, paths: &[&Path], message: &str, auto_commit: bool) -> Result<()> {
    for &p in paths {
        git_add(repo_root, p)?;
    }
    if auto_commit && git_has_staged(repo_root) {
        git_commit(repo_root, message)?;
    }
    Ok(())
}
