# Curio Design Roadmap

This roadmap tracks the next technical and product-shaping steps for Curio after the current init/reset hardening work.

It should read as a sequence of capabilities to build, not just a scratchpad of tasks. Future ideas can be added as later phases or as backlog items at the end.

---

## ✅ Consolidated NORTHSTAR.md + northstar.json (complete 2026-04-14)

**Problem:** The taxonomy lives in two files that must be kept in sync manually:
- `NORTHSTAR.md` — human-readable, authoritative intent and descriptions (92 lines)
- `wiki/_config/northstar.json` — machine-readable, what `northstar.rs` actually parses (113 lines)

`northstar.json` has `"generated_from": "northstar.md"` but is not actually generated — it is maintained by hand in parallel, which means they can and do drift. Any edit to the taxonomy requires touching both files.

**Goal:** `NORTHSTAR.md` is the single source of truth. The taxonomy is expressed as a YAML fenced block inside it. `northstar.rs` parses that YAML directly — `northstar.json` is eliminated entirely.

**Design:**

`NORTHSTAR.md` structure after the change:
````markdown
# Northstar — Knowledge Taxonomy

[prose: purpose, guiding principles, routing rules]

```yaml
# taxonomy
schema_version: 2
nodes:
  - title: Product Tree
    slug: product-tree
    icon: 🌲
    description_markdown: |
      Structured knowledge organized by product.
    children:
      - title: Example Server
        slug: example-server
        ...
```

[prose: additional notes, routing guidance]
````

Parsing approach in `northstar.rs`:
- Read `NORTHSTAR.md` as a string.
- Find the first fenced block tagged `yaml` (or `yaml\n# taxonomy`).
- Deserialize via `serde_yaml` into the existing `NorthstarTaxonomy` struct (or a thin new struct that maps to it).
- No intermediate JSON file written or read.

**Migration steps:**
1. Add `serde_yaml` to `Cargo.toml`.
2. Write `parse_northstar_md(path) -> Result<NorthstarTaxonomy>` in `northstar.rs`.
3. Replace all `northstar.json` load calls with `parse_northstar_md("NORTHSTAR.md")`.
4. Merge `NORTHSTAR.md` + `wiki/_config/northstar.json` into a single updated `NORTHSTAR.md` with the YAML block.
5. Delete `wiki/_config/northstar.json`.
6. Update `curio feedback` (`maybe_update_northstar`) to write back into the YAML block in `NORTHSTAR.md` instead of the JSON file.
7. Update `curio reindex` and any other command that reads northstar.json.

**Critical files:**
- `curio-rs/src/northstar.rs` — primary change
- `curio-rs/Cargo.toml` — add `serde_yaml`
- `NORTHSTAR.md` — absorbs the taxonomy YAML
- `wiki/_config/northstar.json` — deleted
- `curio-rs/src/commands/feedback.rs` — `maybe_update_northstar` writes back to .md
- Any command importing `northstar.json` path directly

---

## Phase 1: Intake Fidelity

Objective:
Make incoming source capture reliable enough that the agent is reasoning over real source signal instead of degraded placeholders.

Planned work:
- Fix Confluence intake extraction for hub and index pages.
- Preserve smart links, children macros, and tables as usable source signal.
- Ensure hub pages do not collapse into near-empty markdown bodies.

Status: **Complete** (commit 752df86 — ac:link rendering, ADF panel dedup, children macro suppression, table cell fix).

## Phase 2: Hierarchy-First Proposal Quality

Objective:
Force proposal generation to produce the best hierarchy for the information, not the first acceptable route match.

Planned work:
- Make the agent propose a full path tree, not just the nearest category match.
- Bias technical-detail content toward deeper paths by default.
- Require explicit rationale whenever a shallower path is chosen over a deeper structural option.
- Reprocess the current troubleshooting review item after extraction improves.
- Decide whether to create `product-tree/example-server/troubleshooting` as a real intermediate branch node.
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
Improve the agent's ability to recognize redundancy, consolidation opportunities, and weak published outcomes.

Planned work:
- Move beyond duplicate-title detection to semantic overlap analysis.
- Compare against nearby peers in the active branch neighborhood.
- Prefer merge or consolidation proposals when overlap is high.
- Route low-signal or weak-content outcomes back to `review` with explicit recommended action.

Operational note:
- Add an explicit editorial publish mode for live KBs that lets Curio use inference to strengthen strong staged pages, publish the best ones, and deliberately leave a visible remainder in staged/review so the corpus still shows work in progress.

Operational note:
- Treat a large source space like SupportServer as a tuning corpus for the harness itself. Use repeated ingest/review/publish passes to discover missing branches, rewrite weak leaf pages, and codify recurring editorial decisions back into harness policy and tests.
- When a source corpus repeatedly produces valid pages in the same branch family, prefer adding or approving that branch family in `NORTHSTAR.md` before forcing individual pages through the wrong route.
- When a page repeatedly fails publish on quality, upgrade the page body first, then the proposal metadata, then the route. Do not treat the publish gate as the place to discover the editorial answer.
- Record the recurring branch families, rejection patterns, and publish-ready traits in the harness docs so future runs start from those learned patterns instead of rediscovering them.

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
- Maintain the hard contract in [curio-core-init.md](curio-core-init.md)
- Extend tests around reset, validation, malformed live trees, and stale managed pages.
- Continue reducing hidden state and ambiguous fallbacks.

