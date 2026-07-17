#!/usr/bin/env bash

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WORK_ROOT="${TMPDIR:-/tmp}"
DEMO_ROOT="$(mktemp -d "${WORK_ROOT%/}/curio-show-hn-demo.XXXXXX")"
KB_DIR="$DEMO_ROOT/wiki-demo"
WIKI_DIR="$KB_DIR/wiki"
ROUTES_FILE="$DEMO_ROOT/routes.json"
MANIFEST_FILE="$DEMO_ROOT/process-prepare.json"
DOCTOR_FILE="$DEMO_ROOT/doctor.json"
STATUS_BEFORE_FILE="$DEMO_ROOT/status-before.json"
STATUS_AFTER_ROUTE_FILE="$DEMO_ROOT/status-after-route.json"
STATUS_AFTER_PUBLISH_FILE="$DEMO_ROOT/status-after-publish.json"
PROCESS_FILE="$DEMO_ROOT/process-apply.json"
PUBLISH_FILE="$DEMO_ROOT/publish.json"
CURIO_BIN="${CURIO_BIN:-}"
CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$DEMO_ROOT/cargo-target}"

cleanup() {
  if [[ "${KEEP_DEMO_DIR:-0}" == "1" ]]; then
    printf 'Retained demo workspace at %s\n' "$DEMO_ROOT"
    return
  fi
  rm -rf "$DEMO_ROOT"
}

fail() {
  printf 'ERROR: %s\n' "$*" >&2
  exit 1
}

find_curio_bin() {
  local candidate

  if [[ -n "$CURIO_BIN" ]]; then
    [[ -x "$CURIO_BIN" ]] || fail "CURIO_BIN is not executable: $CURIO_BIN"
    printf '%s\n' "$CURIO_BIN"
    return
  fi

  for candidate in \
    "$REPO_ROOT/curio-rs/target/debug/curio" \
    "$REPO_ROOT/target/debug/curio" \
    "$CARGO_TARGET_DIR/debug/curio"
  do
    if [[ -x "$candidate" ]]; then
      printf '%s\n' "$candidate"
      return
    fi
  done

  printf 'Building curio...\n' >&2
  mkdir -p "$CARGO_TARGET_DIR"
  env -u RUSTC_WRAPPER cargo \
    --config 'build.rustc-wrapper=""' \
    build \
    --manifest-path "$REPO_ROOT/curio-rs/Cargo.toml" \
    --bin curio \
    --target-dir "$CARGO_TARGET_DIR" \
    >/dev/null

  for candidate in \
    "$REPO_ROOT/curio-rs/target/debug/curio" \
    "$REPO_ROOT/target/debug/curio" \
    "$CARGO_TARGET_DIR/debug/curio"
  do
    if [[ -x "$candidate" ]]; then
      printf '%s\n' "$candidate"
      return
    fi
  done

  fail "Built curio but could not find the binary under target/debug"
}

assert_file() {
  local path="$1"
  [[ -f "$path" ]] || fail "Expected file missing: $path"
}

assert_not_file() {
  local path="$1"
  [[ ! -f "$path" ]] || fail "Unexpected file present: $path"
}

assert_json_check() {
  local file="$1"
  local label="$2"
  local expected="$3"

  python3 - "$file" "$label" "$expected" <<'PY'
import json
import sys

path, label, expected = sys.argv[1:]
with open(path, "r", encoding="utf-8") as handle:
    raw = handle.read()

lines = raw.splitlines()
start = next(
    (index for index, line in enumerate(lines) if line.lstrip().startswith("{")),
    None,
)
if start is None:
    raise SystemExit(f"Could not find JSON object in {path}")

payload = json.loads("\n".join(lines[start:]))
checks = payload["data"]["infrastructure"]
for check in checks:
    if check["label"] == label:
        actual = "true" if check["ok"] else "false"
        if actual != expected:
            raise SystemExit(
                f"Expected {label}={expected} in {path}, found {actual}"
            )
        raise SystemExit(0)
raise SystemExit(f"Did not find infrastructure check {label} in {path}")
PY
}

