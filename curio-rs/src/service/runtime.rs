use crate::git_ops::{
    git_add, git_clone_mirror, git_commit_with_identity, git_fetch_prune, git_has_staged, git_push,
    git_set_remote_url, git_status_porcelain, git_worktree_add, git_worktree_remove,
};
use crate::service::providers::{build_provider_adapter, provider_backend_from_env};
use crate::service::registry::{WorkspaceRegistry, default_registry_path};
use crate::service::types::*;
use anyhow::{Context, Result, bail};
use chrono::Utc;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use tokio::sync::{Mutex as AsyncMutex, OwnedMutexGuard};
use uuid::Uuid;

// ── Git credential guard ───────────────────────────────────────────────────

/// RAII guard that writes a temp git-credentials-format file for the duration
/// of a git clone/fetch, then zero-overwrites and deletes it on drop.
///
/// Git is pointed at the file via GIT_CONFIG_COUNT/KEY/VALUE env vars so that
/// the credential token is never embedded in the remote URL (which would persist
/// in .git/config and appear in error messages).
pub struct GitCredentialGuard {
    path: PathBuf,
    bare_url: String,
    has_creds: bool,
}

impl GitCredentialGuard {
    pub fn create(workspace: &WorkspaceRegistryRecord) -> Result<Self> {
        let token = std::env::var("CURIO_GITLAB_TOKEN")
            .ok()
            .filter(|v| !v.trim().is_empty());
        let username = std::env::var("CURIO_GITLAB_USERNAME")
            .ok()
            .filter(|v| !v.trim().is_empty())
            .unwrap_or_else(|| "oauth2".to_string());

        let bare_url = workspace.repo_url.clone();
        let path = std::env::temp_dir().join(format!(".curio-creds-{}", Uuid::new_v4()));

        let has_creds = if let Some(token) = &token {
            if bare_url.starts_with("https://") {
                let without_scheme = bare_url.trim_start_matches("https://");
                let host = without_scheme.split('/').next().unwrap_or(without_scheme);
                let cred_line = format!("https://{}:{}@{}\n", username, token, host);
                fs::write(&path, &cred_line).with_context(|| {
                    format!("Failed to write git credentials: {}", path.display())
                })?;
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    fs::set_permissions(&path, fs::Permissions::from_mode(0o600))
                        .context("Failed to set permissions on credentials file")?;
                }
                true
            } else {
                false
            }
        } else {
            false
        };

        Ok(Self {
            path,
            bare_url,
            has_creds,
        })
    }

    pub fn bare_url(&self) -> &str {
        &self.bare_url
    }

    pub fn git_env(&self) -> Vec<(String, String)> {
        if !self.has_creds {
            return vec![];
        }
        vec![
            ("GIT_CONFIG_COUNT".to_string(), "1".to_string()),
            (
                "GIT_CONFIG_KEY_0".to_string(),
                "credential.helper".to_string(),
            ),
            (
                "GIT_CONFIG_VALUE_0".to_string(),
                format!("store --file={}", self.path.display()),
            ),
        ]
    }
}

impl Drop for GitCredentialGuard {
    fn drop(&mut self) {
        if self.has_creds && self.path.exists() {
            if let Ok(meta) = fs::metadata(&self.path) {
                let zeros = vec![0u8; meta.len() as usize];
                let _ = fs::write(&self.path, &zeros);
            }
            let _ = fs::remove_file(&self.path);
        }
    }
}

// ── Service config ─────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ServiceConfig {
    pub bind_addr: std::net::SocketAddr,
    pub registry_path: PathBuf,
    pub job_store_path: PathBuf,
    pub audit_log_path: PathBuf,
    pub cache_root: PathBuf,
    pub curio_binary: PathBuf,
    pub read_only: bool,
    pub execute_jobs: bool,
    pub provider_backend: String,
}

