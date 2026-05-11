# Curio Harness — Shared Operating Contract

This file is the canonical, provider-neutral contract for every agent that runs through the Curio harness (Claude, Codex, Gemini, and any future provider). Each provider has a thin entrypoint at the repo root (`CLAUDE.md`, `AGENTS.md`, `GEMINI.md`) that points here, plus a per-provider `providers/<name>/overrides.md` for anything provider-specific.

If a rule applies to **all** providers, it lives here. If it applies to **one** provider, it lives in that provider's `overrides.md`.

## Startup Contract

- Operate from the Curio repo root (the directory containing this file).
- Treat `curio-rs/` as the deterministic execution substrate. The Rust binary makes no LLM calls and performs no editorial routing.
- Treat the tracked `docs/wiki-demo/` tree as a synthetic demo workspace. Production KBs run through an explicit external KB path selected via `--workspace <name>` (registered in `curio.workspaces.toml`) or `--kb-dir <path>`.
- Use `skills/` and plugin-local skills (`plugins/<bundle>/skills/`) as the authored workflow source.
- Use `.agents/plugins/marketplace.json` as the active Curio plugin catalog.
- Honor the shared `CURIO_*` harness environment emitted by `curio agent print-env <provider>`.
- Before any non-trivial curation or publish action, write a short decision record using `docs/design/decision-log-template.md`.
- Infer first, then apply. Do not let the presence of `publish`, `resolve`, or `sync` commands substitute for editorial judgment.

## Multi-KB

A single Curio harness instance manages multiple knowledge bases. Each KB is a git working tree containing a `wiki/` scaffold and a `.curio.yaml` configuration that scopes its Confluence space, parent page, and credentials. KBs are registered in `curio.workspaces.toml`; reference them by name via `--workspace <name>` or by absolute path via `--kb-dir <path>`. Never assume a single global KB — every command that touches content must be scoped to a workspace.

## Command Pipeline

```
curio --workspace <kb> intake --url <url>     # ingest → wiki/intake/
curio --workspace <kb> process                # Phase 1: emit routing manifest (no LLM)
curio --workspace <kb> process --route-file routes.json   # Phase 2: apply routing
curio --workspace <kb> publish <slug>         # staged → published
curio --workspace <kb> tree                   # sync wiki/published/ dirs after NORTHSTAR changes
curio --workspace <kb> reindex                # rebuild co-located index.md files
curio --workspace <kb> sync                   # push wiki/published/ → Confluence
curio --workspace <kb> status                 # pipeline counts + staleness
curio --workspace <kb> lint                   # contradictions / stale claims / orphan refs
curio --workspace <kb> doctor [--scope <path>]                # KB health
curio --workspace <kb> heal --prepare [--scope <path>]        # Phase 1: emit heal manifest
curio --workspace <kb> heal --apply-file /tmp/heal-routes.json  # Phase 2: confidence-gated apply
curio --workspace <kb> reject <slug> [--reason <str>]
curio --workspace <kb> query "question"
```

## Agent-Native Routing (Two-Phase)

`curio process` is agent-native — the Rust binary never calls an LLM. The agent (you) is the router.

Before routing or publishing, state explicitly:
- what shape the corpus appears to have
- which pages are strong enough to publish
- which pages should remain staged or in review
- what evidence is missing

**Phase 1 — Manifest.** Run `curio process` (or `curio process --prepare`). It outputs a JSON manifest containing every page in `wiki/intake/`, the NORTHSTAR tree context, the root index summary, and an `apply_command` template.

**Phase 2 — Apply.** For each page, decide `category`, `status` (`staged` or `review`), `confidence` (0–1), `rationale`, and `alternatives_considered`. Build a route file and run `curio process --route-file /tmp/routes.json`. Each routed page is `git mv`'d, gets a `.analysis.json` sidecar with full provenance, and has its frontmatter updated.

**Disambiguation rules (per KB's NORTHSTAR.md):**
- Use exact route/exclude signals in each tree's `index.md`.
- When confidence < 0.75, route to `review/` — never guess taxonomy.
- Respect product-name boundaries (e.g. "Example Server" ≠ "Intelligence Suite").

## Curation Workflow Rules

- Start every non-trivial curation pass by writing a short decision record using `docs/design/decision-log-template.md`.
- `published` is never the first move for new intake or new curation. The first durable artifact must be `staged` or `review` unless the user explicitly authorizes a manual override.
- Do not directly create, restructure, split, merge, reroute, deduplicate, or substantially rewrite `wiki/published/` content as the first step unless the user explicitly asks for a manual override.
- Use `review` for ambiguity, taxonomy changes, subtree proposals, deduplication decisions, low-signal content, consolidation, or deletion candidates.
- Use `staged` when the route is clear and the content is strong enough to preserve as a proposed curated draft before publication.
- Treat Confluence `Review` as a required human-review surface. Proposals that humans must inspect must appear there, not only in Git.
- Low-signal or placeholder content must not be published — route it to `review` with an explicit recommendation to improve, consolidate, or delete.
- Confidence alone is not enough to publish; also assess information quality and usability.
- If you cannot explain the publish rationale in one sentence, the page is not ready.

## Layer Boundaries

- Orchestration, routing decisions, workspace context, and skill loading: **here** (the Curio harness).
- Command execution, structured output, safety gates, deterministic checks: **`curio-rs`**.
- Provider-neutral workflow guidance: **here**, never inside a provider-specific file.
- Git is the source of truth. Confluence is a **read-only sync target** for humans — Curio never reads back from Confluence as primary state.

## Primary References

- `README.md` — multi-provider overview
- `ARCHITECTURE.md` — layer boundaries and design rules
- Per-KB `NORTHSTAR.md` — routing charter (categories, route-here / exclude rules) for that KB
- `docs/onboarding.md` — bootstrap flow for new instances
- `docs/where-things-live.md` — directory structure reference
- `docs/agent-cli-contract.md` — machine-readable CLI contract
- `docs/provider-matrix.md` — per-provider capability matrix
- `providers/<name>/overrides.md` — provider-specific overrides (read after this file)
- `docs/runbook.md` — day-zero operator guide for the full intake → process → publish → sync flow, including the `curio doctor` infrastructure-check suite and the `--json` error envelope shape

## Day-zero invariants

Before any non-trivial editorial work, the agent and operator should confirm:

- `curio --workspace <name> doctor` shows all 8 infrastructure checks ✓ (config / NORTHSTAR / git / Confluence URL / email / token / space_key / auth probe).
- `wiki/_admin/config.yaml` declares a taxonomy whose top-level slugs match the `wiki/published/` tree the agent expects to route into.
- `connection.token_env` in `.curio.yaml` names a real env var; never embed secrets in YAML.
- The agent has read this file PLUS the per-provider `overrides.md` PLUS the per-KB `NORTHSTAR.md` before running `curio process`.
