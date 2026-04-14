# Curio Processing Design

This document defines the intended end-to-end processing flow for Curio after initialization and setup are already complete.

This is the technical design for how Curio should transform incoming information into the best evolving structure for the knowledge base. It is not a CLI reference and not a code walkthrough.

## Core Point

Curio is not a page router.

Curio is an information transformation system.

Hierarchy is the primary optimization target.

The agent should be biased strongly toward the best hierarchical transformation of information, not the easiest shallow route match.

From the beginning of intake to final publication and later sharpening, the agent must continuously ask:

- what information is present
- how trustworthy and useful it is
- how it relates to the existing knowledge base
- whether it belongs in an existing node
- whether it should merge into an existing page
- whether it justifies a new node in the hierarchy
- how to express the result in the best structure for long-term retrieval and maintenance
- what hierarchy this information should live under
- whether the existing hierarchy is sufficient
- whether the information should create one or more new intermediate nodes before a leaf page is created

New content is never curated in a vacuum. It must always be interpreted relative to the existing knowledge structure.

## Scope

This document starts at the moment new source material is introduced into Curio and ends with:

- curated publication into the Git repo
- curated mirroring into Confluence
- later sharpening proposals for restructuring or consolidating the knowledge base

This document does not cover:

- workspace bootstrap
- auth setup
- environment wiring
- initial Confluence root creation

The deterministic Confluence tree contract for `curio init` lives in [curio-core-init.md](C:/code/agents/curio/docs/design/curio-core-init.md).

## Core Model

Curio has two different surfaces with different responsibilities.

### Git Repo

The Git repo is the system of record.

It stores:

- intake requests
- staged proposals
- review proposals
- published pages
- `_config` source of truth
- audit trail
- local navigation aids
- proposal dossiers
- machine-maintained state that must be visible and reviewable in Git

Git is where the full evidence trail lives.

### Confluence

Confluence is the curated human-facing mirror and the human review surface.

It should contain:

- intentional hierarchy
- curated published pages
- staged proposals that are human-meaningful to inspect
- review proposals that require human judgment
- proposal summaries and rationale when humans must evaluate them

It should not contain:

- raw machine state
- repo-only helper clutter
- accidental duplicate pages
- low-signal placeholders
- opaque machine artifacts with no human review value

Curio sync is one-way:

- Git repo -> Confluence

Confluence is not the source of truth.

## Taxonomy Source of Truth

Taxonomy should be driven by `wiki/_config/northstar.json`.

That JSON should contain:

- the hierarchical node tree
- node identifiers / slugs
- display titles
- descriptions
- any durable metadata needed to validate category paths

`wiki/_config/northstar.md` should remain as the narrative / explanatory wrapper and the source for the Confluence-facing `Northstar` page body.

The JSON drives structure.

The Markdown drives human explanation.

## Main Concepts

### Intake Request

An intake request is the unit of incoming work.

One intake request may contain one or more sources, for example:

- multiple Confluence pages
- one or more URLs
- one or more files or documents
- pasted text
- a human description of the problem or desired knowledge change

Curio must accept as many source types as practical.

### Source

A source is an individual input artifact inside an intake request.

Examples:

- a Confluence page
- a URL
- a PDF
- a markdown file
- a screenshot
- a spreadsheet
- a human-written free-text prompt

### Proposal

A proposal is the core curation unit produced from an intake request.

A proposal is not just “some notes.”

It is a full curation candidate that contains two tightly coupled parts:

1. the proposed page or proposed page-tree change
2. the supporting dossier that explains why that proposal is correct

The supporting dossier should contain:

- all contributing sources
- what was fetched / opened / sampled / inspected
- what alternatives were considered
- what existing pages were compared
- what overlap / merge / deduplication checks were run
- the route evaluation
- the hierarchy evaluation
- confidence metrics
- quality / usability metrics
- dates and timestamps of evaluation
- explicit rationale for the proposed change

An intake request may result in:

- one proposal
- multiple proposals
- no publishable proposal, only a rejection / consolidation recommendation

This is an inference problem. The agent must determine whether multiple sources belong together or should produce separate proposals.

### Proposed Page

Each proposal should include a proposed page artifact.

That page is the candidate knowledge object that would eventually land in `published` if approved.

The top of the page should include a structured information section with:

