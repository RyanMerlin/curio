# Curio — Gemini Entry Point

Curio is the Gemini harness for `curio-rs`.

## Startup Contract

- operate from the repo root
- treat `curio-rs` as the deterministic execution substrate
- consume Curio-authored skills from `skills/`
- use `.agents/plugins/marketplace.json` as the plugin catalog
- use this file as the Gemini-specific entrypoint and keep provider-neutral workflow content elsewhere
- treat the tracked `wiki/` tree as the sample harness workspace; use external workspaces for production KBs

## Gemini Notes

- Curio expects the Gemini launcher to honor the shared `CURIO_*` environment contract emitted by `curio agent print-env gemini`
- the initial Gemini integration target is a launcher compatible with the Go ADK-oriented workflow, but the Curio command model stays provider-stable if the launcher changes later

## Primary References

- `README.md`
- `ARCHITECTURE.md`
- `docs/onboarding.md`
- `docs/provider-matrix.md`
