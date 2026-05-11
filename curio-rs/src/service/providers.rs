use crate::llm;
use crate::service::types::{
    CurioJobRequest, GitMaterializationPlan, ProviderAdapter, ProviderStepOutput,
    WorkspaceRegistryRecord,
};
use anyhow::{Context, Result, bail};
use std::env;

#[derive(Debug, Clone)]
pub struct OpenAIProviderAdapter {
    api_key: String,
    model: String,
    max_tokens: u32,
}

impl OpenAIProviderAdapter {
    pub fn from_env(workspace: Option<&WorkspaceRegistryRecord>) -> Result<Self> {
        let api_key = env::var("OPENAI_API_KEY")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .context("OPENAI_API_KEY is required for the OpenAI provider backend")?;
        let workspace_model = workspace
            .and_then(|record| record.provider_defaults.get("model"))
            .and_then(|value| value.as_str())
            .filter(|value| !value.trim().is_empty())
            .map(|value| value.to_string());
        let model = env::var("CURIO_SERVICE_PROVIDER_MODEL")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .or(workspace_model)
            .unwrap_or_else(|| "gpt-4.1-mini".to_string());
        let max_tokens = env::var("CURIO_SERVICE_PROVIDER_MAX_TOKENS")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(1024);

        Ok(Self {
            api_key,
            model,
            max_tokens,
        })
    }

    fn phase(
        &self,
        phase: &str,
        request: &CurioJobRequest,
        workspace: &WorkspaceRegistryRecord,
        plan: &GitMaterializationPlan,
    ) -> Result<ProviderStepOutput> {
        let system = format!(
            "You are Curio's service-job provider backend. Produce compact JSON only. \
Phase: {phase}. Return an object with keys phase, provider, summary, metadata."
        );
        let user = serde_json::json!({
            "job_id": &plan.commit_metadata.job_id,
            "workspace": {
                "workspace_id": &workspace.workspace_id,
                "display_name": &workspace.display_name,
                "repo_url": &workspace.repo_url,
                "default_branch": &workspace.default_branch,
                "kb_root": &workspace.kb_root,
                "write_policy": &workspace.write_policy,
                "status": &workspace.status,
            },
            "request": {
                "job_type": request.job_type.as_str(),
                "operation": &request.operation,
                "write_mode": &request.write_mode,
                "correlation_id": &request.correlation_id,
                "trigger": &request.trigger,
                "actor": &request.actor,
                "inputs": &request.inputs,
            },
            "materialization": {
                "mirror_dir": &plan.mirror_dir,
                "worktree_dir": &plan.worktree_dir,
                "checkout_ref": &plan.checkout_ref,
                "target_branch": &plan.target_branch,
                "push_refspec": &plan.push_refspec,
                "mutating": plan.mutating,
            },
            "phase": phase,
        })
        .to_string();

        let mut output = self.call_model(&system, &user)?;
        if output.phase.is_empty() || output.phase == "unknown" {
            output.phase = phase.to_string();
        }
        if output.provider.is_empty() {
            output.provider = self.name().to_string();
        }
        if output.metadata.is_null() {
            output.metadata = serde_json::json!({ "model": self.model, "phase": phase });
        }
        Ok(output)
    }

    fn call_model(&self, system: &str, user: &str) -> Result<ProviderStepOutput> {
        let rt =
            tokio::runtime::Runtime::new().context("Failed to create runtime for provider call")?;
        let raw = rt.block_on(llm::call_with_max_tokens(
            &self.api_key,
            &self.model,
            system,
            user,
            self.max_tokens,
        ))?;
        let raw_fallback = raw.clone();
        let value = llm::extract_json(&raw).unwrap_or_else(|_| {
            serde_json::json!({
                "phase": "unknown",
                "provider": self.name(),
                "summary": raw_fallback,
                "metadata": {
                    "model": self.model,
                    "raw": raw_fallback,
                }
            })
        });
        Ok(ProviderStepOutput {
            phase: value
                .get("phase")
                .and_then(|value| value.as_str())
                .unwrap_or("unknown")
                .to_string(),
            provider: value
                .get("provider")
                .and_then(|value| value.as_str())
                .unwrap_or(self.name())
                .to_string(),
            summary: value
                .get("summary")
                .and_then(|value| value.as_str())
                .unwrap_or(raw.as_str())
                .to_string(),
            metadata: value
                .get("metadata")
                .cloned()
                .unwrap_or_else(|| serde_json::json!({ "model": self.model })),
        })
    }
}

