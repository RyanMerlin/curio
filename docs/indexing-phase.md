# Curio Indexing Phase

This document describes the next routing/indexing step for Curio.

## Goal

Curio should behave like an indexed datastore inside Confluence, not like a flat page dump.
The key idea is that every major branch page should expose a compact child index so agents can move
down the tree one hop at a time instead of searching the whole space.

## Core Flow

1. Intake
   - capture the source page, file, or folder
   - normalize source text and preserve source provenance
   - write the intake artifact into Confluence

2. Proposal
   - infer the intended hierarchy for the page
   - record the target path, registry path, rationale, and validation requirements

3. Staged
   - write a Confluence-native staged artifact
   - include a concise summary, source link, source excerpt, and next actions
   - preserve enough structure for a human reviewer and an agent reader

4. Registry
   - write a canonical record for each active Curio artifact
   - keep registry pages hierarchical
   - expose a compact mini-index on each branch page

5. Publish
   - validate the target branch against sibling pages and the current target state
   - move approved content into the canonical Published tree

6. Audit
   - append an immutable record of the action
   - capture who, what, why, source references, and outcome

## Registry Design

The registry should not be a flat metadata shelf.
Instead, each branch page should summarize its direct children and provide links to them.

Recommended branch layout:

- `Config`
  - branch root for `NORTHSTAR`, `CURIO Readme`, and `settings.yaml`
- `Published/index.md`
- `Published/{tree}/index.md`
- `Published/{tree}/{subtree}/index.md`

Each branch page should include:

- a one-paragraph purpose statement
- direct child links
- a short note explaining what belongs here and what does not
- a visible path or breadcrumb-style hint

## Source Excerpt Rules

The staged page should not flatten the source into a plain text dump when Confluence structure is available.
Prefer:

- a clickable source-page link
- preserved ADF blocks where possible
- a short structured excerpt
- fallback text only when the source has no usable native structure

## What Success Looks Like

- A single branch page gives enough context to navigate downward.
- A staged page can be read without losing the source link or the structure of the original page.
- Co-located `index.md` pages act like indexes, not just metadata records.
- Confluence becomes the datastore and the navigation layer at the same time.
