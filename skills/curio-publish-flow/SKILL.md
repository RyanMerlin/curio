---
name: curio-publish-flow
description: Curio publish flow guidance for orchestration and curio-rs content workflows.
---
# Curio Publish Flow

Use this skill when the task crosses between Curio orchestration and `curio-rs` content workflows.

## Rules

1. keep launch and routing concerns in the harness
2. use `curio-rs` commands for deterministic content operations
3. preserve the split between provider startup and Confluence-oriented workflow execution
4. treat proposals as the core curation unit; do not jump directly from intake or structural cleanup to `published`
5. treat the `yaml` block in `NORTHSTAR.md` (repo root) as the single source of truth for taxonomy — `northstar.json` no longer exists
6. treat Confluence as the curated mirror plus human review surface
7. keep audit and sharpening proposal storage under `wiki/_config`
