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

- repo root: `C:\code\agents\curio`
- authored skills: `skills/`
- compatibility skills: `.agents/skills/`
- plugin catalog: `.agents/plugins/marketplace.json`
- provider entrypoints: `AGENTS.md`, `CLAUDE.md`, `GEMINI.md`
- provider profiles: `providers/*.json`

Curio content writes are scoped by Confluence folder ID:

- `CURIO_CONFLUENCE_OUTPUT_ROOT_FOLDER_ID` is the authoritative write root
- Curio's onboarding flow keeps `.env` and `.env.example` aligned on the six Curio keys

For agent integrations, use `--json` on the helper commands and search:

- `.\curio.ps1 doctor --json`
- `.\curio.ps1 agent doctor --json`
- `.\curio.ps1 agent list-providers --json`
- `.\curio.ps1 agent list-skills --json`
- `.\curio.ps1 agent print-env codex --json`
- `.\curio.ps1 search --json`

The JSON shape uses a simple envelope:

- `command`
- `ok`
- `data`

Run onboarding with:

```powershell
.\curio.ps1 onboard
```

`curio bootstrap` now also lays down the Curio overview page, hero artwork, and the base lifecycle documentation under the managed Confluence write root.

## Quickstart

From `C:\code\agents\curio`:

```powershell
.\curio.ps1 onboard
.\curio.ps1 onboard --install
.\curio.ps1 doctor
.\curio.ps1 agent doctor
.\curio.ps1 agent list-providers
.\curio.ps1 agent launch codex
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
