# Where Things Live

## `curio-rs`

- deterministic CLI behavior
- checks and safety gates
- provider launch planning and repo discovery
- helper command JSON output and agent-facing CLI contract

## Curio Root

- provider entrypoints
- onboarding docs
- harness-only skills
- local plugin catalog
- Confluence bootstrap docs and structural pages (`README`, `Config`, `Intake`, `Staged`, `Review`, `Published`)

## `plugins/`

- reusable Curio-local plugin bundles
- plugin-local docs and skills
- runtime helpers owned by the plugin bundle

## `skills/`

- authored harness skills
- compatibility copies live in `.agents/skills/`
- do not make `.agents/skills/` the authored source of truth

## `docs/agent-cli-contract.md`

- machine-readable helper command output
- `--json` usage patterns
- output shapes for agent automation

## `wiki/`

- git-native enterprise knowledge store
- `wiki/intake/` — raw ingested content awaiting routing
- `wiki/staged/` — high-confidence pages awaiting publish
- `wiki/review/` — pages needing human review; `review/auto-approved/` for AI-approved records
- `wiki/published/` — canonical content synced to Confluence
- `wiki/_config/` — `settings.yaml` (heal thresholds, labels), `log.md` (audit log), `northstar.md`

## `docs/`

- Curio architecture and onboarding guidance
- provider and CLI contract notes
- operational docs for the harness and Confluence structure
