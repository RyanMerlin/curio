#!/usr/bin/env bash
set -euo pipefail

if [[ "${CURIO_LIVE_CONFLUENCE:-}" != "1" ]]; then
  echo "Live Confluence acceptance is opt-in. Set CURIO_LIVE_CONFLUENCE=1."
  exit 0
fi

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
if [[ -f "$repo_root/.env" ]]; then
  set -a
  . "$repo_root/.env"
  set +a
fi
: "${CURIO_CONFLUENCE_URL:?Set CURIO_CONFLUENCE_URL}"
: "${CURIO_CONFLUENCE_EMAIL:?Set CURIO_CONFLUENCE_EMAIL}"
: "${CURIO_CONFLUENCE_TOKEN:?Set CURIO_CONFLUENCE_TOKEN}"
: "${CURIO_CONFLUENCE_PARENT_PAGE_ID:?Set CURIO_CONFLUENCE_PARENT_PAGE_ID}"
: "${CURIO_KB_DIR:?Set CURIO_KB_DIR to the sandbox KB context}"
[[ "${CURIO_SPACE_KEY:-CURIO}" == CURIO ]] || { echo "Refusing a non-CURIO space."; exit 1; }
[[ "$CURIO_CONFLUENCE_URL" == https://* ]] || { echo "Refusing a non-HTTPS site."; exit 1; }

base_url="${CURIO_CONFLUENCE_URL%/}"
target_dir="$(cargo metadata --quiet --no-deps --format-version 1 --manifest-path "$repo_root/curio-rs/Cargo.toml" | jq -r '.target_directory')"
echo "Building release curio binary..."
cargo build --release --manifest-path "$repo_root/curio-rs/Cargo.toml" --bin curio >/dev/null
curio_bin="$target_dir/release/curio"
[[ -x "$curio_bin" ]] || { echo "Release curio binary was not created at $curio_bin."; exit 1; }
tmp="$(mktemp -d)"
fixture="$tmp/kb"
cp -a "$repo_root/docs/wiki-demo/." "$fixture/"
ids=()
cleanup() {
  for id in "${ids[@]}"; do
    curl -fsS -u "$CURIO_CONFLUENCE_EMAIL:$CURIO_CONFLUENCE_TOKEN" -X DELETE \
      "$base_url/rest/api/content/$id" >/dev/null 2>&1 || true
  done
  rm -rf "$tmp"
}
trap cleanup EXIT

urlencode() { jq -rn --arg x "$1" '$x|@uri'; }
search() {
  local title="$1" ancestor="${2:-}" url
  url="$base_url/rest/api/content?spaceKey=CURIO&title=$(urlencode "$title")&expand=version,ancestors"
  [[ -z "$ancestor" ]] || url+="&ancestor=$ancestor"
  curl -fsS -u "$CURIO_CONFLUENCE_EMAIL:$CURIO_CONFLUENCE_TOKEN" "$url"
}
page_id() { search "$1" "${2:-}" | jq -r '.results[0].id // empty'; }
version() { curl -fsS -u "$CURIO_CONFLUENCE_EMAIL:$CURIO_CONFLUENCE_TOKEN" "$base_url/rest/api/content/$1?expand=version" | jq -r '.version.number // 0'; }
sync_all() {
  local output
  if ! output=$("$curio_bin" --kb-dir "$fixture" --json sync --all --docs-only 2>&1); then
    echo "$output" >&2
    return 1
  fi
}
create_page() {
  local title="$1" parent="$2" response id
  response="$(jq -n --arg title "$title" --arg parent "$parent" \
    '{type:"page",status:"current",title:$title,body:{storage:{representation:"storage",value:"<p>Acceptance fixture.</p>"}},space:{key:"CURIO"},ancestors:(if $parent == "" then [] else [{id:$parent}] end)}' \
    | curl -fsS -u "$CURIO_CONFLUENCE_EMAIL:$CURIO_CONFLUENCE_TOKEN" \
      -H 'Content-Type: application/json' -X POST "$base_url/rest/api/content" --data-binary @-)"
  id="$(jq -r '.id // empty' <<<"$response")"
  [[ -n "$id" ]] || { echo "Could not create acceptance page." >&2; exit 1; }
  ids+=("$id")
  echo "$id"
}
assert_http() {
  local expected="$1" url="$2" actual
  actual="$(curl -sS -o /dev/null -w '%{http_code}' -u "$CURIO_CONFLUENCE_EMAIL:$CURIO_CONFLUENCE_TOKEN" "$url")"
  [[ "$actual" == "$expected" ]] || { echo "Expected $expected, got $actual: $url"; exit 1; }
}

echo "1/6 initial and idempotent refresh"
sync_all
sync_all

published_id="$(page_id Published "$CURIO_CONFLUENCE_PARENT_PAGE_ID")"
product_id="$(page_id Product-tree "$published_id")"
[[ -n "$published_id" && -n "$product_id" ]] || { echo "Managed pages not found."; exit 1; }
stamp="$(date +%s)"

echo "2/6 preserve an unowned manual page"
manual_id="$(create_page "Curio Acceptance Manual $stamp" "$published_id")"
sync_all
assert_http 200 "$base_url/rest/api/content/$manual_id"

echo "3/6 delete an owned page after local removal"
owned_title="Curio Acceptance Owned $stamp"
cat > "$fixture/published/product-tree/acceptance-owned.md" <<EOF
---
id: acceptance-owned-$stamp
title: $owned_title
status: published
source:
  kind: file
  id: file:acceptance-owned-$stamp
  summary: acceptance fixture
category: [product-tree]
keywords: [acceptance]
created_at: '2026-07-17T00:00:00Z'
updated_at: '2026-07-17T00:00:00Z'
confidence: 0.99
cross_refs: []
content_hash: acceptance-$stamp
confluence_page_id: null
model_used: null
---

Owned acceptance fixture.
EOF
sync_all
owned_id="$(page_id "$owned_title" "$product_id")"
[[ -n "$owned_id" ]] || { echo "Owned page was not created."; exit 1; }
rm "$fixture/published/product-tree/acceptance-owned.md"
sync_all
assert_http 404 "$base_url/rest/api/content/$owned_id"

echo "4/6 propagate a local update"
target="$fixture/published/product-tree/demo-publish.md"
target_id="$(page_id 'Demo Publish Checklist' "$product_id")"
before="$(version "$target_id")"
printf '\nAcceptance update %s.\n' "$stamp" >> "$target"
sync_all
after="$(version "$target_id")"
[[ "$after" -gt "$before" ]] || { echo "Remote version did not advance."; exit 1; }

echo "5/6 reject an outside-root title collision without mutation"
outside_title="Curio Acceptance Outside $stamp"
outside_id="$(create_page "$outside_title" "")"
outside_before="$(version "$outside_id")"
cat > "$fixture/published/product-tree/acceptance-outside.md" <<EOF
---
id: acceptance-outside-$stamp
title: $outside_title
status: published
source:
  kind: file
  id: file:acceptance-outside-$stamp
  summary: acceptance fixture
category: [product-tree]
keywords: [acceptance]
created_at: '2026-07-17T00:00:00Z'
updated_at: '2026-07-17T00:00:00Z'
confidence: 0.99
cross_refs: []
content_hash: acceptance-$stamp
confluence_page_id: null
model_used: null
---

Outside-root title collision.
EOF
sync_all
outside_after="$(version "$outside_id")"
[[ "$outside_after" == "$outside_before" ]] || { echo "Outside-root page was mutated."; exit 1; }
rm "$fixture/published/product-tree/acceptance-outside.md"

echo "6/6 acceptance passed; test-created pages are being removed"
