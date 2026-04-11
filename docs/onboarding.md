# Curio Onboarding

## Base Requirements

- Rust toolchain for `curio-rs`
- provider launcher on `PATH`, or provider command override via environment variable
- run commands from `C:\code\agents\curio`

## Content Root Contract

Curio writes only within the configured Confluence output folder:

- `CURIO_CONFLUENCE_OUTPUT_ROOT_FOLDER_ID` is the primary setting
- `CURIO_ROOT_FOLDER_NAME` is a fallback only for older setups
- `.env` and `.env.example` must contain the same keys

## Provider Overrides

- Codex: `CURIO_CODEX_CMD`
- Claude: `CURIO_CLAUDE_CMD`
- Gemini: `CURIO_GEMINI_CMD`

Optional extra arguments:

- `CURIO_CODEX_ARGS`
- `CURIO_CLAUDE_ARGS`
- `CURIO_GEMINI_ARGS`

Repo-owned provider defaults live in:

- `providers/codex.json`
- `providers/claude.json`
- `providers/gemini.json`

Curio merges launch settings in this order:

1. provider profile file
2. provider-specific `CURIO_*_ARGS`
3. provider-specific `CURIO_*_CMD`

## Verification

```powershell
.\curio.ps1 agent doctor
.\curio.ps1 agent list-providers
.\curio.ps1 agent prepare codex
.\curio.ps1 agent prepare claude
.\curio.ps1 agent prepare gemini
```

## Launch

```powershell
.\curio.ps1 agent launch codex
.\curio.ps1 agent launch claude
.\curio.ps1 agent launch gemini
```