- rationale
- confidence metrics
- quality / usability metrics
- route / hierarchy decision
- related sources and references

The synthesized knowledge content should appear below that decision section.

### Dossier

The dossier is the verbose support record behind the proposal.

It is how Curio remains explainable and reviewable.

## Pipeline Stages

The main content pipeline is:

1. `intake`
2. `proposal generation`
3. `staged`
4. `review`
5. `published`
6. `sharpening`

`proposal generation` is conceptually separate even if parts of it are implemented inside other commands.

### First-move rule

`published` is never the first move for new intake or new curation.

The first durable output of new work must be:

- `staged`
- or `review`

Direct creation or reshaping of `published` content is only valid as an explicit user-authorized manual override.

## Stage 1: Intake

### Goal

Capture one or more sources into Curio in a structure that preserves provenance and supports later synthesis.

### Intake acceptance

Curio should accept as many information types as practical, including:

- Confluence pages
- URLs
- local files
- folders of files
- documents
- human text descriptions
- future structured connectors

The agent should be able to process files and documents directly, but the intake layer must preserve them faithfully enough for that later agent work.

### Required intake record

Every intake request should normalize into a request record with:

- request identity
- request timestamp
- source list
- optional human prompt / subject description
- provenance metadata
- capture status

Each source inside the request should preserve:

- source identity
- source kind
- source location
- source title if available
- source summary if available
- source body reference
- source capture timestamp
- source provenance metadata

### Intake structure

Intake must preserve sources as sources.

The point is not to flatten anything.

The point is to preserve enough hierarchical structure and provenance so the agent can later synthesize the right knowledge object or knowledge-tree change.

### Intake behavior by source kind

#### Confluence source

Curio should preserve:

- original title
- origin URL
- page ID or source identifier
- source body reference
- source structure as faithfully as practical

#### URL source

Curio should preserve:

- canonical URL
- title if available
- source body reference
- any useful page metadata

#### File / document source

Curio should preserve:

- file path or file reference
- media / content type
- file metadata
- reference to the source artifact

Content extraction and summarization are agent responsibilities, not Curio substrate responsibilities.

### Intake checks

Before accepting an intake request, Curio should verify:

- the sources can be opened, fetched, or referenced
- required provenance fields are present
- the sources are not empty or broken in a way that makes analysis impossible
- the request is not an obvious exact duplicate of an existing intake request when that can be determined safely

“Usable” here means the sources can actually be inspected by the later curation step.

If these checks fail, the request should not silently proceed.

## Stage 2: Proposal Generation

### Goal

Convert one intake request with one or more sources into one or more proposed knowledge changes.

This stage must be explicitly hierarchy-first.

The main drive of the agent is not route matching. It is structural transformation of information into the best hierarchy for long-term retrieval, maintenance, and future curation.

The agent must not dump material into the first acceptable existing category. It must first do enough semantic analysis to determine:

- the dominant topic
- the surrounding branch neighborhood
- the likely intermediate nodes required
- the right leaf breakdown
- whether an existing path is sufficient
- whether new intermediate nodes are needed before any leaf page should exist

When the information is technical and detailed, the default bias is toward deeper structure, not flatter placement.

For example, if the agent reads multiple sources about Alteryx Server installation issues involving ODBC driver behavior across versions `23.2`, `24.2`, and `25.1`, the target proposal should lean toward a hierarchy such as:

- `Product-tree`
- `Alteryx Server`
- `Installation`
- `Troubleshooting`
- `Versions`

with one or more version-family leaf pages beneath that branch, depending on signal and natural clustering.

The correct output is often a proposed path tree, not just a proposed page.

### Agent responsibilities

The agent should:

- inspect all provided sources
- inspect the existing knowledge structure
- inspect relevant `index.md` summaries and metadata intentionally
- use the index structure aggressively and recursively as the primary efficient discovery mechanism
- compare against existing peer pages
- determine whether sources belong together or apart
- keep searching the existing hierarchy until it believes it has found the relevant nearby structure, peer pages, and likely overlap targets
- decide whether the result is:
  - a new page
  - an update to an existing page
  - a merge proposal
  - a split proposal
  - a new hierarchy-node proposal
  - a discard / delete recommendation

This is not page routing alone. This is interpretation and synthesis.

