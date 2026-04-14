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
The YAML block below is the machine-readable source of truth — edit it directly to add, rename, or restructure nodes.
`curio` reads this block at runtime; no separate `northstar.json` file is needed.

```yaml
schema_version: 2
nodes:
  - title: Account-tree
    slug: account-tree
    icon: "1f3e2"
    description_markdown: |
      Customer- and account-specific intelligence, deliverables, and reusable account knowledge.
      Not a generic catch-all for any page that mentions an account.
      Metadata to track: source lineage, status, downstream references.
    children: []

  - title: Product-tree
    slug: product-tree
    icon: "1f4e6"
    description_markdown: |
      Product-centric guidance, playbooks, and reference content.
      Not temporary workspace for raw feature notes or launch scraps.
      Metadata to track: product owner, canonical source, related pages.
    children:
      - title: Alteryx Server
        slug: alteryx-server
        icon: "1f5a5"
        description_markdown: |
          Server-specific knowledge: upgrade guides, operational playbooks, and support escalation patterns.
        children:
          - title: API
            slug: api
            icon: "1f517"
            description_markdown: |
              REST API documentation, endpoint references, and integration examples for Alteryx Server.
            children: []
          - title: MongoDB
            slug: mongodb
            icon: "1f4be"
            description_markdown: |
              MongoDB internal database operations, tooling, and troubleshooting for Alteryx Server.
            children: []
          - title: Upgrade
            slug: upgrade
            icon: "1f504"
            description_markdown: |
              Upgrade guides, migration checklists, patch notes, and deployment procedures for Alteryx Server.
            children: []
          - title: Authentication
            slug: authentication
            icon: "1f512"
            description_markdown: |
              Authentication configuration: SAML, Windows Auth, OAuth, Kerberos, and credential management.
            children: []
          - title: Administration
            slug: administration
            icon: "2699"
            description_markdown: |
              System administration, service management, configuration, and operational runbooks.
            children: []
          - title: SQL DB Persistence
            slug: sql-db-persistence
            icon: "1f5c4"
            description_markdown: |
              SQL Server and PostgreSQL persistence layer configuration, migration, and troubleshooting.
            children: []
          - title: High Availability
            slug: high-availability
            icon: "1f4aa"
            description_markdown: |
              High availability deployment patterns, worker node configuration, and failover procedures.
            children: []
          - title: User Management
            slug: user-management
            icon: "1f465"
            description_markdown: |
              User roles, permissions, analytic app access, and gallery management.
            children: []
      - title: Alteryx Designer
        slug: alteryx-designer
        icon: "270f"
        description_markdown: |
          Designer-specific guidance: workflow patterns, best practices, and troubleshooting.
        children: []
      - title: Intelligence Suite
        slug: intelligence-suite
        icon: "1f9e0"
        description_markdown: |
          AI/ML tooling guidance, AutoML patterns, and integration playbooks.
        children: []

  - title: Use-Case-tree
    slug: use-case-tree
    icon: "1f504"
    description_markdown: |
      Repeatable workflows, scenarios, and operating playbooks.
      Not a random bucket for one-off content.
      Metadata to track: trigger conditions, inputs, expected outputs.
    children: []

  - title: Topic-tree
    slug: topic-tree
    icon: "1f4da"
    description_markdown: |
      Subject matter pages when no stronger route applies.
      Not a dumping ground.
      Metadata to track: synonyms, related terms, canonical references.
    children: []
```

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
