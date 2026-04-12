---
id: 42c59af8a4bae559
title: Curio README
status: published
source:
  kind: file
  id: file:README.md
  origin_url: null
  summary: null
category:
- by-topic
keywords:
- readme
- documentation
- overview
created_at: 2026-04-12T18:33:36Z
updated_at: 2026-04-12T18:37:05Z
confidence: 0.8
cross_refs: []
content_hash: sha256:bc73f6918b22feac7b076865326674bda81b5d768e1e1ecac14151faa200acda
confluence_page_id: null
model_used: manual
---

# Curio

Curio is the harness and orchestration repo for `curio-rs`.

The split is deliberate:

- `curio-rs` owns deterministic execution, checks, and CLI primitives
- Curio owns provider launch, prompt routing, skills, plugins, and onboarding
- Curio can later externalize reusable plugin bundles into a separate shared catalog without changing the local harness contract

## Supported Providers

- Codex
- Claude
- Gemini

All three providers are launched from the same Curio workspace contract:

- repo root: the Curio repository root
- authored skills: `skills/`
- compatibility skills: `.agents/skills/`
- plugin catalog: `.agents/plugins/marketplace.json`
- provider entrypoints: `AGENTS.md`, `CLAUDE.md`, `GEMINI.md`
- provider profiles: `providers/*.json`

Curio content writes are scoped by Confluence space:

- `CURIO_SPACE_KEY` is the authoritative write boundary
- `NORTHSTAR.md` seeds the charter page with the project intent text
- Curio's onboarding flow keeps `.env` and `.env.example` aligned on the Curio keys
- Bootstrap creates the `README` landing page plus the structural Confluence layers:
  - `NORTHSTAR`
  - `Intake`
  - `Staged`
  - `Review`
  - `Published`
  - `_templates`
  - `_registry`
  - `_audit`

For agent integrations, use `--json` on the helper commands and search:

- `curio doctor --json`
- `curio agent doctor --json`
- `curio agent list-providers --json`
- `curio agent list-skills --json`
- `curio agent print-env codex --json`
- `curio search --json`

The JSON shape uses a simple envelope:

- `command`
- `ok`
- `data`

Run onboarding with:

```powershell
curio onboard
```

`curio bootstrap` lays down the README landing page, NORTHSTAR charter, hero artwork, the lifecycle documentation, the Published blueprint tree, the template playbook, the registry index, and the audit log inside the configured Curio space. Destructive rebuilds require `--overwrite --confirm-nuke`.
`curio onboard` will prompt for NORTHSTAR intent when the corresponding env value is blank, then offer to repair the tree if required pages are missing.

## Quickstart

From the repo root:

```powershell
curio onboard
curio onboard --install
curio doctor
curio agent doctor
curio agent list-providers
curio agent launch codex
```

If the provider binary is not on `PATH`, set one of:

- `CURIO_CODEX_CMD`
- `CURIO_CLAUDE_CMD`
- `CURIO_GEMINI_CMD`

Curio also supports provider-owned extra args through:

- `CURIO_CODEX_ARGS`
- `CURIO_CLAUDE_ARGS`
- `CURIO_GEMINI_ARGS`

See `docs/onboarding.md` for the full bootstrap flow.
See `docs/agent-cli-contract.md` for the machine-readable CLI contract.