assert_json_path() {
  local file="$1"
  local expression="$2"
  local expected="$3"

  python3 - "$file" "$expression" "$expected" <<'PY'
import json
import sys

path, expression, expected = sys.argv[1:]
with open(path, "r", encoding="utf-8") as handle:
    raw = handle.read()

lines = raw.splitlines()
start = next(
    (index for index, line in enumerate(lines) if line.lstrip().startswith("{")),
    None,
)
if start is None:
    raise SystemExit(f"Could not find JSON object in {path}")

payload = json.loads("\n".join(lines[start:]))

value = payload
for part in expression.split("."):
    if part.isdigit():
        value = value[int(part)]
    else:
        value = value[part]

actual = str(value)
if actual != expected:
    raise SystemExit(
        f"Expected {expression}={expected} in {path}, found {actual}"
    )
PY
}

assert_json_list_len() {
  local file="$1"
  local expression="$2"
  local expected="$3"

  python3 - "$file" "$expression" "$expected" <<'PY'
import json
import sys

path, expression, expected = sys.argv[1:]
with open(path, "r", encoding="utf-8") as handle:
    raw = handle.read()

lines = raw.splitlines()
start = next(
    (index for index, line in enumerate(lines) if line.lstrip().startswith("{")),
    None,
)
if start is None:
    raise SystemExit(f"Could not find JSON object in {path}")

payload = json.loads("\n".join(lines[start:]))

value = payload
for part in expression.split("."):
    if part.isdigit():
        value = value[int(part)]
    else:
        value = value[part]

actual = len(value)
if str(actual) != expected:
    raise SystemExit(
        f"Expected len({expression})={expected} in {path}, found {actual}"
    )
PY
}

create_intake_pages() {
  rm -f "$WIKI_DIR/intake/"*.md

  cat >"$WIKI_DIR/intake/show-hn-staged.md" <<'EOF'
---
id: show-hn-staged
title: Example Server Pre-Upgrade Validation Walkthrough
status: intake
source:
  kind: file
  id: file:wiki/intake/show-hn-staged.md
  origin_url: null
  summary: Synthetic launch demo page that should route cleanly into a staged upgrade guide.
category: []
keywords:
  - example-server
  - upgrade
  - validation
created_at: "2026-07-16T00:00:00Z"
updated_at: "2026-07-16T00:00:00Z"
confidence: null
cross_refs: []
content_hash: "show-hn-staged"
confluence_page_id: null
model_used: null
---

# Example Server Pre-Upgrade Validation Walkthrough

## Goal

This synthetic page demonstrates a launch-safe staged proposal for Example Server operators preparing an upgrade window. It is intentionally detailed enough to pass the publish gate after routing and to show that Curio can move a curated draft from intake to staged and then to published without any live Confluence dependency.

## Pre-upgrade checks

- Confirm the target build, maintenance window, and rollback owner.
- Verify recent backups for the application host and persistence layer.
- Capture the current service version, plugin inventory, and active authentication mode.
- Review environment-specific overrides so the upgrade runbook reflects the actual deployment instead of a generic checklist.

## Validation sequence

1. Export the current configuration snapshot and record where it is stored.
2. Run a smoke test against login, API, and background job paths before touching the deployment.
3. Freeze non-essential changes so the post-upgrade verification compares against a stable baseline.
4. Document the success criteria for rollback versus completion.

## Why this is publishable

The structure is deliberate, the content is synthetic, and the page is useful on its own. That makes it appropriate for a deterministic demo of Curio's staged-to-published flow.
EOF

  cat >"$WIKI_DIR/intake/show-hn-review.md" <<'EOF'
---
id: show-hn-review
title: Example Server TODO Fragments
status: intake
source:
  kind: file
  id: file:wiki/intake/show-hn-review.md
  origin_url: null
  summary: Synthetic weak note that should remain in review.
category: []
keywords:
  - example-server
  - review
created_at: "2026-07-16T00:00:00Z"
updated_at: "2026-07-16T00:00:00Z"
confidence: null
cross_refs: []
content_hash: "show-hn-review"
confluence_page_id: null
model_used: null
---

# Example Server TODO Fragments

- check upgrade later
- maybe ask support
- fill in exact steps
- unclear if this belongs under upgrade or administration

This note is intentionally incomplete so the deterministic route file can prove the review lane still works in a credential-free demo.
EOF
}

