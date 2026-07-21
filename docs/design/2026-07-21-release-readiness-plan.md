# Release Readiness Plan — 2026-07-21

**Status:** approved, in progress
**Decision:** land a focused punch list that fixes a live CI/branch-protection
gap, corrects documentation that undersells shipped capability (particularly
the MCP retrieval surface), cleans up stale GitHub state, and cuts a v1.1.0
release so the public repo, its docs, and its GitHub presentation all agree
with what the code actually does.

This is a verification-and-correction pass, not new feature work. Every item
below was confirmed against live state (CI run logs, branch protection API,
git history, or a local build/test/demo run) before being added here.

## Context

Baseline verification (2026-07-21) found the core project healthy:
`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
`cargo nextest run --all-targets` (120/120), `cargo build --release` (3 bins),
and `./scripts/show-hn-demo.sh` all pass cleanly with no stray files left
behind. The issues below are specific, bounded gaps, not systemic problems.

## Workstream A — CI & branch-protection integrity

**Problem:** `.github/workflows/ci.yml`'s `msrv` job pins
`dtolnay/rust-toolchain@1.100.0`. That ref is interpreted by the action as
both "which version of the action to use" and "which Rust toolchain to
install" unless overridden. Dependabot bumped it from `@1.88.0` (PR #39,
2026-07-20) as a routine action-version update; Rust 1.100.0 does not exist,
so the job now fails with a 404 on every run. The PR's CI failure didn't
block the merge because `msrv` isn't in the branch's required-status-checks
list, and no push-triggered CI has re-verified `main` since, because
GITHUB_TOKEN-authored merges don't retrigger `push`-event workflows on
GitHub. Net effect: `main`'s current HEAD has never had CI run against it,
and the next real PR will show a red, unrelated `msrv` check.

1. Decouple the action ref from the MSRV version: `dtolnay/rust-toolchain@stable`
   with explicit `with: { toolchain: "1.88.0" }`. Dependabot can keep bumping
   the action ref indefinitely without ever touching the tested Rust version
   again — this is a durable fix, not a revert.
2. Add `Rust MSRV (1.88)` to the branch's required status checks.
3. Correct `docs/status-2026-07-16.md`'s branch-protection claims ("enforced
   admin protection," "one approval required") to match verified reality
   (`enforce_admins: false`, no required-review rule). Not changing those
   settings themselves — they're load-bearing for the existing Dependabot
   auto-merge workflow on a solo-maintainer repo; that's a policy call for
   Merlin, not a bug to silently fix.

## Workstream B — Documentation accuracy

4. README.md: replace the "An MCP server and the complementary `fetch`
   contract remain roadmap work" claim (it's shipped: `curio-mcp` is a working
   stdio MCP server — search/fetch/list_categories/knowledge_status — packaged
   into every release archive, with a working credential-free demo entrypoint
   at `scripts/curio-mcp-demo.sh` that nothing currently documents). Add a
   proper section covering it.
5. `docs/design/2026-07-16-adoption-roadmap.md`: update the "Status update"
   paragraph — MCP wrapper, `fetch`, and the retrieval evaluation corpus are
   shipped, not roadmap.
6. Fix 5 dangling relative links found by verification:
   - `README.md:273`, `docs/index.md:39-40` point at
     `docs/archive/launch/*.md`, deleted intentionally in `6b76e889`
     (2026-07-17). Remove the dead references rather than restoring deleted
     content.
   - `docs/design/process.md:47`, `docs/design/TODO.md:166` use a leftover
     Windows absolute path (`C:/code/agents/curio/docs/design/curio-core-init.md`)
     instead of a relative link to the file, which exists locally at
     `docs/design/curio-core-init.md`.
7. Add `docs/design/TODO.md` to `docs/index.md`'s nav — it's a real, partly
   complete design roadmap that's currently orphaned (not linked from the
   index).
8. Note the demo script's implicit prerequisites (python3, perl, git — used
   internally, never documented) near the "Get started" section.

## Workstream C — GitHub surface

9. Close #26 (release-archive smoke testing) and #28 (retrieval evaluation
   corpus) with comments pointing at the commits that already resolved them
   (`68dfb08` / PR #38, and `6b76e88` respectively). Closing #28 empties the
   "v1.1 - Agent retrieval" milestone.
10. Set a custom social-preview image from
    `docs/assets/Curio_curated_intelligence_operator.png` (961×942, usable now;
    a proper 1280×640 banner is a nice-to-have follow-up, not blocking).
11. Clear the self-referential `homepage` URL field.

## Workstream D — Release

12. Once A–C land: move `CHANGELOG.md`'s `[Unreleased]` section into a dated
    `v1.1.0` entry (matches the existing "v1.1 - Agent retrieval" GitHub
    milestone name and is the correct semver bump — this release exposes
    MCP/fetch/retrieval as documented, supported capability for the first
    time), tag `v1.1.0`, let CI build and attach release assets via the
    existing tag-triggered release job, spot-check against
    `docs/release-checklist.md`, close the v1.1 milestone.

## Explicit non-goals

- Not enabling `enforce_admins` / required PR review (see item 3) — flagged
  for Merlin's decision, not changed unilaterally.
- Not producing new banner artwork for the social preview image — using the
  existing logo asset as-is.
- Not touching P3/P4/P5 of the adoption roadmap (source adapters, ACL model,
  feedback metrics) — out of scope for a readiness pass; only the status
  claims about already-shipped work are being corrected.
