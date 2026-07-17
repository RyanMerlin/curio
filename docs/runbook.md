# Curio Operator Runbook

This is the day-zero guide for someone who has been handed a Curio KB and needs to start curating. It does **not** assume you've read `ARCHITECTURE.md` or `process.md`.

If you are evaluating Curio before connecting a real KB, start with
`./scripts/show-hn-demo.sh`. It uses synthetic content in a temporary copy and
does not contact Confluence.

## What you have

- A **KB directory** on the host (e.g. `~/kb/<your-name>/`) — your own copy of the knowledge base. Git is the source of truth here.
- A **Confluence space** (e.g. `myspace`) — the read-only mirror your colleagues will see.
- A **bot user / API token** that authenticates Curio against Confluence.
- Two ways to drive Curio:
  - **Service** (recommended for shared use): `curio-service` running in Docker, you talk to it over HTTP.
  - **CLI** (recommended for local debug): `curio` binary running directly against your KB directory.

The Cloud Run files under `deploy/cloud-run/` are an experimental deployment
path. Do not expose the service to an enterprise network until the inbound
authentication, workspace credential, concurrent state, audit, and
observability gates in the enterprise readiness roadmap are complete.

## Step 0 — Verify your install

Pick whichever path you'll be using.

### Service path

```sh
# from curio-agent/deploy/local/
docker compose up -d
curl -sf http://localhost:8080/healthz
curl -sf http://localhost:8080/v1/workspaces | jq .
curl -sf http://localhost:8080/v1/workspaces/<your-workspace-id>/healthz | jq .
```

If `/v1/workspaces/<id>/healthz` reports `status: "active"` and an empty `issues` array, you're good. If it reports a problem, fix the registry record (`deploy/local/state/workspaces.json`) before continuing.

### CLI path

```sh
# from anywhere
curio --workspace <your-name> doctor
```

This runs the **infrastructure check suite** before scanning your published tree. You should see something like:

```
Infrastructure: 8/8 checks passed
  ✓ kb.config — /path/to/your-kb/.curio.yaml parses cleanly
  ✓ kb.northstar — /path/to/your-kb/wiki/NORTHSTAR.md (4321 bytes)
  ✓ kb.git — git working tree clean
  ✓ kb.confluence.url — https://yourorg.atlassian.net/wiki
  ✓ kb.confluence.email — bot@yourorg.com
  ✓ kb.confluence.token — token resolved from env var CURIO_CONFLUENCE_TOKEN_<NAME>
  ✓ kb.confluence.space_key — myspace
  ✓ kb.confluence.auth — authenticated as Bot User (bot@yourorg.com)
```

If any check is `✖`, follow the printed `hint:` line. The most common failure is `kb.confluence.auth` — usually an email/token mismatch (the email in `.curio.yaml` must match the Atlassian account the token was issued for).

## WSL2 + GCP connectivity

Curio operators on WSL2 should use the Linux Cloud SDK explicitly. The Windows SDK can appear earlier in `PATH`, but the Linux binary is the one that should own the working auth/config state for this repo.

Canonical setup:

```sh
# Use the Linux binary directly.
/usr/bin/gcloud --version

# Put the Cloud SDK config on a writable Linux path.
export CLOUDSDK_CONFIG=/tmp/gcloud-curio
mkdir -p "$CLOUDSDK_CONFIG"

# If you already have a Linux gcloud profile, copy it into the temp config.
cp -a ~/.config/gcloud/. "$CLOUDSDK_CONFIG"/
chmod -R u+rwX "$CLOUDSDK_CONFIG"

# Verify DNS reachability before assuming auth is broken.
getent hosts oauth2.googleapis.com
getent hosts run.googleapis.com
getent hosts pubsub.googleapis.com
getent hosts secretmanager.googleapis.com
getent hosts aiplatform.googleapis.com

# Select the intended account and project.
CLOUDSDK_CONFIG=$CLOUDSDK_CONFIG /usr/bin/gcloud config set account <your-account@example.com>
CLOUDSDK_CONFIG=$CLOUDSDK_CONFIG /usr/bin/gcloud config set project <your-gcp-project-id>
CLOUDSDK_CONFIG=$CLOUDSDK_CONFIG /usr/bin/gcloud auth list
CLOUDSDK_CONFIG=$CLOUDSDK_CONFIG /usr/bin/gcloud config get-value project
```

If `gcloud` still tries to use the Windows SDK, call `/usr/bin/gcloud` directly. If the Linux profile has stale credentials, re-authenticate in WSL2 and keep that session in `/tmp` or another writable Linux path, not under a read-only mount or the Windows profile tree.

Use `deploy/cloud-run/wsl2-gcloud-bootstrap.sh` for a quick environment check and temp-config bootstrap.
If you need one-off defaults, copy `deploy/cloud-run/wsl2-gcloud.local.env.example` to `deploy/cloud-run/wsl2-gcloud.local.env` and keep it untracked.

## Step 1 — Author your charter

Open `<your-kb>/wiki/NORTHSTAR.md`. This is your editorial charter: the categories your KB will support, the routing rules, and what does NOT belong here. The agent reads this every time it routes intake. Spend 15 minutes on it before you ingest anything.

Update `<your-kb>/wiki/_admin/config.yaml` to declare the taxonomy nodes that match your charter. `nodes:` is the authoritative tree; the agent will refuse to publish into a path that isn't declared here.

Commit:

```sh
cd <your-kb>
git add wiki/NORTHSTAR.md wiki/_admin/config.yaml
git commit -m "Author KB charter and taxonomy"
```

## Step 2 — Ingest

