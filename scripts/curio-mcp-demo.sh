#!/usr/bin/env sh
set -eu
# Credential-free local demo. MCP protocol uses stdout; diagnostics go to stderr.
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
exec "$ROOT/curio-rs/target/release/curio-mcp" --kb-dir "$ROOT/docs/wiki-demo"
