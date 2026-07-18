//! Provider-neutral, fail-closed source authorization snapshots.
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum PrincipalKind {
    User,
    Group,
    Service,
    Public,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Principal {
    pub kind: PrincipalKind,
    pub id: String,
    #[serde(default)]
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AclSnapshot {
    pub source_id: String,
    pub source_revision: String,
    pub captured_at: String,
    #[serde(default)]
    pub allow: Vec<Principal>,
    #[serde(default)]
    pub deny: Vec<Principal>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AccessContext {
    pub principal_ids: Vec<String>,
    pub group_ids: Vec<String>,
    pub service_ids: Vec<String>,
}

impl AccessContext {
    pub fn new(principal_ids: impl IntoIterator<Item = String>) -> Self {
        Self {
            principal_ids: principal_ids.into_iter().collect(),
            ..Self::default()
        }
    }
    fn matches(&self, principal: &Principal) -> bool {
        match principal.kind {
            PrincipalKind::Public => true,
            PrincipalKind::User => self.principal_ids.iter().any(|id| id == &principal.id),
            PrincipalKind::Group => self.group_ids.iter().any(|id| id == &principal.id),
            PrincipalKind::Service => self.service_ids.iter().any(|id| id == &principal.id),
        }
    }
}

/// No snapshot means an unrestricted legacy/public page. A present snapshot
/// is authoritative: denies win, and an empty allow list is restricted.
pub fn can_read(snapshot: Option<&AclSnapshot>, access: Option<&AccessContext>) -> bool {
    let Some(snapshot) = snapshot else {
        return true;
    };
    let Some(access) = access else { return false };
    if snapshot.deny.iter().any(|p| access.matches(p)) {
        return false;
    }
    snapshot.allow.iter().any(|p| access.matches(p))
}

pub fn load_snapshot(wiki_dir: &Path, source_id: &str) -> Result<Option<AclSnapshot>> {
    let dir = wiki_dir.join("_admin").join("acl");
    if !dir.exists() {
        return Ok(None);
    }
    for entry in
        std::fs::read_dir(&dir).with_context(|| format!("Failed to read {}", dir.display()))?
    {
        let path = entry?.path();
        if path.extension().and_then(|v| v.to_str()) != Some("json") {
            continue;
        }
        let raw = std::fs::read_to_string(&path)?;
        let snapshot: AclSnapshot = serde_json::from_str(&raw)
            .with_context(|| format!("Invalid ACL snapshot {}", path.display()))?;
        if snapshot.source_id == source_id {
            return Ok(Some(snapshot));
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn p(kind: PrincipalKind, id: &str) -> Principal {
        Principal {
            kind,
            id: id.into(),
            label: None,
        }
    }
    #[test]
    fn deny_wins_and_missing_identity_fails_closed() {
        let s = AclSnapshot {
            source_id: "x".into(),
            source_revision: "1".into(),
            captured_at: "now".into(),
            allow: vec![p(PrincipalKind::Group, "eng")],
            deny: vec![p(PrincipalKind::User, "u")],
        };
        let mut a = AccessContext::default();
        a.group_ids.push("eng".into());
        assert!(can_read(Some(&s), Some(&a)));
        a.principal_ids.push("u".into());
        assert!(!can_read(Some(&s), Some(&a)));
        assert!(!can_read(Some(&s), None));
    }
}
