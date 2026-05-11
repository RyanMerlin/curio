use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum JobType {
    Intake,
    Process,
    Heal,
    Sync,
    Publish,
    Review,
    Search,
    Custom,
}

impl JobType {
    pub fn as_str(&self) -> &'static str {
        match self {
            JobType::Intake => "intake",
            JobType::Process => "process",
            JobType::Heal => "heal",
            JobType::Sync => "sync",
            JobType::Publish => "publish",
            JobType::Review => "review",
            JobType::Search => "search",
            JobType::Custom => "custom",
        }
    }

    pub fn is_mutating(&self) -> bool {
        matches!(
            self,
            JobType::Intake
                | JobType::Process
                | JobType::Heal
                | JobType::Sync
                | JobType::Publish
                | JobType::Review
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum JobState {
    Queued,
    Running,
    Succeeded,
    Failed,
    Rejected,
    Duplicate,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WriteMode {
    ReadOnly,
    DirectPush,
    BranchAndPr,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceStatus {
    Active,
    ReadOnly,
    Disabled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JobActor {
    pub kind: String,
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JobTrigger {
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub received_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CurioJobRequest {
    pub job_type: JobType,
    pub workspace_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    pub trigger: JobTrigger,
    pub actor: JobActor,
    pub operation: String,
    #[serde(default)]
    pub inputs: Value,
    pub write_mode: WriteMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
}

impl CurioJobRequest {
    pub fn effective_idempotency_key(&self) -> String {
        self.idempotency_key.clone().unwrap_or_else(|| {
            format!(
                "{}:{}:{}:{}",
                self.workspace_id,
                self.job_type.as_str(),
                self.operation,
                self.correlation_id
                    .clone()
                    .unwrap_or_else(|| "default".to_string())
            )
        })
    }

    pub fn is_mutating(&self) -> bool {
        self.write_mode != WriteMode::ReadOnly && self.job_type.is_mutating()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommitMetadata {
    pub job_id: String,
    pub actor_id: String,
    pub trigger_kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trigger_source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GitMaterializationPlan {
    pub mirror_dir: PathBuf,
    pub worktree_dir: PathBuf,
    pub checkout_ref: String,
    pub target_branch: String,
    pub push_refspec: String,
    pub mutating: bool,
    pub commit_metadata: CommitMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExecutionResult {
    pub command_line: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    #[serde(default)]
    pub stdout: String,
    #[serde(default)]
    pub stderr: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_revision: Option<String>,
    #[serde(default)]
    pub pushed: bool,
    #[serde(default)]
    pub artifacts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProviderPhaseOutput {
    pub phase: String,
    pub provider: String,
    pub summary: String,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuditEvent {
    pub at: String,
    pub event: String,
    pub job_id: String,
    pub workspace_id: String,
    pub actor_id: String,
    pub trigger_kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkspaceRegistryRecord {
    pub workspace_id: String,
    pub display_name: String,
    pub repo_url: String,
    pub default_branch: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub credential_ref: Option<String>,
    pub kb_root: String,
    #[serde(default)]
    pub allowed_job_types: Vec<String>,
    pub write_policy: WriteMode,
    #[serde(default)]
    pub provider_defaults: Value,
    pub status: WorkspaceStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    pub metadata: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CurioJobStatus {
    pub job_id: String,
    pub idempotency_key: String,
    pub request: CurioJobRequest,
    pub workspace: WorkspaceRegistryRecord,
    pub state: JobState,
    pub queued_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub started_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<ExecutionResult>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(default)]
    pub audit: Vec<AuditEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobSubmissionResponse {
    pub accepted: bool,
    pub duplicate: bool,
    pub job: CurioJobStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthResponse {
    pub ok: bool,
    pub service: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReadinessResponse {
    pub ok: bool,
    pub registry_records: usize,
    pub jobs_cached: usize,
    pub read_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProviderStepOutput {
    pub phase: String,
    pub provider: String,
    pub summary: String,
    #[serde(default)]
    pub metadata: Value,
}

pub trait ProviderAdapter: Send + Sync {
    fn name(&self) -> &str;
    fn prepare(
        &self,
        request: &CurioJobRequest,
        workspace: &WorkspaceRegistryRecord,
        plan: &GitMaterializationPlan,
    ) -> anyhow::Result<ProviderStepOutput>;
    fn analyze(
        &self,
        request: &CurioJobRequest,
        workspace: &WorkspaceRegistryRecord,
        plan: &GitMaterializationPlan,
    ) -> anyhow::Result<ProviderStepOutput>;
    fn route(
        &self,
        request: &CurioJobRequest,
        workspace: &WorkspaceRegistryRecord,
        plan: &GitMaterializationPlan,
    ) -> anyhow::Result<ProviderStepOutput>;
    fn propose_changes(
        &self,
        request: &CurioJobRequest,
        workspace: &WorkspaceRegistryRecord,
        plan: &GitMaterializationPlan,
    ) -> anyhow::Result<ProviderStepOutput>;
    fn summarize(
        &self,
        request: &CurioJobRequest,
        workspace: &WorkspaceRegistryRecord,
        plan: &GitMaterializationPlan,
    ) -> anyhow::Result<ProviderStepOutput>;
}
