# Curio Onboarding

`curio onboard` is the entrypoint for this flow.

By default it prompts to install the user-level Curio shim and treats Enter as yes.
Pass `--install` to force shim installation without prompting.

## Base Requirements

- Rust toolchain for `curio-rs`
- provider launcher on `PATH`, or provider command override via environment variable
- run commands from the repo root

## Content Boundary Contract

Curio writes only within the configured Confluence space:

- `CURIO_SPACE_KEY` is the primary write boundary
- `.env` and `.env.example` must contain the same Curio keys

The onboarding command will:

- merge current shell environment values into `.env`
- keep existing `.env` values when the shell does not override them
- install or update the `curio` shim in the user cargo bin when approved
- validate Confluence auth with the current token
- check the configured space and lifecycle pages
- report provider launcher availability as warnings or failures

The bootstrap command will create or refresh the README landing page, the Config branch, the hero image, and the base lifecycle pages so the documentation layer is ready for human and agent use. If you need to wipe an existing managed tree, Curio now requires `--overwrite --confirm-nuke` so destructive resets are explicit.
`NORTHSTAR.md` is the editable source for the charter text; `curio onboard` ensures it exists and uses it before it offers a repair bootstrap.
It also builds the Curio operating layers:

- `Config` for the project intent and config-source pages
- `Intake` for raw capture
- `Staged` for high-confidence content
- `Review` for ambiguity and human arbitration
- `Published` for canonical output

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
curio onboard
curio onboard --install
curio doctor
curio agent doctor
curio agent list-providers
curio agent prepare codex
curio agent prepare claude
curio agent prepare gemini
```

## Launch

```powershell
curio agent launch codex
curio agent launch claude
curio agent launch gemini
```
