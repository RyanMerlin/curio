#!/usr/bin/env bash
# smoke-test.sh — end-to-end smoke test against a running curio-service
set -euo pipefail

BASE_URL="${CURIO_SERVICE_URL:-http://localhost:8080}"
BEARER_TOKEN="${CURIO_SERVICE_BEARER_TOKEN:-}"
WORKSPACE_ID="${CURIO_SMOKE_WORKSPACE:-}"
MAX_WAIT=120  # seconds to poll for job completion

# ── colour helpers ────────────────────────────────────────────────────────────
green()  { printf '\033[0;32m✓ %s\033[0m\n' "$*"; }
red()    { printf '\033[0;31m✗ %s\033[0m\n' "$*"; }
yellow() { printf '\033[0;33m~ %s\033[0m\n' "$*"; }
bold()   { printf '\033[1m%s\033[0m\n' "$*"; }
fail()   { red "$*"; exit 1; }

auth_header() {
  if [[ -n "$BEARER_TOKEN" ]]; then
    echo "-H" "Authorization: Bearer $BEARER_TOKEN"
  fi
}

# ── 1. healthz ────────────────────────────────────────────────────────────────
bold "==> 1/4  GET /healthz"
resp=$(curl -sf "$(auth_header)" "$BASE_URL/healthz") || fail "GET /healthz failed"
ok=$(echo "$resp" | jq -r '.ok')
[[ "$ok" == "true" ]] || fail "/healthz returned ok=false: $resp"
green "/healthz ok"

# ── 2. readyz ─────────────────────────────────────────────────────────────────
bold "==> 2/4  GET /readyz"
resp=$(curl -sf "$(auth_header)" "$BASE_URL/readyz") || fail "GET /readyz failed"
ok=$(echo "$resp" | jq -r '.ok')
[[ "$ok" == "true" ]] || fail "/readyz returned ok=false: $resp"
green "/readyz ok"

# ── 3. workspace list ─────────────────────────────────────────────────────────
bold "==> 3/4  GET /workspaces"
resp=$(curl -sf "$(auth_header)" "$BASE_URL/workspaces") || fail "GET /workspaces failed"
count=$(echo "$resp" | jq '.workspaces | length')
if [[ "$count" -eq 0 ]]; then
  yellow "No workspaces configured — skipping job submission test."
  bold ""
  bold "Basic smoke test PASSED (no workspaces to run a job against)."
  echo "  Add a workspace via setup-local.sh and re-run to test job submission."
  exit 0
fi

if [[ -z "$WORKSPACE_ID" ]]; then
  WORKSPACE_ID=$(echo "$resp" | jq -r '.workspaces[0].workspace_id')
  yellow "CURIO_SMOKE_WORKSPACE not set — using first workspace: $WORKSPACE_ID"
fi
green "/workspaces returned $count workspace(s), using $WORKSPACE_ID"

# ── 4. submit + poll ──────────────────────────────────────────────────────────
bold "==> 4/4  POST /jobs (read-only analyze)"
job_body=$(jq -n \
  --arg ws "$WORKSPACE_ID" \
  '{
    workspace_id: $ws,
    job_type: "analyze",
    operation: "smoke-test",
    write_mode: "read_only",
    correlation_id: "smoke-test-001",
    trigger: "manual",
    actor: "smoke-test.sh",
    inputs: {"note": "automated smoke test — read-only, no git writes"}
  }')

submit_resp=$(curl -sf "$(auth_header)" \
  -X POST "$BASE_URL/jobs" \
  -H "Content-Type: application/json" \
  -d "$job_body") || fail "POST /jobs failed"

job_id=$(echo "$submit_resp" | jq -r '.job_id // empty')
[[ -n "$job_id" ]] || fail "POST /jobs did not return a job_id: $submit_resp"
green "Job submitted: $job_id"

# Poll for completion
bold "    Polling GET /jobs/$job_id ..."
elapsed=0
while true; do
  status_resp=$(curl -sf "$(auth_header)" "$BASE_URL/jobs/$job_id") || fail "GET /jobs/$job_id failed"
  status=$(echo "$status_resp" | jq -r '.status')
  case "$status" in
    completed)
      green "Job $job_id completed"
      echo "$status_resp" | jq -r '  .result.summary // "(no summary)"'
      break
      ;;
    failed)
      fail "Job $job_id failed: $(echo "$status_resp" | jq -r '.error // .result')"
      ;;
    *)
      printf "    status=%s (${elapsed}s)...\r" "$status"
      sleep 3
      elapsed=$((elapsed + 3))
      if [[ $elapsed -ge $MAX_WAIT ]]; then
        fail "Job $job_id did not complete within ${MAX_WAIT}s (last status: $status)"
      fi
      ;;
  esac
done

echo ""
bold "All smoke tests PASSED"
