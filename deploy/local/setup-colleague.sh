#!/usr/bin/env bash
# setup-colleague.sh — scaffold a new Curio KB for a colleague.
#
# Usage:
#   ./setup-colleague.sh <name> <space-key> <parent-page-id> [token-env-var]
#
# Example:
#   ./setup-colleague.sh alice myspace 9876543210 CURIO_CONFLUENCE_TOKEN_ALICE
#
# Outputs:
#   - ~/curio-kb/<name>/                — KB directory (git init'd, initial commit)
#   - ~/curio-kb/<name>/.curio.yaml     — per-KB config (space, parent page, token env)
#   - ~/curio-kb/<name>/wiki/...        — empty intake/staged/review/published/_admin scaffold
#   - ~/curio-kb/<name>/wiki/NORTHSTAR.md — placeholder charter
#   - ~/curio-kb/<name>/.env.example    — colleague pastes their token here
#   - Registers the workspace in curio-agent/curio.workspaces.toml AND
#     deploy/local/state/workspaces.json (for the docker-compose service).

set -euo pipefail

if [[ $# -lt 3 ]]; then
  cat <<EOF
Usage: $0 <name> <space-key> <parent-page-id> [token-env-var]

  <name>              short workspace id (kebab-case, e.g. "alice" or "fde-uc-repo")
  <space-key>         Confluence space key (e.g. "myspace")
  <parent-page-id>    Confluence numeric page ID where curio will write
  [token-env-var]     env var name holding the bot token
                      (default: CURIO_CONFLUENCE_TOKEN_<NAME_UPPER>)
EOF
  exit 1
fi

NAME="$1"
SPACE_KEY="$2"
PARENT_PAGE_ID="$3"
NAME_UPPER=$(echo "$NAME" | tr '[:lower:]-' '[:upper:]_')
TOKEN_ENV="${4:-CURIO_CONFLUENCE_TOKEN_${NAME_UPPER}}"

# Resolve repo paths relative to this script.
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
HARNESS_DIR="$(cd "${SCRIPT_DIR}/../.." && pwd)"
REPO_PARENT="$(cd "${HARNESS_DIR}/.." && pwd)"
KB_DIR="${CURIO_KB_PARENT:-${REPO_PARENT}}/${NAME}"

if [[ -e "$KB_DIR" ]]; then
  echo "✗ refusing to overwrite existing path: $KB_DIR" >&2
  exit 1
fi

echo "Creating KB scaffold at ${KB_DIR}"
mkdir -p "${KB_DIR}/wiki/"{intake,staged,review,published,_admin}
for sub in intake staged review published _admin; do
  : > "${KB_DIR}/wiki/${sub}/.gitkeep"
done

# NORTHSTAR.md — placeholder charter the colleague edits.
cat > "${KB_DIR}/wiki/NORTHSTAR.md" <<EOF
# NORTHSTAR

## Name

${NAME}

## High-Level Description

Curio knowledge base for ${NAME}. Git is the system of record;
Confluence space \`${SPACE_KEY}\` is the read-only mirror.

## Charter

Describe the purpose of this KB: who it serves, what kinds of knowledge
belong here, and what does NOT belong here. The agent reads this every
time it routes intake, so be specific about boundaries.

## Categories (top-level trees)

> Edit this list and the matching nodes in \`wiki/_admin/config.yaml\`.

- \`example-tree/\` — describe the kind of content that lives here

## Route-Here Rules

- explicit signal X → \`example-tree/sub/\`
- explicit signal Y → \`example-tree/other/\`

## Exclude Rules

- material that does NOT belong here at all (route to a different KB or back to intake)

## Confidence

Route to \`review/\` when confidence < 0.75. When the right node does not
yet exist, propose a new node rather than forcing content into a wrong fit.
EOF

# Workspace config — taxonomy lives here. Empty nodes; colleague adds theirs.
cat > "${KB_DIR}/wiki/_admin/config.yaml" <<EOF
schema_version: 2
nodes:
  - title: Example Tree
    slug: example-tree
    description_markdown: "Replace this seed node with your real taxonomy."
    children: []

heal:
  confidence_threshold: 0.85
  show_auto_heal_callout: true
  auto_heal_label: "curio:auto-healed"
  max_pages_per_run: 20
  stale_threshold_days: 240
  overlap_threshold: 0.6
  external_search_enabled: true
  min_body_words: 50

slack:
  enabled: false
EOF

# .curio.yaml — per-KB Confluence binding.
cat > "${KB_DIR}/.curio.yaml" <<EOF
# Curio KB configuration — ${NAME}
# Confluence space: ${SPACE_KEY}

connection:
  confluence_url: "https://example.atlassian.net/wiki"
  confluence_email: ""        # set to the Atlassian account email matching the API token
  token_env: "${TOKEN_ENV}"

content_model:
  space_key: "${SPACE_KEY}"
  label_namespace: curio

wiki:
  wiki_dir: wiki
  auto_commit: true
  sync:
    enabled: true
    confluence_parent_page_id: "${PARENT_PAGE_ID}"
EOF

# .gitignore + .env stub
cat > "${KB_DIR}/.gitignore" <<EOF
.env
*.env.local
/wiki/_admin/last-sync.txt
EOF

cat > "${KB_DIR}/.env.example" <<EOF
# ${NAME} KB secrets — copy to .env (which is .gitignore'd) and paste your token.
${TOKEN_ENV}=your-confluence-api-token-here
EOF

# README at KB root.
cat > "${KB_DIR}/README.md" <<EOF
# ${NAME}

This is a Curio knowledge base.

- **Source of truth:** the \`wiki/\` tree, in this git repo.
- **Confluence mirror:** \`${SPACE_KEY}\` space, parent page ID \`${PARENT_PAGE_ID}\`.
- **Charter:** see \`wiki/NORTHSTAR.md\`.

## First steps

1. Edit \`wiki/NORTHSTAR.md\` — author your editorial charter.
2. Edit \`wiki/_admin/config.yaml\` — declare your taxonomy nodes.
3. Edit \`.curio.yaml\` — set \`connection.confluence_email\` to the Atlassian account that owns the API token.
4. Copy \`.env.example\` → \`.env\` and paste the token.
5. Run \`curio --workspace ${NAME} doctor\` from the curio-agent directory. All 8 infrastructure checks should pass.
6. Read \`docs/runbook.md\` in the curio-agent repo for the full intake → process → publish → sync flow.
EOF

# git init + initial commit.
( cd "${KB_DIR}" && \
  git init -q -b main && \
  git add -A && \
  git -c user.email=curio@local -c user.name=Curio commit -q -m "Initial KB scaffold for ${NAME}"
)

# Register the workspace in curio.workspaces.toml.
WORKSPACES_TOML="${HARNESS_DIR}/curio.workspaces.toml"
if ! grep -q "name = \"${NAME}\"" "${WORKSPACES_TOML}" 2>/dev/null; then
  cat >> "${WORKSPACES_TOML}" <<EOF

[[workspaces]]
name = "${NAME}"
path = '${KB_DIR}'
description = "${NAME} → Confluence space ${SPACE_KEY}"
EOF
  echo "  registered ${NAME} in ${WORKSPACES_TOML}"
fi

# Register in deploy/local/state/workspaces.json so the service sees it.
REGISTRY="${SCRIPT_DIR}/state/workspaces.json"
if [[ -f "${REGISTRY}" ]]; then
  python3 - "$REGISTRY" "$NAME" "$KB_DIR" "$SPACE_KEY" <<'PY'
import json, sys
path, name, kb_path, space = sys.argv[1:5]
with open(path) as f: data = json.load(f)
records = data.setdefault("records", [])
records[:] = [r for r in records if r.get("workspace_id") != name]
records.append({
    "workspace_id": name,
    "display_name": name,
    "repo_url": f"file:///kb/{name}",
    "default_branch": "main",
    "kb_root": "wiki",
    "allowed_job_types": ["intake","process","publish","sync","review"],
    "write_policy": "direct_push",
    "provider_defaults": {"backend": "passthrough"},
    "status": "active",
    "description": f"{name} → Confluence space {space}",
})
with open(path, "w") as f: json.dump(data, f, indent=2)
PY
  echo "  registered ${NAME} in ${REGISTRY}"
  echo
  echo "NOTE: docker-compose.yml bind-mounts each KB at /kb/<name>. Add this line"
  echo "      to deploy/local/docker-compose.yml under volumes: and restart the stack:"
  echo
  echo "      - ${KB_DIR}:/kb/${NAME}"
fi

echo
echo "✓ ${NAME} ready at ${KB_DIR}"
echo
echo "Next steps for the colleague:"
echo "  1. Edit ${KB_DIR}/wiki/NORTHSTAR.md (charter)"
echo "  2. Edit ${KB_DIR}/wiki/_admin/config.yaml (taxonomy)"
echo "  3. Edit ${KB_DIR}/.curio.yaml — set connection.confluence_email"
echo "  4. cp ${KB_DIR}/.env.example ${KB_DIR}/.env  # then paste token"
echo "  5. curio --workspace ${NAME} doctor          # expect 8/8 infrastructure checks"
echo "  6. Read curio-agent/docs/runbook.md"