impl ServiceConfig {
    pub fn from_env() -> Result<Self> {
        let registry_path = env_path("CURIO_SERVICE_REGISTRY").unwrap_or(default_registry_path()?);
        let cache_root =
            env_path("CURIO_SERVICE_CACHE").unwrap_or_else(|| PathBuf::from("/tmp/curio/cache"));
        let job_store_path = env_path("CURIO_SERVICE_JOBS").unwrap_or_else(|| {
            registry_path
                .parent()
                .unwrap_or(Path::new("."))
                .join("jobs.jsonl")
        });
        let audit_log_path = env_path("CURIO_SERVICE_AUDIT").unwrap_or_else(|| {
            registry_path
                .parent()
                .unwrap_or(Path::new("."))
                .join("audit.jsonl")
        });
        let bind_addr = std::env::var("CURIO_SERVICE_BIND_ADDR")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or_else(|| "0.0.0.0:8080".parse().expect("valid default bind address"));
        let curio_binary = env_path("CURIO_BINARY").unwrap_or_else(|| PathBuf::from("curio"));
        let read_only = env_bool("CURIO_SERVICE_READ_ONLY");
        let execute_jobs = !env_bool("CURIO_SERVICE_PLAN_ONLY");
        let provider_backend = std::env::var("CURIO_SERVICE_PROVIDER_BACKEND")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| provider_backend_from_env());
        Ok(Self {
            bind_addr,
            registry_path,
            job_store_path,
            audit_log_path,
            cache_root,
            curio_binary,
            read_only,
            execute_jobs,
            provider_backend,
        })
    }
}

fn env_path(var: &str) -> Option<PathBuf> {
    std::env::var(var).ok().and_then(|value| {
        let trimmed = value.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(PathBuf::from(trimmed))
        }
    })
}

fn env_bool(var: &str) -> bool {
    matches!(
        std::env::var(var).ok().as_deref().map(|s| s.trim().to_ascii_lowercase()),
        Some(value) if matches!(value.as_str(), "1" | "true" | "yes" | "on")
    )
}

pub trait Materializer: Send + Sync {
    fn materialize(
        &self,
        workspace: &WorkspaceRegistryRecord,
        request: &CurioJobRequest,
        job_id: &str,
    ) -> Result<GitMaterializationPlan>;
}

#[derive(Debug, Clone)]
pub struct GitMaterializer {
    pub cache_root: PathBuf,
}

