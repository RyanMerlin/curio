# Show HN Draft

## Suggested Title

`Show HN: Curio - a Git-native editorial pipeline for enterprise knowledge`

## Submission URL

Use the public repository URL after the milestone commit is pushed. The
repository must be the source of truth for the launch; do not link to a hosted
service that has not passed the enterprise deployment gates.

## Post Body

Most enterprise knowledge systems index documents. Curio takes a different
approach: it turns raw sources into reviewed, hierarchically placed knowledge
objects and mirrors the curated result into Confluence.

The core loop is:

```text
intake -> agent routing -> staged/review -> publish -> Confluence mirror
```

The Rust binary is deterministic and makes no LLM calls. The agent performs
the editorial work: it inspects the taxonomy and nearby pages, proposes a
route, scores quality and overlap, rewrites weak source material when useful,
and records the rationale in a proposal dossier. Git is the system of record.

To try the synthetic demo without credentials:

```sh
git clone https://github.com/RyanMerlin/curio.git
cd curio
./scripts/show-hn-demo.sh
```

The demo uses only synthetic content and a temporary workspace. It exercises
intake, manifest generation, review/staged separation, publish re-gating, and
the final published tree.

## Maker Comment

I built Curio because raw document indexing does not solve the editorial
problem: teams still need to decide what belongs, where it belongs, whether it
duplicates existing knowledge, and whether someone should trust it.

The unusual design choice is the boundary between the harness and Rust. Rust
does deterministic filesystem, Git, taxonomy, quality, overlap, and Confluence
operations. The agent supplies inference and editorial judgment through a
two-phase manifest and route-file protocol. That makes the decisions visible
and reviewable instead of hiding them in an autonomous background job.

The local demo is intentionally credential-free. Real Confluence sync requires
an operator-configured KB and credentials. The Cloud Run deployment files are
experimental: inbound identity verification, workspace-scoped secrets and
RBAC, multi-instance state safety, tamper-evident audit, and production
observability are not claimed as complete yet.

I am especially interested in feedback on:

- whether the proposal dossier is the right review unit
- where the Git/agent/Confluence boundary feels useful or awkward
- how knowledge operators who are not developers should hand setup to their
  agent
- which enterprise sources and permission models should be prioritized next

## Launch Checklist

- Run `./scripts/show-hn-demo.sh` from a clean checkout.
- Confirm the post links to the repository and begins with `Show HN`.
- Keep the maker comment ready before submitting.
- Do not claim a hosted demo, secure SaaS tenancy, or production Cloud Run
  readiness unless those capabilities have separately passed their release
  gates.
- Record recurring setup failures and questions as GitHub issues after launch.
