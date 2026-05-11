# Codex — Provider Overrides

Read `HARNESS.md` first. Everything below is Codex-specific and overrides or extends the shared contract.

## Codex-Specific Layout

- `.agents/skills/` holds compatibility copies of authored skills for Codex's loader. Authored skills still live in `skills/`.
- `.codex-plugin/plugin.json` is the Codex plugin manifest emitted by the harness.

## Notes

- Use `curio agent print-env codex` for the authoritative Codex environment contract.
- Codex's policy framing (no `published` as a first move; structural changes route through `staged` or `review`) is the same as the shared contract — those rules now live in `HARNESS.md` and apply to every provider, not just Codex.