impl Materializer for GitMaterializer {
    fn materialize(
        &self,
        workspace: &WorkspaceRegistryRecord,
        request: &CurioJobRequest,
        job_id: &str,
    ) -> Result<GitMaterializationPlan> {
        let mirror_dir = self
            .cache_root
            .join("mirrors")
            .join(sanitize_segment(&workspace.workspace_id));
        let worktree_dir = self
            .cache_root
            .join("worktrees")
            .join(sanitize_segment(&workspace.workspace_id))
            .join(job_id);
        let checkout_ref = request
            .inputs
            .get("checkout_ref")
            .and_then(|value| value.as_str())
            .filter(|value| !value.is_empty())
            .unwrap_or(&workspace.default_branch)
            .to_string();
        let target_branch = request
            .inputs
            .get("target_branch")
            .and_then(|value| value.as_str())
            .filter(|value| !value.is_empty())
            .unwrap_or(&workspace.default_branch)
            .to_string();
        let push_refspec = format!("HEAD:refs/heads/{}", target_branch);
        let mutating = request.is_mutating() && workspace.write_policy != WriteMode::ReadOnly;
        let commit_metadata = CommitMetadata {
            job_id: job_id.to_string(),
            actor_id: request.actor.id.clone(),
            trigger_kind: request.trigger.kind.clone(),
            trigger_source: request.trigger.source.clone(),
            correlation_id: request.correlation_id.clone(),
        };

        let cred_guard = GitCredentialGuard::create(workspace)?;
        let git_env = cred_guard.git_env();
        if mirror_dir.exists() {
            git_set_remote_url(&mirror_dir, "origin", cred_guard.bare_url())?;
            git_fetch_prune(&mirror_dir, &git_env)?;
        } else {
            git_clone_mirror(cred_guard.bare_url(), &mirror_dir, &git_env)?;
            git_set_remote_url(&mirror_dir, "origin", cred_guard.bare_url())?;
        }
        // cred_guard drops here, zero-wiping the temp credentials file.
        if worktree_dir.exists() {
            let _ = git_worktree_remove(&worktree_dir);
            let _ = fs::remove_dir_all(&worktree_dir);
        }
        git_worktree_add(&mirror_dir, &worktree_dir, &checkout_ref)?;

        Ok(GitMaterializationPlan {
            mirror_dir,
            worktree_dir,
            checkout_ref,
            target_branch,
            push_refspec,
            mutating,
            commit_metadata,
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct NoopMaterializer;

impl Materializer for NoopMaterializer {
    fn materialize(
        &self,
        workspace: &WorkspaceRegistryRecord,
        request: &CurioJobRequest,
        job_id: &str,
    ) -> Result<GitMaterializationPlan> {
        let mirror_dir = PathBuf::from("/tmp/curio/mirror").join(&workspace.workspace_id);
        let worktree_dir = PathBuf::from("/tmp/curio/worktree").join(job_id);
        Ok(GitMaterializationPlan {
            mirror_dir,
            worktree_dir,
            checkout_ref: workspace.default_branch.clone(),
            target_branch: workspace.default_branch.clone(),
            push_refspec: format!("HEAD:refs/heads/{}", workspace.default_branch),
            mutating: request.is_mutating() && workspace.write_policy != WriteMode::ReadOnly,
            commit_metadata: CommitMetadata {
                job_id: job_id.to_string(),
                actor_id: request.actor.id.clone(),
                trigger_kind: request.trigger.kind.clone(),
                trigger_source: request.trigger.source.clone(),
                correlation_id: request.correlation_id.clone(),
            },
        })
    }
}

pub trait CurioExecutor: Send + Sync {
    fn execute(
        &self,
        request: &CurioJobRequest,
        workspace: &WorkspaceRegistryRecord,
        plan: &GitMaterializationPlan,
        provider: &dyn ProviderAdapter,
    ) -> Result<ExecutionResult>;
}

#[derive(Debug, Clone)]
pub struct CommandCurioExecutor {
    pub curio_binary: PathBuf,
    pub execute_jobs: bool,
}

impl CurioExecutor for CommandCurioExecutor {
    fn execute(
        &self,
        request: &CurioJobRequest,
        _workspace: &WorkspaceRegistryRecord,
        plan: &GitMaterializationPlan,
        provider: &dyn ProviderAdapter,
    ) -> Result<ExecutionResult> {
        let command_line = build_curio_command(&self.curio_binary, request, &plan.worktree_dir);
        if !self.execute_jobs {
            return Ok(ExecutionResult {
                command_line,
                exit_code: None,
                stdout: String::new(),
                stderr: "plan-only mode".to_string(),
                current_revision: None,
                pushed: false,
                artifacts: vec![],
            });
        }

        let mut command = build_process_command(&self.curio_binary, request, &plan.worktree_dir);
        let output = command
            .output()
            .with_context(|| format!("Failed to execute Curio command: {}", command_line))?;
        let exit_code = output.status.code();
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !output.status.success() {
            bail!(
                "Curio command failed with status {:?}: {}",
                exit_code,
                stderr
            );
        }

        let status = git_status_porcelain(&plan.worktree_dir)?;
        let mut pushed = false;
        if !status.trim().is_empty() {
            let commit_message = format!(
                "curio job {}: {} {}",
                plan.commit_metadata.job_id,
                request.job_type.as_str(),
                request.operation
            );
            git_add(&plan.worktree_dir, Path::new("."))?;
            if git_has_staged(&plan.worktree_dir) {
                git_commit_with_identity(
                    &plan.worktree_dir,
                    &commit_message,
                    "Curio",
                    "curio@users.noreply.gitlab.com",
                )?;
            }
        }
        if plan.mutating {
            git_push(&plan.worktree_dir, &plan.push_refspec)?;
            pushed = true;
        }

        Ok(ExecutionResult {
            command_line,
            exit_code,
            stdout,
            stderr,
            current_revision: current_revision(&plan.worktree_dir).ok(),
            pushed,
            artifacts: vec![format!("provider={}", provider.name())],
        })
    }
}

fn build_process_command(binary: &Path, request: &CurioJobRequest, worktree_dir: &Path) -> Command {
    let mut command = Command::new(binary);
    command.arg(request.operation.as_str());
    command.arg("--kb-dir").arg(worktree_dir);
    command.arg("--json");
    if let Ok(repo_root) = std::env::var("CURIO_REPO_ROOT") {
        command.current_dir(repo_root);
    }

    if let Some(provider) = &request.provider {
        command.env("CURIO_SERVICE_PROVIDER", provider);
    }
    if let Some(value) = request
        .inputs
        .get("args")
        .and_then(|value| value.as_array())
    {
        for arg in value.iter().filter_map(|value| value.as_str()) {
            command.arg(arg);
        }
    }
    command
}

fn build_curio_command(binary: &Path, request: &CurioJobRequest, worktree_dir: &Path) -> String {
    let mut parts = vec![
        binary.display().to_string(),
        request.operation.clone(),
        "--kb-dir".to_string(),
        worktree_dir.display().to_string(),
        "--json".to_string(),
    ];
    if let Some(provider) = &request.provider {
        parts.push(format!("provider={provider}"));
    }
    if let Some(value) = request
        .inputs
        .get("args")
        .and_then(|value| value.as_array())
    {
        for arg in value.iter().filter_map(|value| value.as_str()) {
            parts.push(arg.to_string());
        }
    }
    parts.join(" ")
}

fn current_revision(worktree_dir: &Path) -> Result<String> {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(worktree_dir)
        .output()
        .with_context(|| {
            format!(
                "Failed to determine current revision in {}",
                worktree_dir.display()
            )
        })?;
    if !output.status.success() {
        bail!("git rev-parse HEAD failed");
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[derive(Debug, Clone, Default)]
pub struct NoopCurioExecutor;

impl CurioExecutor for NoopCurioExecutor {
    fn execute(
        &self,
        request: &CurioJobRequest,
        workspace: &WorkspaceRegistryRecord,
        plan: &GitMaterializationPlan,
        provider: &dyn ProviderAdapter,
    ) -> Result<ExecutionResult> {
        Ok(ExecutionResult {
            command_line: build_curio_command(Path::new("curio"), request, &plan.worktree_dir),
            exit_code: Some(0),
            stdout: serde_json::to_string_pretty(&serde_json::json!({
                "workspace": workspace.workspace_id,
                "provider": provider.name(),
                "operation": request.operation,
            }))?,
            stderr: String::new(),
            current_revision: Some("noop".to_string()),
            pushed: false,
            artifacts: vec![],
        })
    }
}

#[derive(Debug, Clone)]
pub struct JobStore {
    path: PathBuf,
    records: Arc<Mutex<BTreeMap<String, CurioJobStatus>>>,
}

impl JobStore {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let mut records = BTreeMap::new();
        if path.exists() {
            let raw = fs::read_to_string(&path)
                .with_context(|| format!("Failed to read job store: {}", path.display()))?;
            for line in raw.lines().filter(|line| !line.trim().is_empty()) {
                let status: CurioJobStatus = serde_json::from_str(line).with_context(|| {
                    format!("Failed to parse job store line in {}", path.display())
                })?;
                records.insert(status.job_id.clone(), status);
            }
        }
        Ok(Self {
            path,
            records: Arc::new(Mutex::new(records)),
        })
    }

    pub fn len(&self) -> usize {
        self.records.lock().unwrap().len()
    }

    pub fn get(&self, job_id: &str) -> Option<CurioJobStatus> {
        self.records.lock().unwrap().get(job_id).cloned()
    }

    pub fn find_by_idempotency_key(&self, key: &str) -> Option<CurioJobStatus> {
        self.records
            .lock()
            .unwrap()
            .values()
            .find(|status| status.idempotency_key == key)
            .cloned()
    }

    pub fn upsert(&self, status: CurioJobStatus) -> Result<()> {
        {
            let mut guard = self.records.lock().unwrap();
            guard.insert(status.job_id.clone(), status.clone());
        }
        self.append_line(&status)
    }

    pub fn update_state(
        &self,
        job_id: &str,
        state: JobState,
        message: Option<String>,
        result: Option<ExecutionResult>,
        started_at: Option<String>,
        completed_at: Option<String>,
    ) -> Result<CurioJobStatus> {
        let updated = {
            let mut guard = self.records.lock().unwrap();
            let status = guard
                .get_mut(job_id)
                .with_context(|| format!("Job not found: {}", job_id))?;
            status.state = state;
            if let Some(message) = message {
                status.error = Some(message);
            }
            if started_at.is_some() {
                status.started_at = started_at;
            }
            if completed_at.is_some() {
                status.completed_at = completed_at;
            }
            if result.is_some() {
                status.result = result;
            }
            status.clone()
        };
        self.append_line(&updated)?;
        Ok(updated)
    }

    pub fn all(&self) -> Vec<CurioJobStatus> {
        self.records.lock().unwrap().values().cloned().collect()
    }

    fn append_line(&self, status: &CurioJobStatus) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create job store directory: {}", parent.display())
            })?;
        }
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .with_context(|| format!("Failed to open job store: {}", self.path.display()))?;
        use std::io::Write;
        writeln!(file, "{}", serde_json::to_string(status)?)
            .with_context(|| format!("Failed to write job store: {}", self.path.display()))
    }
}

#[derive(Clone)]
pub struct ServiceRuntime {
    registry: Arc<WorkspaceRegistry>,
    job_store: Arc<JobStore>,
    materializer: Arc<dyn Materializer>,
    executor: Arc<dyn CurioExecutor>,
    locks: Arc<AsyncMutex<BTreeMap<String, Arc<AsyncMutex<()>>>>>,
    config: ServiceConfig,
}

impl ServiceRuntime {
    pub fn from_config(config: ServiceConfig) -> Result<Self> {
        let registry = Arc::new(WorkspaceRegistry::load(&config.registry_path)?);
        let job_store = Arc::new(JobStore::load(&config.job_store_path)?);
        fs::create_dir_all(&config.cache_root).with_context(|| {
            format!(
                "Failed to create service cache root: {}",
                config.cache_root.display()
            )
        })?;

        Ok(Self {
            registry,
            job_store,
            materializer: Arc::new(GitMaterializer {
                cache_root: config.cache_root.clone(),
            }),
            executor: Arc::new(CommandCurioExecutor {
                curio_binary: config.curio_binary.clone(),
                execute_jobs: config.execute_jobs,
            }),
            locks: Arc::new(AsyncMutex::new(BTreeMap::new())),
            config,
        })
    }

