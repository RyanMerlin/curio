# Confluence Lifecycle Operator

Use this skill when the task is about the existing Curio Confluence lifecycle in `curio-rs`.

## Rules

1. use `curio-rs` commands for intake, proposal processing, staging, review, publishing, and sync
2. keep provider startup concerns out of the workflow itself
3. preserve the separation between harness guidance and deterministic content mutations
4. treat `northstar.json` as the structural taxonomy source generated from `_config/northstar.md`
5. do not shortcut structural curation directly into `published`; proposals must flow through `staged` or `review` unless the user explicitly authorizes a manual override
6. treat Confluence as a curated human-facing mirror and review surface, not a dump of repo helper state