```sh
# Service path (single page)
curl -sX POST http://localhost:8080/v1/jobs \
  -H 'Content-Type: application/json' \
  -d '{"job_type":"intake","workspace_id":"<your-name>","operation":"intake","actor":{"kind":"human","id":"you"},"trigger":{"kind":"manual"},"write_mode":"direct_push","inputs":{"args":["--url","https://yourorg.atlassian.net/wiki/spaces/<KEY>/pages/<id>"]}}'

# CLI path — single page
curio --workspace <your-name> intake --url https://yourorg.atlassian.net/wiki/spaces/<KEY>/pages/<id>

# CLI path — recursive (page + descendants)
curio --workspace <your-name> intake --url <page-url> --recursive

# CLI path — entire Confluence folder
curio --workspace <your-name> intake --url https://yourorg.atlassian.net/wiki/spaces/<KEY>/folder/<id>
```

After ingest, files land in `wiki/intake/<slug>.md` and an audit row is appended to `wiki/_admin/log.md`. Check `git log` — Curio auto-commits successful intakes. If a prior intake crashed mid-flight, the next intake will finalize the orphan files automatically (you'll see "committed pending intake from a prior partial run").

## Step 3 — Process (the editorial moment)

Process is **two-phase by design**. The Rust binary does no LLM calls; the agent (Claude / Codex / Gemini) does the editorial inference.

```sh
# Phase 1 — emit the routing manifest. This is what you hand to the agent.
curio --workspace <your-name> process --prepare > /tmp/manifest.json
```

The manifest contains:
- All intake pages awaiting routing
- Your full taxonomy from NORTHSTAR
- For each branch in `published/`, the branch index summary **plus up to 5 peer leaf pages** so the agent can judge hierarchy fit against actual peer content
- Strict instructions on hierarchy depth, new-subtree proposals, overlap, and confidence
- A `truncated` flag if the manifest had to drop peer entries to fit under the 64 KB budget (override with `CURIO_MANIFEST_BUDGET_KB`)

Hand `/tmp/manifest.json` to your agent. It returns a routing decisions JSON.

```sh
# Phase 2 — apply the agent's decisions
curio --workspace <your-name> process --route-file /tmp/decisions.json
```

Each decision becomes a `git mv` from `intake/` to `staged/<category>/` or `review/`, plus a `.proposal.json` sidecar with the full dossier (route confidence, quality, hierarchy fit, overlap risk, evidence completeness, usability, freshness; sources; alternatives; rationale; taxonomy mutation if proposed).

## Step 4 — Publish

When a staged page is ready:

```sh
curio --workspace <your-name> publish <slug>
```

Publish runs a **publish-time re-gate**: quality, taxonomy validity, and a fresh peer-overlap pass against `published/`. If the page has drifted (e.g. a near-duplicate appeared since staging), publish refuses and tells you which dimension failed. The page stays in staged.

Override (after manual review) with:

```sh
curio --workspace <your-name> publish <slug> --force
```

Force is logged loudly to `wiki/_admin/log.md` with the bypassed dimensions. Reviewers can audit it later.

## Step 5 — Sync to Confluence

```sh
curio --workspace <your-name> sync
```

One-way Git → Confluence. Confluence is your audience's read-only front door; never edit there expecting it to come back.

Incremental by default (only changed pages). Use `--all` for a full refresh including pruning of stale pages.

## Useful side commands

| Command | What it does |
|---|---|
| `curio --workspace <name> status` | intake / staged / review / published counts; last-sync timestamp |
| `curio --workspace <name> review` | List items in review/ and staged/ |
| `curio --workspace <name> resolve <slug>` | Move a review/ item to staged/ |
| `curio --workspace <name> reject <slug>` | Locally reject (delete) a page; logged in audit |
| `curio --workspace <name> lint` | Find contradictions, stale claims, orphan refs |
| `curio --workspace <name> doctor` | KB health (infra + content). Run this if anything looks off. |
| `curio --workspace <name> sharpen --prepare` | Emit a self-sharpening manifest (consolidate / merge / restructure proposals) for the agent |
| `curio --workspace <name> heal --prepare` | Emit a heal manifest; agent decides; apply with `--apply-file` |

## When something goes wrong

1. **Run `curio doctor`.** Infrastructure checks first, content checks after. If infra is `8/8 ✓` and you still hit a problem, the issue is content-side — read the findings.
2. **Check `wiki/_admin/log.md`.** Every intake / publish / heal / reject writes a line. Force-bypassed publishes are tagged `[FORCE BYPASSED: ...]`.
3. **`git log`** — Curio commits everything it does. Reverting a bad publish is `git revert <commit>` followed by `curio sync` to push the rollback to Confluence.
4. **Service-path errors** — every command in `--json` mode emits a structured error envelope (`{"command": "...", "ok": false, "error": {"code": "...", "message": "...", "hint": "..."}}`) so your scripts can dispatch on `error.code`.

## Boundaries you should respect

- `published/` is **never** the first move for new content. Always go through `staged/` or `review/`.
- If you can't explain why a page belongs in `published/` in one sentence, it's not ready.
- Confidence alone is not enough to publish. Quality and human usability matter equally.
- New taxonomy nodes are agent proposals against your `NORTHSTAR.md`/`config.yaml` — review them before merging the change.
- Confluence is the human review surface, not a database. Anything you want a colleague to inspect must show up in the Confluence Review tree.

## Where to read more

- `HARNESS.md` — the canonical operating contract that governs every provider
- `ARCHITECTURE.md` — layer boundaries (curio-rs vs. harness)
- `docs/design/process.md` — the long-form editorial pipeline design
- `docs/design/operating-contract.md` — the editorial loop colleagues are expected to follow
- `docs/agent-cli-contract.md` — machine-readable JSON shapes for every command