    pub fn with_components(
        config: ServiceConfig,
        registry: WorkspaceRegistry,
        job_store: JobStore,
        materializer: Arc<dyn Materializer>,
        executor: Arc<dyn CurioExecutor>,
    ) -> Self {
        Self {
            registry: Arc::new(registry),
            job_store: Arc::new(job_store),
            materializer,
            executor,
            locks: Arc::new(AsyncMutex::new(BTreeMap::new())),
            config,
        }
    }

    pub fn registry(&self) -> &WorkspaceRegistry {
        &self.registry
    }

    pub fn jobs_cached(&self) -> usize {
        self.job_store.len()
    }

    pub fn is_read_only(&self) -> bool {
        self.config.read_only
    }

    pub fn read_only_mode(&self) -> bool {
        self.config.read_only
    }

    pub async fn submit_job(&self, request: CurioJobRequest) -> Result<JobSubmissionResponse> {
        let workspace = self.registry.resolve(&request.workspace_id)?.clone();
        self.validate_request(&request, &workspace)?;
        let idempotency_key = request.effective_idempotency_key();
        if let Some(existing) = self.job_store.find_by_idempotency_key(&idempotency_key) {
            return Ok(JobSubmissionResponse {
                accepted: false,
                duplicate: true,
                job: existing,
            });
        }

        let job_id = Uuid::new_v4().to_string();
        let queued_at = now();
        let status = CurioJobStatus {
            job_id: job_id.clone(),
            idempotency_key,
            request: request.clone(),
            workspace: workspace.clone(),
            state: JobState::Queued,
            queued_at,
            started_at: None,
            completed_at: None,
            result: None,
            error: None,
            audit: vec![self.audit_event(
                &job_id,
                &workspace.workspace_id,
                &request,
                "queued",
                Some("job queued for execution".to_string()),
                serde_json::json!({
                    "operation": request.operation,
                }),
            )],
        };
        self.job_store.upsert(status.clone())?;
        self.append_audit_event(&status.audit[0])?;

        let runtime = self.clone();
        tokio::spawn(async move {
            if let Err(err) = runtime.execute_job(job_id.clone()).await {
                if let Some(status) = runtime.get_job(&job_id) {
                    let completed_at = now();
                    let _ = runtime.job_store.update_state(
                        &job_id,
                        JobState::Failed,
                        Some(err.to_string()),
                        status.result.clone(),
                        status.started_at.clone(),
                        Some(completed_at),
                    );
                }
            }
        });

        Ok(JobSubmissionResponse {
            accepted: true,
            duplicate: false,
            job: status,
        })
    }

