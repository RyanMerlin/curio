# Production Handoff Plan — Curio Editorial Pipeline

**Status: COMPLETED 2026-05-10.** As of 2026-05-10, all five Tier-1 phases shipped; 72/72 tests passed in the then-current Tier-1 suite; live smoke was green against all 3 KBs; in-container doctor confirmed real Confluence auth. See `CHANGELOG.md` for the per-phase summary. Tier 2 plan to follow.

---

**Target:** hand Curio to colleagues next week so each can drive their own KB end-to-end (intake → process → publish → sync) against their own Confluence space, with no cross-tenant interference, no surprise panics, and a clear runbook when something goes sideways.

**Scope:** the changes in this doc bring Curio from "the data model is right and the happy path works" to "a colleague can run it on Monday without me holding their hand." It does **not** try to ship the full editorial vision (multi-source synthesis with merge/split, embedding-based overlap, page-body rewriting, continuous sharpening) — those are explicitly deferred to a Phase 2 doc.

**Operator decisions (locked):**
- 3-day window. All five Tier-1 phases ship before the demo.
- Deployment shape is provisionally **service-led** (a hosted API or web UI was the intended eventual front door for colleagues at this point in the plan), with the CLI as the local dev/debug fallback. Both paths must be exercised in the smoke test and runbook.
- Confluence is the intended primary human-facing front end. Enhancements to make Confluence a richer review surface (richer Review-tree presentation, action affordances) are bookmarked for Tier 2.
- Tier 2 begins after Tier 1 is verified end-to-end.

---

## What's already in place (no work needed here)

- ✅ Two-layer split: `curio-rs` deterministic, agent does inference. Intake never inferences (correct per spec).
- ✅ `proposal.rs` — full data model: 7-dim `ProposalScores`, 7 `ProposalKind` variants, `ProposalTaxonomyMutation`, `ProposalDossier` with sources/alternatives/overlap-candidates/rationale.
- ✅ `process_intake.rs` already builds and saves a `ProposalRecord` per intake page, including taxonomy mutations and merge candidates.
- ✅ `find_peer_overlap` (Jaccard) and `assess_quality` heuristic feed into proposal scoring.
- ✅ `required_lane(...)` enforces the multi-criteria staged-vs-review gate (route < 0.75, quality < 0.6, hierarchy < 0.7, overlap ≥ 0.7, taxonomy mutation, or explicit review reason → review).
- ✅ Per-KB Confluence credential plumbing (`token_env`).
- ✅ Multi-tenant service registry + atomic registry writes.
- ✅ Per-workspace healthz endpoint.
- ✅ Confluence folder-URL intake.
- ✅ Intake resume-after-crash.

---

## Gap analysis (the actual scope)

### Tier 1 — must ship before colleague handoff

**1. Process Phase 1 manifest enrichment.**
Today's manifest gives the agent intake pages + the full NORTHSTAR tree. It does **not** include peer-page summaries from the likely branch neighborhoods. The design doc explicitly requires the agent to "use the index recursively, walk down through candidate branch indexes, inspect nearby branch nodes and leaf pages." Without peer context in the manifest, the agent can't make a real hierarchy-fit judgment — it can only match keywords against branch descriptions. **This is the single biggest editorial-quality gap that's worth closing now.**

**2. Publish-time re-gate.**
Today `curio publish <slug>` moves staged → published with a frontmatter rewrite and git commit. It does **not** re-run the overlap check, quality re-assessment, or taxonomy-validity check. Per spec, publish must verify the page still clears the gate at promotion time (peer overlap could have changed since staging). One re-check before move; if any gate fails, refuse with an actionable error.

**3. Doctor coverage for colleagues.**
`curio doctor` already exists but doesn't cover the multi-KB invariants colleagues will hit: per-KB Confluence auth probe, NORTHSTAR.md parse check, registry resolution, `.curio.yaml` schema validation, git working-tree cleanliness. Adds ~5 checks; gives colleagues one command to run when something looks off.

**4. Operator runbook for colleagues.**
A single short markdown — "you have a KB, here's how you intake, process, publish, and sync; here's how doctor reports problems; here's how to roll back." Lives in `docs/runbook.md`. Aimed at someone who has not read the architecture doc.

