# Hierarchical Index + LLM-Primary Routing Design

**Date:** 2026-04-13  
**Status:** Approved  
**Scope:** wiki_index.rs rewrite, reconcile.rs LLM-primary routing, analysis sidecar, sync/publish/reindex updates

---

## Problem Statement

Three compounding defects in the current system:

1. **Routing is heuristic-only.** `curio process` uses keyword matching (confidence 0.55 hardcoded) with no LLM involvement. This contradicts the product identity ("enterprise AI curation agent") and causes real misclassifications (e.g., "Rollback a Failed Server Upgrade" routed to `intelligence-suite`).

2. **No routing provenance.** When a page is misclassified, there is no record of why the system made that decision — no rationale, no alternatives considered, no signals. Debugging is forensic.

3. **Monolithic flat index.** `wiki/_index/index.md` dumps all pages into one file. Does not scale past ~100 pages. No descriptions of what subtrees mean, so agents have no disambiguation context. Consumers (LLM agents, Slack bots, Confluence) are served by one format that is optimal for none.

---

## Design

### 1. Hierarchical Co-Located Index

**Replace** `wiki/_index/index.md` with a tree of co-located `index.md` files living inside the content directories. Each level describes its own scope.

**File layout:**
```
wiki/
├── _index/
│   ├── registry.json     ← machine API (search, tooling, Slack) — KEPT
│   └── log.md            ← append-only audit trail — KEPT
│   (index.md DROPPED — replaced by co-located indexes)
│
└── published/
    ├── index.md                              ← root navigation (~1K tokens)
    ├── product-tree/
    │   ├── index.md                          ← tree overview with subtree descriptions
    │   ├── alteryx-server/
    │   │   ├── index.md                      ← leaf index: page table + route hints
    │   │   ├── cryptomigration-in-22-3.md
    │   │   └── cryptomigration-in-22-3.analysis.json
    │   ├── alteryx-designer/
    │   │   └── index.md
    │   └── intelligence-suite/
    │       └── index.md
    ├── account-tree/
    │   └── index.md
    ├── use-case-tree/
    │   └── index.md
    └── topic-tree/
        └── index.md
```

**Root index** (`wiki/published/index.md`):
- Total page count, last updated timestamp
- One entry per top-level tree: name, description, page count, link to tree index
- ~1K tokens regardless of wiki size

**Tree index** (`wiki/published/{tree}/index.md`):
- Tree name, description from NORTHSTAR
- One entry per subtree: name, description, page count, link to leaf index
- Explicit scope: what belongs here and what does not

**Leaf index** (`wiki/published/{tree}/{subtree}/index.md`):
- Subtree name, description from NORTHSTAR
- `Route here for:` — positive signals (injected into LLM routing prompt)
- `Do NOT route here for:` — exclusion rules (injected into LLM routing prompt)
- Full page table: title, summary, keywords, updated date

**`_index/` retains:**
- `registry.json` — machine-readable full catalog, used by `curio search`, Slack integrations, lint
- `log.md` — append-only chronological operation log

**Index generation:** `reindex` command rebuilds all co-located index files from `registry.json` + NORTHSTAR blueprint. Each `publish` operation updates the affected directory indexes incrementally.

---

### 2. LLM-Primary Routing

**Replaces** keyword-based heuristic routing in `reconcile.rs`.

**Flow per intake page (concurrent via `tokio::spawn`):**

```
1. Load page frontmatter + body
2. Run heuristic pre-signal (title token scan → suggested subtree label, NOT a decision)
3. Load NORTHSTAR subtree definitions (name, description, route-here, do-not-route-here)
4. Build routing prompt (see below)
5. LLM call → structured JSON response
6. Validate: category exists in NORTHSTAR, confidence in [0,1], status is staged|review
7. Write .analysis.json sidecar
8. Update frontmatter (category, keywords, confidence, model_used, updated_at)
9. git mv intake/{slug}.md → staged/{tree}/{subtree}/{slug}.md
10. Rebuild affected directory indexes
```

**Routing prompt structure:**