    pub fn get_job(&self, job_id: &str) -> Option<CurioJobStatus> {
        self.job_store.get(job_id)
    }

    pub fn readiness(&self) -> ReadinessResponse {
        let sentinel = self
            .config
            .audit_log_path
            .parent()
            .unwrap_or(std::path::Path::new("/tmp"))
            .join(".curio-readyz-probe");
        let storage_ok = fs::write(&sentinel, b"ok")
            .and_then(|_| fs::remove_file(&sentinel))
            .is_ok();
        ReadinessResponse {
            ok: storage_ok,
            registry_records: self.registry.len(),
            jobs_cached: self.job_store.len(),
            read_only: self.config.read_only,
        }
    }

    async fn execute_job(&self, job_id: String) -> Result<()> {
        let current = self
            .job_store
            .get(&job_id)
            .with_context(|| format!("Job not found: {}", job_id))?;
        let request = current.request.clone();
        let workspace = current.workspace.clone();

        if workspace.status == WorkspaceStatus::Disabled {
            let _ = self.job_store.update_state(
                &job_id,
                JobState::Rejected,
                Some("workspace disabled".to_string()),
                None,
                Some(now()),
                Some(now()),
            );
            return Ok(());
        }

        let mut guard: Option<OwnedMutexGuard<()>> = None;
        if request.is_mutating() {
            let lock = self.lock_for(&workspace.workspace_id).await;
            guard = Some(lock.clone().lock_owned().await);
        }

        let _guard = guard;
        let started_at = now();
        let _ = self.job_store.update_state(
            &job_id,
            JobState::Running,
            None,
            None,
            Some(started_at.clone()),
            None,
        );

        let plan = {
            let materializer = self.materializer.clone();
            let request = request.clone();
            let workspace = workspace.clone();
            let job_id = job_id.clone();
            tokio::task::spawn_blocking(move || {
                materializer.materialize(&workspace, &request, &job_id)
            })
            .await
            .context("Materialization task failed")??
        };
        let provider = self.provider_for(&request, &workspace)?;
        let provider_for_phases = provider.clone();
        let request_for_phases = request.clone();
        let workspace_for_phases = workspace.clone();
        let plan_for_phases = plan.clone();
        let phase_outputs = tokio::task::spawn_blocking(move || {
            Ok::<_, anyhow::Error>(vec![
                provider_for_phases.prepare(
                    &request_for_phases,
                    &workspace_for_phases,
                    &plan_for_phases,
                )?,
                provider_for_phases.analyze(
                    &request_for_phases,
                    &workspace_for_phases,
                    &plan_for_phases,
                )?,
                provider_for_phases.route(
                    &request_for_phases,
                    &workspace_for_phases,
                    &plan_for_phases,
                )?,
                provider_for_phases.propose_changes(
                    &request_for_phases,
                    &workspace_for_phases,
                    &plan_for_phases,
                )?,
                provider_for_phases.summarize(
                    &request_for_phases,
                    &workspace_for_phases,
                    &plan_for_phases,
                )?,
            ])
        })
        .await
        .context("Provider phase task failed")??;
        let executor = self.executor.clone();
        let request_for_exec = request.clone();
        let workspace_for_exec = workspace.clone();
        let plan_for_exec = plan.clone();
        let provider_for_exec = provider.name().to_string();
        let provider_for_exec_task = provider.clone();
        let execution = tokio::task::spawn_blocking(move || {
            executor.execute(
                &request_for_exec,
                &workspace_for_exec,
                &plan_for_exec,
                provider_for_exec_task.as_ref(),
            )
        })
        .await
        .context("Execution task failed")??;

        let mut completed_audit = vec![
            self.audit_event(
                &job_id,
                &workspace.workspace_id,
                &request,
                "provider_phases",
                Some("provider phases completed".to_string()),
                serde_json::json!({ "phases": phase_outputs }),
            ),
            self.audit_event(
                &job_id,
                &workspace.workspace_id,
                &request,
                "materialized",
                Some("workspace materialized".to_string()),
                serde_json::json!({
                    "mirror_dir": &plan.mirror_dir,
                    "worktree_dir": &plan.worktree_dir,
                    "mutating": plan.mutating,
                }),
            ),
            self.audit_event(
                &job_id,
                &workspace.workspace_id,
                &request,
                "executed",
                Some("curio execution completed".to_string()),
                serde_json::json!({
                    "provider": provider_for_exec,
                    "command_line": execution.command_line,
                    "pushed": execution.pushed,
                }),
            ),
        ];
        for event in &completed_audit {
            self.append_audit_event(event)?;
        }

        let updated = self.job_store.update_state(
            &job_id,
            JobState::Succeeded,
            None,
            Some(execution),
            Some(started_at),
            Some(now()),
        )?;
        let mut final_status = updated.clone();
        final_status.audit.extend(completed_audit.drain(..));
        self.job_store.upsert(final_status)?;
        Ok(())
    }

