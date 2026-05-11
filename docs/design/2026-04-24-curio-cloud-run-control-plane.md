# Curio Decision Log

```text
date: 2026-04-24
workspace: /mnt/c/code/agents/curio/curio-agent
goal: add the next Curio phase as a service-oriented control plane with registry-backed workspace resolution and service-safe Curio execution seams
inferred_shape: keep curio-rs as the deterministic substrate; add a separate curio-service binary plus shared service library types, registry, job store, git materialization plan, and HTTP endpoints
selected_pages: curio-rs/src/lib.rs, curio-rs/src/main.rs, curio-rs/src/cli.rs, curio-rs/src/workspace.rs, curio-rs/src/git_ops.rs, curio-rs/src/harness.rs, curio-rs/src/audit_store.rs, docs/where-things-live.md, ARCHITECTURE.md
deferred_pages: broad provider API integrations, Cloud Run/IAM wiring, future database-backed state store
publish_rationale: this is architecture scaffolding and should remain in docs/design while the implementation lands in code
consolidation_rationale: keep the service control plane as a new layer beside the CLI instead of folding it into human-oriented commands
missing_information: exact Cloud Run deployment topology for the production project, provider adapter implementations, GitLab credential flow
next_action: implement the service library, HTTP runtime, registry model, and a local curio-service binary
```
