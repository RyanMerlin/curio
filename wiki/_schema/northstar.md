# NORTHSTAR

## Name

Curio

## High-Level Description

Curio is a Git-native enterprise intelligence workspace.
Git is the canonical data store. Confluence is the visualization layer — a read-only sync target for non-technical consumers.
The LLM maintains a structured markdown wiki in `wiki/`, with self-indexing, semantic reconciliation, and incremental updates.
Inspired by Karpathy's LLM-wiki pattern: knowledge compounds over time rather than being re-derived on every query.

Write the project intent here in plain language.
This file is the source of truth for the charter page that bootstrap will persist into Confluence.

## What Curio Curates

- customer and account knowledge
- product and solution knowledge
- audience-specific guidance
- recurring use-case playbooks
- subject-matter reference material

## Published Tree Blueprint

These bullets define the default `Published` tree.
Bootstrap will create each bullet as a subpage under `Published`.

- `By Account`
  - what it is: customer- or account-specific intelligence, deliverables, and reusable account knowledge
  - what it is not: a generic catch-all for any page that mentions an account
  - useful metadata: source lineage, status, downstream references

- `By Product`
  - what it is: product-centric guidance, playbooks, and reference content
  - what it is not: temporary workspace for raw feature notes or launch scraps
  - useful metadata: product owner, canonical source, related pages

- `By Audience`
  - what it is: knowledge framed for a specific reader or operator group
  - what it is not: duplicated copies of the same page without a purpose
  - useful metadata: audience, intent, reading level

- `By Use Case`
  - what it is: repeatable workflows, scenarios, and operating playbooks
  - what it is not: a random bucket for one-off content
  - useful metadata: trigger conditions, inputs, expected outputs

- `By Topic`
  - what it is: subject matter pages when no stronger route applies
  - what it is not: a dumping ground
  - useful metadata: synonyms, related terms, canonical references

## Structure

- `README`
  - the human landing page
  - the place to start for orientation and usage

- `NORTHSTAR`
  - the project charter and routing contract
  - the source of truth for the Published tree shape

- `Intake`
  - raw capture lane for incoming content
  - content not yet normalized or fully understood

- `Staged`
  - high-confidence content prepared for human review or publish
  - should carry a real proposal body, not a blank shell

- `Review`
  - human arbitration lane for ambiguity, conflict, or risk
  - content that needs judgment before moving forward

- `Published`
  - canonical output surface for approved Curio content
  - organized by the Published tree blueprint above

- `Admin`
  - machine-managed branch for operational structure
  - contains `_templates`, `_registry`, and `_audit`

## Helpful Guidance

- Keep provenance visible.
- Keep published content intentional and narrow.
- Git is the source of truth. Confluence is a mirror.
- Use `wiki/_index/index.md` as the LLM navigation index (~5K tokens for hundreds of pages).
- Prefer stable routes and explicit frontmatter over ad hoc reorganization.
- If a page cannot be routed cleanly, keep it in `Review`.
- Pipeline transitions are `git mv` operations. Every state change is a commit.
