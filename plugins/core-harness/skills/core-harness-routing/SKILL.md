# Core Harness Routing

Use this skill when the task is about provider startup, workspace context, or harness routing.

## Rules

1. keep deterministic execution in `curio-rs`
2. treat Curio as the orchestrator and launch surface
3. route provider-specific behavior through the provider profile before inventing ad hoc launch logic
4. keep workflow policy in the harness, but keep content mutations, validation, taxonomy handling, and sync behavior in `curio-rs`
5. when Curio workflow guidance changes materially, update the harness skills and plugin docs so the agent does not operate from stale command names or stale mental models
6. require an inference-first decision record before any non-trivial curation, publishing, or consolidation pass
7. if a page is thin, duplicated, or ambiguous, prefer rewrite, consolidation, or deferral over immediate publish
8. keep a repeatable inspection order: discover corpus shape, inspect strongest candidates, identify blockers, then mutate
