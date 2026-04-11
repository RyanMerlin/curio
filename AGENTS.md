# Curio Operating Rules

Curio is the Codex-side harness for `curio-rs`.

## Boundaries

- Do not reimplement deterministic `curio-rs` behavior here.
- Keep provider launch, playbook routing, workspace context, and skill loading in Curio.
- Keep reusable Curio plugin content under `plugins/` until a separate shared catalog is justified.

## What Belongs Here

- provider bootstrapping
- skill and plugin routing
- workspace-specific instructions
- orchestration docs
- harness-only policies

## What Stays In `curio-rs`

- command execution
- structured output
- safety gates
- checks that belong to the CLI substrate

## Curio Context

When launched from Curio, use:

- `skills/` as the authored skill source
- `.agents/skills/` as compatibility copies
- `.agents/plugins/marketplace.json` as the plugin catalog
- `plugins/` for Curio-local plugin bundles
- `docs/` for Curio architecture and onboarding
