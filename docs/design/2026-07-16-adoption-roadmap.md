# Curio Adoption Roadmap

**Status:** active proposal, 2026-07-16

**Decision:** position Curio as a knowledge compiler and curation control plane,
not as another enterprise chat interface. Curio should turn noisy source material
into a governed, cited, freshness-aware corpus, then make that corpus available
where people and agents already work.

**Release boundary:** the current public release has the editorial pipeline,
proposal provenance, multi-workspace CLI/service foundations, and a credential-free
demo. It does not ship an MCP retrieval server, a retrieval evaluation harness,
source adapters, page-level ACL filtering, or a production-ready hosted service.
Those are roadmap items, not current capabilities.

**Status update:** `curio retrieve --query ... --json` now provides the first
deterministic, published-only lexical retrieval contract with cited excerpts and
Git/source provenance. The MCP wrapper, `fetch`, evaluation corpus, adapters, and
ACL filtering remain unshipped.

This roadmap follows the first public release. It prioritizes work that can create
an open-source adoption loop while preserving a credible path to enterprise
pilots.

## Why this direction

The major enterprise knowledge products are converging on the same baseline:

- Microsoft 365 Copilot connectors carry content, metadata, and item ACLs into
  Microsoft Graph, while its newer federated connector path uses read-only MCP
  tools for real-time access.
- Atlassian Rovo combines Confluence, Jira, Google Drive, Slack, and other
  connected sources while filtering results to content the current user can
  access.
- Google Gemini Enterprise custom sources accept document ACL metadata and use
  the caller's identity when filtering search results.
- Amazon Q Business connectors synchronize document ACLs and identities, and
  its retrieval controls use metadata such as source authority and recency.

Primary references:

