# Pre-Public v1.0.0 Plan

**Goal:** ship Curio as a public, battle-ready, professional open-source project. Zero sensitive content, zero company-specific identifiers, clean history, real CI, full community-files package, version 1.0.0.

**Scope decision:** this is a tagged v1.0.0 *cut*, not a maintenance pass. Anything not blocking the public cut is a v1.0.x or v1.1 item.

---

## Contamination inventory (current state)

A grep sweep across tracked files turned up the following surfaces. Each gets a phase below.

| Surface | Files | Risk |
|---|---|---|
| Customer names (Albertsons, Papa Johns) | `CHANGELOG.md` (1 file) | **High** — names a real customer engagement |
| Personal identifiers (merlin, real emails) | 2 files | **High** — personally identifying |
| Internal git host (`git.alteryx.com`) | `curio-rs/src/commands/sync.rs` (2 hardcoded URLs in tree-page builder) | **High** — internal hostname |
| Internal Confluence host (`alteryx.atlassian.net`) | `curio-rs/src/commands/intake.rs:984` (test fixture URL) | **Medium** — test data |
| Alteryx product names baked into routing heuristics | `curio-rs/src/reconcile.rs` (alteryxservice content scan + emoji map for alteryx-server / alteryx-designer / intelligence-suite / aah) | **High** — gives away the source company, contradicts "domain-agnostic" positioning |
| Alteryx product names in docs as examples | `README.md`, `HARNESS.md`, `curio-rs/docs/cli_for_*.md`, design docs | **Medium** — easy to genericize |
| Alteryx product names in fixtures + tests | `curio-rs/tests/fixtures/routing/`, `curio-rs/tests/routing_eval.rs`, `curio-rs/heal-manifest.json`, `curio-rs/review.json` | **Medium** — already partially scrubbed per `docs/design/2026-05-04-public-repo-scrub.md`; verify |
| Demo workspace at `docs/wiki-demo/` | tracked, must be 100% synthetic | **Medium** — review for any leftover alteryx-server references |
| Design docs in `docs/design/` written during real engagement | Several dated 2026-04-* and 2026-05-* | **Medium** — may reference customer or internal context |
| `.env` / `.env.bak` / `.env.example` | `.env` and `.env.bak` are gitignored ✓; `.env.example` and `deploy/local/.env.example` tracked | **Low** — verify they carry no real values |
| CHANGELOG mentions of bug-fix sessions against the demo wiki | live customer state names appear in test logs etc. | **Medium** — sanitize entries |

Plus the artifacts NOT in the harness repo: the three KB directories (`curio-kb/`, `partner-business/`, `fde-uc-repo/`) are siblings of `curio-agent/` and **must never go public**. They're separate git repos with no remotes — confirm.

---

## Phase P1 — Secret hygiene & .gitignore (≤30 min)

1. **`.gitignore` review.** Current excludes `.env`, `.terraform/`, `curio.workspaces.toml`, `wiki/_config/last-sync.txt`, `tmp/`. Add explicit blocks for:
   - `*.env.local`
   - `*.env.bak`
   - `secrets/`
   - `credentials.json`
   - `.curio.workspaces.local.toml`
   - `target/` (the Rust artifact dir if not already covered globally)
   - `dist/`, `*.swp`, `*.swo`, `.idea/`, `.vscode/`
2. **Remove `.env.bak`** from disk (left over from the `sed -i.bak` audit-dir fix).
3. **Run a secrets scan** (`gitleaks detect --no-banner --source .` or `trufflehog filesystem .`) against the working tree. **No tokens, API keys, or credentials should appear.**
4. **History audit.** If we decide to keep git history (Phase P8 decision), scan `git log -p` for accidentally-committed secrets. If any are found, `git filter-repo` to scrub them.

## Phase P2 — Scrub personal identifiers (≤15 min)

1. Replace `merlin`, `rmerlin5@pm.me`, `ryan.merlin@alteryx.com` in any tracked file with neutral placeholders (`<your-name>`, `<your-email>`). The setup script already uses `alice`/`bob` — that's fine to keep as canonical examples.
2. Drop any commit `Co-Authored-By:` lines that name a specific human (keep Claude attribution).

## Phase P3 — Genericize Alteryx-specific routing heuristics (≤90 min)

This is the biggest code change. Two paths:

**3a. `reconcile.rs` content heuristics + emoji map → config-driven product registry.**

Today `reconcile.rs:165–200` hard-codes:
- string scans for "alteryxservice", "alteryxserver", etc.
- emoji map: `alteryx-server → 🖥️`, `alteryx-designer → 🎨`, etc.

Refactor:
- New `product_registry.rs` (or section in `_admin/config.yaml`) with a list of `{slug, title, content_signals, emoji}` records.
- Default registry has **generic shapes** (e.g. `core-platform / 📦`, `companion-tool / 🧩`), not Alteryx products.
- Per-KB `_admin/config.yaml` can override with their own product list. The demo wiki's config.yaml carries the fictional registry it wants.
- `reconcile.rs::heuristic_pre_signal` iterates the registry instead of hard-coded strings.

