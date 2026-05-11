# Curio Architecture

Curio is a multi-provider agent harness for `curio-rs`.

## Layers

- `curio-rs`: deterministic CLI substrate
- Curio root: harness orchestration, launch UX, provider entrypoints, skills, plugin wiring
- future shared catalog: optional extraction target for reusable plugin bundles

## Design Goals

- make `codex`, `claude`, and `gemini` first-class launch targets
- keep execution logic out of the harness
- keep provider entrypoints thin and route them through the same Curio context bundle
- keep provider launch defaults in repo-owned profiles under `providers/`
- keep Curio-local plugins extractable later without rewriting the launch model

## Rule of Thumb

- If the behavior is deterministic and safety-sensitive, it belongs in `curio-rs`.
- If the behavior is about composition, workspace context, routing, or provider startup, it belongs in Curio.
