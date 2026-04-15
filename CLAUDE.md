# Curio — Claude Entry Point

Curio is the Claude harness for `curio-rs`. Git is the canonical knowledge store.

## Startup Contract

- operate from the repo root
- treat `curio-rs` as the execution substrate
- treat the tracked `wiki/` tree as the sample harness workspace; production KBs should run through `--workspace` or `--kb-dir`
- use `skills/` and plugin-local skills as the authored workflow source
- use `.agents/plugins/marketplace.json` as the active Curio plugin catalog
- honor the shared `CURIO_*` harness environment from `curio agent print-env claude`

## Pipeline

```
curio intake --url <url>     # ingest → wiki/intake/
curio process                # Phase 1: output routing manifest (agent reads + decides)
curio process --route-file routes.json   # Phase 2: apply routing decisions, write .analysis.json sidecars
curio process --slug <s> --category account-tree --status staged   # direct override
curio publish <slug>         # staged → published
curio tree                   # sync wiki/published/ dirs after NORTHSTAR changes
curio reindex                # rebuild co-located index.md files + _index/ artifacts
curio sync                   # push wiki/published/ → Confluence (requires creds)
curio status                 # show intake/staged/review/published counts + staleness hint
curio lint                   # find contradictions, stale claims, orphan refs
curio doctor [--scope <path>]  # KB health: low-quality, stale, high-overlap, thin branches, orphaned xrefs
curio heal --prepare [--scope <path>] [--out /tmp/heal.json]  # Phase 1: emit heal manifest
curio heal --apply-file /tmp/heal-routes.json [--confidence 0.9] [--auto]  # Phase 2: confidence-gated apply
curio reject <slug> [--reason <str>]  # locally reject a page (no Confluence needed)
curio query "question"       # LLM-powered wiki query
```

## Agent-Native Routing (Two-Phase)

`curio process` is agent-native — no LLM calls in the Rust binary. You (Claude) are the router.

**Phase 1 — Manifest:**
Run `curio process` (or `curio process --prepare`). It outputs a JSON manifest with:
- All pages in `wiki/intake/` (slug, title, source, body preview)
- NORTHSTAR tree context (categories, route-here rules, exclude rules)
- Root index summary
- An `apply_command` template to use in Phase 2

**Phase 2 — Apply:**
Read the manifest. For each page, decide: `category` (e.g. `product-tree/alteryx-server`), `status` (`staged` or `review`), `confidence` (0–1), `rationale`, `alternatives_considered`.

Build a route file and run:
```
curio process --route-file /tmp/routes.json
```

Each routed page gets:
- `git mv` from `intake/` to `staged/{category}/` or `review/`
- `.analysis.json` sidecar with full routing provenance
- Frontmatter updated (category, status, keywords, confidence)

**Disambiguation rules (from NORTHSTAR):**
- Use exact route/exclude signals in each tree's index.md
- When confidence < 0.75, route to `review/` — don't guess
- Alteryx Server ≠ Intelligence Suite — check product name carefully

## Boundaries

- keep orchestration here
- keep deterministic execution in `curio-rs`
- keep provider-neutral workflow guidance in Curio, not in Claude-only files
- Git is source of truth — Confluence is a read-only sync target

## Primary References

- `README.md`
- `ARCHITECTURE.md`
- `NORTHSTAR.md`
- `docs/onboarding.md`
- `docs/provider-matrix.md`
- `docs/where-things-live.md`
- `wiki/published/index.md` (root wiki index — co-located, replaces `_index/index.md`)
- `wiki/published/{tree}/index.md` (per-tree navigation index)
