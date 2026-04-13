---
id: 87d0c8f53b2e4ae1
title: Server Upgrade Issues by Version - 24.1
status: published
source:
  kind: confluence_page
  id: confluence-page:2650999118
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/2650999118
  summary: null
category:
- product-tree
- alteryx-server
keywords:
- upgrade
- version
- 24.1
- schema
- migration
created_at: 2026-04-13T23:20:00Z
updated_at: 2026-04-13T23:20:00Z
confidence: 0.82
cross_refs:
- published/product-tree/alteryx-server/server-upgrade-issues-by-version.md
- published/product-tree/alteryx-server/server-upgrade-version-paths-what-version-can-upgrade-to-what-versions.md
content_hash: sha256:d374ee87a981bfaa7ae4c9e5169236a04a401cedb3b63196ab0b2622952e2455
confluence_page_id: null
model_used: codex-curation
---

> **ℹ️ Info**
>
> Focused issue inventory for the 24.1 Server upgrade family.

## Main Themes

- Python 3.10 transition and connector compatibility
- schema migration fragility
- run-count / run-mode regressions
- UTC / schedule display changes
- custom site color regressions

## Known Issues

| Item | Notes |
| --- | --- |
| Python version upgrade requires connector updates | Review all Python-based connectors and workflows before upgrade. |
| Upgrade from early 21.4 patches is unstable | Patch to the latest 21.4 first to avoid gallery schema migration failure paths. |
| Manual run counts show lower after upgrade | Run the Jira-linked workflow and update queries before users execute manual runs. |
| Run Mode reverts to Safe | If admins manually set execution mode, prepare a Mongo update before upgrade. |
| Service Schema Migration 3 fails with `0001-01-01T00:00:00` | Clean invalid default-date values before upgrade. |
| Custom site colors are removed | Reapply through the defect workaround after upgrade. |
| Schedule times shift after UTC changes | Display values drift even when schedules still run correctly. |
| Gallery schema migration errors on `CustomCss` and `appInfos` | Multiple 24.1 patch-level issues exist; use the relevant Jira fix versions before upgrading. |

## Related Pages

- [Server Upgrade Issues-by-Version](server-upgrade-issues-by-version.md)
- [Server Upgrade Version Paths - What version can upgrade to what versions?](server-upgrade-version-paths-what-version-can-upgrade-to-what-versions.md)
