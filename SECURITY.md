# Security Policy

## Reporting a Vulnerability

If you believe you have found a security vulnerability in Curio, **please do not open a public GitHub issue.** Instead, report it privately so we can fix it before it's disclosed.

**Private channel:** open a GitHub Security Advisory at <https://github.com/RyanMerlin/curio/security/advisories/new>, or email the maintainers directly via the address listed on the repo's GitHub profile.

Please include:

- A description of the issue and its impact.
- Steps to reproduce, or a proof-of-concept if you have one.
- The Curio version (`curio --version`) and your operating environment.
- Whether the vulnerability has been disclosed elsewhere.

We aim to acknowledge reports within 3 business days and to ship a fix or mitigation within 30 days for high-severity issues.

## Scope

In scope:

- The `curio-rs` Rust crate (CLI + service binary).
- The `deploy/cloud-run/` Dockerfile and Terraform.
- The shipped community files (LICENSE, CoC, etc).
- Skills and plugins published under `skills/` or `plugins/` in this repository.

Out of scope:

- Operator-supplied content in any downstream `wiki/` workspace — that's the operator's responsibility.
- Third-party Confluence / OpenAI / git platform vulnerabilities — report those to the upstream vendor.
- Vulnerabilities in dependencies that have not yet shipped a patched version (we'll track them but the fix lives upstream).

## What Curio handles that could be sensitive

- **Confluence API tokens** — resolved per-KB from environment variables named in the KB's `.curio.yaml`. Never committed; `.env*` and `secrets/` are gitignored by default. Tokens are passed via HTTP Basic auth to the configured Confluence endpoint only.
- **Service registry** — `deploy/local/state/workspaces.json` carries the workspace catalog. Writes are atomic (write-tmp + rename). Restrict filesystem access to the directory; the file itself contains no secrets.
- **Audit log** — `wiki/_admin/audit.jsonl` records every editorial action. It contains workspace IDs, page slugs, and operator-provided actor identifiers. Treat it as sensitive operational data.
- **Git remotes** — Curio invokes `git` for mutations. SSH keys or HTTPS credentials live in your OS git-credential store; Curio does not manage them.

## Hardening checklist for operators

Before going live with Curio in production:

- [ ] Every `.curio.yaml` declares a per-KB `connection.token_env` pointing at a KB-scoped env var. Do not share a single token across KBs in production.
- [ ] Service is deployed behind an authenticator (IAP, OIDC, or the bearer-token mode shipped in `service/auth.rs`).
- [ ] `CURIO_SERVICE_AUTH_MODE` is set to `iap` or `oidc` in production; `none` only in local dev.
- [ ] The audit log is shipped to durable storage (Cloud Logging, etc.) — local-only audit is a single point of loss.
- [ ] `--force-publish` use is monitored. The bypassed-dimensions tag in `wiki/_admin/log.md` is the audit trail.
- [ ] `git status --porcelain` is clean before bulk operations. Intake auto-recovers from partial-write crashes, but operator hygiene helps.

## Disclosure

Once a fix is shipped, we'll:

1. Tag a patch release.
2. Publish a GitHub Security Advisory with the CVE if one was assigned.
3. Credit the reporter in the advisory unless they prefer to remain anonymous.
