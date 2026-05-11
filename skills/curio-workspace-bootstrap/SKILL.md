---
name: curio-workspace-bootstrap
description: Curio workspace bootstrap guidance for harness orientation and readiness checks.
---
# Curio Workspace Bootstrap

Use this skill when the agent needs to orient itself inside the Curio harness.

## Steps

1. confirm the repo root is the Curio repository root
2. treat `curio-rs` as the deterministic substrate
3. inspect `docs/`, `skills/`, and `plugins/` before making harness assumptions
4. use `curio onboard` for onboarding and `curio agent doctor` if provider readiness is in question
5. when process or taxonomy design changes materially, verify that skills and plugin docs are still aligned before relying on them
