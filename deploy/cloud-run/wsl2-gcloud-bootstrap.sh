#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
default_env_file="$script_dir/wsl2-gcloud.local.env"

usage() {
  cat <<'EOF'
Usage: wsl2-gcloud-bootstrap.sh [--project PROJECT_ID] [--account ACCOUNT] [--config-dir DIR] [--env-file PATH]

Bootstraps a writable Linux gcloud config for WSL2 and validates basic Google API connectivity.

Defaults:
  --config-dir   /tmp/gcloud-curio
  --project      leave unchanged
  --account      leave unchanged
  --env-file     deploy/cloud-run/wsl2-gcloud.local.env if present

The script refuses to proceed unless it has either:
  - explicit CLI arguments, or
  - a local overlay file with defaults.
EOF
}

project=""
account=""
config_dir="${CLOUDSDK_CONFIG:-/tmp/gcloud-curio}"
gcloud_bin="${CURIO_GCLOUD_SDK:-/usr/bin/gcloud}"
env_file=""
has_explicit_input="false"

if [[ -f "$default_env_file" ]]; then
  # Optional untracked overrides for a single operator machine.
  # shellcheck disable=SC1090
  source "$default_env_file"
  has_explicit_input="true"
fi

while [[ $# -gt 0 ]]; do
  case "$1" in
    --project)
      project="${2:-}"
      has_explicit_input="true"
      shift 2
      ;;
    --account)
      account="${2:-}"
      has_explicit_input="true"
      shift 2
      ;;
    --config-dir)
      config_dir="${2:-}"
      shift 2
      ;;
    --env-file)
      env_file="${2:-}"
      has_explicit_input="true"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

if [[ -n "$env_file" && -f "$env_file" ]]; then
  # shellcheck disable=SC1090
  source "$env_file"
  has_explicit_input="true"
fi

project="${CURIO_GCLOUD_PROJECT:-$project}"
account="${CURIO_GCLOUD_ACCOUNT:-$account}"
config_dir="${CURIO_GCLOUD_CONFIG_DIR:-$config_dir}"
gcloud_bin="${CURIO_GCLOUD_SDK:-$gcloud_bin}"

if [[ "$has_explicit_input" != "true" ]]; then
  echo "Refusing to run without explicit inputs." >&2
  echo "Provide --account/--project or create deploy/cloud-run/wsl2-gcloud.local.env from the example." >&2
  exit 2
fi

if [[ ! -x "$gcloud_bin" ]]; then
  echo "Missing Linux gcloud binary at: $gcloud_bin" >&2
  exit 1
fi

echo "Using Linux Cloud SDK: $gcloud_bin"
"$gcloud_bin" --version | sed -n '1,4p'

mkdir -p "$config_dir"
if [[ -d "${HOME}/.config/gcloud" ]] && [[ ! -e "${config_dir}/credentials.db" ]]; then
  cp -a "${HOME}/.config/gcloud/." "$config_dir"/
fi
chmod -R u+rwX "$config_dir"
export CLOUDSDK_CONFIG="$config_dir"

echo "CLOUDSDK_CONFIG=$CLOUDSDK_CONFIG"

for host in oauth2.googleapis.com run.googleapis.com pubsub.googleapis.com secretmanager.googleapis.com aiplatform.googleapis.com; do
  if ! getent hosts "$host" >/dev/null 2>&1; then
    echo "DNS check failed for $host" >&2
    echo "If you are on WSL2, confirm Windows networking/VPN/DNS is healthy before retrying." >&2
    exit 1
  fi
done

if [[ -n "$account" ]]; then
  "$gcloud_bin" config set account "$account"
fi

if [[ -n "$project" ]]; then
  "$gcloud_bin" config set project "$project"
fi

echo
echo "Active account:"
"$gcloud_bin" auth list --filter=status:ACTIVE --format='value(account)'

echo
echo "Active project:"
"$gcloud_bin" config get-value project

echo
echo "Connectivity check: Google APIs resolve and the Linux SDK can read its config."
