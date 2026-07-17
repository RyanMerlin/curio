<!--
Keep PRs focused and practical for the current public baseline.
- One concern per PR.
- Link the issue for non-trivial changes.
- Keep workflows and docs aligned with the actual repo posture.
- See CONTRIBUTING.md, SUPPORT.md, GOVERNANCE.md, and SECURITY.md for repository expectations.
-->

## Checklist

- [ ] I read the relevant repository guidance (`CONTRIBUTING.md`, `SECURITY.md`, `SUPPORT.md`, and `GOVERNANCE.md` when applicable).
- [ ] This PR addresses one primary concern.
- [ ] I ran the relevant checks locally, or I explain why a check was skipped.
- [ ] I added or updated tests when behavior changed, or I explain why no test was needed.
- [ ] I documented any contract, workflow, or docs changes that reviewers need to verify.
- [ ] If this affects security, workflows, or releases, I called out any required GitHub settings changes.

## What

<!-- One or two sentences on the change. -->

## Why

<!-- Why is this change correct? Reference the editorial principle or operator pain it addresses. -->

## Tests

<!-- What did you add? `cargo nextest run --all-targets --test <name>` should run it. -->

## Checks run

<!-- Example: `cargo fmt --all --check`, `cargo nextest run --all-targets`, config validation, etc. -->

## Breaking changes

<!-- Does this change any of: --json envelope shape, route-file schema, .curio.yaml field semantics, harness contract?
     If yes, describe the migration path. If no, write "none". -->

## Editorial principle alignment

<!-- A sentence reaffirming the change keeps the inference-first / hierarchy-first / agent-owns-judgment promise. -->

## GitHub / operations follow-up

<!-- Note any required repo settings, branch protection, environments, secrets, or "none". -->
