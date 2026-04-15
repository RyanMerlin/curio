use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

const MAX_AUDIT_LINES: usize = 1000;
const RETAIN_AUDIT_LINES: usize = 250;
const MAX_AUDIT_BYTES: u64 = 512 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AuditEntry {
    ts: String,
    kind: String,
    entry: String,
}

fn audit_dir(wiki_dir: &Path) -> PathBuf {
    if let Ok(raw) = env::var("CURIO_AUDIT_DIR") {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            let repo_root = env::var_os("CURIO_REPO_ROOT")
                .map(PathBuf::from)
                .or_else(|| wiki_dir.parent().map(PathBuf::from))
                .unwrap_or_else(|| PathBuf::from("."));
            let expanded = trimmed
                .replace("${REPO_ROOT}", &repo_root.to_string_lossy())
                .replace("$REPO_ROOT", &repo_root.to_string_lossy());
            let path = PathBuf::from(expanded);
            return if path.is_relative() {
                repo_root.join(path)
            } else {
                path
            };
        }
    }
    wiki_dir.join("_config")
}

fn audit_log_path(wiki_dir: &Path) -> PathBuf {
    audit_dir(wiki_dir).join("audit.jsonl")
}

fn sync_marker_path(wiki_dir: &Path) -> PathBuf {
    wiki_dir.join("_config").join("last-sync.txt")
}

fn legacy_audit_log_path(wiki_dir: &Path) -> PathBuf {
    wiki_dir.join(".curio").join("audit.jsonl")
}

fn ensure_audit_dir(wiki_dir: &Path) -> Result<()> {
    let dir = audit_dir(wiki_dir);
    fs::create_dir_all(&dir)
        .with_context(|| format!("Failed to create audit directory: {}", dir.display()))
}

fn write_sync_marker(wiki_dir: &Path, ts: &str) -> Result<()> {
    let marker_path = sync_marker_path(wiki_dir);
    if let Some(parent) = marker_path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "Failed to create sync marker directory: {}",
                parent.display()
            )
        })?;
    }
    fs::write(&marker_path, format!("{}\n", ts))
        .with_context(|| format!("Failed to write sync marker: {}", marker_path.display()))
}

fn parse_kind(entry: &str) -> String {
    entry
        .split_once(':')
        .map(|(kind, _)| kind.trim().to_ascii_lowercase())
        .filter(|kind| !kind.is_empty())
        .unwrap_or_else(|| "event".to_string())
}

fn read_entries(wiki_dir: &Path) -> Result<Vec<AuditEntry>> {
    let log_path = if audit_log_path(wiki_dir).exists() {
        audit_log_path(wiki_dir)
    } else {
        legacy_audit_log_path(wiki_dir)
    };
    if !log_path.exists() {
        return Ok(Vec::new());
    }

    let file = fs::File::open(&log_path)
        .with_context(|| format!("Failed to open audit log: {}", log_path.display()))?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        if let Ok(entry) = serde_json::from_str::<AuditEntry>(&line) {
            entries.push(entry);
        }
    }

    Ok(entries)
}

fn write_entries(wiki_dir: &Path, entries: &[AuditEntry]) -> Result<()> {
    ensure_audit_dir(wiki_dir)?;
    let log_path = audit_log_path(wiki_dir);
    let mut file = fs::File::create(&log_path)
        .with_context(|| format!("Failed to rewrite audit log: {}", log_path.display()))?;

    for entry in entries {
        let line = serde_json::to_string(entry).context("Failed to serialize audit entry")?;
        writeln!(file, "{}", line).context("Failed to write audit entry")?;
    }

    Ok(())
}