impl ProviderAdapter for OpenAIProviderAdapter {
    fn name(&self) -> &str {
        "openai"
    }

    fn prepare(
        &self,
        request: &CurioJobRequest,
        workspace: &WorkspaceRegistryRecord,
        plan: &GitMaterializationPlan,
    ) -> Result<ProviderStepOutput> {
        self.phase("prepare", request, workspace, plan)
    }

    fn analyze(
        &self,
        request: &CurioJobRequest,
        workspace: &WorkspaceRegistryRecord,
        plan: &GitMaterializationPlan,
    ) -> Result<ProviderStepOutput> {
        self.phase("analyze", request, workspace, plan)
    }

    fn route(
        &self,
        request: &CurioJobRequest,
        workspace: &WorkspaceRegistryRecord,
        plan: &GitMaterializationPlan,
    ) -> Result<ProviderStepOutput> {
        self.phase("route", request, workspace, plan)
    }

    fn propose_changes(
        &self,
        request: &CurioJobRequest,
        workspace: &WorkspaceRegistryRecord,
        plan: &GitMaterializationPlan,
    ) -> Result<ProviderStepOutput> {
        self.phase("propose_changes", request, workspace, plan)
    }

    fn summarize(
        &self,
        request: &CurioJobRequest,
        workspace: &WorkspaceRegistryRecord,
        plan: &GitMaterializationPlan,
    ) -> Result<ProviderStepOutput> {
        self.phase("summarize", request, workspace, plan)
    }
}

#[derive(Debug, Clone)]
pub struct GeminiProviderAdapter {
    /// Vertex AI project ID — required when api_key is None.
    project_id: Option<String>,
    location: String,
    model: String,
    max_tokens: u32,
    /// Static OAuth bearer token (overrides metadata-server lookup).
    access_token: Option<String>,
    /// Gemini Developer API key (https://aistudio.google.com/apikey).
    /// When set, uses generativelanguage.googleapis.com instead of Vertex AI.
    /// Simpler for local dev — no GCP project or OAuth needed.
    api_key: Option<String>,
}

impl GeminiProviderAdapter {
    pub fn from_env(workspace: Option<&WorkspaceRegistryRecord>) -> Result<Self> {
        // API key takes priority — enables local dev without a GCP project.
        let api_key = env::var("CURIO_GEMINI_API_KEY")
            .ok()
            .filter(|value| !value.trim().is_empty());

        let project_id = env::var("CURIO_VERTEX_PROJECT_ID")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .or_else(|| {
                env::var("GOOGLE_CLOUD_PROJECT")
                    .ok()
                    .filter(|value| !value.trim().is_empty())
            })
            .or_else(|| {
                env::var("GCP_PROJECT")
                    .ok()
                    .filter(|value| !value.trim().is_empty())
            });

        if api_key.is_none() && project_id.is_none() {
            anyhow::bail!(
                "Gemini backend requires either CURIO_GEMINI_API_KEY (local dev) \
                 or CURIO_VERTEX_PROJECT_ID (production Vertex AI)"
            );
        }
        let location = env::var("CURIO_VERTEX_LOCATION")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "us-central1".to_string());
        let workspace_model = workspace
            .and_then(|record| record.provider_defaults.get("model"))
            .and_then(|value| value.as_str())
            .filter(|value| !value.trim().is_empty())
            .map(|value| value.to_string());
        let model = env::var("CURIO_SERVICE_PROVIDER_MODEL")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .or(workspace_model)
            .unwrap_or_else(|| "gemini-2.5-pro".to_string());
        let max_tokens = env::var("CURIO_SERVICE_PROVIDER_MAX_TOKENS")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(1024);
        let access_token = env::var("CURIO_GEMINI_ACCESS_TOKEN")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .or_else(|| {
                env::var("GOOGLE_OAUTH_ACCESS_TOKEN")
                    .ok()
                    .filter(|value| !value.trim().is_empty())
            });

