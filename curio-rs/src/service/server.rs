use crate::service::auth::{
    AuthMode, AuthState, PrincipalKind, VerifiedPrincipal, verify_bearer, verify_iap_jwt,
    verify_pubsub_oidc,
};
use crate::service::runtime::{ServiceConfig, ServiceRuntime};
use crate::service::types::{
    CurioJobRequest, HealthResponse, ReadinessResponse, WorkspaceRegistryRecord, WorkspaceStatus,
};
use anyhow::{Context, Result};
use axum::extract::{Path, Request, State};
use axum::http::StatusCode;
use axum::middleware::{self, Next};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Extension, Json, Router};
use base64::Engine;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::trace::TraceLayer;
use uuid::Uuid;

// ── App state ──────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct AppState {
    pub runtime: Arc<ServiceRuntime>,
    pub auth: Arc<AuthState>,
}

// ── Request extensions ─────────────────────────────────────────────────────

#[derive(Clone)]
pub struct CorrelationId(pub String);

// ── Error helpers ──────────────────────────────────────────────────────────

fn error_response_coded(code: &str, cid: &str) -> serde_json::Value {
    serde_json::json!({
        "ok": false,
        "error_code": code,
        "correlation_id": cid,
    })
}

// ── Input validation ───────────────────────────────────────────────────────

const MAX_ARGS: usize = 20;
const MAX_ARG_LEN: usize = 512;
const MAX_WORKSPACE_ID_LEN: usize = 64;

fn validate_job_inputs(
    request: &CurioJobRequest,
) -> Result<(), (StatusCode, Json<serde_json::Value>)> {
    let cid = "validation";
    if request.workspace_id.is_empty() || request.workspace_id.len() > MAX_WORKSPACE_ID_LEN {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(error_response_coded("workspace_id_invalid", cid)),
        ));
    }
    if request.workspace_id.contains('\0') {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(error_response_coded("workspace_id_invalid", cid)),
        ));
    }
    if let Some(args) = request.inputs.get("args").and_then(|v| v.as_array()) {
        if args.len() > MAX_ARGS {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(error_response_coded("too_many_args", cid)),
            ));
        }
        for arg in args {
            let s = arg.as_str().unwrap_or("");
            if s.len() > MAX_ARG_LEN {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(error_response_coded("arg_too_long", cid)),
                ));
            }
            if s.contains('\0') {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(error_response_coded("arg_invalid", cid)),
                ));
            }
        }
    }
    Ok(())
}

// ── Auth middleware ────────────────────────────────────────────────────────

async fn human_auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> axum::response::Response {
    let cid = Uuid::new_v4().to_string();

    let result: Result<VerifiedPrincipal> = match &state.auth.config.mode {
        AuthMode::None => Ok(VerifiedPrincipal {
            email: "anonymous@local".to_string(),
            kind: PrincipalKind::System,
        }),
        AuthMode::Iap => {
            let token = request
                .headers()
                .get("x-goog-iap-jwt-assertion")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("")
                .to_string();
            let audience = state
                .auth
                .config
                .iap_audience
                .as_deref()
                .unwrap_or("")
                .to_string();
            verify_iap_jwt(&token, &audience, &state.auth.cache).await
        }
        AuthMode::Bearer => {
            let token = request
                .headers()
                .get("authorization")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.strip_prefix("Bearer "))
                .unwrap_or("")
                .to_string();
            let expected = state
                .auth
                .config
                .bearer_token
                .as_deref()
                .unwrap_or("")
                .to_string();
            verify_bearer(&token, &expected)
        }
    };

    match result {
        Ok(principal) => {
            request.extensions_mut().insert(CorrelationId(cid));
            request.extensions_mut().insert(principal);
            next.run(request).await
        }
        Err(err) => {
            tracing::warn!(cid = %cid, err = %err, "auth rejected");
            (
                StatusCode::UNAUTHORIZED,
                Json(error_response_coded("unauthorized", &cid)),
            )
                .into_response()
        }
    }
}

async fn pubsub_auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> axum::response::Response {
    let cid = Uuid::new_v4().to_string();

    let sa_email = state
        .auth
        .config
        .pubsub_sa_email
        .as_deref()
        .unwrap_or("")
        .to_string();
    let audience = state
        .auth
        .config
        .service_url
        .as_deref()
        .unwrap_or("")
        .to_string();

    // In None/Bearer modes with no pubsub SA configured: allow through.
    if (sa_email.is_empty() || audience.is_empty()) && state.auth.config.mode != AuthMode::Iap {
        request.extensions_mut().insert(CorrelationId(cid));
        request.extensions_mut().insert(VerifiedPrincipal {
            email: "pubsub@local".to_string(),
            kind: PrincipalKind::ServiceAccount,
        });
        return next.run(request).await;
    }

    if sa_email.is_empty() || audience.is_empty() {
        tracing::error!(cid = %cid, "pubsub auth misconfigured: missing sa_email or service_url");
        return (
            StatusCode::UNAUTHORIZED,
            Json(error_response_coded("unauthorized", &cid)),
        )
            .into_response();
    }

    let token = request
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .unwrap_or("")
        .to_string();

    match verify_pubsub_oidc(&token, &sa_email, &audience, &state.auth.cache).await {
        Ok(principal) => {
            tracing::info!(email = %principal.email, cid = %cid, "pubsub push authenticated");
            request.extensions_mut().insert(CorrelationId(cid));
            request.extensions_mut().insert(principal);
            next.run(request).await
        }
        Err(err) => {
            tracing::warn!(cid = %cid, err = %err, "pubsub auth rejected");
            (
                StatusCode::UNAUTHORIZED,
                Json(error_response_coded("unauthorized", &cid)),
            )
                .into_response()
        }
    }
}

