#!/usr/bin/env bash
set -euo pipefail

if [[ "${CURIO_LIVE_CONFLUENCE:-}" != "1" ]]; then
  echo "Live Confluence smoke test is opt-in. Set CURIO_LIVE_CONFLUENCE=1."
  exit 0
fi

: "${CURIO_CONFLUENCE_URL:?Set CURIO_CONFLUENCE_URL to the sandbox site /wiki URL}"
: "${CURIO_CONFLUENCE_EMAIL:?Set CURIO_CONFLUENCE_EMAIL}"
: "${CURIO_CONFLUENCE_TOKEN:?Set CURIO_CONFLUENCE_TOKEN (it is never printed)}"
: "${CURIO_CONFLUENCE_PARENT_PAGE_ID:?Set CURIO_CONFLUENCE_PARENT_PAGE_ID to the dedicated sandbox CURIO root}"
: "${CURIO_KB_DIR:?Set CURIO_KB_DIR to the sandbox KB directory containing wiki/ and .curio.yaml}"
space_key="${CURIO_SPACE_KEY:-CURIO}"
if [[ "$space_key" != "CURIO" ]]; then
  echo "Refusing live smoke test outside the dedicated CURIO sandbox."
  exit 1
fi
if [[ "$CURIO_CONFLUENCE_URL" != https://* ]]; then
  echo "Refusing live smoke test unless CURIO_CONFLUENCE_URL uses HTTPS."
  exit 1
fi

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
manifest="$repo_root/curio-rs/Cargo.toml"

echo "Running credential-redacted Confluence doctor against the CURIO sandbox..."
cargo run --quiet --manifest-path "$manifest" --bin curio -- --kb-dir "$CURIO_KB_DIR" doctor --json
echo "Running credential-redacted full-refresh smoke sync..."
cargo run --quiet --manifest-path "$manifest" --bin curio -- --kb-dir "$CURIO_KB_DIR" sync --all --docs-only
