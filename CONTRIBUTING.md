# Contributing to Curio

Thanks for your interest. Curio is an open editorial-pipeline harness for knowledge bases; contributions that sharpen the editorial substrate or extend the provider/skill surface are very welcome.

## Quick orientation

- Read [`HARNESS.md`](HARNESS.md) for the operating contract that governs every provider.
- Read [`docs/design/process.md`](docs/design/process.md) for the editorial pipeline design.
- Read [`docs/runbook.md`](docs/runbook.md) for the operator-facing workflow (intake → process → publish → sync).
- Read [`ARCHITECTURE.md`](ARCHITECTURE.md) for the curio-rs ↔ harness layer split.

## The editorial-philosophy promise

Curio is **not a page router**. It is an information-transformation system. Every contribution should keep that promise alive:

1. **Inference first.** Use commands to apply decisions, not to discover whether a decision exists.
2. **Hierarchy is the primary optimization target.** Default toward deeper paths for technical / scenario-specific content.
3. **`published` is never the first move.** New work lands in `staged` or `review`.
4. **The agent owns editorial judgment; `curio-rs` owns deterministic execution.** Don't put LLM calls in the Rust binary; don't put deterministic safety gates in the harness.
5. **The product registry is the SSOT for domain customization.** If you find yourself hard-coding product names, taxonomies, or domain keywords into Rust, stop — extend the registry instead.

PRs that violate these are unlikely to be accepted as-is; we'll usually ask you to refactor toward them.

## What's in scope

- Bug fixes against the documented behavior (`docs/agent-cli-contract.md` and `docs/runbook.md` are the contract).
- New CLI subcommands or service routes that extend the editorial pipeline.
- New skills or plugins under `skills/` or `plugins/`.
- Documentation polish.
- Provider adapters (a fourth supported provider beyond Claude / Codex / Gemini).
- Tests for any of the above.

## What's out of scope (for now)

- Adding LLM calls to the `curio-rs` crate. LLM calls live in the harness layer.
- Hard-coding company-specific taxonomies, product names, or emojis. These belong in the per-KB `_admin/config.yaml` registry.
- Changes that break the `--json` envelope contract without a clear migration path.

## How to develop

```sh
# Build
cd curio-rs
cargo build --release --bin curio --bin curio-service

# Lint + format
cargo fmt --all
cargo clippy --all-targets -- -D warnings

# Test
cargo test
```

Tests live in `curio-rs/tests/` (integration) and as `#[cfg(test)]` modules inside source files (unit). Integration tests use `tempfile::tempdir` so they don't touch your local KBs.

## Submitting changes

1. Open an issue first for non-trivial changes so we can align on the approach.
2. Fork, branch from `main`, commit, push.
3. Open a PR. The PR template will ask you to describe **what / why / tests / breaking**.
4. Keep PRs focused. One concern per PR is much easier to review than three.

### Commit messages

We use conventional-commit-ish prefixes when they help, but it's not strict. The body should explain **why** the change is correct, not just **what** it changes.

### Tests

A change without a test is a change we have to take on faith. Please add at least one test that would have failed before your change. Tests run against `tempfile::tempdir` workspaces so they're hermetic — see `tests/multi_kb.rs` for a template.

## Code style

- Rust 2024 edition, `cargo fmt --all` defaults.
- Avoid `unwrap()` / `expect()` in user-reachable paths. Use `?` with `anyhow::Context` and a helpful error message. Static-string infallibles (e.g. `Selector::parse("body")`) are fine.
- Comments explain **why**, not **what**. The code already says what; the comment says why the code is what it is.

## Filing issues

See [`SECURITY.md`](SECURITY.md) for security issues — those need a private channel, not a GitHub issue.

For everything else, use the [issue templates](.github/ISSUE_TEMPLATE/):
- **Bug report** — include reproduction steps, expected vs. actual, and the `--json` output when relevant.
- **Feature request** — explain the editorial-pipeline gap or operator-UX gap the feature would close.

## Conduct

By participating you agree to our [Code of Conduct](CODE_OF_CONDUCT.md).

## Licensing

Curio is licensed under the [Apache License, Version 2.0](LICENSE). By submitting a contribution you agree your contribution is licensed under the same terms.