create_route_file() {
  cat >"$ROUTES_FILE" <<'EOF'
[
  [
    "show-hn-staged",
    {
      "category": ["product-tree", "example-server", "upgrade"],
      "keywords": ["example-server", "upgrade", "validation"],
      "confidence": 0.94,
      "status": "staged",
      "summary": "Synthetic staged upgrade validation walkthrough for the Show HN demo.",
      "cross_refs": [],
      "review_reason": null,
      "proposed_new_subtree": null,
      "proposal_rationale": null,
      "merge_target": null,
      "model_used": "manual",
      "decision_section_markdown": "## Curation Decision\n\n- Route: `product-tree/example-server/upgrade`\n- Recommended action: stage and publish after deterministic verification\n- Why: the body is specific, structured, and directly useful to operators\n",
      "proposed_body_markdown": "# Example Server Pre-Upgrade Validation Walkthrough\n\n## Scope\n\nThis synthetic Curio demo page shows a launch-safe staged proposal for Example Server upgrades. It is written as a curated operator guide rather than a raw intake capture so the publish step exercises the same quality gate a real proposal would face.\n\n## Required preflight checks\n\n- Confirm the target build, rollback owner, and maintenance window.\n- Capture the current service version, active authentication mode, and plugin inventory.\n- Verify recent backups for the application host and persistence layer.\n- Freeze non-essential changes so the post-upgrade comparison uses a stable baseline.\n\n## Validation sequence\n\n1. Export the current configuration snapshot and record where it is stored.\n2. Run a smoke test against login, API, and background job paths before the upgrade begins.\n3. Record success criteria for completion versus rollback, including the exact signals reviewers should inspect.\n4. After the upgrade, re-run the smoke test and confirm the expected version string and service health checks.\n\n## Launch demo notes\n\nThis page is synthetic, deterministic, and credential-free. It exists to demonstrate that Curio can route strong intake content into `staged/`, preserve the editorial rationale, and publish the resulting page locally without any Confluence credentials.\n",
      "body_rewrite_kind": "full_synthesis",
      "merge_into_slug": null
    }
  ],
  [
    "show-hn-review",
    {
      "category": ["product-tree", "example-server", "administration"],
      "keywords": ["example-server", "review", "todo"],
      "confidence": 0.42,
      "status": "review",
      "summary": "Synthetic weak note that should remain in review.",
      "cross_refs": [],
      "review_reason": "Content is incomplete and does not support publication without a real operating procedure.",
      "proposed_new_subtree": null,
      "proposal_rationale": null,
      "merge_target": null,
      "model_used": "manual",
      "decision_section_markdown": "## Curation Decision\n\n- Route: `product-tree/example-server/administration`\n- Recommended action: keep in review\n- Why: the note is intentionally incomplete and taxonomy-fit is still ambiguous\n",
      "proposed_body_markdown": null,
      "body_rewrite_kind": "none",
      "merge_into_slug": null
    }
  ]
]
EOF
}

normalize_demo_config() {
  perl -0pi -e 's/confluence_email: "\$\{CURIO_CONFLUENCE_EMAIL\}"/confluence_email: ""/' "$KB_DIR/.curio.yaml"
  perl -0pi -e 's/space_key: "\$\{CURIO_SPACE_KEY\}"/space_key: ""/' "$KB_DIR/.curio.yaml"
  perl -0pi -e 's/wiki_dir: "\."/wiki_dir: "wiki"/' "$KB_DIR/.curio.yaml"
}

reshape_workspace_layout() {
  rm -f "$KB_DIR/_config/northstar.md"
  mkdir -p "$WIKI_DIR"
  mv "$KB_DIR/NORTHSTAR.md" "$WIKI_DIR/NORTHSTAR.md"
  mv "$KB_DIR/_admin" "$WIKI_DIR/_admin"
  mv "$KB_DIR/_config" "$WIKI_DIR/_config"
  mv "$KB_DIR/intake" "$WIKI_DIR/intake"
  mv "$KB_DIR/staged" "$WIKI_DIR/staged"
  mv "$KB_DIR/review" "$WIKI_DIR/review"
  mv "$KB_DIR/published" "$WIKI_DIR/published"
}