    fn validate_request(
        &self,
        request: &CurioJobRequest,
        workspace: &WorkspaceRegistryRecord,
    ) -> Result<()> {
        if self.config.read_only && request.is_mutating() {
            bail!("service is running in read-only mode");
        }
        if workspace.status == WorkspaceStatus::ReadOnly && request.is_mutating() {
            bail!("workspace is read-only");
        }
        if !workspace.allowed_job_types.is_empty()
            && !workspace.allowed_job_types.iter().any(|job_type| {
                job_type.eq_ignore_ascii_case(request.job_type.as_str())
                    || job_type.eq_ignore_ascii_case(&request.operation)
            })
        {
            bail!(
                "job type '{}' is not allowed for workspace {}",
                request.job_type.as_str(),
                workspace.workspace_id
            );
        }
        if request.write_mode == WriteMode::DirectPush
            && workspace.write_policy == WriteMode::ReadOnly
        {
            bail!("workspace policy is read-only");
        }
        Ok(())
    }

    fn provider_for(
        &self,
        request: &CurioJobRequest,
        workspace: &WorkspaceRegistryRecord,
    ) -> Result<Arc<dyn ProviderAdapter>> {
        let backend = request
            .provider
            .as_deref()
            .filter(|value| !value.is_empty())
            .or_else(|| {
                workspace
                    .provider_defaults
                    .get("backend")
                    .and_then(|value| value.as_str())
            })
            .unwrap_or(self.config.provider_backend.as_str());
        build_provider_adapter(backend, Some(workspace))
    }