## Phase 8: Confluence Feedback Loop ✅ (complete 2026-04-14)

Objective:
Close the round-trip so Confluence reviewer signals drive wiki state changes without requiring hand-edits.

Implemented:
- **E1** — `.sync-refs.json` sidecar written on sync, persisting `confluence_review_page_id` + `pinned_comment_id`.
- **E2** — `ConfluenceClient` extended with: `get_page_labels_v2`, `get_page_footer_comments`, `get_page_inline_comments`, `get_comment_reactions`, `create_footer_comment`, `update_footer_comment`.
- **E3** — `curio sync` posts a pinned reaction-instruction footer comment on every review page and persists the comment ID in the sidecar. Idempotent on re-sync.
- **E4** — `curio feedback [--dry-run]` command: reads labels + pinned-comment reactions + free-form comments; dispatches approve/reject/rewrite/capture; updates NORTHSTAR.md taxonomy on taxonomy-mutation approvals; appends to `_config/log.md`.
- **E5** — `curio process` manifest includes `reviewer_feedback` field when a `<slug>.feedback.md` sidecar exists, so the routing LLM sees prior reviewer commentary on resubmitted pages.

Signal map: `curio:approve` label or 👍 reaction → approve; `curio:reject` or 👎 → reject; `curio:rewrite` or ❓ → rewrite; free-form comments → `feedback.md` only.

## Phase 9: Confluence-Native Intake (next)

Objective:
Let Confluence users submit intake requests directly in the browser without needing CLI access.

Design:
- A **protected page template** in the CURIO Confluence space provides a structured form:
  `Source URL`, `Requested by`, `Priority` (high / normal / low), `Notes / context for the router`.
- Users create a page from this template under an **"Intake Requests"** parent page in the CURIO space.
- `curio pull` (new command) reads child pages of the Intake Requests parent:
  1. Skips pages already labelled `curio:processed`.
  2. Parses the structured fields from storage-format body.
  3. Runs the normal intake pipeline against `Source URL` (same as `curio intake --url`).
  4. Labels the request page `curio:processed`.
  5. Posts a footer comment on the request page: "Intake complete — review proposal at [Confluence review link]".
- Provenance: the request page ID is stored in the resulting wiki page frontmatter as `intake_request_ref`, so the source of the request is always traceable.
- `curio sync` can surface a "Pending Requests" count in the CURIO Confluence root page.

Planned work:
- Create the Confluence page template (storage-format XML) and register it in the CURIO space.
- Add `curio pull [--dry-run]` command that implements the poll-and-process loop above.
- Register `pull` in `cli.rs` and `main.rs`.
- Store `intake_request_ref: Option<String>` in `Frontmatter` (alongside existing `confluence_page_id`).
- Add `--from-queue` flag to `curio intake` as an alternative entry point if preferred over a separate subcommand.

Critical files:
- `curio-rs/src/commands/pull.rs` (new)
- `curio-rs/src/cli.rs`
- `curio-rs/src/main.rs`
- `curio-rs/src/lib.rs` (Frontmatter field)
- `curio-rs/src/confluence.rs` (add_label helper already exists; reuse)

## Phase 10: Slack Integration (planned)

Objective:
Surface Curio's review queue and curation workflow inside Slack so reviewers never have to leave their primary tool.

Design:

**Inbound (Slack → Curio):**
- `/curio intake <url>` slash command → triggers `curio intake --url` pipeline; replies with "Queued for intake — [n] pages found."
- `/curio ask <question>` → runs `curio query`; replies in-thread with the answer and source citations.
- `/curio status` → replies with current intake/staged/review/published counts.

**Outbound (Curio → Slack):**
- When `curio sync` creates new review proposals, post a digest to `#curio-review`:
  - Title, proposed path, confidence score, rationale excerpt.
  - Block Kit buttons: **Approve** / **Reject** / **Rewrite** (write the corresponding `curio:*` label to Confluence and trigger `curio feedback`).
- When `curio feedback` processes a batch, post a completion summary.

**Architecture:**
- Slack app with slash commands + interactive components (Block Kit).
- Webhook receiver (small HTTP server or AWS Lambda) translates button clicks into Confluence label writes + `curio feedback` invocations.
- OAuth for workspace install; tokens stored in `.env` / keyring alongside Confluence credentials.
- The Rust binary remains source-of-truth; Slack is just a UI surface — no business logic lives in the webhook receiver.

