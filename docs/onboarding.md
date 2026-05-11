# Curio Onboarding

`curio onboard` is the entrypoint for this flow.

By default it prompts to install the user-level Curio shim and treats Enter as yes.
Pass `--install` to force shim installation without prompting.

## Base Requirements

- Rust toolchain for `curio-rs`
- provider launcher on `PATH`, or provider command override via environment variable
- run commands from the repo root

## Workspace Model

- `curio-agent` is the harness repo
- the tracked `wiki/` directory is only a sample workspace for demos, docs, and harness validation
- day-to-day KB work should target an external workspace chosen with `--workspace <name>` or `--kb-dir <path>`
- `curio.workspaces.toml` stores local workspace registrations and is not committed

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

The bootstrap command will create or refresh the README landing page, the Admin branch, the hero image, and the base lifecycle pages so the documentation layer is ready for human and agent use. If you need to wipe an existing managed tree, Curio now requires `curio init --reset --confirm-nuke` so destructive resets are explicit.
`NORTHSTAR.md` is the editable source for the charter text, and `wiki/_admin/config.yaml` is the deterministic YAML source for taxonomy plus runtime settings. `curio onboard` ensures the charter exists before it offers a repair bootstrap.
It also builds the Curio operating layers:

- `Admin` for the project intent and config-source pages
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

- `providers/codex/profile.json` (with `providers/codex/overrides.md` for Codex-specific docs)
- `providers/claude/profile.json` (with `providers/claude/overrides.md` for Claude-specific docs)
- `providers/gemini/profile.json` (with `providers/gemini/overrides.md` for Gemini-specific docs)

Curio merges launch settings in this order:

1. provider profile file
2. provider-specific `CURIO_*_ARGS`
3. provider-specific `CURIO_*_CMD`

Every provider receives the same harness baseline:

- `CURIO_HARNESS_DIR`
- `CURIO_REPO_ROOT`
- `CURIO_CRATE_ROOT`
- `CURIO_DOCS_DIR`
- `CURIO_SKILLS_DIR`
- `CURIO_AGENTS_SKILLS_DIR`
- `CURIO_PLUGINS_DIR`
- `CURIO_MARKETPLACE_PATH`
- `CURIO_ENTRYPOINT`
- `CURIO_PROVIDER`
- `CURIO_PROVIDER_PROFILE`
- `CURIO_BOOTSTRAP_SUMMARY`
- `CURIO_WIKI_DIR`

Provider extras are additive rather than part of the shared contract:

- Codex: `CURIO_CODEX_PLUGIN_MANIFEST`
- Claude: `CURIO_CLAUDE_SETTINGS_PATH`
- Gemini: `CURIO_GEMINI_RUNTIME`

## Verification

```powershell
curio onboard
curio onboard --install
curio doctor
curio agent doctor
curio agent list-providers
curio workspace list
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
