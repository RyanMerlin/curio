#!/usr/bin/env bash
# migrate-to-gcp.sh — copy local state to GCS and print Cloud Run env var commands
#
# Usage:
#   ./deploy/cloud-run/migrate-to-gcp.sh \
#     --project   your-project-id \
#     --bucket    your-project-curio-state \
#     --service   curio-control-plane \
#     --region    us-central1
#
# Prerequisites:
#   - gcloud authenticated: gcloud auth login
#   - gsutil/gcloud available
#   - docker-compose service stopped (or drained)
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
STATE_DIR="$SCRIPT_DIR/state"

# ── defaults (override via flags or env) ──────────────────────────────────────
PROJECT="${GCP_PROJECT:-}"
BUCKET="${GCS_BUCKET:-}"
SERVICE="${CLOUD_RUN_SERVICE:-curio-control-plane}"
REGION="${GCP_REGION:-us-central1}"
DRY_RUN=0

# ── colour helpers ────────────────────────────────────────────────────────────
green()  { printf '\033[0;32m✓ %s\033[0m\n' "$*"; }
red()    { printf '\033[0;31m✗ %s\033[0m\n' "$*"; }
yellow() { printf '\033[0;33m~ %s\033[0m\n' "$*"; }
bold()   { printf '\033[1m%s\033[0m\n' "$*"; }
fail()   { red "$*"; exit 1; }

usage() {
  echo "Usage: $0 [--project ID] [--bucket NAME] [--service NAME] [--region REGION] [--dry-run]"
  exit 1
}

# ── arg parsing ───────────────────────────────────────────────────────────────
while [[ $# -gt 0 ]]; do
  case "$1" in
    --project)  PROJECT="$2";  shift 2 ;;
    --bucket)   BUCKET="$2";   shift 2 ;;
    --service)  SERVICE="$2";  shift 2 ;;
    --region)   REGION="$2";   shift 2 ;;
    --dry-run)  DRY_RUN=1;     shift   ;;
    -h|--help)  usage ;;
    *) fail "Unknown flag: $1" ;;
  esac
done

[[ -n "$PROJECT" ]] || fail "--project is required (or set GCP_PROJECT)"
[[ -n "$BUCKET"  ]] || fail "--bucket is required (or set GCS_BUCKET)"

run() {
  if [[ $DRY_RUN -eq 1 ]]; then
    yellow "  [dry-run] $*"
  else
    "$@"
  fi
}

# ── dependency check ──────────────────────────────────────────────────────────
bold "==> Checking dependencies"
for dep in gcloud gsutil jq; do
  command -v "$dep" &>/dev/null || fail "Missing dependency: $dep"
  green "$dep found"
done

# ── 1. Upload state files ─────────────────────────────────────────────────────
bold ""
bold "==> Uploading state/ to gs://$BUCKET/"

for f in workspaces.json jobs.jsonl audit.jsonl; do
  src="$STATE_DIR/$f"
  if [[ -f "$src" ]]; then
    size=$(wc -c < "$src")
    yellow "  Uploading $f ($size bytes)"
    run gsutil -q cp "$src" "gs://$BUCKET/$f"
    green "  $f uploaded"
  else
    yellow "  $f not found — skipping (will be created by service on first run)"
  fi
done

# ── 2. GitLab token → Secret Manager ─────────────────────────────────────────
bold ""
bold "==> GitLab token → Secret Manager"

ENV_FILE="$SCRIPT_DIR/.env"
if [[ -f "$ENV_FILE" ]]; then
  GITLAB_TOKEN=$(grep '^CURIO_GITLAB_TOKEN=' "$ENV_FILE" | cut -d= -f2- | tr -d '"' | tr -d "'")
else
  GITLAB_TOKEN=""
fi

SECRET_NAME="CURIO_GITLAB_TOKEN"
if [[ -z "$GITLAB_TOKEN" ]]; then
  yellow "  No CURIO_GITLAB_TOKEN found in .env — skipping Secret Manager upload."
  yellow "  Create it manually:"
  yellow "    echo -n 'glpat-...' | gcloud secrets create $SECRET_NAME --project=$PROJECT --data-file=-"
else
  if gcloud secrets describe "$SECRET_NAME" --project="$PROJECT" &>/dev/null; then
    yellow "  Secret $SECRET_NAME already exists — adding a new version."
    if [[ $DRY_RUN -eq 1 ]]; then
      yellow "  [dry-run] printf %s '$SECRET_NAME' | gcloud secrets versions add $SECRET_NAME --project=$PROJECT --data-file=-"
    else
      printf %s "$GITLAB_TOKEN" | gcloud secrets versions add "$SECRET_NAME" --project="$PROJECT" --data-file=-
    fi
  else
    if [[ $DRY_RUN -eq 1 ]]; then
      yellow "  [dry-run] printf %s '$SECRET_NAME' | gcloud secrets create $SECRET_NAME --project=$PROJECT --replication-policy=automatic --data-file=-"
    else
      printf %s "$GITLAB_TOKEN" | gcloud secrets create "$SECRET_NAME" --project="$PROJECT" --replication-policy=automatic --data-file=-
    fi
  fi
  green "  Secret $SECRET_NAME updated"
