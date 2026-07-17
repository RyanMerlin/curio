# Curio Changelog

## [Unreleased]

- Hardened the Confluence Cloud mirror with effective HTTP timeouts, bounded
  transient retries, rate-limit handling, sanitized diagnostics, strict `/wiki`
  URL validation, same-origin continuation validation, and bounded pagination
  for descendants, children, folders, and CQL results.
- Made `curio sync --all` ownership-safe: only pages carrying the Curio-owned
  `curio-sync` property can be deleted; unowned or malformed pages are
  preserved, and cleanup failures are surfaced in structured output.
- Added credential-free Confluence contract tests, an API compatibility matrix,
  and the opt-in `scripts/confluence-live-smoke.sh` sandbox harness.
- Added deterministic, read-only `curio retrieve` ranking over canonical
  `wiki/published/` pages, with cited excerpts, stable local IDs, source and Git
  provenance, machine-readable validation errors, and strict lane isolation.

## [1.0.1] — 2026-07-16 · public milestone readiness

- Added a credential-free `scripts/show-hn-demo.sh` path that verifies the
  synthetic intake, routing, review, staged, and publish lifecycle in a
  temporary workspace.
- Added agent-led setup guidance for knowledge operators, including explicit
  approval boundaries before publish or Confluence sync.
- Reconciled the public status with the shipped page rewriting, review-tree,
  multi-source consolidation, and cached-overlap capabilities.
- Clarified that hosted enterprise readiness still requires verified deployment
  identity, workspace-scoped secret resolution, durable concurrent state, audit
  integrity, and observability.
- Added CI packaging for tagged cross-platform release archives and SHA-256
  checksums, corrected public crate metadata, and made the packaged crate
  self-contained. Release assets remain subject to the verification checklist.
- Added a research-backed adoption roadmap centered on cited MCP retrieval,
  permission-aware source adapters, and measurable retrieval quality.

## [1.0.0] — 2026-05-10 · first public release

Brings Curio from internal tooling to a publicly-usable editorial knowledge-base harness. Apache 2.0 licensed.

### Highlights

- **Two-layer architecture.** Deterministic Rust substrate (`curio-rs`) + multi-provider agent harness. LLM calls live only in the harness; the binary stays predictable.
- **Multi-KB from day one.** A single Curio harness can manage N knowledge bases — each with its own taxonomy, Confluence space, and credentials. Atomic registry writes, per-KB infrastructure checks, true `--dry-run`. Production service hardening is not implied.
- **Editorial pipeline.** Two-phase agent-native routing, seven-dimension proposal scoring (route / quality / hierarchy fit / overlap risk / evidence completeness / usability / freshness), proposal dossiers as `.proposal.json` sidecars, taxonomy mutations against `NORTHSTAR.md`.
- **Multi-source synthesis.** One intake invocation can consolidate related sources into one proposal after agent judgment. Consolidation is recorded as `ProposalKind::Consolidation` with full source provenance; split application remains future work.
- **Page-body rewriting.** The agent ships curated knowledge objects, not raw captures. Structured decision sections explain rationale, scores, alternatives.
- **Confluence as the review surface.** Polished Review-tree rendering with score bars, status badges, taxonomy-mutation note macro, alternatives-considered lists, pinned reviewer-feedback comment with 👍 / 👎 / ❓ semantics.
- **Domain-agnostic engine + config-driven SSOT.** No company-specific product names, taxonomies, or emojis baked into the binary. Operators bring their own `wiki/_admin/config.yaml::products` registry to teach Curio about their domain.
- **Three providers, one contract.** Claude, Codex, Gemini all launch from the same `HARNESS.md` operating contract. Adding a fourth provider is two files.
- **Deterministic safety boundaries.** Atomic registry writes, intake resume-after-crash, publish-time re-gate (quality + overlap + taxonomy validity rechecked at promotion), `--force` escape hatch with audit logging, JSON error envelopes, and a comprehensive test suite (84/84 passing). Cloud Run production hardening remains a separate track.

### Community files

Apache 2.0 LICENSE, NOTICE, CONTRIBUTING, CODE_OF_CONDUCT, SECURITY, GitHub issue + PR templates, GitHub Actions CI workflow (fmt + clippy + test + build), Dependabot configuration.

### What's coming next

- Split proposal application and richer overlap evaluation
- Continuous self-sharpening scheduler
- Tuning-corpus learning (codify recurring rejection reasons into harness policy)
- Cloud Run authentication, workspace-scoped secrets, durable concurrent state, audit integrity, and observability

See [`docs/release-checklist.md`](docs/release-checklist.md) and the
[`enterprise readiness roadmap`](docs/design/2026-04-26-enterprise-readiness-roadmap.md)
for the public milestone and production-service tracks.

### Pre-public scrub

This release was preceded by a deliberate scrub (`docs/design/2026-05-10-pre-public-v1-plan.md`): hard-coded vendor-specific product names → config-driven `products` registry; hard-coded private git host URLs → config-driven `admin_related_repos`; personal-operator identifiers removed; stale `CURIO_AUDIT_DIR=${REPO_ROOT}/wiki/_config` env var removed; expanded `.gitignore` for every `.env` variant. The public repo ships zero domain assumptions in the engine; an example registry in `docs/wiki-demo/_admin/config.yaml` shows the pattern.

---

## 2026-05-10 — Tier 1: Production handoff readiness