        Ok(Self {
            project_id,
            location,
            model,
            max_tokens,
            access_token,
            api_key,
        })
    }

    fn phase(
        &self,
        phase: &str,
        request: &CurioJobRequest,
        workspace: &WorkspaceRegistryRecord,
        plan: &GitMaterializationPlan,
    ) -> Result<ProviderStepOutput> {
        let system = format!(
            "You are Curio's service-job provider backend. Produce compact JSON only. \
Phase: {phase}. Return an object with keys phase, provider, summary, metadata."
        );
        let user = serde_json::json!({
            "job_id": &plan.commit_metadata.job_id,
            "workspace": {
                "workspace_id": &workspace.workspace_id,
                "display_name": &workspace.display_name,
                "repo_url": &workspace.repo_url,
                "default_branch": &workspace.default_branch,
                "kb_root": &workspace.kb_root,
                "write_policy": &workspace.write_policy,
                "status": &workspace.status,
            },
            "request": {
                "job_type": request.job_type.as_str(),
                "operation": &request.operation,
                "write_mode": &request.write_mode,
                "correlation_id": &request.correlation_id,
                "trigger": &request.trigger,
                "actor": &request.actor,
                "inputs": &request.inputs,
            },
            "materialization": {
                "mirror_dir": &plan.mirror_dir,
                "worktree_dir": &plan.worktree_dir,
                "checkout_ref": &plan.checkout_ref,
                "target_branch": &plan.target_branch,
                "push_refspec": &plan.push_refspec,
                "mutating": plan.mutating,
            },
            "phase": phase,
        })
        .to_string();

        let mut output = self.call_model(&system, &user)?;
        if output.phase.is_empty() || output.phase == "unknown" {
            output.phase = phase.to_string();
        }
        if output.provider.is_empty() {
            output.provider = self.name().to_string();
        }
        if output.metadata.is_null() {
            output.metadata = serde_json::json!({ "model": self.model, "phase": phase });
        }
        Ok(output)
    }

    fn call_model(&self, system: &str, user: &str) -> Result<ProviderStepOutput> {
        let rt =
            tokio::runtime::Runtime::new().context("Failed to create runtime for provider call")?;
        let raw = rt.block_on(self.generate_content(system, user))?;
        let raw_fallback = raw.clone();
        let value = llm::extract_json(&raw).unwrap_or_else(|_| {
            serde_json::json!({
                "phase": "unknown",
                "provider": self.name(),
                "summary": raw_fallback,
                "metadata": {
                    "model": self.model,
                    "raw": raw_fallback,
                }
            })
        });
        Ok(ProviderStepOutput {
            phase: value
                .get("phase")
                .and_then(|value| value.as_str())
                .unwrap_or("unknown")
                .to_string(),
            provider: value
                .get("provider")
                .and_then(|value| value.as_str())
                .unwrap_or(self.name())
                .to_string(),
            summary: value
                .get("summary")
                .and_then(|value| value.as_str())
                .unwrap_or(raw.as_str())
                .to_string(),
            metadata: value
                .get("metadata")
                .cloned()
                .unwrap_or_else(|| serde_json::json!({ "model": self.model })),
        })
    }

    async fn generate_content(&self, system: &str, user: &str) -> Result<String> {
        let body = serde_json::json!({
            "systemInstruction": {
                "parts": [{ "text": system }]
            },
            "contents": [
                { "role": "user", "parts": [{ "text": user }] }
            ],
            "generationConfig": {
                "temperature": 0.2,
                "maxOutputTokens": self.max_tokens,
                "responseMimeType": "application/json"
            }
        });

        let client = reqwest::Client::new();
        let (endpoint, request) = if let Some(key) = &self.api_key {
            // Gemini Developer API — no GCP project or OAuth needed.
            let url = format!(
                "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
                self.model, key
            );
            let req = client.post(url).json(&body);
            (String::from("generativelanguage.googleapis.com"), req)
        } else {
            // Vertex AI — requires a project ID and an OAuth bearer token.
            let project_id = self
                .project_id
                .as_deref()
                .context("Vertex AI requires CURIO_VERTEX_PROJECT_ID")?;
            let url = format!(
                "https://aiplatform.googleapis.com/v1/projects/{}/locations/{}/publishers/google/models/{}:generateContent",
                project_id, self.location, self.model
            );
            let token = self.access_token().await?;
            let req = client.post(url).bearer_auth(token).json(&body);
            (String::from("aiplatform.googleapis.com"), req)
        };

        let response = request
            .send()
            .await
            .with_context(|| format!("Failed to reach Gemini API ({})", endpoint))?;
        let status = response.status();
        let text = response
            .text()
            .await
            .context("Failed to read Gemini response")?;
        if !status.is_success() {
            bail!("Gemini API error {} ({}): {}", status, endpoint, text);
        }

        let raw = extract_gemini_text(&text).unwrap_or_else(|| text.clone());
        Ok(raw)
    }

    async fn access_token(&self) -> Result<String> {
        if let Some(token) = &self.access_token {
            return Ok(token.clone());
        }

        let client = reqwest::Client::new();
        let response = client
            .get("http://metadata.google.internal/computeMetadata/v1/instance/service-accounts/default/token")
            .header("Metadata-Flavor", "Google")
            .send()
            .await
            .context("Failed to reach Google metadata server for Vertex AI credentials")?;
        let status = response.status();
        let text = response
            .text()
            .await
            .context("Failed to read metadata token response")?;
        if !status.is_success() {
            bail!("Failed to fetch Google access token {}: {}", status, text);
        }
        let json: serde_json::Value = serde_json::from_str(&text)
            .with_context(|| format!("Failed to parse metadata token response: {}", text))?;
        json.get("access_token")
            .and_then(|value| value.as_str())
            .map(|value| value.to_string())
            .ok_or_else(|| anyhow::anyhow!("metadata server response missing access_token"))
    }
}

