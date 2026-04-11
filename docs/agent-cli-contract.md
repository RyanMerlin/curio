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

## Notes

- `--json` is intended for helper and discovery commands.
- `curio agent launch` remains streaming and human-oriented.
- `curio onboard` remains interactive and may prompt to install the user-level shim.
- If a command fails in JSON mode, Curio still emits JSON first and then exits non-zero.