Brings Curio from "happy path works on one KB" to "a colleague can drive their own KB end-to-end without help." Five phases covered by `docs/design/2026-05-10-production-handoff-plan.md`.

### Repo reorganization (Phase 1, prior session)

- New canonical `HARNESS.md` — provider-neutral operating contract.
- `providers/{claude,codex,gemini}/{profile.json, overrides.md}` — every provider gets a folder; root `CLAUDE.md` / `AGENTS.md` / `GEMINI.md` become thin stubs that delegate to `HARNESS.md`.
- `harness.rs` loader updated; tests updated; `providers/*.json` removed.

### Multi-KB foundations (prior session)

- `connection.token_env` per `.curio.yaml` so a single Curio instance manages multiple KBs with distinct Confluence credentials. `ConnectionConfig::resolve_token()` is the single source of truth — every Confluence client construction site goes through it.
- `curio-service` HTTP multi-tenancy: `GET /v1/workspaces`, `/v1/workspaces/:id`, `/v1/workspaces/:id/healthz`. `WorkspaceRegistry::save` is now atomic (write-tmp + rename).
- Confluence folder URLs supported in intake (`extract_confluence_folder_id` + `get_folder_descendants_v2`). 16-page demo folder ingested live.
- Intake resume-after-crash: `finalize_pending_intake` commits orphaned files left over from a prior partial run before a new intake begins.
- `wiki_index` walker no longer warns on `NORTHSTAR.md` / `README.md` at the wiki root.
- `tracing_subscriber` now wired to `--log-level` (was a dead flag).

### Phase A — Multi-tenant hardening

- **Standardized `--json` error envelope.** `main()` now wraps every dispatch error: when `--json` is set, emits `{command, ok:false, error:{code, message, hint}}` and exits non-zero. `output::emit_json_error` is the helper.
- **`curio doctor` extended with infrastructure checks** (8 per-KB probes): `kb.config`, `kb.northstar`, `kb.git`, `kb.confluence.url`, `kb.confluence.email`, `kb.confluence.token`, `kb.confluence.space_key`, and a real `kb.confluence.auth` probe against `/rest/api/user/current`. Each check carries a `fix_hint` line for human operators.
- **`tests/multi_tenant_safety.rs`** (3 tests) — proves filesystem isolation, token-env isolation under env contamination, and concurrent doctor runs against 2 KBs.

### Phase B — Phase 1 manifest enrichment

- Routing manifest bumped to `schema_version: 2`. Each branch in `hierarchy_context` now carries up to 5 `peer_pages` with title + 240-char summary + keywords from frontmatter — the editorial signal `docs/design/process.md` requires for hierarchy-fit judgment.
- Byte-budget guard (default 64 KB; override via `CURIO_MANIFEST_BUDGET_KB`). Drops peer entries from the largest branches first; sets `truncated:true` and `dropped_peer_pages:N`.
- 3 unit tests covering peer collection cap and budget enforcement.
- `docs/agent-cli-contract.md` documents the v2 schema.

### Phase C — Publish-time re-gate

- `gold_publish.rs` already enforced quality / taxonomy / overlap / `is_publish_ready` at promotion time. New: `publish --force` escape hatch that bypasses **editorial** gates (quality / overlap / proposal-ready) but **never** structural gates (taxonomy validity).
- Force-bypassed dimensions are recorded to `wiki/_admin/log.md` with a `[FORCE BYPASSED: ...]` tag and surfaced in the JSON envelope under `force_bypassed`.
- 2 integration tests in `tests/publish_regate.rs`.

### Phase D — Runbook + per-colleague onboarding

- **`docs/runbook.md`** — operator-facing guide aimed at someone who hasn't read `ARCHITECTURE.md`. Covers both service HTTP and CLI paths; doctor; intake (page / recursive / folder URL); two-phase process; publish + force; sync; rollback; `--json` error envelope shape.
- **`deploy/local/setup-colleague.sh`** — scaffolds a new KB, writes `.curio.yaml` / `NORTHSTAR.md` / `_admin/config.yaml` / `.gitignore` / `.env.example` / `README.md`, runs `git init` + initial commit, and registers the workspace in both `curio.workspaces.toml` and `deploy/local/state/workspaces.json`.
- **`curio init-kb`** now also writes a colleague-facing `README.md` at the KB root with a 6-step "first day" checklist.

### Phase E — Live verification

- Image rebuilt; container restarted; smoke test green for all 3 KBs.
- In-container `curio doctor` against a real KB: 8/8 infrastructure checks pass, including real Confluence auth.
- In-container `curio process --prepare`: schema_version 2, peer pages populated, budget honored.
- Live publish re-gate exercise: `publish` correctly refused on quality; `publish --force` correctly proceeded past quality and was correctly stopped by structural taxonomy validity (force never bypasses structural gates).

### Test count

72 tests pass: 50 lib + 2 demo + 2 multi_kb + 3 multi_tenant_safety + 2 publish_regate + 12 routing eval + 1 doctest.

### Bookmarked for Tier 2

- Multi-source synthesis (1 intake request → N proposals after merge/split).
- Page-body rewriting in process Phase 2.
- Continuous self-sharpening loop (today: manual `curio sharpen`).
- Embeddings-based overlap (today: Jaccard token overlap).
- Tuning-corpus learning (codify recurring rejection reasons into harness policy).
- Confluence-as-richer-front-end enhancements.
