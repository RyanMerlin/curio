# Curio — Enterprise Production Readiness Review & Roadmap

## Context

You're about to deploy the curio-service backbone to GCP and want an enterprise-grade posture: real auth (OIDC), multi-repo / multi-credential support, and direction on the human review UI. The codebase is well-factored at the boundary (deterministic Rust CLI vs. agent harness), but the service binary that fronts Cloud Run has critical gaps that should not ship as-is. This plan calls out the pre-deploy blockers, the multi-repo refactor, and a UI direction recommendation. Aim is to sequence work so the backbone goes live safely first, then unlocks multi-repo + UI.

**Confirmed scope:**
- **Tenancy: single-org, multi-repo.** One company, many internal repos under one Cloud Run deployment. Workspace-level RBAC is sufficient; no per-tenant KMS / per-tenant SA / SaaS-grade isolation needed yet. Keep the door open by using `workspace_id` as the isolation key but don't over-build.
- **Auth: Cloud IAP + Workforce Identity Federation.** IAP fronts Cloud Run; federation to the existing IdP (Okta / Azure AD / etc.). Curio-service verifies the IAP-injected JWT (`X-Goog-IAP-JWT-Assertion`) — no in-app OIDC client. Pub/Sub push retains its own Google-signed OIDC verification path.

## Current State (audit summary)

**Boundary you got right — keep it:**
- `curio-rs` is deterministic, no LLM calls; routing is agent-native; git is source of truth; Confluence is read-only mirror. Don't dilute this.
- `WorkspaceRegistryRecord` (`service/types.rs:193`) already models `repo_url`, `default_branch`, `credential_ref`, `write_policy`, `provider_defaults` per workspace — schema is multi-tenant-ready.
- NORTHSTAR per-KB resolution (`northstar.rs:48`) and per-KB `.env` overlay (`config.rs:316`) are already per-workspace. No changes needed here.

**What's actually wired**: registry schema is multi-tenant; everything else is single-tenant by env var.

**Critical gaps (blockers for prod deploy)** — all in `curio-agent/curio-rs/src/`:

1. **No inbound auth.** `service/server.rs:18-30` builds a bare router with zero middleware. The Pub/Sub OIDC token is never verified at `/v1/pubsub/jobs` (`server.rs:69-88`). `request.actor` is whatever the caller claims (`runtime.rs:851`). Cloud Run IAM is the only gate, and `INGRESS_TRAFFIC_ALL` is set in `deploy/cloud-run/terraform/main.tf:64`.
2. **Credential model is process-global.** `service/runtime.rs:904 authenticated_repo_url()` reads `CURIO_GITLAB_TOKEN` from env and embeds it into the URL string — token leaks via argv/error output, and there is exactly one of it. `credential_ref` on the registry record is never read.
3. **Audit log is not tamper-evident.** `audit_store.rs:126-204` is plain JSONL with destructive compaction (rewrites file keeping last 250 lines). No hash chain, no Cloud Logging sink, no retention lock. Auditors will reject.
4. **Concurrency unsafe at scale > 1.** State is GCS-FUSE-mounted JSONL (`terraform/main.tf:78-84,102-105`); per-workspace coordination is an in-process `tokio::Mutex` (`runtime.rs:832`). Two Cloud Run instances will race on `jobs.jsonl` / `audit.jsonl` and on git mirror dirs.
5. **No observability.** No `tracing`, no metrics, no request IDs, raw `eprintln!`. `/readyz` returns ok unconditionally (`runtime.rs:620`). Errors return raw `anyhow` strings to clients (`server.rs:101-106`).
6. **Container hygiene.** Dockerfile copies entire `/workspace` into runtime (`deploy/cloud-run/Dockerfile:27`), runs as root, no pinned digests.
7. **No UI / reviewer surface.** Today the only human entry points are the CLI and Confluence labels/comments. `curio review` (`commands/review.rs`) just prints a table.

## Phased Roadmap

