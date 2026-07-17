# Curio Support

This repository is maintained as a small public project. Support is best-effort and GitHub-first.

## Where to ask

- Usage questions, setup trouble, and troubleshooting: open a GitHub issue if no existing issue answers it.
- Security concerns: follow [SECURITY.md](SECURITY.md) and use a private reporting path.
- Feature, connector, or roadmap ideas: use the matching issue template under `.github/ISSUE_TEMPLATE/`.

## Before opening an issue

Check the documents that define the current public contract:

- [README.md](README.md) for the project overview and supported paths
- [CONTRIBUTING.md](CONTRIBUTING.md) for contribution expectations
- [HARNESS.md](HARNESS.md) for the provider-neutral operating contract
- [docs/runbook.md](docs/runbook.md) for operator workflow details
- [docs/agent-cli-contract.md](docs/agent-cli-contract.md) for `--json` command envelopes
- [SECURITY.md](SECURITY.md) for vulnerability handling

## What to include

For the fastest triage, include:

- the exact command or workflow you ran,
- the Curio version and OS details,
- relevant `--json` output when available,
- whether you are using the CLI, local service, or a synthetic demo path,
- whether the problem reproduces on the current `main` branch.

## Response expectations

- Maintainer response times are best-effort, not an SLA.
- Security reports are handled according to the timelines in [SECURITY.md](SECURITY.md).
- If a request needs design work before implementation, maintainers may redirect it into an issue discussion before taking a pull request.