**3b. `sync.rs` `git.alteryx.com` hardcoded URLs → config-driven repos list.**

The two `<a href=...>` blocks in the multi-source / admin pages builder list the harness + content repos. Today they're hardcoded `https://git.alteryx.com/cro/curio.git` and `.../curio-wiki.git`. Refactor:
- Move the list to `_admin/config.yaml` as `admin.related_repos: [{title, url, description}]`.
- The renderer reads from there; default registry is empty (no links rendered if none configured).

## Phase P4 — Documentation scrub (≤45 min)

1. **README.md** — replace the Albertsons/Papa Johns + alteryx-server lifecycle examples with a synthetic example. Use the fictional "Acme Corp" or just `product-tree/<your-product>`.
2. **HARNESS.md** — `"Alteryx Server" ≠ "Intelligence Suite"` example → generic "two products in the same family should not collapse into one category."
3. **`curio-rs/docs/cli_for_*.md`** — replace `product-tree/alteryx-server` examples with `product-tree/<your-product>` and the "Alteryx Server 2024.1" query example with a synthetic one.
4. **Design docs in `docs/design/`:**
   - Audit each dated `2026-04-*` and `2026-05-*` doc for company-internal context.
   - **Keep:** generic design rationale (process.md, operating-contract.md, source-corpus-tuning.md, the Tier 1/2 plans).
   - **Genericize:** any doc that references the live curio-wiki / customer engagements / internal URLs.
   - **Move out:** anything that's company-internal context (e.g. `docs/design/2026-04-26-enterprise-readiness-roadmap.md` if it names internal teams). Move to a separate private notes repo before going public.
5. **CHANGELOG.md** — the 2026-05-10 entry mentions Albertsons / Papa Johns in the live-test summary. Rewrite the live-test bullets to describe the change shape, not the specific customer. Drop customer names entirely.

## Phase P5 — Demo workspace + fixtures (≤30 min)

1. **`docs/wiki-demo/`** — grep for `alteryx-server`, `alteryx-designer`, etc. Replace with the fictional product registry (e.g. `core-platform`, `companion-tool`).
2. **`curio-rs/tests/fixtures/routing/*`** — same.
3. **`curio-rs/tests/routing_eval.rs`** — "Alteryx Server 2024.1 Upgrade Guide" → "Core Platform 2024.1 Upgrade Guide".
4. **`curio-rs/heal-manifest.json` / `review.json`** — already partly scrubbed per the 2026-05-04 doc; verify.
5. **`curio-rs/src/commands/intake.rs:984`** test — `alteryx.atlassian.net` → `example.atlassian.net`.

## Phase P6 — Community files + license (≤45 min)

Add the standard open-source package:

1. **`LICENSE`** — Apache 2.0 recommended (permissive, patent grant, enterprise-friendly). MIT is a reasonable alternative.
2. **`CONTRIBUTING.md`** — how to file an issue, run tests, propose changes, the editorial-philosophy summary.
3. **`CODE_OF_CONDUCT.md`** — Contributor Covenant v2.1 (drop-in standard).
4. **`SECURITY.md`** — how to report security issues (private channel), what's in scope, how we triage.
5. **`.github/ISSUE_TEMPLATE/bug_report.md`** + `feature_request.md`.
6. **`.github/PULL_REQUEST_TEMPLATE.md`** — short, "what / why / tests / breaking".
7. **`.github/workflows/ci.yml`** — runs `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` on push + PR. Already has `.github/workflows/rust.yml` (release build matrix) — keep that, add a separate CI workflow.
8. **`.github/dependabot.yml`** — weekly Cargo updates.
9. **`README.md` badge** — replace the static `tests-80/80-passing` badge with a real `actions/workflows/ci.yml/badge.svg` once the CI workflow is live.

## Phase P7 — Version bump to 1.0.0 + final CHANGELOG (≤15 min)

1. **`curio-rs/Cargo.toml`** — `version = "0.1.5"` → `"1.0.0"`.
2. **`curio-rs/Cargo.lock`** — regenerated by build.
3. **`CHANGELOG.md`** — new top section:
   ```
   ## [1.0.0] — 2026-05-XX

   First public release. Brings Curio from internal-tooling to publicly-usable
   editorial knowledge-base harness.

   ### Highlights
   * Two-layer architecture (deterministic Rust substrate + multi-provider agent harness)
   * Multi-tenant from day one (N KBs, one harness)
   * Two-phase agent-native routing with seven-dimension proposal scoring
   * Multi-source synthesis (1 intake request → 1 or N proposals)
   * Page-body rewriting with structured decision sections
   * Confluence sync as a read-only mirror surface
   * Provider-neutral via HARNESS.md + per-provider folders

   ### Editorial pipeline
   T2-A page-body rewriting; T2-B Confluence Review-tree polish;
   T2-C multi-source synthesis.

   ### Production-readiness
   Tier 1 hardening: per-KB Confluence credential isolation, atomic registry
   writes, eight per-KB infrastructure checks, --json error envelope, true
   --dry-run on sync, intake resume-after-crash, multi-tenant safety tests.

   ### Coming next
   T2-D embeddings-based overlap, T2-E continuous sharpening scheduler,
   T2-F tuning-corpus learning. See docs/design/2026-05-10-tier2-plan.md.
   ```
