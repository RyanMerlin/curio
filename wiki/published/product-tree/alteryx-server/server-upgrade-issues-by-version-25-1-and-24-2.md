---
id: 6b4d0b9a3ef7401a
title: Server Upgrade Issues by Version - 25.1 and 24.2
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
- 25.1
- 24.2
- mongodb
created_at: 2026-04-13T23:20:00Z
updated_at: 2026-04-13T23:20:00Z
confidence: 0.82
cross_refs:
- published/product-tree/alteryx-server/server-upgrade-issues-by-version.md
- published/product-tree/alteryx-server/server-upgrade-version-paths-what-version-can-upgrade-to-what-versions.md
- published/product-tree/alteryx-server/mongodb-upgrade-folder-structure.md
- published/product-tree/alteryx-server/mongo-database-upgrade-error-you-are-upgrading-from-a-version-of-server-that-utilizes-mongodb-version-older-than-6-0.md
content_hash: sha256:ed338de1c4acde289e52db7e39c935214c62397c561a391dca294ac49ec83412
confluence_page_id: null
model_used: codex-curation
---

> **ℹ️ Info**
>
> Focused issue inventory for the 25.1 and 24.2 Server upgrade families.

# 25.1

| Item | Notes |
| --- | --- |
| Unhandled Exception when Starting Designer | Prevent by uninstalling or updating Copilot before upgrading to 25.1. Release notes and the linked Confluence article call this out. |
| Error publishing with a credential - Invalid username or password | Affects publishing to 25.1 from older Designer versions. Track the related Confluence article and Jira reference. |

# 24.2

| Item | Notes |
| --- | --- |
| Embedded MongoDB upgrade no longer backs up rollback data | Snapshot with the Service stopped. The `Mongo_PreUpgrade` folder is not a rollback substitute. |
| License Server Admin Command 401 after upgrade | Reset password post-upgrade when the Admin password appears to revert. |
| MongoDB won’t upgrade to 7.0 because it thinks it is on 4.2 | Validate `ASMongoDBVersion.bin` and avoid stacking multiple Mongo version upgrades in one move. |
| Data loss during MongoDB Version Upgrade | Review `migration.log`; some errors are expected noise, but silent data loss cases still need targeted review. |
| Gallery stops responding / CPU grows over time | Watch for API-driven CPU growth after `24.2.1.14`; periodic service restart and reduced API load are the current mitigations. |

## Related Pages

- [Server Upgrade Issues-by-Version](server-upgrade-issues-by-version.md)
- [Server Upgrade Version Paths - What version can upgrade to what versions?](server-upgrade-version-paths-what-version-can-upgrade-to-what-versions.md)
- [MongoDB Upgrade Folder Structure](mongodb-upgrade-folder-structure.md)
- [Mongo Database Upgrade Error - You are upgrading from a version of Server that utilizes MongoDB version older than 6.0](mongo-database-upgrade-error-you-are-upgrading-from-a-version-of-server-that-utilizes-mongodb-version-older-than-6-0.md)
