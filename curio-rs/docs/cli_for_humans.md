# Curio CLI Reference (for Humans)

This document is the current human-facing reference for the `curio` CLI. For the agent-facing workflow, see `cli_for_agents.md`.

The Curio harness repo may contain a tracked sample `wiki/`, but production KB work should normally target an external workspace selected with `--workspace` or `--kb-dir`.

## Global Flags

These flags are accepted by all commands:

- `--config <path>` — path to the curio config file (default: `curio.toml`)
- `--kb-dir <path>` — override the KB root directory (normally resolved from config)
- `--workspace <name>` — select a named workspace from `curio.workspaces.toml`
- `--dry-run` — print what would be done without making changes
- `--json` — emit output as a JSON envelope `{ "command": "...", "ok": true, "data": {...} }`
- `--log-level <level>` — set log verbosity (`error`, `warn`, `info`, `debug`, `trace`)

---

## Commands

### `onboard [--install]`

Inspect and sync Curio onboarding state. Without `--install`, reports what is missing or misconfigured. With `--install`, applies repairs (prompts for NORTHSTAR intent if blank, installs binary, aligns `.env`).

```bash
curio onboard
curio onboard --install
```

---

### `doctor [--scope <path>]`

Run a structural health report on the KB. Checks for low-quality pages, stale content, high-overlap entries, orphaned cross-references, thin branches, and missing keywords. Output is a human-readable report by default; use `--json` for machine-readable output. For provider and harness health, use `curio agent doctor` instead.

```bash
curio doctor
curio doctor --scope wiki/published/product-tree
curio doctor --json
```

---

### `init-kb [--path <path>] [--name <name>] [--description <desc>]`

Create a new KB store at the given path. Initializes the directory structure, `curio.toml`, and seed index files. Use `--name` and `--description` to set KB metadata in the config.

```bash
curio init-kb --path ~/my-kb --name "My KB" --description "Internal knowledge base"
```

---

### `workspace list|add|remove`

Manage named KB workspaces in `curio.workspaces.toml`. `list` shows all registered workspaces. `add` registers a new workspace by name and path. `remove` deregisters a workspace.

```bash
curio workspace list
curio workspace add --name prod --path /data/prod-kb
curio workspace remove --name prod
```

---

### `init [--reset] [--confirm-nuke]`

Create the `wiki/` scaffold and seed the index files. Use `--reset` with `--confirm-nuke` to tear down and rebuild an existing wiki directory (destructive).

```bash
curio init
curio init --reset --confirm-nuke
```

---

### `intake --url|--file|--folder [--title] [--subject-hint] [--recursive]`

Ingest content into `wiki/intake/`. Exactly one of `--url`, `--file`, or `--folder` must be supplied. Use `--title` to override the detected title, `--subject-hint` to guide routing, and `--recursive` when ingesting a folder.

```bash
curio intake --url "https://example.com/release-notes"
curio intake --file ./notes.md --title "Q2 Release Notes"
curio intake --folder ./export/ --recursive
```

---

### `process [--limit N] [--all] [--prepare] [--route-file <path>] [--slug <s>] [--category <c>] [--status <s>] [--keywords <kw>] [--confidence <n>] [--summary <text>]`

Route intake pages — agent-native, two-phase. Phase 1: run `curio process` (or `--prepare`) to emit a JSON routing manifest. Phase 2: read the manifest, make routing decisions, then run `curio process --route-file /tmp/routes.json` to apply them. For a single-page direct override, pass `--slug`, `--category`, and `--status` together.

```bash
curio process --prepare                                          # Phase 1
curio process --route-file /tmp/routes.json                     # Phase 2
curio process --slug my-page --category product-tree --status staged  # direct override
```

---

### `status [--all]`

Show pipeline counts (intake / staged / review / published) and a staleness hint. Use `--all` to include counts for all workspaces.

```bash
curio status
curio status --all
```

---

### `review [--lane all|review|staged]`

List items in the review or staged lanes with status summaries. Defaults to showing both lanes.

```bash
curio review
curio review --lane review
```

---

### `resolve <slug> [--category <c>]`

