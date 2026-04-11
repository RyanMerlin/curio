# Curio Agent CLI Contract

Curio is designed to be usable by humans and by automated agents.

Use `--json` when the caller needs machine-readable output from helper and discovery commands.

## Supported JSON Commands

- `curio doctor --json`
- `curio agent doctor --json`
- `curio agent prepare <provider> --json`
- `curio agent list-providers --json`
- `curio agent list-skills --json`
- `curio agent list-plugins --json`
- `curio agent print-env <provider> --json`
- `curio search --json`
- `curio bootstrap --json`
- `curio intake-create --json`
- `curio process-intake --json`
- `curio agent-analyze --json`
- `curio gold-resolve --json`
- `curio gold-publish --json`
- `curio review approve --json`
- `curio review reject --json`

## Output Shapes

- Helper commands use a common envelope:
  - `command`
  - `ok`
  - `data`
- `doctor` data includes `provider`, `ok`, `failed`, and `checks`.
- `prepare` data includes provider metadata, repo paths, command line, skill count, and enabled plugin count.
- `list-providers` data includes a `providers` array.
- `list-skills` data includes a `skills` array.
- `list-plugins` data includes a `plugins` array.
- `print-env` data includes `provider` and an `env` map.
- `search` data includes the CQL query, a result count, and the raw Confluence result array.
- `bootstrap` data includes the managed root folder ID, the README landing page ID, the structural roots (`_templates`, `_registry`, `_audit`), and the ensured base pages.
- `intake-create` data includes source item counts, handled items, duplicate skips, and unavailable skips.
- `process-intake` data includes the intake count, handled count, staged count, and review-required count.
- `agent-analyze` data includes the requested page count and analyzed page count.
- `gold-resolve` data includes the page ID and the number of proposed changes.
- `gold-publish` data includes the page ID and how many changes were applied.
- `review approve` and `review reject` data include the page ID, resulting status, and dry-run flag.

## Notes

- `--json` is intended for helper and discovery commands.
- `--json` is also available on write commands for deterministic agent automation.
- `curio agent launch` remains streaming and human-oriented.
- `curio onboard` remains interactive and may prompt to install the user-level shim.
- If a command fails in JSON mode, Curio still emits JSON first and then exits non-zero.