run_demo() {
  local curio_bin="$1"

  printf 'Preparing synthetic Show HN demo workspace at %s\n' "$KB_DIR"
  cp -R "$REPO_ROOT/docs/wiki-demo" "$KB_DIR"
  normalize_demo_config
  reshape_workspace_layout
  git -C "$KB_DIR" init -b main >/dev/null
  git -C "$KB_DIR" config user.name "Curio Demo"
  git -C "$KB_DIR" config user.email "curio-demo@example.invalid"
  git -C "$KB_DIR" add .
  git -C "$KB_DIR" commit -m "seed synthetic demo workspace" >/dev/null

  unset CURIO_CONFLUENCE_URL
  unset CURIO_CONFLUENCE_EMAIL
  unset CURIO_SPACE_KEY
  unset CURIO_CONFLUENCE_TOKEN

  create_intake_pages
  create_route_file

  printf 'Running curio doctor/status/process --prepare...\n'
  "$curio_bin" --kb-dir "$KB_DIR" doctor --json >"$DOCTOR_FILE"
  "$curio_bin" --kb-dir "$KB_DIR" status --json >"$STATUS_BEFORE_FILE"
  "$curio_bin" --kb-dir "$KB_DIR" process --prepare >"$MANIFEST_FILE"

  assert_json_check "$DOCTOR_FILE" "kb.config" "true"
  assert_json_check "$DOCTOR_FILE" "kb.northstar" "true"
  assert_json_check "$DOCTOR_FILE" "kb.git" "true"
  assert_json_check "$DOCTOR_FILE" "kb.confluence.url" "true"
  assert_json_check "$DOCTOR_FILE" "kb.confluence.email" "false"
  assert_json_check "$DOCTOR_FILE" "kb.confluence.token" "false"
  assert_json_check "$DOCTOR_FILE" "kb.confluence.space_key" "false"
  assert_json_check "$DOCTOR_FILE" "kb.confluence.auth" "false"
  assert_json_path "$MANIFEST_FILE" "page_count" "2"

  printf 'Applying deterministic route file...\n'
  "$curio_bin" --kb-dir "$KB_DIR" process --route-file "$ROUTES_FILE" --json >"$PROCESS_FILE"
  "$curio_bin" --kb-dir "$KB_DIR" status --json >"$STATUS_AFTER_ROUTE_FILE"

  assert_file "$WIKI_DIR/staged/product-tree/example-server/upgrade/show-hn-staged.md"
  assert_file "$WIKI_DIR/staged/product-tree/example-server/upgrade/show-hn-staged.analysis.json"
  assert_file "$WIKI_DIR/review/product-tree/example-server/administration/show-hn-review.md"
  assert_file "$WIKI_DIR/review/product-tree/example-server/administration/show-hn-review.analysis.json"
  assert_not_file "$WIKI_DIR/intake/show-hn-staged.md"
  assert_not_file "$WIKI_DIR/intake/show-hn-review.md"
  assert_json_list_len "$PROCESS_FILE" "data.errors" "0"

  printf 'Publishing the staged proposal...\n'
  "$curio_bin" --kb-dir "$KB_DIR" publish show-hn-staged --category product-tree/example-server/upgrade --json >"$PUBLISH_FILE"
  "$curio_bin" --kb-dir "$KB_DIR" status --json >"$STATUS_AFTER_PUBLISH_FILE"

  assert_not_file "$WIKI_DIR/staged/product-tree/example-server/upgrade/show-hn-staged.md"
  assert_file "$WIKI_DIR/published/product-tree/example-server/upgrade/show-hn-staged.md"
  assert_file "$WIKI_DIR/published/product-tree/example-server/upgrade/show-hn-staged.analysis.json"
  assert_file "$WIKI_DIR/review/product-tree/example-server/administration/show-hn-review.md"
  assert_json_path "$PUBLISH_FILE" "data.slug" "show-hn-staged"
  assert_json_path "$PUBLISH_FILE" "data.published_to" "published/product-tree/example-server/upgrade/show-hn-staged.md"

  printf '\n%s\n' 'Verification summary'
  printf '%s\n' '- doctor confirms KB structure is healthy, the demo URL placeholder is present, and email/token/space/auth checks remain unavailable as expected'
  printf '%s\n' '- process --prepare emitted a 2-page manifest from the synthetic intake set'
  printf '%s\n' '- route file produced one review proposal and one staged proposal'
  printf '%s\n' '- publish moved the staged proposal into published while leaving the review proposal untouched'
  printf '\n%s\n' 'Artifacts'
  printf '%s\n' "- demo workspace: $KB_DIR"
  printf '%s\n' "- doctor report: $DOCTOR_FILE"
  printf '%s\n' "- manifest: $MANIFEST_FILE"
  printf '%s\n' "- route file: $ROUTES_FILE"
  printf '%s\n' "- publish report: $PUBLISH_FILE"
}

trap cleanup EXIT

CURIO_BIN="$(find_curio_bin)"
run_demo "$CURIO_BIN"