// ── Router ─────────────────────────────────────────────────────────────────

pub fn router(runtime: ServiceRuntime, auth: AuthState) -> Router {
    let state = AppState {
        runtime: Arc::new(runtime),
        auth: Arc::new(auth),
    };

    let health_routes = Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz));

    let pubsub_routes = Router::new()
        .route("/v1/pubsub/jobs", post(submit_pubsub_job))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            pubsub_auth_middleware,
        ));

    let protected_routes = Router::new()
        .route("/v1/jobs", post(submit_job))
        .route("/v1/jobs/:job_id", get(get_job))
        .route("/v1/workspaces", get(list_workspaces))
        .route("/v1/workspaces/:workspace_id", get(get_workspace))
        .route(
            "/v1/workspaces/:workspace_id/healthz",
            get(workspace_healthz),
        )
        .layer(middleware::from_fn_with_state(
            state.clone(),
            human_auth_middleware,
        ));

    Router::new()
        .merge(health_routes)
        .merge(pubsub_routes)
        .merge(protected_routes)
        .layer(RequestBodyLimitLayer::new(1024 * 1024))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

pub async fn serve(runtime: ServiceRuntime, bind_addr: SocketAddr) -> Result<()> {
    let auth = AuthState::from_env();
    let app = router(runtime, auth);
    let listener = tokio::net::TcpListener::bind(bind_addr)
        .await
        .with_context(|| format!("Failed to bind {}", bind_addr))?;
    axum::serve(listener, app)
        .await
        .context("HTTP server exited unexpectedly")
}

// ── Handlers ───────────────────────────────────────────────────────────────

async fn healthz() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(HealthResponse {
            ok: true,
            service: "curio-control-plane".to_string(),
        }),
    )
}

async fn readyz(State(state): State<AppState>) -> impl IntoResponse {
    let readiness: ReadinessResponse = state.runtime.readiness();
    let status = if readiness.ok {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    (status, Json(readiness))
}

async fn submit_job(
    State(state): State<AppState>,
    Extension(cid): Extension<CorrelationId>,
    Json(request): Json<CurioJobRequest>,
) -> impl IntoResponse {
    if let Err(e) = validate_job_inputs(&request) {
        return e.into_response();
    }
    submit_job_impl(state, request, cid.0).await
}

async fn submit_pubsub_job(
    State(state): State<AppState>,
    Extension(cid): Extension<CorrelationId>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let request = if payload.get("message").is_some() {
        decode_pubsub_request(payload)
    } else {
        serde_json::from_value::<CurioJobRequest>(payload)
            .context("failed to parse CurioJobRequest")
    };

    match request {
        Ok(req) => {
            if let Err(e) = validate_job_inputs(&req) {
                return e.into_response();
            }
            submit_job_impl(state, req, cid.0).await
        }
        Err(err) => {
            tracing::warn!(cid = %cid.0, err = %err, "pubsub payload parse failed");
            (
                StatusCode::BAD_REQUEST,
                Json(error_response_coded("bad_request", &cid.0)),
            )
                .into_response()
        }
    }
}

async fn get_job(
    State(state): State<AppState>,
    Extension(cid): Extension<CorrelationId>,
    Path(job_id): Path<String>,
) -> impl IntoResponse {
    match state.runtime.get_job(&job_id) {
        Some(job) => (StatusCode::OK, Json(job)).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(error_response_coded("job_not_found", &cid.0)),
        )
            .into_response(),
    }
}

#[derive(serde::Serialize)]
struct WorkspaceListResponse {
    workspaces: Vec<WorkspaceRegistryRecord>,
}

#[derive(serde::Serialize)]
struct WorkspaceHealthResponse {
    ok: bool,
    workspace_id: String,
    display_name: String,
    status: WorkspaceStatus,
    repo_url: String,
    kb_root: String,
    write_policy: crate::service::types::WriteMode,
    issues: Vec<String>,
}

async fn list_workspaces(State(state): State<AppState>) -> impl IntoResponse {
    let workspaces: Vec<WorkspaceRegistryRecord> =
        state.runtime.registry().iter().cloned().collect();
    (StatusCode::OK, Json(WorkspaceListResponse { workspaces }))
}

