# Curio

Curio is the harness and orchestration repo for `curio-rs`.

The split is deliberate:

- `curio-rs` owns deterministic execution, checks, and CLI primitives
- Curio owns provider launch, prompt routing, skills, plugins, and onboarding
- Curio can later externalize reusable plugin bundles into a separate shared catalog without changing the local harness contract

## Repo Model

- this repository is the Curio harness
- the tracked `wiki/` tree is a small sample workspace for docs, demos, and harness validation
- production KBs should live in external repos or directories and be selected with `--workspace <name>` or `--kb-dir <path>`
- `curio.workspaces.toml` is local operator state and is intentionally gitignored

## Supported Providers

- Codex
- Claude
- Gemini

All three providers are launched from the same Curio workspace contract:

- repo root: the Curio repository root
- harness dir: `CURIO_HARNESS_DIR`
- authored skills: `skills/`
- compatibility skills: `.agents/skills/`
- plugin catalog: `.agents/plugins/marketplace.json`
- provider entrypoints: `AGENTS.md`, `CLAUDE.md`, `GEMINI.md`
- provider profiles: `providers/*.json`
- sample wiki: `wiki/` unless an external workspace overrides `CURIO_WIKI_DIR`

Provider-specific extras remain explicit:

- Codex also receives `CURIO_CODEX_PLUGIN_MANIFEST`
- Claude may use `CURIO_CLAUDE_SETTINGS_PATH` when the repo-local settings file exists
- Gemini may use `CURIO_GEMINI_RUNTIME` to describe the expected launcher shape

Curio content writes are scoped by Confluence space:

- `CURIO_SPACE_KEY` is the authoritative write boundary
- `NORTHSTAR.md` seeds the charter branch content under `Config`
- Curio's onboarding flow keeps `.env` and `.env.example` aligned on the Curio keys
- Bootstrap creates the `README` landing page plus the structural Confluence layers:
  - `Config`
  - `Intake`
  - `Staged`
  - `Review`
  - `Published`

For agent integrations, use `--json` on the helper commands and search:

- `curio doctor --json` (KB structural health report)
- `curio agent doctor --json` (provider/harness health check)
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

`curio bootstrap` lays down the README landing page, the Config branch, hero artwork, the lifecycle documentation, and the Published blueprint tree inside the configured Curio space. Destructive rebuilds require `--overwrite --confirm-nuke`.
`curio onboard` will prompt for NORTHSTAR intent when the corresponding env value is blank, then offer to repair the tree if required pages are missing.

## Quickstart

From the repo root:

```powershell
curio onboard
curio onboard --install
curio doctor
curio agent doctor
curio agent list-providers
curio workspace list
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
