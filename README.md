<div align="center">

<img src="docs/assets/Curio_curated_intelligence_operator.png" alt="Curio — Curated Intelligence Operator" width="320" />

# Curio

**A knowledge compiler and curation control plane for enterprise knowledge.** Git-native, multi-provider, and multi-workspace. Curio turns raw sources into a curated hierarchy and mirrors the result into Confluence for the humans who consume it.

[![milestone](https://img.shields.io/badge/milestone-public%20demo-success)](CHANGELOG.md)
[![CI](https://github.com/RyanMerlin/curio/actions/workflows/ci.yml/badge.svg)](https://github.com/RyanMerlin/curio/actions/workflows/ci.yml)
[![providers](https://img.shields.io/badge/providers-Claude%20%7C%20Codex%20%7C%20Gemini-blue)](#providers)
[![license](https://img.shields.io/badge/license-Apache%202.0-blue)](LICENSE)

</div>

---

## Why Curio

Most knowledge tools index documents. **Curio compiles them into governed knowledge, then mirrors the result into Confluence.**

Every intake source is judged against the existing tree, scored on seven dimensions, optionally rewritten, optionally consolidated with peers, and routed to `staged/`, `review/`, or `published/` with a proposal dossier that explains the decision. The Rust binary handles deterministic execution; the agent handles editorial judgment. Git is the system of record. Confluence is the read-only mirror your audience actually opens.

**Curio is not a page router. It is an information architect.**

Curio is the editorial control plane between source material and downstream
retrieval. The current product is deliberately proposal-first: agents make the
curation decisions, while `curio-rs` enforces deterministic filesystem, Git,
quality, and publication boundaries.

> "If a curation step cannot be explained as inference‑driven judgment, the step is too early."
> — `docs/design/operating-contract.md`

## What Curio does well

- **Hierarchy‑first synthesis.** The agent walks the existing tree's `index.md` summaries and peer pages near likely branches before proposing a path. Default bias is toward *deeper* placement, never the easiest shallow match.
- **Multi‑source consolidation.** Multiple sources from a single intake request can be consolidated into one curated proposal; the proposal dossier records every contributing source. Split workflows remain a future extension.
- **Agent‑authored bodies.** The agent ships a curated knowledge object, not a raw capture. Body rewrites land with a structured decision section explaining the scores, alternatives, and recommended action.
- **Continuous overlap detection.** Pages with high semantic overlap are flagged for merge or consolidation instead of becoming silent duplicates.
- **Self‑sharpening.** `curio sharpen --prepare` emits a manifest of restructure / merge / split candidates for the agent to act on.
- **Taxonomy mutations.** When no existing node fits, the agent proposes a new node against `NORTHSTAR.md`. Reviewers see a dedicated "Taxonomy mutation proposed" panel in Confluence before approving.
- **Deterministic safety boundaries.** Atomic registry writes, intake resume-after-crash, publish-time re-gate (quality + overlap + taxonomy validity rechecked at promotion), `--force` escape hatch with audit logging, JSON error envelopes, and a full `--dry-run` mode that **never** touches Confluence. Production Cloud Run hardening remains deferred.

## Why this holds up

A few things are deliberately concrete:

- **Deterministic core.** `curio-rs` makes no LLM calls and keeps the safety-sensitive pieces in Rust.
- **Explicit workspace boundaries.** Every command is scoped to a named workspace or KB path; the harness does not assume one global corpus.
- **Real quality gates.** Routing, publishing, and sync all re-check the corpus state instead of trusting stale inputs.
- **Automated verification.** The Rust suite runs under `cargo nextest`; CI exercises fmt, clippy, tests, and release builds.
- **Synthetic public fixtures.** The demo workspace and tracked examples are synthetic so the public repo stays safe to inspect and fork.
- **Deployment path is documented.** The GCP / Vertex AI path is described in-repo, with its current experimental status and production gaps called out explicitly.

## Enterprise knowledge posture

Curio follows the parts of enterprise knowledge management that remain useful
even when the retrieval model changes:

- **Trusted sources and provenance.** Every proposal retains source lineage,
  alternatives, rationale, and the curation decision that produced it.
- **Explicit structure.** `NORTHSTAR.md` and the YAML taxonomy define an
  inspectable ontology instead of relying on an opaque index or tags alone.
- **Human review for uncertainty.** Weak, stale, duplicative, or ambiguous
  material is routed to `review/` rather than silently becoming canonical.
- **Freshness and maintenance.** Quality, overlap, freshness, and usability
  are re-evaluated at publication; `sharpen` and `heal` provide proposal-only
  maintenance loops.
- **Permission-aware deployment is still ahead.** A production service must
  add authenticated identity, workspace-scoped credentials, and permission
  trimming before it can safely ground enterprise users across multiple
  repositories.
- **Published-only retrieval is now deterministic.** The CLI exposes a stable
  `retrieve --query ... --json` contract with cited excerpts and provenance.
  An MCP server and the complementary `fetch` contract remain roadmap work.
- **Adapters, evaluation, and page-level ACLs are roadmap work.** The adoption
  roadmap sequences source adapters, cited retrieval evaluation, provenance
  fields, and permission-preserving retrieval before enterprise connector claims.

## The lifecycle

```
  ┌──────────┐     ┌───────────┐     ┌──────────┐     ┌──────────┐     ┌─────────────┐
  │  intake  │ ──▶ │  process  │ ──▶ │  staged  │ ──▶ │  publish │ ──▶ │ sync to     │
  │ wiki/    │     │  (agent)  │     │  /review │     │          │     │ Confluence  │
  │  intake/ │     └───────────┘     └──────────┘     └──────────┘     └─────────────┘
  └──────────┘           │                  ▲
       │                 │ proposal,        │ feedback signals
       │                 │ dossier,         │ (👍 / 👎 / ❓ from
       │                 │ scores           │ reviewers in Confluence)
       │                 ▼                  │
       │           ┌──────────┐             │
       └────────▶  │   heal   │ ────────────┘
                   │ sharpen  │
                   └──────────┘
```

Every transition is git‑committed; every editorial decision lives in a `.proposal.json` dossier next to the page. The audit trail is the repo.

## Target use cases

| Use case | What Curio organizes | Why it fits |
|---|---|---|
| **Customer / account intelligence** | sanitized engagement notes, recurring patterns, post-engagement learnings | Keeps related accounts together and prevents duplicate lessons from scattering. |
| **Product knowledge bases** | install, configure, upgrade, troubleshoot per product / version | Deeper paths keep technical content discoverable as products evolve. |
| **Partner / channel knowledge** | partner programs, tiering, joint motions, partner-led playbooks | Confluence mirrors let partner-facing teams consume curated output without git literacy. |
| **FDE / SE use-case catalogs** | engagement patterns, solution recipes, anti-patterns | The sharpening loop turns failure analyses into reusable policy. |
| **Subject-matter references** | structured technical references with explicit taxonomy | NORTHSTAR.md is the editorial charter, and the agent enforces it. |

Each KB is a separate git repo with its own taxonomy, Confluence space, and credentials. A single Curio harness instance manages **N KBs** without cross-tenant leak.

## Get started

### First run: credential-free synthetic demo

The recommended public path needs no Confluence account, provider account, or
live enterprise data. It uses a temporary copy of the synthetic workspace and
verifies the full local lifecycle:

```sh
./scripts/show-hn-demo.sh
```

The demo exercises deterministic intake fixtures, agent-style routing, staged
and review lanes, proposal dossiers, publish re-gating, and the final
published tree. It never modifies `docs/wiki-demo/`.

For an agent-led setup intended for knowledge operators, see
[docs/agent-setup.md](docs/agent-setup.md).

You can also drive Curio three ways. Pick the path that matches your work.

### Option 1 — Service via Docker (local shared use)

This is the local service path for shared development. The Cloud Run deployment
files are an experimental staging path, not a claim that the hosted service is
ready for unrestricted enterprise production.

```sh
# from this repo
cd deploy/local
cp .env.example .env            # paste your Confluence token
docker compose up -d --build
curl http://localhost:8080/healthz
./smoke-test.sh                 # exercises all 3 KBs end‑to‑end
```

Then post a job:

```sh
curl -X POST http://localhost:8080/v1/jobs \
  -H 'Content-Type: application/json' \
  -d '{"job_type":"intake","workspace_id":"demo-workspace","operation":"intake",
       "actor":{"kind":"human","id":"you"},"trigger":{"kind":"manual"},
       "write_mode":"direct_push",
       "inputs":{"args":["--url","https://yourorg.atlassian.net/wiki/spaces/X/pages/123"]}}'
```

### Option 2 — CLI (local development and real KBs)

```sh
cd curio-rs && cargo build --release --bin curio
./target/release/curio --workspace <name> doctor   # infrastructure + content checks
./target/release/curio --workspace <name> intake --url <confluence-or-web-url>
./target/release/curio --workspace <name> process --prepare > manifest.json
# hand manifest.json to your agent, get routes.json back
./target/release/curio --workspace <name> process --route-file routes.json
./target/release/curio --workspace <name> publish <slug>
./target/release/curio --workspace <name> sync
```

Run tests with `cargo nextest run --all-targets` from `curio-rs/`.

### Option 3 — Onboard a new colleague

```sh
deploy/local/setup-colleague.sh <name> <space-key> <parent-page-id>
# Scaffolds the KB, registers the workspace in both curio.workspaces.toml
# and deploy/local/state/workspaces.json (demo/local registry), runs git init, and prints next steps.
```

See [**docs/runbook.md**](docs/runbook.md) for the full day‑zero operator guide.

## The Confluence side

What reviewers actually see when they open a page Curio synced to their space:

- **Banner** explaining the lane (`Review`, `Staged`) and what action is expected
- **Curation Decision** table with all seven score dimensions as inline bars (route, quality, hierarchy fit, overlap risk, evidence completeness, usability, freshness)
- **Recommendation** panel with proposal kind, recommended action, merge target if any, review reason
- **Taxonomy mutation proposed** note macro when approving the proposal also adds a new node to `NORTHSTAR.md` — with full new‑node title, slug, parent path, description, rationale
- **Alternatives considered** list (the paths the agent rejected and why)
- **Body rewrite badge** — `none` / `light_edit` / `full_synthesis` so reviewers know what they're reading
- **Branch landing pages** with status badges on every child link (proposal kind, route confidence, overlap warning, "+new node" tag)
- **Pinned reviewer comment** — react 👍 to approve, 👎 to reject, ❓ to request a rewrite; equivalent labels work too; `curio feedback` applies the signals

## Multi‑workspace from day one

```
   curio harness                 ┌─────────────┐
   (one Docker container)        │ Confluence  │
        │                        │  spaces     │
        ├── wiki #1   ◀──sync──▶ │  • PARTNER  │
        ├── wiki #2   ◀──sync──▶ │  • FDE      │
        └── wiki #3   ◀──sync──▶ │  • CURIO    │
```

- Each KB declares its Confluence URL, email, space key, and token env var in its own `.curio.yaml`.
- The token env var is **per‑KB** — colleagues can each have their own bot account if they want.
- The workspace registry is atomic (write‑tmp + rename) so concurrent restarts can't corrupt it.
- `curio doctor` runs eight per‑KB infrastructure checks including a real Confluence auth probe.
- `curio sync --all` deletes only explicitly Curio-owned pages below managed roots; unowned pages are preserved and cleanup failures are reported.
- Confluence contract tests run without credentials. The dedicated sandbox smoke test is opt-in via `CURIO_LIVE_CONFLUENCE=1 ./scripts/confluence-live-smoke.sh`; see [`docs/runbook.md`](docs/runbook.md).

## Two‑layer architecture

```
   ┌──────────────────────────────────────────┐
   │   Curio harness (this repo)              │
   │   • Provider entrypoints + profiles      │  ◀── editorial / inference layer
   │   • Skills, plugins, marketplace catalog │      (Claude / Codex / Gemini)
   │   • Manifest + dossier composition       │
   │   • Operator UX                          │
   └──────────────────┬───────────────────────┘
                      │
   ┌──────────────────▼───────────────────────┐
   │   curio-rs (deterministic Rust crate)    │
   │   • CLI primitives (intake, process,     │  ◀── deterministic / safety layer
   │     publish, sync, doctor, heal, etc.)   │      (no LLM calls in the binary)
   │   • Git mutations                        │
   │   • Confluence v2 client                 │
   │   • Filesystem + index management        │
   │   • Service (Cloud Run / Docker)         │
   └──────────────────────────────────────────┘
```

The rule: **if behavior is deterministic and safety‑sensitive, it lives in `curio-rs`. If it is about composition, routing, or editorial judgment, it lives in the harness.**

## Providers

All three providers launch from the same workspace contract — taxonomy, skills, plugin catalog, manifest, environment.

| Provider | Entrypoint | Profile | Notes |
|---|---|---|---|
| **Claude** | `CLAUDE.md` (stub → `HARNESS.md` + `providers/claude/overrides.md`) | `providers/claude/profile.json` | Primary curation agent today; `.claude/settings.local.json` optional |
| **Codex** | `AGENTS.md` (same pattern) | `providers/codex/profile.json` | `.agents/skills/` compatibility mirror |
| **Gemini** | `GEMINI.md` (same pattern) | `providers/gemini/profile.json` | ADK‑Go launcher target |

Adding a fourth provider = one folder under `providers/<name>/` plus one root stub. See `HARNESS.md` for the shared operating contract.

## Tests and quality

- **84 tests pass** — lib unit tests, demo workspace, multi-KB isolation,
  multi-workspace safety, publish re-gate, page-body rewrite, multi-source
  synthesis, routing evaluation, doctest.
- Every command supports `--json` with a stable envelope `{command, ok, data}` or, on errors, `{command, ok: false, error: {code, message, hint}}`.
- `curio doctor` validates eight per‑KB infrastructure invariants (config / NORTHSTAR / git / Confluence URL / email / token / space key / auth probe).
- `--dry-run` on every write command is a true read‑only preview — `sync --dry-run` never constructs a Confluence client.
- Atomic registry writes prevent corruption across concurrent service restarts.
- Multi-workspace integration tests verify zero cross-KB filesystem or env-var leak.

## Documentation

| Doc | Audience | What's in it |
|---|---|---|
| [`HARNESS.md`](HARNESS.md) | Agents | Canonical, provider‑neutral operating contract |
| [`ARCHITECTURE.md`](ARCHITECTURE.md) | Engineers | Layer boundaries (curio‑rs ↔ harness) and design rules |
| [`docs/runbook.md`](docs/runbook.md) | Operators / colleagues | Day‑zero guide: intake → process → publish → sync, recovery, rollback |
| [`docs/agent-setup.md`](docs/agent-setup.md) | Knowledge operators + agents | Copy-paste setup contract, approval boundaries, and readiness report |
| [`docs/release-checklist.md`](docs/release-checklist.md) | Maintainers | Release verification and launch claims |
| [`docs/show-hn-launch.md`](docs/show-hn-launch.md) | Launch owner | Show HN draft, maker comment, and launch boundaries |
| [`docs/index.md`](docs/index.md) | Everyone | Navigation map for active docs, design notes, and archives |
| [`docs/archive/launch/`](docs/archive/launch) | Everyone | Historical launch artifacts preserved for reference only |
| [`docs/design/process.md`](docs/design/process.md) | Anyone shaping the editorial pipeline | Long‑form spec for the proposal model |
| [`docs/design/operating-contract.md`](docs/design/operating-contract.md) | Curation agents + reviewers | The inference‑first loop |
| [`docs/agent-cli-contract.md`](docs/agent-cli-contract.md) | Tool integrators | Machine‑readable JSON shapes for every command |
| [`docs/design/2026-05-10-production-handoff-plan.md`](docs/design/2026-05-10-production-handoff-plan.md) | Implementers | The Tier 1 handoff plan, completed |
| [`docs/design/2026-05-10-tier2-plan.md`](docs/design/2026-05-10-tier2-plan.md) | Implementers | Tier 2 editorial completion plan, in flight |
| [`docs/design/2026-07-16-adoption-roadmap.md`](docs/design/2026-07-16-adoption-roadmap.md) | Maintainers / contributors | Active proposal for MCP retrieval, evaluation, adapters, ACLs, and enterprise hardening |
| [`CHANGELOG.md`](CHANGELOG.md) | Everyone | Most recent shipped changes |

## Repo layout

```
curio-agent/
├── HARNESS.md                     # canonical shared contract
├── CLAUDE.md / AGENTS.md / GEMINI.md   # thin provider stubs
├── providers/
│   ├── claude/{profile.json, overrides.md, settings.template.json}
│   ├── codex/{profile.json, overrides.md}
│   └── gemini/{profile.json, overrides.md}
├── skills/                        # authored Curio skills
├── plugins/                       # plugin bundles (skills + tools)
├── curio-rs/                      # the deterministic Rust crate
│   ├── src/
│   │   ├── commands/              # one file per CLI subcommand
│   │   ├── service/               # HTTP service for Docker / Cloud Run
│   │   └── *.rs                   # config, confluence, git_ops, ...
│   └── tests/                     # integration tests
├── deploy/
│   ├── cloud-run/                 # Cloud Run Dockerfile + Terraform
│   └── local/                     # docker-compose for local 3-KB testing
└── docs/                          # all the docs above
```

## Status

| Tier | Status |
|---|---|
| **Tier 1** — multi‑tenant hardening, doctor, runbook, colleague onboarding | ✅ Shipped |
| **Editorial milestone** — page‑body rewriting, Review‑tree polish, multi‑source consolidation, cached overlap | ✅ Shipped |
| **Editorial backlog** — split proposals, continuous sharpening, tuning‑corpus learning | 🟨 Next |
| **Enterprise service hardening** — verified hosted identity/configuration, workspace secrets/RBAC, durable concurrent state, tamper-evident audit, observability | 🗓 Deferred |

See [`docs/release-checklist.md`](docs/release-checklist.md) for the public
milestone gate and [`docs/design/2026-04-26-enterprise-readiness-roadmap.md`](docs/design/2026-04-26-enterprise-readiness-roadmap.md)
for the deferred production-service track.

## Current limitations

- The public demo is local and synthetic; it does not provide a hosted sandbox.
- Real Confluence synchronization requires operator-supplied credentials and a
  configured KB.
- The Cloud Run control plane should be treated as experimental until its
  deployment identity, workspace-scoped secret resolution, multi-instance state
  safety, audit integrity, and observability are configured and verified.
- Curio provides an editorial pipeline and review surface; it is not a general
  enterprise search replacement or an automatic publisher.

## Onboarding a colleague (one command)

```sh
deploy/local/setup-colleague.sh alice myspace 9876543210
```

Scaffolds `~/kb/alice/` with a wiki tree, NORTHSTAR.md template, `.curio.yaml` pointed at the right Confluence space, `.env.example` for the token, a colleague‑facing README, and `git init`. Registers the workspace in both `curio.workspaces.toml` and the docker-compose demo/local service registry. Prints the colleague's first five steps.

## Operating principles

1. **Git is the source of truth.** Confluence is the read‑only mirror.
2. **Inference first.** Use commands to *apply* decisions, not to discover whether a decision exists.
3. **`published` is never the first move.** New work lands in `staged` or `review`.
4. **Hierarchy is the primary optimization target.** Default toward deeper paths for technical or scenario‑specific content.
5. **Confidence alone is not enough to publish.** Quality and usability matter equally; the publish gate re‑checks at promotion.
6. **Anything a human must review must surface in Confluence.** If it's only in Git, it's operationally invisible.
7. **Low‑signal content goes back to review.** Never publish a placeholder with a weak confidence‑only justification.

---

<div align="center">
<sub>Curio · Curated Intelligence Operator · Git‑native, multi‑provider, multi‑workspace</sub>
</div>
