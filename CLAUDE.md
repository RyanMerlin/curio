# Curio — Claude Entry Point

Curio is the Claude harness for `curio-rs`. Git is the canonical knowledge store.

## Startup Contract

- operate from the repo root
- treat `curio-rs` as the execution substrate
- wiki knowledge lives in `wiki/` — use `curio reindex` to rebuild indexes
- use `skills/` and plugin-local skills as the authored workflow source
- use `.agents/plugins/marketplace.json` as the active Curio plugin catalog

## Pipeline

```
curio intake --url <url>     # ingest → wiki/intake/
curio process                # auto-route via heuristics (or agent-guided)
curio process --slug <s> --category by-account --status staged
curio publish <slug>         # staged → published
curio tree                   # sync wiki/published/ dirs after NORTHSTAR changes
curio reindex                # rebuild wiki/_index/ from filesystem
curio sync                   # push wiki/published/ → Confluence (requires creds)
curio lint                   # find contradictions, stale claims, orphan refs
curio query "question"       # LLM-powered wiki query
```

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
- `docs/where-things-live.md`
- `wiki/_index/index.md` (runtime wiki state)