    async fn lock_for(&self, workspace_id: &str) -> Arc<AsyncMutex<()>> {
        let mut guard = self.locks.lock().await;
        guard
            .entry(workspace_id.to_string())
            .or_insert_with(|| Arc::new(AsyncMutex::new(())))
            .clone()
    }

    fn audit_event(
        &self,
        job_id: &str,
        workspace_id: &str,
        request: &CurioJobRequest,
        event: &str,
        message: Option<String>,
        metadata: serde_json::Value,
    ) -> AuditEvent {
        AuditEvent {
            at: now(),
            event: event.to_string(),
            job_id: job_id.to_string(),
            workspace_id: workspace_id.to_string(),
            actor_id: request.actor.id.clone(),
            trigger_kind: request.trigger.kind.clone(),
            message,
            metadata,
        }
    }

    fn append_audit_event(&self, event: &AuditEvent) -> Result<()> {
        if let Some(parent) = self.config.audit_log_path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create audit log directory: {}", parent.display())
            })?;
        }
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.config.audit_log_path)
            .with_context(|| {
                format!(
                    "Failed to open audit log: {}",
                    self.config.audit_log_path.display()
                )
            })?;
        use std::io::Write;
        writeln!(file, "{}", serde_json::to_string(event)?).with_context(|| {
            format!(
                "Failed to write audit log: {}",
                self.config.audit_log_path.display()
            )
        })
    }
}

fn sanitize_segment(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == '.' {
                ch
            } else {
                '-'
            }
        })
        .collect()
}

fn now() -> String {
    Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}