This must be hierarchy-first synthesis.

The agent should prefer:

- the best branch path for the information
- the right intermediate node structure
- the right leaf breakdown

over:

- dumping the content into the first acceptable existing subtree
- creating a flat list of pages under a shallow category
- matching on obvious keywords and stopping there

### Existing knowledge structure input

Proposal generation must consider:

- `northstar.json`
- branch-node descriptions
- co-located `index.md` summaries and metadata
- relevant peer pages in the likely neighborhood

The agent should use the index recursively:

1. start from the root and likely branch nodes
2. walk down through candidate branch indexes
3. inspect nearby branch nodes and leaf pages
4. continue until it believes it has found the relevant surrounding structure and overlaps

Only after that recursive structure walk should it finalize the dossier and make the curation judgment call.

The agent should prefer intentional structural context over blind word search.

### Primary proposal dimensions

The agent must evaluate at least:

- dominant topic
- value of information
- route fit
- depth of fit in the hierarchy
- semantic overlap with peer pages
- whether the information belongs in a new or existing node
- whether a merge or consolidation is better than a new page
- whether the information should create a deeper branch path before any leaf page is created
- whether a cluster of version-, scenario-, or subtopic-specific leaves should live under a new intermediate node

### Required proposal outputs

Each proposal should contain:

- proposed page title
- proposed hierarchical path
- proposed full hierarchy path tree to the data, not just the nearest shallow category
- proposed status
- proposed page body
- source list
- rationale
- alternatives considered
- merge target if applicable
- new-node proposal if applicable
- confidence metrics
- quality metrics
- usability metrics
- timestamp of evaluation

## Confidence and Scoring

Confidence must not be a single scalar.

Curio should track multiple confidence / scoring dimensions, including:

- route confidence
- quality confidence
- hierarchy-fit confidence
- merge / deduplication confidence
- evidence completeness
- date / freshness of evaluation

These scores support judgment. They do not replace judgment.

## Stage 3: Staged

### Goal

Hold proposals that are strong enough to preserve as candidate curated outcomes, but are not yet final published knowledge.

### Meaning of staged

`staged` means:

- Curio has a concrete proposed page or hierarchy change
- the proposal is coherent and worth preserving
- it may still need human approval or later refinement before publish

### Staged contents

A staged proposal should include:

- the proposed page content
- the structured decision section at the top
- provenance and source references
- route / hierarchy path
- all confidence and quality scores
- rationale
- comparison with nearby pages where relevant

### Staged checks

The system should verify:

- the proposal record is structurally valid
- required provenance exists
- the proposed hierarchical path is valid against `northstar.json`

If the path does not exist in the taxonomy, Curio should not pretend the route is already valid.

Instead, the proposal should explicitly include:

- the taxonomy mutation being proposed
- the new node to add to `northstar.json`
- the corresponding Git path / page-tree change

That may still land in `staged` if confidence is high and the proposal is complete, but it becomes a taxonomy-change proposal rather than a simple page-route proposal.

## Stage 4: Review

### Goal

Hold proposals that require human judgment or a higher-confidence curation pass.

### What sends a proposal to review

A proposal must go to `review` when:

- the dominant topic is ambiguous
- multiple hierarchical paths are plausible
- the evidence for route fit is weak
- the best action is merge / consolidation rather than simple publication
- the taxonomy needs a new node and the justification is not trivial
- semantic overlap with existing peer pages is high
- the information quality is low
- the human usability is low
- the agent lacks enough context to make a defensible decision

### Clear distinction for review

“Awaiting judgment” should mean one of three concrete things:

1. the agent lacks enough context
2. the agent has multiple plausible outcomes with insufficient separation
3. the proposal changes knowledge structure or published content in a way that needs explicit human approval

That distinction should be stated directly in the review artifact.

### Review artifact requirements

A review artifact must include:

- the proposed page or hierarchy change
- the full supporting dossier
- route alternatives and why they were rejected
- merge / overlap analysis if relevant
- taxonomy proposal if relevant
- full traceability to all sources
- all scoring dimensions
- explicit unresolved questions
- explicit recommended action

### Confluence review behavior

Anything a human must review must surface in Confluence `Review`.

That includes:

