# Curio Design Roadmap

This roadmap tracks the next technical and product-shaping steps for Curio after the current init/reset hardening work.

It should read as a sequence of capabilities to build, not just a scratchpad of tasks. Future ideas can be added as later phases or as backlog items at the end.

## Phase 1: Intake Fidelity

Objective:
Make incoming source capture reliable enough that the agent is reasoning over real source signal instead of degraded placeholders.

Planned work:
- Fix Confluence intake extraction for hub and index pages.
- Preserve smart links, children macros, and tables as usable source signal.
- Ensure hub pages do not collapse into near-empty markdown bodies.

Current driver:
- The current review item at [troubleshooting.md](/C:/code/agents/curio/wiki/review/product-tree/alteryx-server/troubleshooting.md) is correctly held in `review`, but it cannot be elevated until extraction quality improves.

## Phase 2: Hierarchy-First Proposal Quality

Objective:
Force proposal generation to produce the best hierarchy for the information, not the first acceptable route match.

Planned work:
- Make the agent propose a full path tree, not just the nearest category match.
- Bias technical-detail content toward deeper paths by default.
- Require explicit rationale whenever a shallower path is chosen over a deeper structural option.
- Reprocess the current troubleshooting review item after extraction improves.
- Decide whether to create `product-tree/alteryx-server/troubleshooting` as a real intermediate branch node.
- If approved, generate that branch node plus child leaf proposals instead of flattening the content.

## Phase 3: Branch Nodes as First-Class Knowledge Objects

Objective:
Make branch nodes useful, human-readable, and durable in both Git and Confluence.

Planned work:
- Strengthen branch-node generation so branch proposals include:
  - branch description
  - child index
  - short child descriptions
  - usage guidance
- Ensure Confluence branch pages always render those elements cleanly.
- Add validation and heal rules so branch nodes do not become empty or low-value shells.

## Phase 4: Stronger Curation and Overlap Judgment

Objective:
Improve the agent’s ability to recognize redundancy, consolidation opportunities, and weak published outcomes.

Planned work:
- Move beyond duplicate-title detection to semantic overlap analysis.
- Compare against nearby peers in the active branch neighborhood.
- Prefer merge or consolidation proposals when overlap is high.
- Route low-signal or weak-content outcomes back to `review` with explicit recommended action.

## Phase 5: Proposal Dossiers and Review Surface

Objective:
Make every proposal fully explainable and easy for humans to review in Git and Confluence.

Planned work:
- Formalize a stable dossier schema containing:
  - source set
  - inspected artifacts
  - nearby pages considered
  - alternatives rejected
  - overlap candidates
  - unresolved questions
  - recommended action
- Ensure every human-reviewable proposal is clearly surfaced in Confluence `Review`.
- Keep machine-only artifacts out of Confluence.

## Phase 6: Doctor, Heal, and Continuous Curation

Objective:
Turn Curio into a recursive knowledge-maintenance engine, not just an intake and publish pipeline.

Planned work:
- Add a Confluence tree doctor command.
- Validate root placement, required children, body content, hero attachment, and orphaned managed pages.
- Report concrete defects without improvising repair.
- Add healing flows that recurse through the current hierarchy:
  - inspect branch indexes recursively
  - detect weak branches, low-signal leaves, overlap clusters, and empty nodes
  - generate review proposals for restructuring

## Phase 7: Deterministic Core Maintenance

Objective:
Keep init, sync, and status deterministic as the system grows.

Planned work:
- Maintain the hard contract in [curio-core-init.md](/C:/code/agents/curio/docs/design/curio-core-init.md)
- Extend tests around reset, validation, malformed live trees, and stale managed pages.
- Continue reducing hidden state and ambiguous fallbacks.

## Test/Dev Run Findings — 2026-04-14 (SupportServer ingest)

### What was run
- Space: `SupportServer` (Alteryx Confluence) — 152 pages
- All pages ingested, routed, reindexed, linted
- 8 new subtrees proposed under `product-tree/alteryx-server`: api, mongodb, upgrade, authentication, administration, sql-db-persistence, high-availability, user-management

### What worked well
- Hub page synthesis: sparse Confluence pages with only children macros now get usable bodies
- Shallow-route gate: 1-component category routes get demoted to review correctly
- Taxonomy mutation proposals: every new subtree produces a clean `taxonomy_change` proposal with `proposed_new_subtree` and `node_description`
- Proposal dossier quality: full body preserved, rationale meaningful, overlap candidates listed
- Narrative log (`_config/log.md`) recording every pipeline action append-only
- 41 tests all green including new routing_eval integration harness

### Open issues found
1. **Query returns 0 results until publish** — expected behavior but needs UX clarity. A `curio query` call against content sitting in `review/` finds nothing. Consider adding a `--include-review` flag or separate `curio review-query` for the curation workflow.
2. **Process runs 10 pages per invocation** — the `process --route-file` command processes in batches of 10 (appears to be a hard limit). For a 152-page ingest, 15+ re-runs were needed. Either raise the batch size or add a `--all` flag to drain the queue in one pass.
3. **Branch node descriptions not auto-propagated to northstar.json** — after approving 8 new subtrees via review and adding them to northstar.json manually, reindex does not validate that description_markdown is populated. The branch validation warns on reindex but only for published nodes, not for newly added taxonomy nodes.
4. **`proposed_new_subtree` path not normalized** — the route file used `"product-tree/alteryx-server/upgrade"` (string), but northstar.rs expects a `Vec<String>` path. The Rust side appears to handle this but the gen-routes script should emit the array form.
5. **Merge proposal auto-population uses title similarity only** — the draft-HA overlap was caught, but the merge_target population is heuristic. A semantic embedding-based overlap pass would improve precision here.

### Next iteration priorities
1. Approve the 8 proposed taxonomy subtrees → promote review items with high confidence to staged → publish first batch
2. Run `curio sync` to push to Confluence; verify branch pages render correctly with child indexes
3. Implement `--all` flag (or batch-size config) for `curio process --route-file` to avoid the 10-page limit
4. Add `curio review-query` or `--include-review` so curation work has search visibility before publish

## Backlog Ideas

These are valid future directions but are not yet committed to a near-term build sequence.

- richer multi-source intake request model
- stronger taxonomy mutation workflow
- deeper proposal state machine
- automated healing suggestions for weak branch layouts
- ranking and prioritization of review proposals by expected signal gain
