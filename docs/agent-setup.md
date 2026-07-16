# Agent-Led Setup

This is the shortest path for a knowledge operator who wants an agent to
prepare Curio without learning the repository architecture first. The agent
may create configuration and prepare proposals, but it must not publish or
sync content without explicit human approval.

## Give This To Your Agent

```text
You are setting up a Curio knowledge base. Read HARNESS.md, providers/<your-provider>/overrides.md, and this document first.

1. Run `curio agent doctor --json` and `curio agent print-env <provider> --json`.
2. Create or select an external KB with `curio init-kb --path <path> --name <name>`.
3. Read the generated KB README and edit `wiki/NORTHSTAR.md` plus `wiki/_admin/config.yaml` to match the operator's stated purpose and taxonomy.
4. Run `curio --kb-dir <path> doctor --json`. Do not continue if configuration, taxonomy, git, or credentials fail.
5. For the first source, run intake and `process --prepare`; inspect the manifest and explain the proposed route, quality, evidence, and missing information.
6. Apply routes only to `staged/` or `review/`. Never route new content directly to `published/`.
7. Show the operator the resulting page, proposal dossier, and recommended next action.
8. Ask for explicit approval before `publish` or `sync`.

Report: KB path, charter summary, taxonomy summary, provider status, doctor result, pages staged, pages in review, and any unresolved questions.
```

## Operator Inputs

The operator should provide:

- the KB purpose and intended readers
- examples of content that belongs and does not belong
- the source URL, file, or folder for the first intake
- the provider the agent should use
- Confluence URL, email, space key, parent page, and token environment variable
  only when real synchronization is required

The agent should turn these into a concrete `NORTHSTAR.md` charter and a
matching `wiki/_admin/config.yaml` taxonomy. A taxonomy is not complete merely
because it parses; its top-level nodes must match the corpus the operator
intends to curate.

## Credential-Free Demo

For the public synthetic demo, no provider or Confluence credentials are
needed:

```sh
./scripts/show-hn-demo.sh
```

The script uses a temporary copy of `docs/wiki-demo/`, applies deterministic
route decisions, and verifies staged, review, and published outcomes without
modifying tracked fixtures.

## Readiness Report

Before handing the KB to an operator, the agent should confirm:

- `curio agent doctor` passes for the selected provider
- the KB has a meaningful `NORTHSTAR.md`
- the YAML taxonomy parses and contains the expected route nodes
- `curio doctor` passes the relevant infrastructure checks
- the first intake has a proposal dossier
- ambiguous or weak content is in `review/`, not `published/`
- no publish or Confluence sync happened without approval

If Confluence is not configured, report that limitation explicitly. The local
Git-native workflow remains usable without it.