- page proposals
- merge proposals
- split proposals
- taxonomy / hierarchy proposals
- low-signal rejection candidates
- consolidation recommendations

If a human cannot see it in the review surface, it is operationally useless.

## New Node Proposal Behavior

There is no valid `published/uncategorized` outcome.

If the existing hierarchy has no strong fit, the proposal must include a new-node proposal.

That proposal must include:

- proposed parent path
- proposed new node title
- proposed new node slug / identifier
- node description
- rationale
- why nearby existing nodes do not fit
- source evidence
- confidence
- expected impact on future curation

The taxonomy change should target `northstar.json`, and the corresponding Git path and branch page should be part of the same proposal.

## Stage 5: Publish

### Goal

Move approved staged proposals into the published knowledge corpus in Git.

### Publish is not raw promotion

Publishing is not just file movement.

At publish time Curio must confirm:

- the proposal is approved
- the title is intentional and fits the peer neighborhood
- the route is correct
- the hierarchy placement is correct
- the information quality is high enough
- the page is usable for a human reader
- semantic overlap with peer pages is low enough, or a merge decision has already been made

### Publish checks

Before publish, Curio should verify:

- the target hierarchical path exists in `northstar.json`, or the approved taxonomy mutation has already been applied
- the target Git path exists or is created as part of the approved change
- the proposal is not low-signal placeholder material
- the proposal clears the minimum quality / usability gate
- the peer-level overlap check does not indicate a likely merge instead

### Overlap rule

Duplicate detection is not enough.

Curio should look for semantic overlap with nearby peer pages.

If overlap is high enough that a new page would create redundant knowledge, the correct outcome is:

- merge proposal
- consolidation proposal
- or return to review

not “publish another page and hope for the best.”

## Published Output in Git

Published Git content should be:

- human-meaningful
- hierarchically well placed
- traceable
- free of low-signal placeholders
- stable enough to mirror to Confluence

## Published Tree Design

The published tree should be defined by `wiki/_config/northstar.json`.

The hierarchy may have an arbitrary number of nested levels.

It must not be limited conceptually to only one tree plus one subtree layer.

### Tree semantics

Every node in the hierarchy represents an intentional knowledge container.

Leaf pages live beneath those nodes.

### Node creation principle

New nodes should not be proposed because “the stack got too tall.”

New nodes should be proposed because they are the best transformation of the information into the right structure for the knowledge base.

The main drive for the agent should be toward meaningful hierarchy.

Default toward deeper paths for technical-detail documents.

If content is:

- operational
- version-specific
- scenario-specific
- troubleshooting-specific
- implementation-specific
- narrowly procedural

then the agent should prefer a deeper hierarchy path or propose new intermediate nodes by default.

Only keep content at a higher level when it is clearly:

- broad
- cross-cutting
- intentionally overview-oriented
- serving as a branch-level landing page

If the information naturally implies:

- product -> lifecycle phase -> issue family -> version family
- platform -> workflow area -> task type -> scenario
- domain -> concept -> subtopic -> leaf

then the proposal should capture that hierarchy explicitly instead of flattening the result into the first acceptable existing branch.

The agent must continuously reevaluate:

- what new information exists
- how it changes the existing structure
- whether a better node arrangement is now justified

### Branch node behavior

A real branch node is not a useless summary page.

It should contain:

- what the node is
- why the node exists
- the index of child pages
- short descriptions of child pages
- guidance for how the section should be used

Branch nodes need deliberate validation and heal behavior so they never become empty shells.

The agent should treat branch nodes as first-class knowledge objects.

They are not filler.

They are how the knowledge base scales and stays understandable.

## Reindexing and Navigation

Git may contain co-located `index.md` files for local navigation.

However, the Confluence requirement is stronger:

- every branch page in the Confluence tree must describe what that page is
- every branch page must show its child index
- every child in that index should have a brief description

The distinction is:

- machine-only helper indexes stay out of Confluence
- intentional branch indexes must be rendered into the parent branch page in Confluence

Curio needs validation and healability for this.

## Sync to Confluence

### Goal

Mirror the curated Git state into a human-facing Confluence tree.

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
- staged proposals that humans need to inspect
- review proposals that humans need to inspect
- branch descriptions and child-page indexes
- config reference pages under `Config`

### What should not sync

Confluence should not receive:

