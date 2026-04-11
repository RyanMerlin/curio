# Curio — Claude Entry Point

Curio is the Claude harness for `curio-rs`.

## Startup Contract

- operate from `C:\code\agents\curio`
- treat `curio-rs` as the execution substrate
- use `skills/` and plugin-local skills as the authored workflow source
- use `.agents/plugins/marketplace.json` as the active Curio plugin catalog

## Boundaries

- keep orchestration here
- keep deterministic execution in `curio-rs`
- keep provider-neutral workflow guidance in Curio, not in Claude-only files

## Primary References

- `README.md`
- `ARCHITECTURE.md`
- `docs/onboarding.md`
- `docs/where-things-live.md`