```
You are a routing agent for an enterprise knowledge wiki.
Your job: classify an article into exactly one subtree.

## Available Routes

### product-tree/alteryx-server
Description: Server-specific operational knowledge.
Route here for: server upgrades, cryptomigration, MongoDB ops, installation failures, service recovery, encryption migration.
Do NOT route here for: Designer workflows, AutoML, AI/ML pipelines, general product announcements.

### product-tree/intelligence-suite
Description: AI/ML tooling and AutoML guidance.
Route here for: AutoML patterns, machine learning pipelines, Intelligence Suite integrations.
Do NOT route here for: server administration, upgrade procedures, MongoDB, installation issues.

[... all subtrees from NORTHSTAR ...]

## Heuristic pre-signal (hint — override freely if wrong)
Title token match suggests: product-tree/alteryx-server

## Article
Title: {title}
Body: {body up to ~3K tokens}

## Output (JSON, no other text)
{
  "category": ["product-tree", "alteryx-server"],
  "confidence": 0.93,
  "rationale": "one or two sentences explaining the decision",
  "alternatives_considered": [
    {"path": ["product-tree", "intelligence-suite"], "score": 0.07, "ruled_out_because": "..."}
  ],
  "keywords": ["up", "to", "8", "terms"],
  "summary": "max 200 chars describing page content",
  "status": "staged",
  "review_reason": null,
  "cross_refs": []
}
```

**Confidence thresholds:**
- `>= 0.75` → route to `staged`
- `< 0.75` → route to `review` (LLM sets `status: "review"` and populates `review_reason`)
- LLM may also force `review` regardless of confidence if it detects ambiguity or conflict

**Error handling:**
- JSON parse failure → retry once, then route to `review` with flag
- Invalid category → route to `review` with flag
- LLM timeout/error → route to `review`, preserve intake file, log error

---

### 3. Analysis Sidecar

Every page that passes through `process` gets a `.analysis.json` file written alongside it. The file travels with the content through `publish` (git mv copies it). It is excluded from Confluence sync (sync filters on `.md` only).

**Schema (schema_version: 1):**
```json
{
  "schema_version": 1,
  "analyzed_at": "ISO 8601",
  "model": "claude-sonnet-4-6",
  "inputs": {
    "title": "string",
    "source_url": "string or null",
    "content_hash": "sha256:...",
    "content_preview": "first 500 chars of body"
  },
  "routing": {
    "decision": ["tree", "subtree"],
    "confidence": 0.93,
    "rationale": "string",
    "alternatives_considered": [
      {"path": ["tree", "subtree"], "score": 0.07, "ruled_out_because": "string"}
    ],
    "flags": [],
    "review_reason": null
  },
  "signals": {
    "heuristic_pre_signal": "subtree slug or null",
    "title_tokens": ["array", "of", "tokens"],
    "keywords_extracted": ["array"]
  }
}
```

**Lint integration:** `curio lint` scans `.analysis.json` files to surface:
- Pages with confidence < 0.75 that were force-staged
- Pages where heuristic pre-signal disagreed with LLM decision
- Pages in `review` with no resolution after N days

---

### 4. Sync / Publish / Reindex Updates

**sync:**
- `index.md` files sync as **section pages** (existing behavior for directory pages — they get a `<ac:structured-macro ac:name="children"/>` body listing child pages)
- `.analysis.json` files are **never synced** — filtered by extension before upload
- Stale page detection excludes index pages (they are always regenerated, never stale)

**publish:**
- After `git mv staged/{path}/{slug}.md → published/{path}/{slug}.md`, also move `staged/{path}/{slug}.analysis.json → published/{path}/{slug}.analysis.json` if it exists
- Rebuild the affected leaf `index.md` after publish

**reindex:**
- Walks `wiki/published/**/*.md` (excluding `index.md` files)
- Rebuilds `registry.json` from frontmatter
- Regenerates all co-located `index.md` files from registry + NORTHSTAR blueprint
- `_index/index.md` is deleted if it exists (migration cleanup)

---

## What Does NOT Change

- `wiki/_index/registry.json` — format, location, consumers unchanged
- `wiki/_index/log.md` — append-only, unchanged
- `Frontmatter` schema — `routing` fields added to existing schema, all existing fields preserved
- `curio search` — reads registry.json, unaffected
- `curio sync` — minimal changes (filter .analysis.json, treat index.md as section pages)
- Confluence page hierarchy — co-located index.md files become section overview pages, improving navigation

---

## Success Criteria

- `curio reindex` generates co-located `index.md` at every directory level in `published/`
- `curio process` calls LLM per intake page, writes `.analysis.json`, routes with confidence >= 0.75 to staged
- "Rollback a Failed Server Upgrade" routes to `alteryx-server`, not `intelligence-suite`
- `curio sync` pushes co-located `index.md` files as Confluence section pages, skips `.analysis.json`
- `cargo test` passes
- `_index/index.md` no longer exists