- raw helper files
- machine-only indexes
- low-signal placeholders
- repo-only operational artifacts that humans do not need to inspect

### Sync checks

Before or during sync, Curio must validate:

- all synced pages are intended human-facing content
- parent-child placement matches the approved hierarchy
- every branch page has a description
- every branch page has a child index with brief descriptions
- write target is inside the managed Confluence root

### Stale page cleanup

The managed Confluence subtree must be pruned when pages are removed or relocated in Git.

Otherwise the mirror stops representing the repo.

## Audit and Proposal Storage

Audit and proposal state should live under `_config`, not under `wiki/.curio`.

Target locations should be:

- `wiki/_config/audit.jsonl`
- `wiki/_config/sharpening-proposals/`

`wiki/.curio` should be eliminated.

Audit should remain Git-tracked and should never mirror to Confluence.

Proposals should remain Git-tracked and should also surface into Confluence `Review` when human review is needed.

## Self-Sharpening

### Goal

Review the existing knowledge base and propose improvements without auto-applying them.

### What sharpening should detect

Sharpening should detect:

- semantic duplicates
- near-duplicates
- merge opportunities
- split opportunities
- weak titles
- weak routes
- weak hierarchy
- low-signal pages
- opportunities for better node structure

Sharpening, healing, doctoring, and merge flows must all recurse through the existing branch/index structure in the same way:

- understand the current hierarchy
- understand the child index at each branch
- detect overlap, inconsistency, low-signal pages, and meaningless branches
- propose the best transformed hierarchy as a cohesive whole

### Sharpening outputs

Sharpening produces proposals, not direct mutations.

All such proposals must go to `Review`.

They may also be stored in Git as durable proposal records.

## End-to-End Flow

1. One intake request is created with one or more sources.
2. Curio verifies that the sources are usable for analysis.
3. The agent inspects the sources and the existing knowledge structure.
4. The agent decides how many proposals should result from the intake request.
5. For each proposal, the agent synthesizes:
   - the proposed page or hierarchy change
   - the supporting dossier
6. The agent evaluates route fit, hierarchy fit, overlap, quality, usability, and value of information.
7. Strong proposals move to `staged`.
8. Ambiguous, weak, structural, or approval-requiring proposals move to `review`.
9. Taxonomy mutations produce explicit node proposals targeting `northstar.json`.
10. Humans review the necessary staged / review items in Confluence and Git.
11. Approved staged items publish into the Git `published` tree.
12. Reindex updates local navigation.
13. Sync mirrors published, staged, and review surfaces into Confluence.
14. Stale Confluence descendants are removed.
15. Sharpening periodically creates new review proposals for consolidation or restructuring.

## Non-Negotiable Rules

### Rule 1

There is no valid published uncategorized content.

### Rule 2

Routing and curation are based on title, dominant content topic, value of information, and the existing knowledge structure.

Hierarchy is the main design objective.

### Rule 3

Incidental mentions are weak evidence.

### Rule 4

Confluence is curated output and human review surface, not a mirror of repo noise.

### Rule 5

If the hierarchy is insufficient, the agent must propose a new node rather than forcing bad publication.

### Rule 6

Real branch nodes must have usable descriptions and child indexes in Confluence.

### Rule 7

Semantic overlap is a curation warning signal, not just exact duplicate titles.

### Rule 8

The Confluence managed subtree must be pruned so the mirror stays faithful to Git.

### Rule 9

`published` is never the first move. Do not shortcut new curation or structural curation directly into `published`. Proposed changes must pass through `staged` or `review` first unless the user explicitly authorizes a manual override.

### Rule 10

Anything a human must review operationally must be visible in Confluence `Review`.

### Rule 11

Low-signal content must be rejected back to `review` for improvement, consolidation, or deletion rather than being published with a weak confidence-only justification.

## Open Design Follow-Ups

1. Define the exact schema for intake requests with multiple sources.
2. Define the exact schema for proposal dossiers.
3. Define the exact schema for `northstar.json`.
4. Implement branch-page child indexes and descriptions in Confluence as a first-class validated feature.
5. Replace title-only duplicate logic with stronger semantic overlap checks.
6. Move audit and proposal storage under `_config` and remove `wiki/.curio`.
7. Add explicit validation / healing for branch-node descriptions and child indexes.
