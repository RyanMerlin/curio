use crate::service::types::{WorkspaceRegistryRecord, WorkspaceStatus};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct WorkspaceRegistryFile {
    #[serde(default)]
    records: Vec<WorkspaceRegistryRecord>,
}

#[derive(Debug, thiserror::Error)]
pub enum WorkspaceRegistryError {
    #[error("workspace not found: {0}")]
    NotFound(String),
    #[error("workspace disabled: {0}")]
    Disabled(String),
}

#[derive(Debug, Clone)]
pub struct WorkspaceRegistry {
    path: PathBuf,
    records: BTreeMap<String, WorkspaceRegistryRecord>,
}

impl WorkspaceRegistry {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        if !path.exists() {
            return Ok(Self {
                path,
                records: BTreeMap::new(),
            });
        }

        let raw = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read workspace registry: {}", path.display()))?;
        let file: WorkspaceRegistryFile = serde_json::from_str(&raw)
            .with_context(|| format!("Failed to parse workspace registry: {}", path.display()))?;
        let mut records = BTreeMap::new();
        for record in file.records {
            records.insert(record.workspace_id.clone(), record);
        }
        Ok(Self { path, records })
    }

    pub fn load_default() -> Result<Self> {
        Self::load(default_registry_path()?)
    }

    pub fn save(&self) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!(
                    "Failed to create workspace registry directory: {}",
                    parent.display()
                )
            })?;
        }
        let file = WorkspaceRegistryFile {
            records: self.records.values().cloned().collect(),
        };
        let raw = serde_json::to_string_pretty(&file).context("Failed to serialize registry")?;

        // Atomic write: stage to <path>.tmp then rename. Prevents truncation
        // and partial-write states under concurrent service restarts.
        let mut tmp_path = self.path.clone();
        let mut tmp_name = self
            .path
            .file_name()
            .map(|n| n.to_os_string())
            .unwrap_or_default();
        tmp_name.push(".tmp");
        tmp_path.set_file_name(tmp_name);

        fs::write(&tmp_path, raw).with_context(|| {
            format!(
                "Failed to write workspace registry tmp file: {}",
                tmp_path.display()
            )
        })?;
        fs::rename(&tmp_path, &self.path).with_context(|| {
            format!(
                "Failed to atomically replace workspace registry: {}",
                self.path.display()
            )
        })
    }

    pub fn upsert(&mut self, record: WorkspaceRegistryRecord) {
        self.records.insert(record.workspace_id.clone(), record);
    }

    pub fn remove(&mut self, workspace_id: &str) -> Option<WorkspaceRegistryRecord> {
        self.records.remove(workspace_id)
    }

    pub fn resolve(&self, workspace_id: &str) -> Result<&WorkspaceRegistryRecord> {
        let record = self
            .records
            .get(workspace_id)
            .ok_or_else(|| WorkspaceRegistryError::NotFound(workspace_id.to_string()))?;
        if record.status == WorkspaceStatus::Disabled {
            return Err(WorkspaceRegistryError::Disabled(workspace_id.to_string()).into());
        }
        Ok(record)
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &WorkspaceRegistryRecord> {
        self.records.values()
    }
}

pub fn default_registry_path() -> Result<PathBuf> {
    if let Ok(path) = env::var("CURIO_SERVICE_REGISTRY")
        && !path.trim().is_empty()
    {
        return Ok(PathBuf::from(path));
    }

    let root = discover_repo_root()?;
    Ok(root.join(".curio").join("service").join("workspaces.json"))
}

pub fn discover_repo_root() -> Result<PathBuf> {
    if let Ok(path) = env::var("CURIO_REPO_ROOT")
        && !path.trim().is_empty()
    {
        return Ok(PathBuf::from(path));
    }

    let mut current = env::current_dir().context("Failed to resolve current working directory")?;
    loop {
        if current.join("curio-rs").is_dir() {
            return Ok(current);
        }
        if !current.pop() {
            break;
        }
    }

    anyhow::bail!(
        "Could not locate the Curio repository root. Set CURIO_REPO_ROOT or run from the repo."
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::types::{JobType, WorkspaceStatus, WriteMode};
    use std::collections::BTreeMap;

    fn temp_registry_path() -> PathBuf {
        std::env::temp_dir().join(format!(
            "curio-service-registry-{}.json",
            uuid::Uuid::new_v4()
        ))
    }

    #[test]
    fn registry_resolves_workspace_by_id() {
        let path = temp_registry_path();
        let mut registry = WorkspaceRegistry::load(&path).expect("registry load");
        registry.upsert(WorkspaceRegistryRecord {
            workspace_id: "acme".to_string(),
            display_name: "Acme".to_string(),
            repo_url: "https://example.invalid/acme.git".to_string(),
            default_branch: "main".to_string(),
            credential_ref: Some("secret/gitlab/acme".to_string()),
            kb_root: "wiki".to_string(),
            allowed_job_types: vec![JobType::Sync.as_str().to_string()],
            write_policy: WriteMode::DirectPush,
            provider_defaults: serde_json::json!({"provider": "passthrough"}),
            status: WorkspaceStatus::Active,
            description: Some("Acme workspace".to_string()),
            metadata: BTreeMap::new(),
        });
        registry.save().expect("registry save");

        let loaded = WorkspaceRegistry::load(&path).expect("registry reload");
        let record = loaded.resolve("acme").expect("workspace lookup");
        assert_eq!(record.display_name, "Acme");
        assert_eq!(record.allowed_job_types, vec!["sync"]);
        let _ = std::fs::remove_file(path);
    }
}
