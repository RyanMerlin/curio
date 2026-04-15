# Provider Matrix

| Provider | Entrypoint | Profile | Default command | Override env |
|---|---|---|---|---|
| Codex | `AGENTS.md` | `providers/codex.json` | `codex` | `CURIO_CODEX_CMD` |
| Claude | `CLAUDE.md` | `providers/claude.json` | `claude` | `CURIO_CLAUDE_CMD` |
| Gemini | `GEMINI.md` | `providers/gemini.json` | `gemini`, then `adk` | `CURIO_GEMINI_CMD` |

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