**5. Multi-tenant safety smoke test.**
Today's smoke test verifies all 3 KBs are reachable. Add: run `intake → process → publish` in parallel for two KBs (different content, different spaces) and verify no cross-pollination — assert each KB's git log only contains its own commits, each Confluence space only got its own pages, registry state is consistent.

**6. Onboard validation.**
`curio onboard` already prompts for env vars but doesn't validate the per-KB token reaches Confluence. Add an explicit auth probe at the end so colleagues hit the auth wall on day 0, not on day 3 when they try to sync.

**7. JSON-mode error envelope consistency.**
Some commands emit `--json` envelope `{command, ok, data}`; others print a Rust error to stderr and exit non-zero without JSON. Standardize: when `--json` is set, every error path emits `{command, ok: false, error: {code, message, hint}}`. Critical for colleagues running curio from scripts.

### Tier 2 — editorial completion (defer to Phase 2 unless there's time)

- **Multi-source synthesis** — one intake request → N proposals after merge/split. Design doc Section "Stage 2 / Agent responsibilities" demands it; today it's 1:1.
- **Page-body rewriting in Phase 2** — agent should be able to author the proposed page body, not just route the raw intake.
- **Continuous sharpening loop** — `curio sharpen` exists but is manual. A scheduled or triggered version closes the design-doc requirement of "continuous self-sharpening."
- **Embeddings-based overlap** — Jaccard token overlap is a first cut. Replace with embeddings + cosine when LLM costs are budgeted.
- **Tuning corpus learning** — `source-corpus-tuning.md` describes codifying repeated rejection reasons back into harness policy. No infrastructure exists yet.

### Tier 3 — observability & ops (after handoff)

- Audit log surfacing in Confluence Admin tree.
- Per-KB metrics (intake count, publish rate, time-in-review, overlap-flag rate).
- Slack hooks for review queue escalation.
- Cloud Run migration (already partially scoped; deferred per user direction).

---

## Implementation phases

### Phase A — Multi-tenant hardening (Day 1)

A1. **Standard `--json` error envelope.** Touch the half-dozen sites that bypass `emit_json` on the error path. Wrap them in a small helper that emits `{ok:false, error:{code,message,hint}}` and still exits non-zero.

A2. **Doctor extensions.** Add per-KB checks: Confluence auth probe (calls `/rest/api/user/current` with the resolved token), NORTHSTAR.md parses, `.curio.yaml` parses, registry record present (when running through service), git status clean. Each check returns a `CheckResult` with `ok`, `label`, `detail`, and an optional `fix_hint`.

A3. **Onboard auth probe.** Append to the existing `run_onboard` flow: call the doctor's Confluence-auth check; surface 401/403/404 as actionable errors with the right env var name from the per-KB `token_env` config.

A4. **Multi-tenant safety integration test.** New `tests/multi_tenant_safety.rs`: spin up 2 KBs in tempdirs, run intake/process/publish concurrently in tokio, assert each KB's git log + frontmatter changes are scoped to itself.

### Phase B — Editorial Phase 1 manifest enrichment (Day 1–2)

B1. **Peer-neighborhood discovery.** For each intake page, the manifest emitter should compute a "likely branch list" (top 3 NORTHSTAR branches by keyword overlap or pre-signal heuristic). For each, include the branch's `index.md` summary and 3–5 nearest peer pages with title + 200-char summary + score. The agent now has real context to make a hierarchy-fit call instead of guessing.

B2. **Manifest schema bump.** Add `hierarchy_context` field to the manifest. Bump `schema_version`. Document the field in `docs/agent-cli-contract.md`.

B3. **Manifest size guard.** With peer pages added, the manifest can grow. Add a `--manifest-budget=KB` flag (default 64KB) that truncates peer summaries lowest-score-first if exceeded — the agent gets warned in a `truncated` field rather than getting silent missing context.

### Phase C — Publish-time re-gate (Day 2)

C1. **Re-run gate at publish.** In `commands/gold_publish.rs`, before the git mv: re-load the proposal sidecar, re-run `find_peer_overlap` against current `published/` peers, re-run `assess_quality`, re-validate the target NORTHSTAR path. Compare to stored scores; if any drifted into the review-required range, refuse with `ProposalKind::Merge` or `ProposalKind::Consolidation` recommendation, surfacing the specific dimension that failed.