impl ProviderAdapter for GeminiProviderAdapter {
    fn name(&self) -> &str {
        "gemini"
    }

    fn prepare(
        &self,
        request: &CurioJobRequest,
        workspace: &WorkspaceRegistryRecord,
        plan: &GitMaterializationPlan,
    ) -> Result<ProviderStepOutput> {
        self.phase("prepare", request, workspace, plan)
    }

    fn analyze(
        &self,
        request: &CurioJobRequest,
        workspace: &WorkspaceRegistryRecord,
        plan: &GitMaterializationPlan,
    ) -> Result<ProviderStepOutput> {
        self.phase("analyze", request, workspace, plan)
    }

    fn route(
        &self,
        request: &CurioJobRequest,
        workspace: &WorkspaceRegistryRecord,
        plan: &GitMaterializationPlan,
    ) -> Result<ProviderStepOutput> {
        self.phase("route", request, workspace, plan)
    }

    fn propose_changes(
        &self,
        request: &CurioJobRequest,
        workspace: &WorkspaceRegistryRecord,
        plan: &GitMaterializationPlan,
    ) -> Result<ProviderStepOutput> {
        self.phase("propose_changes", request, workspace, plan)
    }

    fn summarize(
        &self,
        request: &CurioJobRequest,
        workspace: &WorkspaceRegistryRecord,
        plan: &GitMaterializationPlan,
    ) -> Result<ProviderStepOutput> {
        self.phase("summarize", request, workspace, plan)
    }
}

