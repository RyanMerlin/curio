use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;
use std::process::Stdio;

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

    // Stage source if untracked so git mv can operate on it
    let _ = Command::new("git")
        .args(["add", "--"])
        .arg(from)
        .current_dir(repo_root)
        .status();

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
    // Keep command JSON envelopes clean; the exit status still reports commit failures.
    let status = Command::new("git")
        .args(["commit", "-m", message])
        .current_dir(repo_root)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .context("Failed to spawn git commit")?;

    if !status.success() {
        anyhow::bail!("git commit failed (exit {})", status);
    }
    Ok(())
}

/// Run `git -c user.name=<name> -c user.email=<email> commit -m <message>` in the given repo root.
pub fn git_commit_with_identity(
    repo_root: &Path,
    message: &str,
    user_name: &str,
    user_email: &str,
) -> Result<()> {
    let status = Command::new("git")
        .arg("-c")
        .arg(format!("user.name={}", user_name))
        .arg("-c")
        .arg(format!("user.email={}", user_email))
        .args(["commit", "-m"])
        .arg(message)
        .current_dir(repo_root)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .context("Failed to spawn git commit with identity")?;

    if !status.success() {
        anyhow::bail!("git commit with identity failed (exit {})", status);
    }
    Ok(())
}

/// Run `git clone --mirror <repo_url> <mirror_dir>`.
pub fn git_clone_mirror(
    repo_url: &str,
    mirror_dir: &Path,
    extra_env: &[(String, String)],
) -> Result<()> {
    if let Some(parent) = mirror_dir.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create mirror parent {}", parent.display()))?;
    }

    let status = Command::new("git")
        .args(["clone", "--mirror", repo_url])
        .arg(mirror_dir)
        .envs(extra_env.iter().map(|(k, v)| (k.as_str(), v.as_str())))
        .status()
        .context("Failed to spawn git clone --mirror")?;

    if !status.success() {
        anyhow::bail!("git clone --mirror failed (exit {})", status);
    }
    Ok(())
}

/// Run `git fetch --prune` in a mirror repository.
pub fn git_fetch_prune(repo_root: &Path, extra_env: &[(String, String)]) -> Result<()> {
    let status = Command::new("git")
        .args(["fetch", "--prune"])
        .current_dir(repo_root)
        .envs(extra_env.iter().map(|(k, v)| (k.as_str(), v.as_str())))
        .status()
        .context("Failed to spawn git fetch --prune")?;

    if !status.success() {
        anyhow::bail!("git fetch --prune failed (exit {})", status);
    }
    Ok(())
}

/// Set the URL of a named remote (e.g. to strip embedded credentials after a clone).
pub fn git_set_remote_url(repo_root: &Path, remote: &str, url: &str) -> Result<()> {
    let status = Command::new("git")
        .args(["remote", "set-url", remote, url])
        .current_dir(repo_root)
        .status()
        .context("Failed to spawn git remote set-url")?;
    if !status.success() {
        anyhow::bail!("git remote set-url failed (exit {})", status);
    }
    Ok(())
}

/// Run `git worktree add --force <worktree_dir> <ref>` against a mirror repository.
pub fn git_worktree_add(mirror_dir: &Path, worktree_dir: &Path, checkout_ref: &str) -> Result<()> {
    if let Some(parent) = worktree_dir.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create worktree parent {}", parent.display()))?;
    }

    let status = Command::new("git")
        .args(["--git-dir"])
        .arg(mirror_dir)
        .args(["worktree", "add", "--force"])
        .arg(worktree_dir)
        .arg(checkout_ref)
        .status()
        .context("Failed to spawn git worktree add")?;

    if !status.success() {
        anyhow::bail!("git worktree add failed (exit {})", status);
    }
    Ok(())
}

/// Run `git worktree remove --force <worktree_dir>`.
pub fn git_worktree_remove(worktree_dir: &Path) -> Result<()> {
    let status = Command::new("git")
        .args(["worktree", "remove", "--force"])
        .arg(worktree_dir)
        .status()
        .context("Failed to spawn git worktree remove")?;

    if !status.success() {
        anyhow::bail!("git worktree remove failed (exit {})", status);
    }
    Ok(())
}

/// Run `git status --porcelain` and return the raw output.
pub fn git_status_porcelain(repo_root: &Path) -> Result<String> {
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(repo_root)
        .output()
        .context("Failed to spawn git status --porcelain")?;

    if !output.status.success() {
        anyhow::bail!("git status --porcelain failed (exit {})", output.status);
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Run `git push origin <refspec>` in the given repo root.
pub fn git_push(repo_root: &Path, refspec: &str) -> Result<()> {
    let status = Command::new("git")
        .args(["push", "origin", refspec])
        .current_dir(repo_root)
        .status()
        .context("Failed to spawn git push")?;

    if !status.success() {
        anyhow::bail!("git push failed (exit {})", status);
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
pub fn stage_and_commit(
    repo_root: &Path,
    paths: &[&Path],
    message: &str,
    auto_commit: bool,
) -> Result<()> {
    for &p in paths {
        git_add(repo_root, p)?;
    }
    if auto_commit && git_has_staged(repo_root) {
        git_commit(repo_root, message)?;
    }
    Ok(())
}