async fn get_workspace(
    State(state): State<AppState>,
    Extension(cid): Extension<CorrelationId>,
    Path(workspace_id): Path<String>,
) -> impl IntoResponse {
    match state.runtime.registry().resolve(&workspace_id) {
        Ok(record) => (StatusCode::OK, Json(record.clone())).into_response(),
        Err(err) => {
            // Distinguish disabled vs missing for clearer client behavior.
            let downcast = err.downcast_ref::<crate::service::WorkspaceRegistryError>();
            let (status, code) = match downcast {
                Some(crate::service::WorkspaceRegistryError::Disabled(_)) => {
                    (StatusCode::GONE, "workspace_disabled")
                }
                _ => (StatusCode::NOT_FOUND, "workspace_not_found"),
            };
            (status, Json(error_response_coded(code, &cid.0))).into_response()
        }
    }
}

async fn workspace_healthz(
    State(state): State<AppState>,
    Extension(cid): Extension<CorrelationId>,
    Path(workspace_id): Path<String>,
) -> impl IntoResponse {
    // Read the raw record (including disabled), then assemble a health summary.
    let registry = state.runtime.registry();
    let record = match registry.iter().find(|r| r.workspace_id == workspace_id) {
        Some(r) => r.clone(),
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(error_response_coded("workspace_not_found", &cid.0)),
            )
                .into_response();
        }
    };

    let mut issues = Vec::new();
    if record.status == WorkspaceStatus::Disabled {
        issues.push("workspace is disabled in registry".to_string());
    }
    if record.repo_url.is_empty() {
        issues.push("repo_url is empty".to_string());
    }
    if record.kb_root.is_empty() {
        issues.push("kb_root is empty".to_string());
    }

    let response = WorkspaceHealthResponse {
        ok: issues.is_empty() && record.status != WorkspaceStatus::Disabled,
        workspace_id: record.workspace_id.clone(),
        display_name: record.display_name.clone(),
        status: record.status.clone(),
        repo_url: record.repo_url.clone(),
        kb_root: record.kb_root.clone(),
        write_policy: record.write_policy.clone(),
        issues,
    };
    let status = if response.ok {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    (status, Json(response)).into_response()
}

async fn submit_job_impl(
    state: AppState,
    request: CurioJobRequest,
    cid: String,
) -> axum::response::Response {
    match state.runtime.submit_job(request).await {
        Ok(response) => (StatusCode::ACCEPTED, Json(response)).into_response(),
        Err(err) => {
            tracing::error!(cid = %cid, err = %err, "submit_job failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(error_response_coded("internal_error", &cid)),
            )
                .into_response()
        }
    }
}

// ── Pub/Sub decode ─────────────────────────────────────────────────────────

pub fn decode_pubsub_request(payload: serde_json::Value) -> anyhow::Result<CurioJobRequest> {
    let message = payload
        .get("message")
        .ok_or_else(|| anyhow::anyhow!("missing Pub/Sub message"))?;
    let data = message
        .get("data")
        .and_then(|value| value.as_str())
        .ok_or_else(|| anyhow::anyhow!("missing Pub/Sub message.data"))?;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(data)
        .context("failed to decode Pub/Sub payload")?;
    let raw = String::from_utf8(bytes).context("Pub/Sub payload is not UTF-8")?;
    let request: CurioJobRequest =
        serde_json::from_str(&raw).context("failed to parse CurioJobRequest")?;
    Ok(request)
}

pub fn runtime_from_config(config: ServiceConfig) -> Result<ServiceRuntime> {
    ServiceRuntime::from_config(config)
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::types::{CurioJobRequest, JobActor, JobTrigger, JobType, WriteMode};
    use base64::Engine;

    #[test]
    fn pubsub_payload_decodes_into_job_request() {
        let request = CurioJobRequest {
            job_type: JobType::Sync,
            workspace_id: "acme".to_string(),
            provider: Some("passthrough".to_string()),
            trigger: JobTrigger {
                kind: "pubsub".to_string(),
                source: Some("topic/jobs".to_string()),
                request_id: Some("message-1".to_string()),
                received_at: Some("2026-04-24T00:00:00Z".to_string()),
            },
            actor: JobActor {
                kind: "service".to_string(),
                id: "curio".to_string(),
                display_name: Some("Curio".to_string()),
            },
            operation: "sync".to_string(),
            inputs: serde_json::json!({"args": ["--all"]}),
            write_mode: WriteMode::DirectPush,
            correlation_id: Some("corr-1".to_string()),
            idempotency_key: None,
        };

        let body = serde_json::json!({
            "message": {
                "data": base64::engine::general_purpose::STANDARD
                    .encode(serde_json::to_string(&request).expect("serialize")),
                "messageId": "message-1"
            }
        });

        let decoded = decode_pubsub_request(body).expect("decode pubsub request");
        assert_eq!(decoded.workspace_id, "acme");
        assert_eq!(decoded.job_type, JobType::Sync);
        assert_eq!(decoded.write_mode, WriteMode::DirectPush);
        assert_eq!(decoded.trigger.kind, "pubsub");
        assert_eq!(decoded.actor.kind, "service");
        assert_eq!(decoded.trigger.request_id.as_deref(), Some("message-1"));
        assert_eq!(decoded.correlation_id.as_deref(), Some("corr-1"));
    }
}
