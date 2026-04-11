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
