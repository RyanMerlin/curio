# Curio Processing Design

This document defines the intended end-to-end processing flow for Curio after initialization and setup are already complete.

This is not a CLI reference and not a code walkthrough. It is the technical operating design for how content should move through Curio, what information is collected, what the agent must infer, what the deterministic system must validate, what gets written to Git, and what gets mirrored to Confluence.

## Scope

This document starts at the moment new source material is introduced into Curio and ends with:

- curated publication into the Git repo
- curated mirroring into Confluence
- later review / sharpening proposals for restructuring the knowledge base

This document does not cover:

- workspace bootstrap
- auth setup
- environment wiring
- initial Confluence root creation

## Core Model

Curio has two different surfaces with different responsibilities.

### Git Repo

The Git repo is the system of record.

It stores:

- all intake content
- all staged content
- all review content
- all published content
- `_config` source of truth
- audit trail
- local navigation aids
- proposal artifacts
- machine-maintained state that must be visible and reviewable in Git

The Git repo may contain machine-oriented helper artifacts and operational structure.

### Confluence

Confluence is the curated human-facing mirror.

It should contain only intentional, human-meaningful structure and pages.

It should not contain:

- audit logs
- raw machine state
- repo-only helper clutter
- accidental duplicate pages
- low-signal placeholders

Curio sync is one-way:

- Git repo -> Confluence

Confluence is not the source of truth.

### Review visibility rule

Proposal state may be stored in Git, but proposals that require human review must also surface in Confluence under `Review`.

Confluence `Review` is the human operational review surface for:

- subtree proposals
- merge / split proposals
- deduplication proposals
- deletion candidates
- low-signal content decisions
- other curation questions awaiting judgment

## Pipeline Stages

The main content pipeline is:

1. `intake`
2. `staged`
3. `review`
4. `published`
5. `sharpening`

`sharpening` is not a forward stage in the same sense as the others. It is a periodic review function over existing curated content.

## Stage 1: Intake

### Goal

Capture new source material into Curio in a consistent structure without pretending that it is already curated.

### Possible source types

Intake may begin from:

- a Confluence page
- a URL/web page
- a local file
- pasted or generated text
- other future structured connectors

### Required intake record

Every intake item should be normalized into a content record with these conceptual fields:

- source identity
- source kind
- source location
- source title
- source summary if extractable
- body or content reference
- capture timestamp
- provenance metadata
- initial subject hints if available

### Intake structure

The intake artifact should preserve source provenance clearly.

Conceptually it needs:

- a stable Curio page ID
- source metadata
- raw or summarized content depending on source kind
- enough extracted text for routing and later review
- a clear indication that this is uncurated intake

### Intake behavior by source kind

#### Confluence page source

Curio should collect:

- original page title
- original page URL
- page ID / source identifier
- summary extract
- readable body content or reference-card body, depending on source policy

The goal is not to flatten the source beyond usefulness. The item should remain attributable to the source page.

#### URL source

Curio should collect:

- canonical URL
- page title if extractable
- summary extract
- readable content or captured excerpt

#### File source

Curio should collect:

- file path or file reference
- detected media/content type
- extracted content if feasible
- a summary or preview

### Intake checks

Before the item is accepted into intake, deterministic checks should verify:

- the source can be read or fetched
- the intake item is not an exact duplicate of an already-tracked source artifact when duplicate detection is possible
- required provenance fields are present
- the content is non-empty enough to be processed

If these checks fail, the item should not silently proceed.

## Stage 2: Routing Analysis

### Goal

Determine what the content is mainly about, where it belongs in the curated taxonomy, and whether it is ready for `staged` or requires `review`.

This is the most important agent judgment step in Curio.

### Inputs to routing

The routing decision should consider:

- title
- dominant content topic
- source summary
- extracted body content
- provenance
- existing taxonomy in `_config/northstar.md`
- existing published corpus when necessary for disambiguation

### Primary routing principle

Routing must be based on:

- title
- dominant content topic

Secondary or incidental mentions are weak evidence.

