---
id: c12ac8d6d8f6407c
title: Server Upgrade Issues by Version - 23.2 and 23.1
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
- 23.2
- 23.1
- cryptomigration
created_at: 2026-04-13T23:20:00Z
updated_at: 2026-04-13T23:20:00Z
confidence: 0.82
cross_refs:
- published/product-tree/alteryx-server/server-upgrade-issues-by-version.md
- published/product-tree/alteryx-server/servicedata-blob-removal-in-23-2.md
content_hash: sha256:a3e37ea5f0bd91b5634ea8bf28837fb88348e00a2a0b7f006fcf1f7ffc3337af
confluence_page_id: null
model_used: codex-curation
---

> **ℹ️ Info**
>
> Focused issue inventory for the 23.2 and 23.1 Server upgrade families.

# 23.2

| Item | Notes |
| --- | --- |
| Mongo 6.0 upgrade: unexpected `4.0.10` in `ASMongoDBVersion.bin` | Validate both the starting Mongo version and the file content before upgrade. |
| Missing `ASMongoDBVersion.bin` | Recreate the file when the installer misplaces it. |
| Service Schema Migration 2 fails on `AS_Queue` | Review migrator and service logs before attempting restart. |
| Existing workflow revision/version numbers display as `1` | Current workaround is script-based and post-upgrade. |
| Data loss during MongoDB Version Upgrade | Review `migration.log` carefully; diagnostic tooling exists to filter expected errors. |

# 23.1

| Item | Notes |
| --- | --- |
| Missing `AS_Versions` collection causes upgrade failure | Affects some already-CryptoMigrated installs and can trigger false CryptoMigration attempts. |
| Lucene indexing replaced | Post-upgrade indexing symptoms can look like missing data. |
| Embedded R upgrade | Customers may need to update R code. |
| UI framework replacement | Patch-level UI defects were common during this family. |

## Related Pages

- [Server Upgrade Issues-by-Version](server-upgrade-issues-by-version.md)
- [ServiceData Blob Removal in 23.2](servicedata-blob-removal-in-23-2.md)
