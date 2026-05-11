#!/usr/bin/env bash
# setup-local.sh — interactive setup for local curio-service (docker-compose)
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ENV_FILE="$SCRIPT_DIR/.env"
WORKSPACES_FILE="$SCRIPT_DIR/state/workspaces.json"

# ── colour helpers ────────────────────────────────────────────────────────────
green()  { printf '\033[0;32m%s\033[0m\n' "$*"; }
yellow() { printf '\033[0;33m%s\033[0m\n' "$*"; }
red()    { printf '\033[0;31m%s\033[0m\n' "$*"; }
bold()   { printf '\033[1m%s\033[0m\n' "$*"; }

prompt() {
  local var_name="$1" prompt_text="$2" default="${3:-}"
  if [[ -n "$default" ]]; then
    read -rp "$prompt_text [$default]: " val
    eval "$var_name='${val:-$default}'"
  else
    read -rp "$prompt_text: " val
    eval "$var_name='$val'"
  fi
}

# ── dependency check ──────────────────────────────────────────────────────────
bold "==> Checking dependencies"
missing=0
for dep in docker docker-compose curl jq; do
  if ! command -v "$dep" &>/dev/null; then
    red "  missing: $dep"
    missing=1
  else
    green "  found: $dep"
  fi
done
if [[ $missing -eq 1 ]]; then
  red "Install missing dependencies and re-run."
  exit 1
fi

echo ""

# ── .env setup ────────────────────────────────────────────────────────────────
bold "==> Configuring .env"

if [[ -f "$ENV_FILE" ]]; then
  yellow "  $ENV_FILE already exists. Edit it manually or delete it to re-run setup."
else
  cp "$SCRIPT_DIR/.env.example" "$ENV_FILE"
  green "  Created $ENV_FILE from .env.example"

  echo ""
  yellow "  Gemini API key (get one free at https://aistudio.google.com/apikey)"
  yellow "  This is the simplest local-dev auth — no GCP project needed."
  prompt gemini_key "  CURIO_GEMINI_API_KEY" ""
  if [[ -n "$gemini_key" ]]; then
    sed -i "s|^CURIO_GEMINI_API_KEY=.*|CURIO_GEMINI_API_KEY=$gemini_key|" "$ENV_FILE"
    green "  API key saved."
  else
    yellow "  No key set — you can add it to $ENV_FILE later."
  fi

  echo ""
  yellow "  GitLab token (required if workspace repo_url is a private GitLab repo)"
  yellow "  Leave blank for read-only public repos or to skip for now."
  prompt gitlab_token "  CURIO_GITLAB_TOKEN" ""
  if [[ -n "$gitlab_token" ]]; then
    sed -i "s|^CURIO_GITLAB_TOKEN=.*|CURIO_GITLAB_TOKEN=$gitlab_token|" "$ENV_FILE"
    green "  GitLab token saved."
  fi
fi

echo ""

# ── workspace setup ───────────────────────────────────────────────────────────
bold "==> Configuring workspaces"
mkdir -p "$SCRIPT_DIR/state"

if [[ -f "$WORKSPACES_FILE" ]]; then
  yellow "  $WORKSPACES_FILE already exists."
  echo "  Current workspaces:"
  jq -r '.records[] | "    \(.workspace_id)  \(.repo_url)  [\(.status)]"' "$WORKSPACES_FILE" 2>/dev/null || echo "    (could not parse)"
  echo ""
  read -rp "  Add another workspace? [y/N]: " add_ws
  [[ "$add_ws" != "y" && "$add_ws" != "Y" ]] && add_ws="n"
else
  cat > "$WORKSPACES_FILE" <<'JSON'
{"records": []}
JSON
  green "  Created empty $WORKSPACES_FILE"
  add_ws="y"
fi

if [[ "$add_ws" == "y" ]]; then
  echo ""
  yellow "  Enter workspace details (press Enter to accept defaults):"
  prompt ws_id      "  workspace_id (slug, no spaces)" "main"
  prompt ws_name    "  display_name" "Main KB"
  prompt ws_url     "  repo_url (HTTPS git URL)" ""
  prompt ws_branch  "  default_branch" "main"
  prompt ws_kb_root "  kb_root (path within repo)" "wiki"

  if [[ -z "$ws_url" ]]; then
    yellow "  No repo_url provided — workspace not added."
  else
    existing=$(jq '.records' "$WORKSPACES_FILE")
    new_record=$(jq -n \
      --arg id "$ws_id" \
      --arg name "$ws_name" \
      --arg url "$ws_url" \
      --arg branch "$ws_branch" \
      --arg kb_root "$ws_kb_root" \
      '{
        workspace_id: $id,
        display_name: $name,
        repo_url: $url,
        default_branch: $branch,
        kb_root: $kb_root,
        write_policy: "read_only",
        status: "active",
        provider_defaults: {}
      }')
    jq --argjson r "$new_record" '.records += [$r]' "$WORKSPACES_FILE" > /tmp/ws_tmp.json
    mv /tmp/ws_tmp.json "$WORKSPACES_FILE"
    green "  Workspace '$ws_id' added."
  fi
fi

echo ""

# ── build + start ─────────────────────────────────────────────────────────────
bold "==> Building Docker image"
docker-compose -f "$SCRIPT_DIR/docker-compose.yml" build

echo ""
bold "==> Starting curio-service"
docker-compose -f "$SCRIPT_DIR/docker-compose.yml" up -d

echo ""
bold "==> Waiting for service to become ready..."
timeout=30
elapsed=0
while ! curl -sf http://localhost:8080/healthz &>/dev/null; do
  sleep 1
  elapsed=$((elapsed + 1))
  if [[ $elapsed -ge $timeout ]]; then
    red "Service did not become healthy within ${timeout}s."
    red "Check logs: docker-compose -f deploy/cloud-run/docker-compose.yml logs"
    exit 1
  fi
done

green ""
green "curio-service is running at http://localhost:8080"
green ""
green "Next steps:"
green "  Run smoke tests:   ./deploy/cloud-run/smoke-test.sh"
green "  View logs:         docker-compose -f deploy/cloud-run/docker-compose.yml logs -f"
green "  Stop:              docker-compose -f deploy/cloud-run/docker-compose.yml down"