fi

# ── 3. Gemini API key → Secret Manager ───────────────────────────────────────
bold ""
bold "==> Gemini API key → Secret Manager"

if [[ -f "$ENV_FILE" ]]; then
  GEMINI_KEY=$(grep '^CURIO_GEMINI_API_KEY=' "$ENV_FILE" | cut -d= -f2- | tr -d '"' | tr -d "'")
else
  GEMINI_KEY=""
fi

if [[ -n "$GEMINI_KEY" ]]; then
  yellow "  Found CURIO_GEMINI_API_KEY in .env."
  yellow "  On Cloud Run you should use the service account + Vertex AI instead."
  yellow "  Storing it in Secret Manager as CURIO_GEMINI_API_KEY just in case."
  GEMINI_SECRET="CURIO_GEMINI_API_KEY"
  if gcloud secrets describe "$GEMINI_SECRET" --project="$PROJECT" &>/dev/null; then
    if [[ $DRY_RUN -eq 1 ]]; then
      yellow "  [dry-run] printf %s '$GEMINI_SECRET' | gcloud secrets versions add $GEMINI_SECRET --project=$PROJECT --data-file=-"
    else
      printf %s "$GEMINI_KEY" | gcloud secrets versions add "$GEMINI_SECRET" --project="$PROJECT" --data-file=-
    fi
  else
    if [[ $DRY_RUN -eq 1 ]]; then
      yellow "  [dry-run] printf %s '$GEMINI_SECRET' | gcloud secrets create $GEMINI_SECRET --project=$PROJECT --replication-policy=automatic --data-file=-"
    else
      printf %s "$GEMINI_KEY" | gcloud secrets create "$GEMINI_SECRET" --project="$PROJECT" --replication-policy=automatic --data-file=-
    fi
  fi
  green "  Secret $GEMINI_SECRET updated"
else
  yellow "  No CURIO_GEMINI_API_KEY — Cloud Run will use Vertex AI via service account (correct)."
fi

# ── 4. Print Cloud Run env var update commands ────────────────────────────────
bold ""
bold "==> Post-migration: update Cloud Run env vars"
bold "    Run the following gcloud command to point the service at GCS state:"
echo ""

cat <<CMD
gcloud run services update "$SERVICE" \\
  --project="$PROJECT" \\
  --region="$REGION" \\
  --update-env-vars \\
CURIO_SERVICE_REGISTRY=/state/workspaces.json,\\
CURIO_SERVICE_JOBS=/state/jobs.jsonl,\\
CURIO_SERVICE_AUDIT=/state/audit.jsonl,\\
CURIO_SERVICE_CACHE=/tmp/curio/cache,\\
CURIO_SERVICE_PROVIDER_BACKEND=gemini,\\
CURIO_SERVICE_PROVIDER_MODEL=gemini-2.5-pro,\\
CURIO_SERVICE_AUTH_MODE=iap
CMD

echo ""
yellow "  Also wire the IAP audience (get from terraform output iap_audience):"
cat <<CMD
gcloud run services update "$SERVICE" \\
  --project="$PROJECT" \\
  --region="$REGION" \\
  --update-env-vars CURIO_IAP_AUDIENCE=/projects/PROJECT_NUMBER/global/backendServices/BACKEND_SERVICE_ID
CMD

# ── 5. Summary ────────────────────────────────────────────────────────────────
echo ""
if [[ $DRY_RUN -eq 1 ]]; then
  bold "Dry run complete — no changes made."
else
  bold "Migration complete."
  green "State files uploaded to gs://$BUCKET/"
  green "Secrets created/updated in project $PROJECT"
  bold ""
  bold "Checklist before cutting over traffic:"
  echo "  [ ] Run: gcloud run services update (commands printed above)"
  echo "  [ ] Point DNS A record for your domain at the terraform load_balancer_ip output"
  echo "  [ ] Wait for managed SSL cert to provision (10-30 min after DNS propagates)"
  echo "  [ ] Wire IAP audience: gcloud run services update ... CURIO_IAP_AUDIENCE=..."
  echo "  [ ] Run smoke-test.sh against the Cloud Run URL"
  echo "  [ ] Stop local docker-compose"
fi