- [Microsoft 365 Copilot connector permissions](https://learn.microsoft.com/en-us/microsoft-365/copilot/connectors/manage-access-permissions)
- [Microsoft custom federated connectors with MCP](https://learn.microsoft.com/en-us/microsoft-365/copilot/connectors/set-up-custom-federated-connectors)
- [Atlassian Rovo search and source permissions](https://support.atlassian.com/rovo/docs/search/)
- [Google custom-source access controls](https://docs.cloud.google.com/agentspace/docs/identity)
- [Amazon Q Business connector concepts](https://docs.aws.amazon.com/amazonq/latest/qbusiness-ug/connector-concepts.html)
- [Amazon Q Business relevance tuning](https://docs.aws.amazon.com/amazonq/latest/qbusiness-ug/relevancy-tuning.html)
- [Model Context Protocol server concepts](https://modelcontextprotocol.io/docs/develop/build-server)

Curio should not compete with these products on breadth of chat UI. Its wedge is
the step they largely leave to customers: deciding what deserves to become
canonical knowledge, rewriting it, consolidating duplicates, preserving the
decision record, and continuously improving the governed hierarchy.

## Current state

### Strong and differentiated

- The deterministic Rust layer and agent judgment layer have an explicit,
  provider-neutral contract.
- Git is the auditable source of truth; Confluence is a curated mirror and review
  surface.
- Intake is scored, rewritten, consolidated, and routed through proposal dossiers
  rather than copied directly into a search index.
- Publish-time gates, crash recovery, dry-run behavior, structured JSON output,
  and multi-workspace isolation are tested.
- The credential-free demo proves the core editorial loop without vendor access.

### Adoption constraints

- The default public path still builds from source; there is no one-command
  cross-platform binary installation story.
- The CLI accepts URLs, local files and folders, and Confluence content. There is
  no source-adapter contract for community-contributed connectors.
- Published knowledge has a deterministic local retrieval command, but no MCP
  server or complementary `fetch` contract for agent clients.
- Provenance exists, but page-level source ACLs are not modeled or enforced.
- Quality gates measure editorial fitness at publication time, but there is no
  repeatable retrieval evaluation set or usage-quality dashboard.
- The production service hardening roadmap remains relevant for hosted enterprise
  deployments.

## North-star outcomes

The roadmap is successful when Curio can demonstrate all of the following:

| Outcome | Target |
|---|---|
| First value | A new user runs the credential-free demo in 5 minutes or less. |
| Installation | Supported users install a release binary without compiling Rust. |
| Agent usefulness | Any MCP-compatible client can search and fetch published Curio knowledge with citations. |
| Trust | Every retrieval result includes stable provenance, freshness metadata, and an authority signal. |
| Access safety | Permission fixtures produce zero unauthorized search or fetch results. |
| Retrieval quality | A checked-in evaluation corpus reaches recall@5 >= 0.85 before semantic retrieval becomes the default. |
| Connector leverage | A contributor can add a source adapter without modifying the editorial pipeline. |
| Operator value | A pilot can report time-to-publish, acceptance rate, stale-content rate, and common rejection reasons. |

## Priority order

| Rank | Target | Adoption value | Enterprise value | Effort | Decision |
|---|---|---:|---:|---:|---|
| 0 | Release distribution and five-minute activation | Very high | Medium | Small | Start now |
| 1 | Read-only MCP search/fetch over `published/` | Very high | High | Medium | Build next |
| 2 | Retrieval evaluation and provenance contract | High | Very high | Medium | Build with MCP |
| 3 | Source-adapter SDK plus GitHub/local Markdown adapter | High | High | Medium | Build after MCP contract |
| 4 | ACL model and permission-filtered retrieval | Medium | Critical | Large | Required before enterprise connector claims |
| 5 | SharePoint/OneDrive or Google Drive adapter | High | Very high | Large | Choose from pilot demand |
| 6 | Feedback metrics and scheduled sharpening | Medium | High | Medium | Build after real usage exists |
| 7 | Hosted-service production hardening | Low for HN | Critical for hosted pilots | Large | Follow existing enterprise roadmap |

## P0: release distribution and activation

**Goal:** remove compilation and configuration as barriers to understanding the
project.

Deliverables:

1. Correct public package metadata, including the repository URL.
2. Attach Linux, macOS, and Windows CLI binaries to versioned GitHub Releases.
3. Publish checksum files and document how release assets are produced.
4. Keep `scripts/show-hn-demo.sh` credential-free and make it work with either a
   downloaded binary or a source checkout.
5. Add a concise installation matrix to the README after release assets are
   verified.
6. Resolve the Docker image export problem before claiming Docker as a verified
   launch path.

Exit criteria:

- A clean Linux environment downloads a release asset and completes the demo.
- macOS and Windows artifacts start and print `curio --help` in CI.
- Release checksums are generated by CI, not a maintainer laptop.
- The README does not require a Rust toolchain for the primary demo path.

## P1: Curio MCP retrieval surface

**Goal:** make curated knowledge immediately useful to existing agent clients
without building a Curio chat UI.

Start with a local stdio server. Add authenticated Streamable HTTP only after the
service identity model is ready.

Read-only tools:

- `search(query, workspace?, category?, freshness?, limit?)`
- `fetch(id)`
- `list_categories(workspace?)`
- `knowledge_status(workspace?)`

Result contract:

```json
{
  "id": "curio://workspace/category/page-slug",
  "title": "Canonical page title",
  "excerpt": "Query-relevant excerpt",
  "score": 0.91,
  "category": "product/install",
  "source_uris": ["https://source.example/item"],
  "published_commit": "<git-sha>",
  "updated_at": "2026-07-16T00:00:00Z",
  "freshness": "current",
  "authority": "published",
  "content_hash": "sha256:..."
}
```

This is a proposed result shape, not a shipped API. Field names and provenance
semantics should be finalized with the evaluation fixtures before an MCP server
is presented as a supported integration.

Implementation boundaries:

- Retrieval reads only `wiki/published/`; review and staged content are excluded.
- Stable IDs derive from workspace plus relative path, not an absolute filesystem
  location.
- `fetch` returns the canonical Markdown body and complete provenance.
- The first retrieval implementation may use deterministic lexical scoring.
  Semantic retrieval cannot become the default until the evaluation harness
  proves it improves results.
- The MCP layer calls shared Rust library code. It must not duplicate curation,
  routing, or filesystem policy.
- Stdio logs go to stderr so protocol output remains valid.

Exit criteria:

- Contract and permission tests cover every tool.
- A standard MCP client can discover, search, and fetch the synthetic demo corpus.
- Every result is cited and traceable to a Git commit and source URI.
- The server is packaged with release assets and is eligible for the official MCP
  Registry after the remote/auth story is ready.

## P2: retrieval evaluation and trust metadata

**Goal:** prove that curation improves answers instead of relying on demos and
intuition.

Deliverables:

1. Extend the shipped `curio retrieve --query ... --json` contract with the
   evaluation and trust metadata required by MCP clients.
2. Check in a synthetic evaluation corpus containing queries, expected document
   IDs, irrelevant near-matches, stale pages, and conflicting authorities.
3. Report recall@k, mean reciprocal rank, citation coverage, stale-result rate,
   and ACL-leak count.
4. Add explicit frontmatter or dossier fields for `source_updated_at`,
   `curated_at`, `reviewed_at`, `owner`, and `authority` where they do not already
   exist.
5. Define freshness policies in `wiki/_admin/config.yaml`; do not hard-code one
   expiration window for every knowledge domain.

Exit criteria:

- Evaluation is deterministic and runs in CI.
- Citation coverage is 100 percent for published results.
- ACL-leak count is zero once P4 lands.
- Retrieval backend changes include before/after evaluation output.

## P3: source-adapter contract

**Goal:** let Curio gain source breadth without coupling provider APIs to the
editorial pipeline.

Adapter interface responsibilities:

- Enumerate and fetch source items.
- Emit stable source IDs, canonical URLs, content, MIME type, timestamps, owner,
  parent relationships, and raw ACL principals.
- Persist an opaque sync cursor.
- Emit create, update, delete, move, and ACL-change events.
- Declare capability flags such as incremental sync, recursive traversal, ACLs,
  comments, and attachments.

First adapters:

1. Refactor current web and Confluence intake behind the adapter contract without
   changing their CLI behavior.
2. Add local Markdown/GitHub repository ingestion as the reference community
   adapter. This has low setup friction and is useful to the open-source audience.
3. Select SharePoint/OneDrive or Google Drive from an identified pilot, not from
   speculative breadth. Slack/Jira should follow the same contract.

Exit criteria:

- Adapter conformance tests are reusable by third-party adapters.
- Re-running an unchanged sync creates no duplicate intake.
- Deletes and source moves produce explicit proposals rather than silently
  deleting canonical knowledge.
- A new adapter does not modify proposal routing or publish code.

## P4: permission-preserving knowledge

**Goal:** make enterprise retrieval safe without turning Git paths into an
implicit authorization scheme.

Model:

- Store normalized principals and groups separately from editorial content.
- Record source ACL snapshots and their source revision in proposal provenance.
- Define the policy for synthesized pages with multiple sources. The safe default
  is the intersection of readable audiences; widening access requires an explicit
  reviewer decision.
- Apply permission filtering before scoring and excerpt generation.
- Treat ACL changes as sync events even when source content is unchanged.
- Fail closed when identity resolution or ACL state is missing for a restricted
  workspace.

Exit criteria:

- Tests cover users, groups, deny rules, removed access, multi-source synthesis,
  and cross-workspace isolation.
- Search result counts do not reveal inaccessible document existence.
- Fetch cannot bypass search filtering by guessing an ID.
- Permission behavior is documented independently of any one identity provider.

## P5: operator feedback and continuous improvement

**Goal:** show that Curio makes a knowledge base healthier over time.

Metrics:

- Intake-to-proposal and proposal-to-publish duration.
- Publish, review, rewrite, merge, and reject rates.
- Fresh, stale, and ownerless published-page counts.
- Overlap-warning rate and consolidation rate.
- Search zero-result rate and accepted-result rate, without retaining sensitive
  query text by default.
- Recurring rejection and review reasons.

Use those metrics to drive the existing proposal-only sharpening and
tuning-corpus work. Scheduling remains future work. Do not add autonomous
publishing; sharpening outputs remain proposals.

## Hosted enterprise track

The service hardening work in
[`2026-04-26-enterprise-readiness-roadmap.md`](2026-04-26-enterprise-readiness-roadmap.md)
remains the prerequisite for a hosted enterprise claim. Re-audit that plan before
implementation because parts of the auth and observability stack have changed
since it was written.

Minimum gates before a hosted pilot:

- Verified inbound identity and workspace authorization.
- Secret-manager-backed per-workspace credentials.
- Durable idempotent jobs and distributed workspace locking.
- Tamper-evident audit retention.
- Non-root, minimal containers and a reproducible image build.
- Real readiness checks, structured telemetry, request limits, and safe errors.
- P4 permission filtering for any multi-audience workspace.

## Public adoption loop

Each milestone should produce something independently demonstrable:

1. **Install:** release binaries plus a five-minute credential-free demo.
2. **Use:** an MCP client searches and fetches the curated demo corpus.
3. **Prove:** retrieval evaluation shows why curated output beats raw intake.
4. **Extend:** a contributor adds a source adapter through a documented contract.
5. **Trust:** a permission test demonstrates that restricted knowledge does not
   leak.

Launch material should lead with the working artifact and architectural tension,
not an enterprise feature checklist: "Most knowledge tools index everything.
Curio makes an agent propose what should become canonical, then keeps the result
auditable in Git."

## Explicit non-goals

- Building a general-purpose chat UI before retrieval is useful through existing
  clients.
- Replacing Confluence as the first curated human mirror.
- Making hosted semantic embeddings mandatory for local use.
- Claiming permission preservation before source ACLs are modeled and tested.
- Adding connectors without incremental sync, provenance, and deletion semantics.
- Allowing an automated sharpening loop to bypass proposal review.
- Optimizing GitHub stars as a substitute for activation and retained use.

## Next implementation slice

The next bounded engineering slice after distribution is:

1. Add `fetch` for the stable IDs emitted by the shipped `retrieve` command.
2. Build retrieval evaluation fixtures and metrics around the deterministic
   lexical baseline.
3. Wrap the shared retrieval library in a stdio MCP server.
4. Add an MCP demo command and the evaluation corpus to CI.

This slice is valuable without enterprise credentials, does not weaken Curio's
layer boundaries, and creates the shortest path from the public release to daily
agent usage.
