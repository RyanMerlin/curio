<!--
Thanks for the PR! Quick checklist:
- One concern per PR (we'll ask you to split if a PR mixes concerns).
- `cargo fmt --all` + `cargo clippy --all-targets -- -D warnings` + `cargo nextest run --all-targets` all green.
- A test that would have failed before this change (or a deliberate "no test needed" rationale).
- See CONTRIBUTING.md for the editorial-philosophy promise this codebase keeps.
-->

## What

<!-- One or two sentences on the change. -->

## Why

<!-- Why is this change correct? Reference the editorial principle or operator pain it addresses. -->

## Tests

<!-- What did you add? `cargo nextest run --all-targets --test <name>` should run it. -->

## Breaking changes

<!-- Does this change any of: --json envelope shape, route-file schema, .curio.yaml field semantics, harness contract?
     If yes, describe the migration path. If no, write "none". -->

## Editorial principle alignment

<!-- A sentence reaffirming the change keeps the inference-first / hierarchy-first / agent-owns-judgment promise. -->