### Phase 0 — Pre-deploy hardening (must do before GCP go-live)

Goal: make the public Cloud Run endpoint safe to expose. ~3–5 days of focused work.

1. **Inbound auth middleware** in `service/server.rs` — two verifiers, one middleware stack:
   - **Human path (`/v1/jobs`, future UI/API reads):** verify `X-Goog-IAP-JWT-Assertion` against IAP's JWKS (`https://www.gstatic.com/iap/verify/public_key-jwk`), check audience = `/projects/<project-number>/global/backendServices/<svc-id>`, extract `email` + `sub` as the verified principal. Workforce Identity federation handles the IdP login upstream — no app code needed for that.
   - **Pub/Sub path (`/v1/pubsub/jobs`):** verify Google-signed OIDC token (issuer `https://accounts.google.com`, audience = service URL, signing SA = the one configured at `terraform/main.tf:226-229`) using `jsonwebtoken` crate + Google JWKS cached in memory with TTL.
   - Replace caller-supplied `actor` (`runtime.rs:851`) with the verified principal in both paths — never trust client-side actor.
   - Reject everything else with 401, no body details.

2. **Tighten ingress + IAP wiring** in TF:
   - Flip `INGRESS_TRAFFIC_ALL` → `INGRESS_TRAFFIC_INTERNAL_AND_CLOUD_LOAD_BALANCING` (`terraform/main.tf:64`).
   - Provision external HTTPS LB + serverless NEG → Cloud Run; enable IAP on the backend service.
   - Configure Workforce Identity Federation pool for the corporate IdP; grant `roles/iap.httpsResourceAccessor` to the workforce principal set.
   - Pub/Sub push continues to reach the service directly (it uses the Cloud Run invoker SA path, not through IAP); verify the SA still has `roles/run.invoker` and document the two ingress paths clearly.

3. **Error hygiene**: change `error_response` (`server.rs:101-106`) to return `{ok:false, error_code, correlation_id}`; log full error detail server-side under that correlation ID only — never leak `anyhow` strings to clients.

4. **Pin `max_instances = 1`** in TF until Phase 2 fixes concurrency. Mark it with a `# TODO: lift after Phase 2 Firestore migration` comment.

5. **Move all secrets to Secret Manager** via `value_source.secret_key_ref`: `OPENAI_API_KEY`, `CURIO_GEMINI_ACCESS_TOKEN` override, Confluence token. Stop embedding GitLab token in URL strings (`runtime.rs:914-918`) — replace with a git credential helper script that fetches the token from Secret Manager at git-ask time and never writes it to argv or disk.

6. **Container hardening** in `deploy/cloud-run/Dockerfile`: distroless or Debian-slim base, non-root `USER`, drop the broad `COPY /workspace /workspace`, pin base image digest.

7. **Bootstrap observability**: add `tracing` + `tracing-subscriber` (JSON layer for Cloud Logging), `tower-http::trace` middleware, generate a `correlation_id` per request and thread it through all log events, propagate `X-Cloud-Trace-Context`. Expose `/metrics` via `axum-prometheus`. Make `/readyz` actually probe: registry file readable + state path writable + one Vertex AI ping.

8. **Schema validation & limits**: enforce max payload size on `/v1/jobs`; cap `inputs.args` length and element count to prevent argv injection into the spawned `curio` subprocess (`runtime.rs:288-310`); add `serde` validation on `JobType` / `WriteMode` enums to reject unknown values.

**Verification:**
- Smoke test sandbox: unauthenticated `curl` → 401; valid IAP-signed request → 202; valid Pub/Sub OIDC push → 202; tampered JWT (wrong audience) → 401.
- `cargo test` passes + new `service::auth` unit tests with mocked JWKS.
- `gcloud run deploy` to staging; `gcloud logging read` shows structured JSON with correlation IDs.

---

### Phase 1 — Multi-repo / multi-credential support

Goal: make the registry schema actually load-bearing so one curio-service instance can serve N repos (within one org) with N credential sets. ~1–2 weeks.