#[derive(Debug, Clone, Default)]
pub struct PassthroughProviderAdapter {
    name: String,
}

impl ProviderAdapter for PassthroughProviderAdapter {
    fn name(&self) -> &str {
        if self.name.is_empty() {
            "passthrough"
        } else {
            self.name.as_str()
        }
    }

    fn prepare(
        &self,
        request: &CurioJobRequest,
        workspace: &WorkspaceRegistryRecord,
        plan: &GitMaterializationPlan,
    ) -> Result<ProviderStepOutput> {
        Ok(self.step("prepare", request, workspace, plan))
    }

    fn analyze(
        &self,
        request: &CurioJobRequest,
        workspace: &WorkspaceRegistryRecord,
        plan: &GitMaterializationPlan,
    ) -> Result<ProviderStepOutput> {
        Ok(self.step("analyze", request, workspace, plan))
    }

    fn route(
        &self,
        request: &CurioJobRequest,
        workspace: &WorkspaceRegistryRecord,
        plan: &GitMaterializationPlan,
    ) -> Result<ProviderStepOutput> {
        Ok(self.step("route", request, workspace, plan))
    }

    fn propose_changes(
        &self,
        request: &CurioJobRequest,
        workspace: &WorkspaceRegistryRecord,
        plan: &GitMaterializationPlan,
    ) -> Result<ProviderStepOutput> {
        Ok(self.step("propose_changes", request, workspace, plan))
    }

    fn summarize(
        &self,
        request: &CurioJobRequest,
        workspace: &WorkspaceRegistryRecord,
        plan: &GitMaterializationPlan,
    ) -> Result<ProviderStepOutput> {
        Ok(self.step("summarize", request, workspace, plan))
    }
}

impl PassthroughProviderAdapter {
    fn step(
        &self,
        phase: &str,
        request: &CurioJobRequest,
        workspace: &WorkspaceRegistryRecord,
        plan: &GitMaterializationPlan,
    ) -> ProviderStepOutput {
        ProviderStepOutput {
            phase: phase.to_string(),
            provider: self.name().to_string(),
            summary: format!(
                "{} phase for job_type={} operation={} workspace={} checkout={}",
                phase,
                request.job_type.as_str(),
                request.operation,
                workspace.workspace_id,
                plan.checkout_ref
            ),
            metadata: serde_json::json!({
                "workspace_id": workspace.workspace_id,
                "job_type": request.job_type.as_str(),
                "operation": request.operation,
                "write_mode": request.write_mode,
                "checkout_ref": plan.checkout_ref,
                "target_branch": plan.target_branch,
            }),
        }
    }
}

pub fn provider_backend_from_env() -> String {
    env::var("CURIO_SERVICE_PROVIDER_BACKEND")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "gemini".to_string())
}

pub fn build_provider_adapter(
    backend: &str,
    workspace: Option<&WorkspaceRegistryRecord>,
) -> Result<std::sync::Arc<dyn ProviderAdapter>> {
    match backend {
        "gemini" | "vertex" | "google" => Ok(std::sync::Arc::new(GeminiProviderAdapter::from_env(
            workspace,
        )?)),
        "openai" => Ok(std::sync::Arc::new(OpenAIProviderAdapter::from_env(
            workspace,
        )?)),
        "passthrough" => Ok(std::sync::Arc::new(PassthroughProviderAdapter::default())),
        other => bail!("unsupported provider backend: {}", other),
    }
}

fn extract_gemini_text(text: &str) -> Option<String> {
    let json: serde_json::Value = serde_json::from_str(text).ok()?;
    let candidates = json.get("candidates")?.as_array()?;
    let first = candidates.first()?;
    let content = first.get("content")?;
    let parts = content.get("parts")?.as_array()?;
    let mut combined = String::new();
    for part in parts {
        if let Some(part_text) = part.get("text").and_then(|value| value.as_str()) {
            combined.push_str(part_text);
        }
    }
    if combined.trim().is_empty() {
        None
    } else {
        Some(combined)
    }
}
