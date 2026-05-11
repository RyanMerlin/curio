# Claude — Provider Overrides

Read `HARNESS.md` first. Everything below is Claude-specific and overrides or extends the shared contract.

## Claude-Specific Settings

- Apply repo-local Claude settings from `.claude/settings.local.json` (path is also exposed as `CURIO_CLAUDE_SETTINGS_PATH`).
- Claude is the primary curation agent for Curio today. The full agent-native routing flow described in `HARNESS.md` is authored against Claude's tool surface; other providers may need adapter shims.

## Notes

- Use the `curio agent print-env claude` output as the authoritative Claude environment contract.
- When invoking sub-agents via the Agent tool, prefer the `Explore` subagent for read-only KB inspection and reserve the general-purpose agent for multi-step routing dry-runs.