The system must not over-weight:

- side mentions
- prerequisite mentions
- uninstall or compatibility side notes
- incidental references to adjacent products

### What the agent must infer

The agent must infer:

- primary topic
- best existing tree
- best existing subtree
- whether the item is truly about that subtree or only mentions it
- whether the current title is human-meaningful
- whether the content is specific enough to publish eventually
- whether the content conflicts with existing knowledge
- whether the content suggests a new subtree is needed

### Required routing output

The routing analysis should produce a structured decision with at least:

- title
- proposed status
- proposed category path
- confidence
- rationale
- notable evidence
- review reason if not stageable
- new-subtree proposal if needed

### Routing statuses

The agent may route an intake item to:

- `staged`
- `review`

It must not route directly to `published`.

### Stageable content

A page can move to `staged` when:

- the dominant topic is clear
- an existing tree and subtree fit confidently
- the content is coherent enough to preserve
- the title is acceptable or can be refined safely

### Review-required content

A page must move to `review` when:

- the dominant topic is ambiguous
- multiple subtrees are plausible
- the current taxonomy has no good fit
- the source is low quality or contradictory
- a likely duplicate exists
- the item should trigger consolidation rather than new publication
- a new subtree appears necessary
- information quality is too low to justify publication
- usability for a human reader is too low to justify publication

## New Subtree Proposal Behavior

This is a critical rule.

There is no valid `published/uncategorized` outcome.

If the agent cannot confidently fit content into an existing subtree, the content must:

- go to `review`
- include a proposal for a new subtree

That proposal should include:

- candidate parent tree
- candidate subtree title
- candidate subtree slug
- rationale
- why existing subtrees do not fit
- source evidence
- confidence
- expected impact on future curation

This is how Curio scales. It does not solve taxonomy gaps by publishing uncategorized content.

The subtree proposal must also be visible in Confluence `Review` so a human can evaluate it there.

## Stage 3: Staged

### Goal

Hold content that has been routed confidently enough to preserve in a proposed curated location, but is not yet approved for final publication.

### What should be written into staged

The staged artifact should include:

- normalized title
- preserved provenance
- category path
- confidence
- rationale
- keywords
- cross references if known
- body content or reference-style content appropriate to the source type

`staged` is for proposed curation drafts, not invisible shortcuts to `published`.

### What staged means

`staged` means:

- Curio believes it knows where this belongs
- the item is promising enough to keep moving
- a human or a later agent step can still refine it before publish

### Deterministic checks before allowing staged

The system should validate:

- category path exists in current `northstar`
- status is valid
- title is non-empty
- frontmatter is structurally valid
- content file can be written into the proposed location

If these fail, the item should be diverted to `review`.

## Stage 4: Review

### Goal

Hold items that require human or higher-judgment agent attention before they can safely become curated knowledge.

### Reasons content lands in review

Common reasons:

- ambiguous routing
- suspected duplication
- low confidence
- taxonomy gap
- poor title quality
- poor content quality
- conflicting evidence
- possible need to merge into existing published content

### Review artifact requirements

A review item should make the unresolved issue explicit.

It should include:

- proposed status or disposition
- rationale
- unresolved questions
- source evidence
- category candidates if any
- subtree proposal if applicable
- duplicate suspicion if applicable
- information quality assessment
- usability assessment
- recommended action if the item is too weak to publish

Review artifacts that humans need to inspect should be mirrored into Confluence `Review`, not kept only as Git-local proposal files.

### Review outcomes

Review should resolve into one of:

- approve route to `staged`
- rewrite and then stage
- merge with an existing page
- reject / discard
- create or approve a new subtree proposal

## Stage 5: Publish

### Goal

Move curated content from working state into the published knowledge corpus in Git.

### Publish is not raw promotion

Publishing is not just file movement.

At publish time Curio must confirm:

- the content is intentionally titled
- the route is correct
- the page is useful enough to deserve published status
- the page is not a likely duplicate that should instead merge into existing knowledge
- the information quality is high enough to be useful
- the page is usable by a human reader without requiring major reconstruction