Note: single-org scope means **no** per-tenant KMS keys, per-tenant SAs, or customer-isolated audit storage needed in this phase. Workspace-level RBAC is the isolation boundary.

Files to modify (all under `curio-agent/curio-rs/src/`):

1. **Introduce a `SecretResolver` trait** (new `service/secrets.rs`) with implementations:
   - `EnvResolver` — dev fallback, reads `std::env::var`
   - `GcpSecretManagerResolver` — prod, calls Secret Manager REST API, caches with configurable TTL
   - `FileResolver` — test fixture, reads from a temp dir

   Keys are strings by convention (e.g. `gitlab/repo-x/token`, `confluence/workspace-y/token`). Wire into `ServiceRuntime` at construction.

2. **Extend `WorkspaceRegistryRecord`** (`service/types.rs:193`):
   ```
   git_auth: GitAuth  // replaces credential_ref
     HttpsToken { secret_ref: String }
     SshKey { secret_ref: String, known_hosts_ref: String }
   confluence: Option<ConfluenceCreds>  // replaces global env vars
     { base_url, email, token_ref, space_key, parent_page_id }
   llm_keys: HashMap<String, String>  // provider → secret_ref
   allowed_principals: Vec<String>    // verified IAP emails / SA emails
   ```

3. **Refactor `authenticated_repo_url`** (`service/runtime.rs:904`) to consume `record.git_auth` via the secret resolver. For SSH: write key to an in-memory tmpfile (prefer `/dev/shm` or `memfd_create`), set `GIT_SSH_COMMAND="ssh -i <keyfile> -o StrictHostKeyChecking=yes -o UserKnownHostsFile=<hostsfile>"`, clean up immediately after git subprocess exits. Never persist secrets to disk.

4. **Per-job config overlay**: rework `config.rs:251 load_config` so `ConfluenceClient` and `LlmConfig::effective_api_key` (`config.rs:49`) resolve from the active workspace record + secret resolver. Process env stays as the dev fallback only; in production a missing `credential_ref` should be a hard error, not a silent fallback.

5. **Unify CLI workspace with service registry**: today there are two models — `Workspace` in `workspace.rs:17` (TOML, path-only) and `WorkspaceRegistryRecord` in `service/types.rs`. Teach the CLI to optionally read from the registry JSON (read-only, no secret resolution) so `curio --workspace acme` resolves the same identity the service uses. Keep TOML as local-dev override.

6. **Per-workspace audit stream**: split `audit.jsonl` into `audit/<workspace_id>/audit.jsonl` so streams are isolated. Tied to Phase 2 for immutability, but do the path split now.

7. **Workspace-level RBAC**: middleware checks verified principal ∈ `workspace.allowed_principals` before dispatch; return 403 (not 404) on unauthorized workspace access. Admin principals live in a top-level config allowlist.

8. **Quota & rate limiting per principal**: `tower::limit::RateLimitLayer` per verified principal; per-workspace daily-job cap stored in the registry record.

**Verification:**
- Provision two test workspaces with different GitLab repos and different tokens in Secret Manager; submit jobs to both; confirm correct push targets and no token cross-contamination.
- Integration test `service::multi_workspace` using `FileResolver` with two distinct fake credentials.
- RBAC test: principal not in `allowed_principals` → 403.

---

### Phase 2 — Concurrency, audit integrity, and durable state

Goal: lift `max_instances` past 1 safely. ~1 week.

1. **Replace JSONL state with Firestore.** `jobs.jsonl` → `jobs/{job_id}` documents (Firestore transactions for state transitions); `workspaces.json` → `workspaces/{workspace_id}` documents. GCS FUSE mount remains only for the git mirror cache; all job and audit state moves off it.

2. **Hash-chained audit log**: each entry stores `sha256(prev_hash || canonical_json(entry))`. Mirror to Cloud Logging (immutable by default) and to a GCS bucket with **retention policy + bucket lock**. Remove the destructive compaction in `audit_store.rs:126-163`; replace with a periodic signed-checkpoint pointer if size management is needed.

