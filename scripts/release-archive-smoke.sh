#!/usr/bin/env bash
set -euo pipefail

archive="${1:?usage: release-archive-smoke.sh ARCHIVE}"
[[ -f "$archive" ]] || { echo "Archive not found: $archive" >&2; exit 1; }

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

case "$archive" in
  *.tar.gz) tar -xzf "$archive" -C "$tmp" ;;
  *.zip) unzip -q "$archive" -d "$tmp" ;;
  *) echo "Unsupported archive format: $archive" >&2; exit 1 ;;
esac

for binary in curio curio-mcp; do
  path="$tmp/$binary"
  [[ -x "$path" ]] || { echo "Missing executable in archive: $binary" >&2; exit 1; }
  "$path" --help >/dev/null
done

echo "Archive smoke test passed: $archive"