### Required publish checks

Before publish, deterministic checks should validate:

- category path exists
- target location is valid
- title is present
- frontmatter is complete enough
- the content is not trying to publish into a nonexistent subtree
- the content is not low-signal placeholder material
- the content clears the minimum information quality / usability gate

### Publish quality gate

Publish should depend on more than routing confidence.

Curio should evaluate at least these dimensions:

- routing confidence
- taxonomy fit
- information quality
- human usability
- duplication risk

Low confidence can block publication, but high confidence alone must not allow publication of weak content.

Examples of content that should be rejected back to `review`:

- placeholder pages with almost no real information
- pages that repeat obvious fragments without usable guidance
- pages that are likely duplicates of stronger existing pages
- pages that need consolidation or deletion more than publication

### Duplicate-title policy

Duplicate titles in published are a curation risk signal.

Default rule:

- the agent should avoid creating them through good curation

If a duplicate title still occurs, Curio may use the explicit duplicate fallback:

- publish the new page as `"{title} (dup)"`
- add a visible duplicate notice
- include a reference to the conflicting page

This is an exception path, not the normal curation model.

### Published output in Git

Published Git content should be:

- human-meaningful
- routed into the correct tree/subtree path
- stable enough to mirror to Confluence
- free of placeholders and junk nodes

## Published Tree Design

The published tree is defined by `_config/northstar.md`.

### Tree semantics

Top-level tree pages represent durable navigation domains.

Subtree pages represent narrower curated domains inside those trees.

Published leaf pages live under those nodes.

### When a new branch node is needed

If a topic cluster becomes dense enough that flat pages are no longer navigable, the agent should propose a new branch node.

Example pattern:

- existing flat sibling set becomes too large or too repetitive
- the agent identifies a stable organizing dimension
- the agent proposes a new subtree node
- child pages move under it
- the new branch page becomes the landing page and index for those children

This is the correct scaling pattern for 100s or 1000s of pages.

### Branch node behavior

A real branch node is not a useless summary page.

It should contain:

- a clear description of the topic area
- navigation to its child pages
- optional guidance for how to use that section

It should serve as the human index for that branch.

## Reindexing and Local Navigation

Git may contain co-located `index.md` files to support local navigation.

Those indexes are useful on the Git side because they:

- summarize the contents of a folder
- make tree structure visible in-repo
- help the agent understand local topology

However, not every Git `index.md` should automatically become a Confluence page.

The correct rule is:

- only intentional branch indexes should mirror to Confluence
- raw machine-helper indexes should remain Git-only

That distinction matters because Confluence must stay curated.

## Sync to Confluence

### Goal

Mirror the curated Git-published corpus into a human-facing Confluence tree.

### Confluence write model

Curio writes only under the managed `CURIO` root page inside the configured Confluence space.

Under `CURIO`, the visible structure is:

- `Published`
- `Intake`
- `Staged`
- `Review`
- `Config`

### What should sync

Confluence should receive:

- curated published branch pages
- curated published leaf pages
- human-meaningful lane pages where appropriate
- config reference pages under `Config`

### What should not sync

Confluence should not receive:

- audit trail
- sharpening proposals
- raw helper files
- machine-only indexes
- low-signal placeholders
- repo-only operational artifacts

### Sync checks

Before or during sync, Curio must validate:

- all published pages have a valid route
- all synced pages are intended human-facing content
- parent-child placement matches the Git tree
- write target is inside the managed Confluence root

### Confluence branch page rule

If a Git node represents a real curated branch, Confluence should get a corresponding branch page.

That branch page should:

- live in the correct place in the page tree
- include navigational index content for child pages

Without this, the Confluence tree becomes unusable as the corpus grows.

### Stale page cleanup

The managed Confluence subtree must be pruned when pages are removed or relocated in Git.

Otherwise stale pages accumulate and the mirror stops representing the repo.

The `Published` subtree should be treated as authoritative under the Curio-managed root.

## Audit Trail

