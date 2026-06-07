# Provider Matrix

| Provider | Entrypoint | Profile | Default command | Override env |
|---|---|---|---|---|
| Codex | `AGENTS.md` | `providers/codex/profile.json` (+ `overrides.md`) | `codex` | `CURIO_CODEX_CMD` |
| Claude | `CLAUDE.md` | `providers/claude/profile.json` (+ `overrides.md`) | `claude` | `CURIO_CLAUDE_CMD` |
| Gemini | `GEMINI.md` | `providers/gemini/profile.json` (+ `overrides.md`) | `gemini`, then `adk` | `CURIO_GEMINI_CMD` |

All three root entrypoints (`AGENTS.md`, `CLAUDE.md`, `GEMINI.md`) are thin stubs that delegate to the shared `HARNESS.md` (canonical operating contract) plus their per-provider `overrides.md`. Add a new provider by creating `providers/<name>/{profile.json, overrides.md}` and one root entrypoint stub.

## Common Contract

Each provider receives:

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

Provider-specific extras:

- Codex: `CURIO_CODEX_PLUGIN_MANIFEST`
- Claude: `CURIO_CLAUDE_SETTINGS_PATH`
- Gemini: `CURIO_GEMINI_RUNTIME`

For machine-readable provider and harness inspection, use `--json` with:

- `curio doctor`
- `curio agent doctor`
- `curio agent prepare <provider>`
- `curio agent list-providers`
- `curio agent list-skills`
- `curio agent list-plugins`
- `curio agent print-env <provider>`

The canonical JSON envelopes, including the structured error shape, are documented in [`docs/agent-cli-contract.md`](agent-cli-contract.md).
