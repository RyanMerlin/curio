# Curio Release Checklist

This checklist is for release verification and launch claims. It is intentionally explicit so a release owner can prove what was checked, what was built, and what was only asserted.

## Code Health

- [ ] `cargo fmt --all --check --manifest-path curio-rs/Cargo.toml`
- [ ] `cargo clippy --all-targets --manifest-path curio-rs/Cargo.toml -- -D warnings`
- [ ] `cargo nextest run --all-targets --manifest-path curio-rs/Cargo.toml`
- [ ] Review the working tree before tagging. Do not cut a release with unexplained local changes.

## Confluence Integration

- [ ] Run the credential-free Confluence contract tests in ordinary CI.
- [ ] Confirm `sync --all` cannot delete an unowned page and reports cleanup failures.
- [ ] Confirm timeout, bounded retry, same-origin continuation, and conflict tests pass.
- [ ] Before a release changing Confluence behavior, run `CURIO_LIVE_CONFLUENCE=1 ./scripts/confluence-live-smoke.sh` against the dedicated sandbox and retain only a redacted transcript or summary artifact (no raw secrets, tokens, or unredacted workspace data).
- [ ] Before a release changing cleanup or write-safety behavior, run `CURIO_LIVE_CONFLUENCE=1 CURIO_KB_DIR=<sandbox-kb> ./scripts/confluence-live-acceptance.sh` and retain only a redacted scenario transcript or summary artifact (no raw secrets, tokens, or unredacted workspace data). The harness now builds the release `curio` binary itself before syncing.
- [ ] Do not add Confluence credentials as a required public-PR or ordinary CI secret.

## Release Binaries

- [ ] Build the release CLI: `cargo build --release --manifest-path curio-rs/Cargo.toml --bin curio`
- [ ] Capture the binary path and platform used for the release verification.
- [ ] Run `./curio-rs/target/release/curio --help` or the equivalent resolved release binary to confirm the artifact starts.
- [ ] If multiple target triples are promised in the launch notes, build each one and record the exact commands used.

## Docker

- [ ] Build the Docker image used for the launch claim, if the release notes claim a Docker/service artifact.
- [ ] Record the exact Dockerfile or build context if more than one path exists.
- [ ] Start the image once and confirm the expected entrypoint responds without interactive repair steps.

## Demo

- [ ] Run `scripts/show-hn-demo.sh` from the repo root.
- [ ] Confirm the helper builds or reuses `curio`, copies `docs/wiki-demo` into a temporary git repo, runs `doctor`, `status`, and `process --prepare`, applies a deterministic route file, publishes a valid staged page, and verifies review/staged/published outcomes locally.
- [ ] Keep the temp directory with `KEEP_DEMO_DIR=1` when you need the generated workspace as launch evidence.
- [ ] Treat the demo as credential-free only if all Confluence checks are absent or skipped exactly because the relevant environment variables are unset.

## Scrub

- [ ] Confirm the tracked demo workspace under `docs/wiki-demo/` is synthetic and does not rely on live credentials.
- [ ] Re-check release-facing docs, scripts, and fixtures for private URLs, tokens, customer names, or workstation-specific paths.
- [ ] Validate that any workspace aliases or example config files point to synthetic or operator-supplied placeholders, not private infrastructure.

## Launch Claims

- [ ] Every public claim in the release notes can be tied to a test, script, demo, or built artifact.
- [ ] Count-based claims use exact numbers captured on release day, not stale values from earlier dry runs.
- [ ] Performance or compatibility claims state the platform, target, and command used to verify them.
- [ ] Any known gaps are disclosed in the release notes instead of being silently omitted.
- [ ] Archive the final launch evidence set: command transcript, binary details, Docker build result, demo result, and the published release URL.