Curio maintains a Git-tracked audit trail at `wiki/.curio/audit.jsonl`.

### Purpose

The audit trail records operational events such as:

- sync actions
- major pipeline changes
- exceptional actions

### Constraints

Audit should be:

- Git-visible
- compacted periodically
- never mirrored to Confluence

## Self-Sharpening

### Goal

Review existing curated knowledge and propose structural or content improvements without auto-applying them.

This is a periodic curation improvement loop, not the same as intake routing.

### What sharpening should look for

Sharpening should detect:

- likely duplicate pages
- near-duplicate pages
- oversized pages that should split
- fragmented clusters that should consolidate
- weak titles
- weak routing
- low-signal pages that should be demoted or removed
- clusters that justify a new subtree

### What sharpening produces

Sharpening should produce proposals, not direct automatic mutations.

A proposal should include:

- proposal type
- affected pages
- rationale
- evidence
- confidence
- expected benefit

Sharpening proposals may be stored in Git, but proposals requiring human review should also be represented in Confluence `Review`.

### Important scaling rule

When a topic cluster becomes too broad, the agent should not create:

- one useless summary plus many siblings

It should propose:

- a real new branch node
- child pages beneath it
- a usable branch index

That is the only scalable pattern for large corpora.

## End-to-End Decision Flow

This is the intended operational sequence after content enters Curio.

1. New source material is captured into `intake` with provenance.
2. Deterministic intake checks confirm the source is usable.
3. The agent analyzes title, dominant topic, and content.
4. The agent selects the best existing tree/subtree or determines that no existing subtree fits.
5. The agent emits a structured routing decision.
6. Deterministic validation checks the decision against `northstar`.
7. If the route is valid and confidence is sufficient, the item moves to `staged`.
8. The item must also clear information quality and usability thresholds before it is eligible to move toward publication.
9. If the route is ambiguous, the quality is weak, or no good subtree exists, the item moves to `review`.
10. If no subtree fits, the review item includes a new-subtree proposal.
11. Review items and proposals that humans need to inspect should surface in Confluence `Review`.
12. A human or higher-judgment curation pass resolves review items.
13. Eligible staged content is published into the Git `published/` tree.
14. Reindex updates local Git navigation.
15. Sync mirrors the curated published tree to Confluence.
16. Stale Confluence descendants are removed from the managed subtree.
17. Periodic sharpening analyzes the published corpus and emits improvement proposals.

## Non-Negotiable Rules

The following rules are critical.

### Rule 1

There is no valid published uncategorized content.

### Rule 2

Routing is based primarily on title plus dominant content topic.

### Rule 3

Incidental mentions are weak evidence.

### Rule 4

Confluence is curated output, not a mirror of all repo noise.

### Rule 5

If the tree is insufficient, the agent must propose a new branch node rather than forcing bad publication.

### Rule 6

Real branch nodes in the tree must have a usable index / landing page in Confluence.

### Rule 7

Duplicate titles are a curation warning signal, not a normal operating condition.

### Rule 8

The Confluence managed subtree must be pruned so the mirror stays faithful to Git.

### Rule 9

Do not shortcut new curation or structural curation directly into `published`. Proposed changes must pass through `staged` or `review` first unless the user explicitly authorizes a manual override.

### Rule 10

Anything a human must review operationally should be visible in Confluence `Review`, not only in Git.

### Rule 11

Low-signal content must be rejected back to `review` for improvement, consolidation, or deletion rather than being published with a weak confidence-only justification.

## Open Design Follow-Ups

These are the next technical design items that follow from this document.

1. Define the exact structure of the routing analysis artifact.
2. Define the exact structure of the new-subtree proposal artifact.
3. Distinguish explicitly between Git-only helper indexes and Confluence branch indexes.
4. Implement branch-node publishing so subtree landing pages in Confluence include their child index.
5. Tighten duplicate detection so duplicate-title fallback becomes rare.
6. Add a density / cluster threshold that triggers branch-node proposals earlier.
7. Define the minimum information quality / usability gate for publish decisions.
