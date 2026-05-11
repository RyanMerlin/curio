# Curio — Claude Entry Point

**Read `HARNESS.md` first** — it is the canonical, provider-neutral operating contract. Everything below is Claude-specific.

After `HARNESS.md`, read `providers/claude/overrides.md` for Claude-specific settings, environment contract, and notes.

## Quick Start (Claude only)

- Apply repo-local Claude settings from `.claude/settings.local.json`.
- Curio's agent-native routing flow is authored against Claude's tool surface; you are the primary curation agent.
- Run `curio agent print-env claude` for the full environment contract.