3. **Distributed lock for git mirror dirs**: per-`workspace_id` Firestore lease (acquire → do work → release with TTL). Each Cloud Run instance must hold the lease before touching `cache_root/<workspace_id>`. Reject or queue if lease is held.

4. **Idempotency**: make `idempotency_key` on `CurioJobRequest` required; reject duplicates at submit time using a Firestore unique-constraint check. Return `202 Accepted` with the existing job record for duplicate keys.

**Verification:** load test with 10 concurrent jobs across 3 workspaces at `max_instances = 3`; verify no audit gaps, no git push collisions, no double-processing. Confirm Firestore transaction conflicts surface as 409, not 500.

---

### Phase 3 — UI direction

**Recommendation: don't build a custom curation web app yet. Build a minimal reviewer console only where the CLI/Confluence loop demonstrably fails.**

Why: today the review surface is markdown + git + Confluence labels/comments — a good fit for the agent-native model. A bespoke React app is a large surface area and a second source of truth for state that lives in git.

**Stage A — JSON-first read API** (1–2 days, do this in Phase 1):
Add three read endpoints to `service/server.rs`:
- `GET /v1/workspaces` — list workspaces the caller has access to (RBAC-filtered)
- `GET /v1/workspaces/{id}/queue` — intake/staged/review counts + page slug list
- `GET /v1/pages/{slug}` — page metadata + frontmatter + `.analysis.json` rationale

This is the contract any future UI builds on. It also makes `curio review --json` redundant (CLI can call it).

**Stage B — minimal reviewer console** (1–2 weeks, build when you can name two non-CLI humans who will triage weekly):

A small **SvelteKit** app (lean, no React ecosystem churn) behind IAP, three screens:
- **Queue view**: pages in `review/` with confidence score, NORTHSTAR-suggested category, and agent rationale from `.analysis.json`. Filterable by workspace.
- **Diff view**: rendered markdown on the left, agent rationale on the right, frontmatter panel.
- **Action bar**: Approve (→ `curio publish`), Reject with reason, Re-route to different category. All mutations are `POST /v1/jobs` — UI is stateless and dumb.

Auth: IAP-injected `X-Goog-IAP-JWT-Assertion`, same verification as Phase 0.

Hosting: serve static SvelteKit build via `tower-http::services::ServeDir` from the same Cloud Run service. No second service.

**Stage C — opinionated dashboards** (only after Stage B has real users):
- Per-tree quality/health dashboard (powered by `curio doctor --json`)
- Staleness / freshness charts
- `curio heal` proposal review queue

**What I'd skip entirely:** in-app markdown editor, provider playground, workspace settings UI (use TF).

---

## Critical Files

| File | Change |
|---|---|
| `service/server.rs` | Add auth middleware, Stage A endpoints |
| `service/runtime.rs:904` | Credential resolution refactor |
| `service/types.rs:193` | `WorkspaceRegistryRecord` extensions |
| `service/registry.rs` | Consume new fields |
| `audit_store.rs` | Hash-chained + Cloud Logging sink |
| `config.rs:251,316` | Per-workspace config overlay |
| `workspace.rs:17` | Unify with registry record |
| `commands/onboard.rs:15` | Workspace-scoped env contract |
| `deploy/cloud-run/terraform/main.tf:64,167-193` | Ingress, IAP, Secret Manager |
| `deploy/cloud-run/Dockerfile:27` | Container hardening |

## Out of Scope (intentionally)

- Rewriting the CLI in another language.
- Bespoke web UI before Stage A proves traction.
- Replacing Confluence as the published-content surface.
- Per-user fine-grained ACLs inside a workspace — workspace-level RBAC first; defer page-level.
- SaaS multi-tenancy (per-tenant KMS, per-tenant SA) — single-org scope confirmed.