Move a `review/` item to `staged/`. Supply `--category` to place it in the correct subtree.

```bash
curio resolve my-page --category product-tree/alteryx-server
```

---

### `publish <slug> [--category <c>]`

Publish a `staged/` page to `published/`. The page becomes the canonical source for Confluence sync.

```bash
curio publish my-page
```

---

### `search [--keywords <kw>] [--category <c>] [--status <s>] [--text <q>] [--limit N]`

Search the wiki registry. Filter by keywords, category, pipeline status, or free-text. Combine filters freely.

```bash
curio search --text "release notes" --status published --limit 10
curio search --category product-tree --keywords "server"
```

---

### `sharpen [--prepare] [--proposal-file <path>] [--limit N]`

Run knowledge-sharpening reviews to improve existing published pages. `--prepare` emits a proposal manifest; `--proposal-file` applies a previously prepared proposal.

```bash
curio sharpen --prepare --limit 20
curio sharpen --proposal-file /tmp/sharpen.json
```

---

### `reindex`

Rebuild all co-located `index.md` files and `_index/` artifacts from the current `wiki/published/` tree.

```bash
curio reindex
```

---

### `tree`

Sync `wiki/published/` directory structure to match the NORTHSTAR blueprint. Run after editing NORTHSTAR to apply structural changes.

```bash
curio tree
```

---

### `sync [--parent-page-id <id>] [--dry-run] [--all]`

Push `wiki/published/` content to Confluence. Requires Confluence credentials in the environment. Use `--dry-run` to preview without writing. Use `--all` to ensure all review pages have pinned comments for the feedback loop.

```bash
curio sync
curio sync --dry-run
curio sync --all
```

---

### `feedback [--dry-run]`

Read Confluence review signals (pinned comments, reactions) and apply them back to the local wiki. Use `--dry-run` to preview the changes.

```bash
curio feedback
curio feedback --dry-run
```

---

### `reject <slug-or-path> [--reason <str>] [--force]`

Locally reject a wiki page without touching Confluence. Records the reason in the audit log and removes the page from the active pipeline. Use `--force` to bypass the confirmation prompt.

```bash
curio reject my-page --reason "Duplicate of canonical-page"
curio reject wiki/intake/my-page.md --force
```

---

### `lint [--fix]`

Scan the wiki for contradictions, stale claims, and orphaned cross-references. With `--fix`, applies safe automatic repairs.

```bash
curio lint
curio lint --fix
```

---

### `query <question> [--save]`

Answer a question by querying the wiki with LLM assistance. Use `--save` to persist the answer as a wiki page.

```bash
curio query "What changed in Alteryx Server 2024.1?"
curio query "What is the retention policy?" --save
```

---

### `heal [--prepare] [--apply-file <path>] [--scope <path>] [--out <path>] [--confidence <n>] [--auto]`

AI self-healing loop for KB quality issues. Two-phase like `process`. Phase 1: `--prepare` emits a heal manifest (optionally scoped to a subtree with `--scope`, written to `--out`). Phase 2: `--apply-file` applies a previously prepared heal plan. Use `--confidence` to gate which fixes are auto-applied. Use `--auto` to apply all fixes above the confidence threshold without prompting.

```bash
curio heal --prepare --scope wiki/published/product-tree --out /tmp/heal.json
curio heal --apply-file /tmp/heal-routes.json --confidence 0.9
curio heal --apply-file /tmp/heal-routes.json --auto
```

---

### `agent prepare|launch|doctor|list-providers|list-skills|list-plugins|print-env`

Harness commands for provider and plugin management. These operate on the Curio workspace itself, not on wiki content.

- `prepare <provider>` — inspect the launch plan for a provider
- `launch <provider>` — start a provider in the Curio workspace
- `doctor` — check provider and harness health (distinct from `curio doctor` which checks the KB)
- `list-providers` — list configured providers
- `list-skills` — list available skills
- `list-plugins` — list loaded plugins
- `print-env <provider>` — print the resolved environment for a provider

```bash
curio agent doctor
curio agent doctor --json
curio agent prepare claude
curio agent launch claude
curio agent list-providers --json
curio agent print-env codex --json
```
