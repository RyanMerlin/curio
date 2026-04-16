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
- `docs/wiki-demo/` only as the tracked synthetic demo workspace; use an explicit external KB path for real KB operations

## Curation Workflow Rules

- Do not bypass the Curio content process for curation work.
- `published` is never the first move for new intake or new curation. The first durable artifact must be `staged` or `review` unless the user explicitly authorizes a manual override.
- Do not directly create, restructure, split, merge, reroute, deduplicate, or substantially rewrite `wiki/published/` content as the first step unless the user explicitly asks for a manual override.
- For new curation work and structural curation changes, the first artifact must go through `staged` or `review`.
- Use `review` when the work involves ambiguity, taxonomy changes, subtree proposals, deduplication decisions, low-signal content, consolidation, or deletion candidates.
- Use `staged` when the route is clear and the content is strong enough to preserve as a proposed curated draft before publication.
- Treat Confluence `Review` as a required human-review surface. Proposals that humans need to inspect must appear there, not only in Git.
- If you are about to edit `published/` directly for anything other than a narrow user-authorized manual cleanup, stop and route the work through `staged` or `review` first.
- Low-signal or placeholder content must not be published. Route it to `review` with an explicit recommendation to improve, consolidate, or delete.
- Confidence alone is not enough to publish. Also assess information quality and usability.
