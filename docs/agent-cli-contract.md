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
- `curio retrieve --query <text> --json`
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
- `retrieve` is a read-only lexical retrieval command over canonical Markdown pages in
  `wiki/published/`. Its request flags are:
  - `--query <text>` (required)
  - `--category <path>` (optional; includes the category and its descendants)
  - `--limit <n>` (optional; defaults to `5`)
  - `--json` is the global machine-readable output flag.
- `retrieve --json` returns the stable envelope
  `{ "command": "retrieve", "ok": true, "data": ... }`. The `data` shape is:
  `{ "query": string, "category": string|null, "limit": number, "count": number,
  "results": [...] }`.
- Every retrieve result has this exact shape:
  - `id`: `local:<16 lowercase hex characters>`, derived from
    `sha256("published/" + path)`; this is local to the KB and is deliberately not a
    workspace-global identity.
  - `title`: published page frontmatter title.
  - `path`: slash-separated path relative to `wiki/published/`.
  - `category`: frontmatter category joined by `/`; if absent, the published parent
    directory is used.
  - `excerpt`: deterministic query-relevant Markdown excerpt, at most 280 characters.
  - `score`: numeric lexical score. Query terms contribute 100 for title, 60 for
    keywords, 30 for source summary, and 10 for body. Ranking first prioritizes the
    strongest matching field (title, keyword, summary, body), then score descending,
    then `path` ascending, so ties are stable.
  - `source_uri`: source `origin_url`, or `null` when absent.
  - `content_hash`: stored page frontmatter `content_hash`; a `sha256:<hex>` body
    hash is used only when the stored value is blank.
  - `updated_at`: published page frontmatter timestamp.
  - `authority`: always `"published"`.
  - `last_commit`: `{ "hash": string, "author": string, "committed_at": string,
    "subject": string }`, or `null` when the KB is not a Git worktree or lookup
    cannot be completed.
- Retrieval walks only `wiki/published/**/*.md`, excludes every `index.md` and known
  Markdown sidecars, and never reads intake, staged, review, registry, or generated
  index content. Empty and stopword-only queries fail with
  `{ "command": "retrieve", "ok": false, "error": { "code": "invalid_query",
  "message": string, "hint": string } }`; they never return all documents.
- Retrieval does not write pages, registry files, sidecars, audit logs, or query logs.
  `--dry-run` is accepted as a global flag and has no effect. No network or LLM call
  is made.
- Stability boundary: result IDs remain stable when page content changes, but change
  when the page’s relative published path changes. Paths, frontmatter fields, scoring
  weights, result fields, and error codes above are the adoption-slice contract.
- `bootstrap` data includes the configured space key, the README landing page ID, the Config branch page ID, and the ensured base pages.
- `onboard` ensures `NORTHSTAR.md` exists, then offers a repair bootstrap for missing required structure pages.
- `intake-create` data includes source item counts, handled items, duplicate skips, and unavailable skips.
- `process-intake` data includes the intake count, handled count, staged count, and review-required count.
- `process --prepare` (and `process` with no flags) emits a routing manifest with the following shape (schema_version 2):
  - `schema_version`: integer (currently 2; increment on breaking changes)
  - `action`: `"route_intake_pages"`
  - `page_count`: number of intake pages awaiting routing
  - `manifest_budget_bytes`: byte budget applied (default 64 KB; override with env `CURIO_MANIFEST_BUDGET_KB`)
  - `truncated`: bool — true when peer_pages were dropped to fit under budget
  - `dropped_peer_pages`: integer — count of peer entries removed by truncation
  - `taxonomy`, `northstar_context`, `workspace_config_yaml`, `index_summary`: KB charter and structure for the agent's recursive walk
  - `hierarchy_context`: array of branch entries; each carries `path`, `title`, `summary`, and up to 5 `peer_pages` with `path`/`title`/`summary`/`keywords` so the agent can judge hierarchy fit against actual peer leaves, not branch labels alone
  - `pages`: array of intake pages awaiting routing decisions
  - `instructions`: structured rules the agent must follow (hierarchy-first, depth, recursive index walk, new-subtree, overlap, hub-page) plus the `apply_command` template
  - **Doctor (`curio doctor --json`)** also returns an `infrastructure` array of per-KB checks (config / NORTHSTAR / git / Confluence URL / email / token / space_key / auth_probe) with `label`, `ok`, `detail`, optional `fix_hint`. Run before any colleague handoff.
- `agent-analyze` data includes the requested page count and analyzed page count.
- `gold-resolve` data includes the page ID and the number of proposed changes.
- `gold-publish` data includes the page ID and how many changes were applied.
- `review approve` and `review reject` data include the page ID, resulting status, and dry-run flag.

## Curio Curation Protocol

- Before a substantial publish or consolidation pass, write a short decision record with the inferred corpus shape, selected pages, deferred pages, and publish rationale.
- If the agent cannot articulate why a page belongs in `published`, keep it in `staged` or `review`.
- The command layer is for applying judgment; it is not a substitute for judgment.

## Notes

- `--json` is intended for helper and discovery commands.
- `--json` is also available on write commands for deterministic agent automation.
- `curio agent launch` remains streaming and human-oriented.
- `curio onboard` remains interactive and may prompt to install the user-level shim.
- If a command fails in JSON mode, Curio still emits JSON first and then exits non-zero.
- `agent doctor` is the harness integrity check; it now covers authored-vs-compat skill mirror parity and enabled plugin path validity in addition to provider launch readiness.
- `agent print-env` exposes the shared harness contract (`CURIO_HARNESS_DIR`, docs/skills/plugins/catalog paths, provider metadata, and effective wiki path) plus provider-specific extras where applicable.
