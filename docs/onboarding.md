# Curio Onboarding

`curio onboard` is the entrypoint for this flow.

By default it prompts to install the user-level Curio shim and treats Enter as yes.
Pass `--install` to force shim installation without prompting.

## Base Requirements

- Rust toolchain for `curio-rs`
- provider launcher on `PATH`, or provider command override via environment variable
- run commands from `C:\code\agents\curio`

## Content Root Contract

Curio writes only within the configured Confluence output folder:

- `CURIO_CONFLUENCE_OUTPUT_ROOT_FOLDER_ID` is the primary setting
- `.env` and `.env.example` must contain the same Curio keys

The onboarding command will:

- merge current shell environment values into `.env`
- keep existing `.env` values when the shell does not override them
- install or update the `curio` shim in the user cargo bin when approved
- validate Confluence auth with the current token
- check the managed output folder and lifecycle pages
- report provider launcher availability as warnings or failures

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
.\curio.ps1 onboard
.\curio.ps1 onboard --install
.\curio.ps1 doctor
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
