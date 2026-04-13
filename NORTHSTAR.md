# NORTHSTAR

## Name

Curio

## High-Level Description

:::confluence-info What is Curio?
Curio is a **Git-native enterprise intelligence workspace**.

Git is the canonical data store. Confluence is the visualization layer — a read-only sync target for non-technical consumers.

The LLM maintains a structured markdown wiki in `wiki/`, with self-indexing, semantic reconciliation, and incremental updates. Inspired by Karpathy's LLM-wiki pattern: knowledge compounds over time rather than being re-derived on every query.
:::

## What Curio Curates

- Customer and account knowledge
- Product and solution knowledge
- Audience-specific guidance
- Recurring use-case playbooks
- Subject-matter reference material

## Published Tree Blueprint

Tree definitions below drive the `published/` wiki structure and the Confluence hierarchy.
Each `###` heading defines a top-level knowledge tree. Each `####` heading defines a named subtree under it.
The filesystem slug is derived from the tree name: `Account-tree` → `wiki/published/account-tree/`.

### Account-tree
> Customer- and account-specific intelligence, deliverables, and reusable account knowledge.
> Not a generic catch-all for any page that mentions an account.

**Icon:** 1f3e2
**Metadata to track:** source lineage, status, downstream references

### Product-tree
> Product-centric guidance, playbooks, and reference content.
> Not temporary workspace for raw feature notes or launch scraps.

**Icon:** 1f4e6
**Metadata to track:** product owner, canonical source, related pages

#### Alteryx Server
> Server-specific knowledge: upgrade guides, operational playbooks, and support escalation patterns.

**Icon:** 1f5a5
#### Alteryx Designer
> Designer-specific guidance: workflow patterns, best practices, and troubleshooting.

**Icon:** 270f
#### Intelligence Suite
> AI/ML tooling guidance, AutoML patterns, and integration playbooks.

**Icon:** 1f9e0
### Use-Case-tree
> Repeatable workflows, scenarios, and operating playbooks.
> Not a random bucket for one-off content.

**Icon:** 1f504
**Metadata to track:** trigger conditions, inputs, expected outputs

### Topic-tree
> Subject matter pages when no stronger route applies.
> Not a dumping ground.

**Icon:** 1f4da
**Metadata to track:** synonyms, related terms, canonical references

## Structure

| Stage | Role | Description |
|-------|------|-------------|
| `README` | Human entry point | Landing page; start here for orientation and usage |
| `Config` | Machine-managed | Internal structure: `_config/northstar.md`, `_config/readme.md`, `_config/settings.yaml` |
| `Intake` | Pipeline: Stage 1 | Raw capture lane for incoming content not yet normalized or fully understood |
| `Staged` | Pipeline: Stage 2 | High-confidence content prepared for human review or publish |
| `Review` | Pipeline: Stage 2b | Human arbitration lane for ambiguity, conflict, or risk |
| `Published` | Output surface | Canonical wiki organized by the tree blueprint above |

## Helpful Guidance

:::confluence-tip Operating Principles
- Keep provenance visible.
- Keep published content intentional and narrow.
- Git is the source of truth. Confluence is a mirror.
- Use co-located `index.md` files as the LLM navigation index.
- Prefer stable routes and explicit frontmatter over ad hoc reorganization.
- If a page cannot be routed cleanly, keep it in `Review`.
- Pipeline transitions are `git mv` operations. Every state change is a commit.
:::
