#!/usr/bin/env bash
# Local 3-KB smoke test for curio-service running in docker-compose.
# Exercises both the HTTP service path and the in-container CLI path.

set -euo pipefail

BASE_URL="${CURIO_SERVICE_URL:-http://localhost:8080}"
WORKSPACES=("curio-wiki" "partner-business" "fde-uc-repo")

green()  { printf '\033[0;32m✓ %s\033[0m\n' "$*"; }
red()    { printf '\033[0;31m✗ %s\033[0m\n' "$*"; }
bold()   { printf '\033[1m%s\033[0m\n' "$*"; }
fail()   { red "$*"; exit 1; }

# ── 1. service liveness ───────────────────────────────────────────────────────
bold "==> /healthz"
curl -sf "$BASE_URL/healthz" | jq -e '.ok == true' >/dev/null \
  || fail "/healthz did not return ok=true"
green "/healthz ok"

bold "==> /readyz"
curl -sf "$BASE_URL/readyz" | jq -e '.ok == true' >/dev/null \
  || fail "/readyz did not return ok=true"
green "/readyz ok"

# ── 2. workspace registry exposes all 3 KBs ───────────────────────────────────
bold "==> /v1/workspaces"
resp=$(curl -sf "$BASE_URL/v1/workspaces")
count=$(echo "$resp" | jq '.workspaces | length')
[[ "$count" == "3" ]] || fail "expected 3 workspaces, got $count: $resp"
green "found 3 workspaces"

# ── 3. per-workspace healthz ──────────────────────────────────────────────────
for ws in "${WORKSPACES[@]}"; do
  bold "==> /v1/workspaces/$ws/healthz"
  status=$(curl -sf "$BASE_URL/v1/workspaces/$ws/healthz" | jq -r '.status')
  [[ "$status" == "active" ]] || fail "workspace $ws status=$status (expected active)"
  green "$ws active"
done

# ── 4. CLI path inside the container ──────────────────────────────────────────
for ws in "${WORKSPACES[@]}"; do
  bold "==> docker compose exec curio --kb-dir /kb/$ws status"
  out=$(docker compose exec -T curio-service curio --kb-dir "/kb/$ws" status 2>&1) \
    || fail "CLI status failed for $ws:\n$out"
  echo "$out" | grep -q "intake" || fail "unexpected status output for $ws:\n$out"
  green "CLI status ok for $ws"
done

bold ""
bold "Local 3-KB smoke test PASSED."
