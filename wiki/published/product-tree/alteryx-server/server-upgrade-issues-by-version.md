---
id: 76ab286cdda87a35
title: Server Upgrade Issues-by-Version
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
- error
- upgrade
- version
- jira
- migration
created_at: 2026-04-13T01:55:58Z
updated_at: 2026-04-13T23:20:00Z
confidence: 0.82
cross_refs:
- published/product-tree/alteryx-server/server-upgrade-version-paths-what-version-can-upgrade-to-what-versions.md
- published/product-tree/alteryx-server/server-upgrade-issues-by-version-25-1-and-24-2.md
- published/product-tree/alteryx-server/server-upgrade-issues-by-version-24-1.md
- published/product-tree/alteryx-server/server-upgrade-issues-by-version-23-2-and-23-1.md
- published/product-tree/alteryx-server/server-upgrade-issues-by-version-22-3-and-22-1.md
content_hash: sha256:95dc07530ddb3ad796f3e4b0fab90bca769471d9a1dd4da0c8a5fb6d9240b240
confluence_page_id: null
model_used: codex-curation
---

> **ℹ️ Info**
>
> This page is now an overview. The detailed issue inventory is split into focused version-family pages so each set can be maintained and reviewed independently.

## How To Use This Page

- Use this page to choose the version family that matches the planned upgrade.
- Use [Server Upgrade Version Paths - What version can upgrade to what versions?](server-upgrade-version-paths-what-version-can-upgrade-to-what-versions.md) first when you need upgrade-path validation.
- Use the detailed pages below for issue tracking, mitigations, and related references.

## Version-Family Pages

1. [Server Upgrade Issues by Version - 25.1 and 24.2](server-upgrade-issues-by-version-25-1-and-24-2.md)
2. [Server Upgrade Issues by Version - 24.1](server-upgrade-issues-by-version-24-1.md)
3. [Server Upgrade Issues by Version - 23.2 and 23.1](server-upgrade-issues-by-version-23-2-and-23-1.md)
4. [Server Upgrade Issues by Version - 22.3 and 22.1](server-upgrade-issues-by-version-22-3-and-22-1.md)

## Quick Triage

| Version family | Main themes |
| --- | --- |
| 25.1 and 24.2 | Copilot compatibility, credential publish issues, Mongo 7.0 upgrade behavior, CPU growth after patching |
| 24.1 | Python 3.10 transition, run-count / run-mode regressions, timezone and schema migration issues |
| 23.2 and 23.1 | Mongo 6.0 transition, `__ServiceData` changes, AS_Versions defects, Lucene / UI framework changes |
| 22.3 and 22.1 | CryptoMigration, SAML URL / ACS issues, controller token transition, host recovery and gallery migration defects |