4. **README.md** — version badge → `v1.0.0`.

## Phase P8 — Git history decision (REQUIRES USER INPUT)

Two options:

| Option | What | Pro | Con |
|---|---|---|---|
| **A — Keep history** | Audit existing commits, `git filter-repo` to scrub any sensitive blobs, force-push the rewritten history | Preserves change attribution + commit-by-commit story | Time-consuming; small risk of missing a leak |
| **B — Squash to single root commit** (recommended) | One commit titled "Initial public release v1.0.0", `git checkout --orphan public-main` and squash everything | Zero history risk; clean public starting point; fast | Loses individual commit attribution + the Tier-1 / Tier-2 narrative |

**Recommendation: B.** Curio's narrative lives in `CHANGELOG.md` and the design docs in `docs/design/`. The git history isn't load-bearing for a public consumer. Squashing eliminates an entire class of leak risk.

**If A is chosen:** also scan all commit messages (`git log --pretty=full`) for `Co-Authored-By:` lines that name specific humans, customer references, internal hostnames, etc.

## Phase P9 — Final verification gate (≤30 min)

1. `cargo build --release --bin curio --bin curio-service` — clean.
2. `cargo test` — 80/80 (or whatever the count is post-genericization).
3. `cargo clippy --all-targets -- -D warnings` — clean.
4. `cargo fmt -- --check` — clean.
5. **Final contamination grep:**
   ```sh
   grep -rin "alteryx\|merlin\|albertsons\|papa.johns\|rmerlin5" \
     --exclude-dir=.git --exclude-dir=target \
     --exclude-dir=fde-uc-repo --exclude-dir=partner-business --exclude-dir=curio-kb \
     curio-agent/ | grep -v "(allowed:)"
   ```
   Must return zero. Allowed strings (if any) carry an inline `(allowed: <reason>)` comment so the grep filter can ignore them.
6. **`docker compose -f deploy/local/docker-compose.yml build && up -d`** — image builds, healthz returns 200, smoke test green.
7. **Manual README pass** as if seeing it for the first time. Look for jargon, broken links, anything that screams "internal tool."
8. **Manual run-through of the runbook** start-to-finish against a freshly-init'd test KB.

## Phase P10 — Cut the release (≤15 min)

1. `git tag -a v1.0.0 -m "Curio 1.0.0 — first public release"`
2. **Make the GitHub repo public.** (Set via repo settings; tags push along.)
3. `git push origin main && git push origin v1.0.0`
4. Open a v1.0.0 GitHub release pointing at the CHANGELOG entry.
5. (Optional) write a short launch announcement / blog post.

---

## Acceptance criteria

Public-ready means **all** of these:

- [ ] Zero matches in grep for: alteryx, merlin, albertsons, papa-johns, real emails, internal hostnames
- [ ] LICENSE, CONTRIBUTING, CODE_OF_CONDUCT, SECURITY, issue templates, PR template, CI workflow all present
- [ ] `cargo test` 80/80 (or N/N) passing
- [ ] `cargo clippy --all-targets -- -D warnings` clean
- [ ] `cargo fmt --check` clean
- [ ] gitleaks scan clean
- [ ] README renders cleanly on GitHub (hero image, badges, sections all visible)
- [ ] Demo workspace under `docs/wiki-demo/` is purely synthetic
- [ ] Cargo.toml version is `1.0.0`
- [ ] CHANGELOG has a 1.0.0 entry
- [ ] Either: history is clean (Option A) OR squashed to a single root commit (Option B)

## Out of scope for the v1.0.0 cut

- T2-D embeddings-based overlap, T2-E continuous sharpening, T2-F tuning-corpus learning — these are post-1.0 in `docs/design/2026-05-10-tier2-plan.md`. They ship as v1.1.x.
- The sibling KB repos (`curio-kb/`, `partner-business/`, `fde-uc-repo/`) stay private. They are not part of this repo and are not going public.
- Cloud Run deploy (`deploy/cloud-run/terraform/`) — the Dockerfile + compose are public; the Terraform stays in the repo but with placeholder values (no project IDs, no service-account emails).

---

## Time estimate

P1+P2 (hygiene + identifiers): 45 min
P3 (code genericization): 90 min
P4+P5 (docs + fixtures): 75 min
P6 (community files): 45 min
P7 (version bump): 15 min
P9 (verification): 30 min
P10 (cut): 15 min
**Subtotal: ~5 hours**

P8 (history rewrite or squash): +30–60 min depending on option

**Total: ~6 hours of focused work.**