fn maybe_compact(wiki_dir: &Path) -> Result<()> {
    let log_path = audit_log_path(wiki_dir);
    let metadata = match fs::metadata(&log_path) {
        Ok(metadata) => metadata,
        Err(_) => return Ok(()),
    };

    let entries = read_entries(wiki_dir)?;
    if entries.len() <= MAX_AUDIT_LINES && metadata.len() <= MAX_AUDIT_BYTES {
        return Ok(());
    }

    let retained_start = entries.len().saturating_sub(RETAIN_AUDIT_LINES);
    let retained = entries[retained_start..].to_vec();
    let last_sync = entries
        .iter()
        .rev()
        .find(|entry| entry.kind == "sync")
        .map(|entry| entry.ts.clone());

    let checkpoint = AuditEntry {
        ts: Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        kind: "checkpoint".to_string(),
        entry: format!(
            "checkpoint: compacted audit log; retained {} recent entries; previous entries={}{}",
            retained.len(),
            entries.len().saturating_sub(retained.len()),
            last_sync
                .map(|ts| format!("; last_sync={}", ts))
                .unwrap_or_default()
        ),
    };

    let mut compacted = Vec::with_capacity(retained.len() + 1);
    compacted.push(checkpoint);
    compacted.extend(retained);
    write_entries(wiki_dir, &compacted)
}

pub fn append_entry(wiki_dir: &Path, entry: &str) -> Result<()> {
    ensure_audit_dir(wiki_dir)?;
    let log_path = audit_log_path(wiki_dir);
    let legacy_path = legacy_audit_log_path(wiki_dir);
    if !log_path.exists() && legacy_path.exists() {
        fs::copy(&legacy_path, &log_path).with_context(|| {
            format!(
                "Failed to migrate legacy audit log from {} to {}",
                legacy_path.display(),
                log_path.display()
            )
        })?;
    }
    let audit_entry = AuditEntry {
        ts: Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        kind: parse_kind(entry),
        entry: entry.to_string(),
    };

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .with_context(|| {
            format!(
                "Failed to open audit log for append: {}",
                log_path.display()
            )
        })?;

    let line = serde_json::to_string(&audit_entry).context("Failed to serialize audit entry")?;
    writeln!(file, "{}", line).context("Failed to append audit entry")?;
    drop(file);

    if audit_entry.kind == "sync" {
        write_sync_marker(wiki_dir, &audit_entry.ts)?;
    }

    maybe_compact(wiki_dir)
}

/// Append a human-readable entry to the narrative log at `wiki/_config/log.md`.
///
/// Karpathy-style: append-only, one dated line per intake/process/publish/query,
/// so the log compounds into a readable history of what the KB has learned.
pub fn append_log_md(wiki_dir: &Path, entry: &str) -> Result<()> {
    let log_path = wiki_dir.join("_config").join("log.md");
    let ts = Utc::now().format("%Y-%m-%d %H:%M UTC").to_string();
    let line = format!("- {} — {}\n", ts, entry);

    // Initialise with a header if the file is new
    let needs_header = !log_path.exists()
        || std::fs::metadata(&log_path)
            .map(|m| m.len() == 0)
            .unwrap_or(true);
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .with_context(|| format!("Failed to open log.md for append: {}", log_path.display()))?;
    if needs_header {
        use std::io::Write;
        writeln!(
            file,
            "# Curio Knowledge Log\n\nAppend-only record of ingests, routing runs, publications, and queries.\n"
        )?;
    }
    use std::io::Write;
    file.write_all(line.as_bytes())
        .with_context(|| format!("Failed to append to log.md: {}", log_path.display()))
}

pub fn read_last_sync(wiki_dir: &Path) -> Result<Option<String>> {
    let marker_path = sync_marker_path(wiki_dir);
    if marker_path.exists() {
        let raw = fs::read_to_string(&marker_path)
            .with_context(|| format!("Failed to read sync marker: {}", marker_path.display()))?;
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            return Ok(Some(trimmed.to_string()));
        }
    }
    let entries = read_entries(wiki_dir)?;
    Ok(entries
        .iter()
        .rev()
        .find(|entry| entry.kind == "sync")
        .map(|entry| entry.ts.clone()))
}
