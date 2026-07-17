# Curio Governance

Curio is currently maintained as a small public repository with a single primary maintainer. This document sets expectations for decision-making and release stewardship without pretending there is a larger formal body behind the project yet.

## Maintainer model

- The repository owner is the default maintainer and release approver.
- Maintainers review and merge pull requests, triage issues, cut releases, and keep the public contract coherent across docs, workflows, and code.
- Additional maintainers may be added over time. When that happens, update this file and `CODEOWNERS` in the same pull request.

## How decisions are made

- Changes to published behavior should be anchored in the documented contract: `HARNESS.md`, `docs/runbook.md`, `docs/agent-cli-contract.md`, and `ARCHITECTURE.md`.
- Non-trivial product or workflow changes should start with an issue before implementation.
- When behavior, scope, or roadmap tradeoffs are unclear, the maintainer makes the final call after reviewing the issue or pull request record.

## Change classes

### Routine changes

- Bug fixes
- Documentation improvements
- Test-only changes
- Dependency updates
- Low-risk workflow maintenance

These can merge after normal review and passing checks.

### Contract-affecting changes

- `--json` envelope changes
- Route-file schema changes
- `.curio.yaml` semantics changes
- Editorial-policy shifts that change what Curio claims or guarantees
- Security policy, governance, or support process changes

These should include an explicit migration or rollout note in the pull request.

## Release posture

- Releases are cut from `main`.
- Tagged releases should reflect passing CI and a coherent changelog entry.
- Security fixes may ship on an accelerated timeline; in those cases, follow `SECURITY.md` first and document public details after the fix is available.

## Communication

- Use GitHub issues for bugs, features, connector ideas, and roadmap requests.
- Use GitHub Security Advisories or the private maintainer contact path for vulnerabilities.
- Use `SUPPORT.md` for usage questions and troubleshooting paths.

## Project scope discipline

Curio is still tightening the public baseline. Maintainers may defer requests that assume:

- hosted infrastructure the project does not operate,
- enterprise support commitments that are not staffed,
- compatibility promises not yet captured in the documented contract.

Deferring a request is not a rejection of the underlying problem. It usually means the repository needs a smaller or more local-first path first.
