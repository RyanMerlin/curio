# Curio CLI Reference (for Agents)

This document is the machine-facing workflow reference for agents using the `curio` CLI. For human-facing command details and flag listings, see `cli_for_humans.md`.

The Curio harness repo may contain a tracked sample `wiki/`, but production KB automation should normally run against an external workspace selected with `--workspace` or `--kb-dir`.

## JSON Envelope

All commands support `--json`. The output shape is:

```json
{
  "command": "status",
  "ok": true,
  "data": { ... }
}
```

On failure, `ok` is `false` and `data` contains an `error` string. Always pass `--json` in automated pipelines to get structured output.

---

## Standard Pipeline

### 1. Check pipeline state

```bash
curio status --json
```

Returns counts for intake / staged / review / published and a staleness hint. Use this to decide whether intake, routing, or publishing is the priority action.

### 2. Ingest content

```bash
curio intake --url <url>
curio intake --file <path> --subject-hint "<hint>"
curio intake --folder <path> --recursive
```

Content lands in `wiki/intake/` as a markdown file with frontmatter. The slug is derived from the title.

### 3. Route intake pages (two-phase)

**Phase 1 — emit manifest:**

```bash
curio process --prepare --json
```

Output includes:
- All pages in `wiki/intake/` (slug, title, source, body preview)
- NORTHSTAR tree context (categories, `route-here` rules, `exclude` rules)
- Root index summary
- An `apply_command` template

**Phase 2 — apply routing decisions:**

Read the manifest. For each page, decide:
- `category` — e.g. `product-tree/alteryx-server`
- `status` — `staged` (confidence ≥ 0.75) or `review` (confidence < 0.75)
- `confidence` — float 0–1
- `rationale` — one sentence
- `alternatives_considered` — list of other candidate categories

Build a route file and apply:

```bash
curio process --route-file /tmp/routes.json --json
```

Route file schema:

```json
{
  "routes": [
    {
      "slug": "my-page",
      "category": "product-tree/alteryx-server",
      "status": "staged",
      "confidence": 0.87,
      "rationale": "Content describes a Server feature.",
      "alternatives_considered": ["product-tree/intelligence-suite"]
    }
  ]
}
```

Each routed page gets:
- `git mv` from `intake/` to `staged/{category}/` or `review/`
- `.analysis.json` sidecar with full routing provenance
- Frontmatter updated (category, status, keywords, confidence)

**Disambiguation rules:**
- Use exact `route-here` and `exclude` signals from each tree's `index.md`
- When confidence < 0.75, route to `review/` — do not guess
- Alteryx Server ≠ Intelligence Suite — check product name carefully

### 4. Inspect staged and review lanes

```bash
curio review --json
curio review --lane review --json
curio review --lane staged --json
```

### 5. Promote pages

Move a `review/` page to `staged/` after human or AI resolution:

```bash
curio resolve <slug> --category <category> --json
```

Publish a `staged/` page to `published/`:

```bash
curio publish <slug> --json
```

### 6. Push to Confluence

```bash
curio sync --json
```

Use `--dry-run` to preview without writing. Use `--all` to ensure all review pages have pinned comments before running the feedback loop.

---

## Self-Healing Loop

Run this loop to improve KB quality automatically.

### 1. Get health report

```bash
curio doctor --json
curio doctor --scope wiki/published/product-tree --json
```

Output contains a list of findings: low-quality pages, stale content, high-overlap entries, orphaned xrefs, thin branches, missing keywords.

### 2. Prepare heal manifest (Phase 1)

```bash
curio heal --prepare --scope wiki/published/product-tree --out /tmp/heal.json
```

Read `/tmp/heal.json`. For each finding, decide whether to accept, modify, or skip the proposed fix.

Write your decisions to `/tmp/heal-routes.json` using the same structure with an `accept` boolean and optional `override` fields.

### 3. Apply heal plan (Phase 2)

```bash
curio heal --apply-file /tmp/heal-routes.json --confidence 0.9 --json
```

- `--confidence <n>` — only apply fixes with confidence ≥ n
- `--auto` — apply all fixes above the threshold without prompting
- `--dry-run` — preview changes without writing

---

## Feedback Loop

Run this loop after a sync to incorporate Confluence review signals.

### 1. Ensure all review pages have pinned comments

```bash
curio sync --all --json
```

### 2. Read and apply signals

```bash
curio feedback --json
```

Use `--dry-run` to preview. Confluence signals (pinned comments, reactions) are mapped back to wiki frontmatter updates and staged as local changes.

---

## Local Rejection

To reject a page without touching Confluence:

```bash
curio reject <slug-or-path> --reason "<reason>" --json
curio reject <slug-or-path> --force --json   # skip confirmation
```

The audit log (`wiki/_config/log.md`) records the rejection.

---

## Search

```bash
curio search --text "<query>" --status published --json
curio search --category product-tree --keywords "server" --limit 20 --json
```

---

## Linting

```bash
curio lint --json
curio lint --fix --json
```

---

## Querying

```bash
curio query "What changed in Alteryx Server 2024.1?" --json
```

---

## Harness Health (not KB health)

```bash
curio agent doctor --json
curio agent list-providers --json
curio agent list-skills --json
curio agent print-env codex --json
```

Note: `curio agent doctor` checks provider/harness readiness. `curio doctor` checks KB structural health. These are distinct commands.