C2. **`--force-publish` escape hatch.** Operator override for cases where the agent has explicitly approved (e.g. user says "I've reviewed, ship it"). Logged loudly in audit + `wiki/_admin/log.md`.

C3. **Test coverage.** New `tests/publish_regate.rs`: stage a low-overlap proposal, drop a near-duplicate into `published/`, run publish — assert refusal with merge recommendation. Then re-run with `--force-publish` — assert success with audit entry.

### Phase D — Operator runbook & handoff materials (Day 2)

D1. **`docs/runbook.md`.** Concrete commands the colleague will run: `curio onboard`, `curio doctor`, `curio --workspace mine intake --url X`, `curio --workspace mine process --prepare`, `curio --workspace mine process --route-file routes.json`, `curio --workspace mine publish <slug>`, `curio --workspace mine sync`. Plus rollback (`git revert HEAD`) and "if doctor reports X, do Y." Each command shows its own output shape. Aimed at someone who hasn't read ARCHITECTURE.md.

D2. **Per-colleague onboarding script.** `deploy/local/setup-colleague.sh <name> <git-remote> <space-key> <parent-page-id>`: scaffolds a new KB dir, writes `.curio.yaml`, registers the workspace in both `curio.workspaces.toml` and the in-container registry, and prints a one-line "you're ready to go" with their first command.

D3. **README at the KB root.** Each KB scaffold gets a README.md telling the colleague: "this is your KB; the wiki/ tree is your source of truth; read NORTHSTAR.md for your charter; run `curio --workspace <name> doctor` first." Generated automatically by `init-kb`.

### Phase E — Live verification (Day 3)

E1. **Build and ship the image.** Single release build covering all phases.

E2. **Run the full smoke flow against all 3 KBs.** Use real Confluence (now that auth works). Intake a small folder into each, route, publish one page from each into a separate Confluence space, verify isolation.

E3. **Hand a colleague a 30-minute test.** Pick one (the user picks); walk through the runbook live; capture every confusion point as a `// runbook:` follow-up.

---

## Phase 2 (post-handoff, not in this plan)

- Multi-source synthesis (intake request as a unit, agent decides 1→N proposals).
- Page-body rewriting in process Phase 2.
- Embeddings-based overlap (replace Jaccard).
- Continuous sharpening (scheduled or triggered loop emitting review proposals).
- Tuning-corpus learning (codify repeated rejection reasons into harness policy).
- Audit log surfacing in Confluence Admin tree.
- Cloud Run deploy.

---

## Critical files

**Phase A:**
- `curio-rs/src/commands/doctor.rs` — extend checks
- `curio-rs/src/commands/onboard.rs` — append auth probe
- `curio-rs/src/output.rs` — error envelope helper
- `curio-rs/src/main.rs` — convert error paths through envelope when `--json`
- `curio-rs/tests/multi_tenant_safety.rs` (new)

**Phase B:**
- `curio-rs/src/commands/process_intake.rs` — manifest emit + hierarchy_context field
- `curio-rs/src/wiki_index.rs` — peer summary extraction (may already have most of what's needed)
- `curio-rs/src/northstar.rs` — branch keyword scoring
- `curio-agent/docs/agent-cli-contract.md` — schema docs

**Phase C:**
- `curio-rs/src/commands/gold_publish.rs` — re-gate logic
- `curio-rs/src/cli.rs` — `--force-publish` flag
- `curio-rs/tests/publish_regate.rs` (new)

**Phase D:**
- `curio-agent/docs/runbook.md` (new)
- `curio-agent/deploy/local/setup-colleague.sh` (new)
- `curio-rs/src/commands/init_kb.rs` — emit README.md

---

## Verification

- All Tier-1 items have unit or integration test coverage.
- `cargo nextest run --all-targets` clean (at plan completion this had reached 84 tests).
- `cargo clippy --all-targets -- -D warnings` clean.
- `deploy/local/smoke-test.sh` extended to exercise the multi-tenant scenario.
- The runbook is dogfooded against a real KB end-to-end before colleagues see it.