Planned work:
- Create Slack app manifest (slash commands, interactive endpoints, OAuth scopes).
- Add `CURIO_SLACK_TOKEN` + `CURIO_SLACK_CHANNEL` to config.
- Add `curio notify` command: takes a JSON payload describing the event, posts to Slack.
- Hook `curio sync` (end of review sync pass) and `curio feedback` (completion summary) to call `curio notify`.
- Build minimal webhook receiver (can be a separate binary in `curio-rs/src/bin/curio-slack-webhook.rs`).

Critical files:
- `curio-rs/src/commands/notify.rs` (new)
- `curio-rs/src/bin/curio-slack-webhook.rs` (new)
- `curio-rs/src/cli.rs`, `main.rs`
- `curio-rs/Cargo.toml` (add `axum` or `warp` for webhook receiver)

## Test/Dev Run Findings — 2026-04-14 (SupportServer ingest)

### What was run
- Space: `SupportServer` (company Confluence) — 152 pages
- All pages ingested, routed, reindexed, linted
- 8 new subtrees proposed under `product-tree/example-server`: api, mongodb, upgrade, authentication, administration, sql-db-persistence, high-availability, user-management

### What worked well
- Hub page synthesis: sparse Confluence pages with only children macros now get usable bodies
- Shallow-route gate: 1-component category routes get demoted to review correctly
- Taxonomy mutation proposals: every new subtree produces a clean `taxonomy_change` proposal with `proposed_new_subtree` and `node_description`
- Proposal dossier quality: full body preserved, rationale meaningful, overlap candidates listed
- Narrative log (`_config/log.md`) recording every pipeline action append-only
- 41 tests all green including new routing_eval integration harness
- Confluence feedback loop: pinned reaction comments posted on all review pages after sync; `curio feedback --dry-run` operational

### Persistent sync errors (pre-existing, not resolved)

These two pages error on every `curio sync` run and are skipped:

- **`orasi-labs.md`** — Confluence rejects with HTTP 400 `"Content contains unsupported extensions and cannot be edited in Fabric editor"`. The source page uses a Fabric-editor extension that the v1 storage-format API cannot represent. Options: (a) strip the offending extension during intake and re-intake; (b) suppress this page from sync and note it in `log.md`; (c) investigate which macro/extension is the culprit and add a suppression rule to `intake.rs`.

- **`how-to-use-vendor-engine-cmd-to-queue-a-job-from-a-bat-using-the-api.md`** — Confluence rejects with HTTP 400 `"A page already exists with the same TITLE in this space"`. The generated title `Review - How to use vendor-engine-cmd to queue a job from a BAT using the API` collides with a pre-existing page (likely from a prior partial sync or a manually created page). Options: (a) detect the collision and append a disambiguator suffix (e.g. `… (2)`); (b) find and delete the orphaned colliding page; (c) truncate titles above a safe length before sync to reduce collision surface.

### Open issues found
1. **Query returns 0 results until publish** — expected behavior but needs UX clarity. A `curio query` call against content sitting in `review/` finds nothing. Consider adding a `--include-review` flag or separate `curio review-query` for the curation workflow.
2. **Process runs 10 pages per invocation** — the `process --route-file` command processes in batches of 10 (appears to be a hard limit). For a 152-page ingest, 15+ re-runs were needed. Either raise the batch size or add a `--all` flag to drain the queue in one pass.
3. **Branch node descriptions not auto-propagated to NORTHSTAR.md** — after approving 8 new subtrees via review and adding them to NORTHSTAR.md manually, reindex does not validate that description_markdown is populated. The branch validation warns on reindex but only for published nodes, not for newly added taxonomy nodes.
4. **`proposed_new_subtree` path not normalized** — the route file used `"product-tree/example-server/upgrade"` (string), but northstar.rs expects a `Vec<String>` path. The Rust side appears to handle this but the gen-routes script should emit the array form.
5. **Merge proposal auto-population uses title similarity only** — the draft-HA overlap was caught, but the merge_target population is heuristic. A semantic embedding-based overlap pass would improve precision here.

### Next iteration priorities
1. Validate `curio sync` pinned comments landed correctly on all 153 review pages
2. Test `curio feedback --dry-run` against a page with a real label or reaction
3. Implement Phase 9 (Confluence-native intake template + `curio pull`)
4. Plan Phase 10 (Slack integration)

## Backlog Ideas

These are valid future directions but are not yet committed to a near-term build sequence.

- richer multi-source intake request model
- stronger taxonomy mutation workflow
- deeper proposal state machine
- automated healing suggestions for weak branch layouts
- ranking and prioritization of review proposals by expected signal gain
- `SourceProvider` trait and plugin pattern (Block G from prior design) — filesystem, GitHub wiki, Notion, SharePoint as additional sources
- Confluence metadata enrichment on intake: labels, owner, last-updated-by, attachments, word count, watcher count (Block F from prior design)
- `--include-review` flag or `curio review-query` so curation work has search visibility before publish
- `--all` flag (or configurable batch size) for `curio process --route-file` to drain the queue in one pass
