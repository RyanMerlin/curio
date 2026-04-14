---
id: b9ac518153229f71
title: Server Upgrade Version Paths - What version can upgrade to what versions?
status: review
source:
  kind: confluence_page
  id: confluence-page:2843344956
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/2843344956
  summary: null
category:
- product-tree
- alteryx-server
- upgrade
keywords:
- upgrade
- version-paths
- supported-upgrades
- planning
created_at: 2026-04-14T15:18:28Z
updated_at: 2026-04-14T15:20:41Z
confidence: 0.9
cross_refs: []
content_hash: sha256:50a8fd4b5972944eac2bd33c911bb486c6727b66e4b4ddcd6b79a14d65d76979
confluence_page_id: null
model_used: claude-sonnet-4-6
---

> **ℹ️ Info**
>
> This document helps plan what versions you can upgrade to directly

| **Key Articles** | [Server Upgrade Issues-by-Version](https://alteryx.atlassian.net/wiki/search?text=Server+Upgrade+Issues-by-Version)  [MongoDB Upgrade Folder Structure](https://alteryx.atlassian.net/wiki/search?text=MongoDB+Upgrade+Folder+Structure) |
| --- | --- |
| **Help** | <https://help.alteryx.com/current/en/server/install/install-or-upgrade-server/version-to-version-server-upgrade-guide-supported-versions.html>       > <https://help.alteryx.com/current/en/server/install/install-or-upgrade-server/version-to-version-server-upgrade-guide-unsupported-versions.html>  <https://help.alteryx.com/current/en/server/install/install-or-upgrade-server/server-upgrade-checklist.html>  <https://help.alteryx.com/current/en/server/install/install-or-upgrade-server.html>  <https://help.alteryx.com/current/en/server/configure/database-management/mongodb-management/mongodb-schema-reference.html> |

# Embedded Mongo

| **Version** |  |  |  |  |  |  |  |  |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **25.2** | Mongo 8.0 |  |  |  |  |  |  |  |
| **25.1** | Mongo 7.0 | Mongo 7.0 | Mongo 7.0 | Mongo 7.0 |  |  |  |  |
| **24.2** |  | Mongo 7.0 | Mongo 7.0 | Mongo 7.0 |  |  |  |  |
| **24.1** |  |  | Mongo 6.0 | Mongo 6.0 | Mongo 6.0 | Mongo 6.0 | Mongo 6.0 | Mongo 6.0 |
|  |  |  |  | - Python 3.8.16 to 3.10.13 requires reinstalling connectors (Help / Confluence) |  |  |  |  |
| **23.2** |  |  |  | Mongo 6.0 | Mongo 6.0 | Mongo 6.0 | Mongo 6.0 | Mongo 6.0 |
|  |  |  |  |  | - __ServiceData blob removed (Confluence) |  |  |  |
| **23.1** |  |  |  |  | Mongo 4.2 | Mongo 4.2 | Mongo 4.2 | Mongo 4.2 |
| **22.3** |  |  |  |  |  | Mongo 4.2 | Mongo 4.2 | Mongo 4.2 |
|  |  |  |  |  | - Run CryptoMigration Prep before upgrade to 22.3 (requires 64-char Token)   (Help / Conf_1 / Conf_2)    - Designer 22.3_Patch3 req’d for Server 22.3+ (Help)    - SAML Auth: SAML ACS endpoint must be all lowercase (KB)    - Controller Token auto lengthened 40- to 64-char (Confluence) |  |  |  |
| **22.1** |  |  |  |  |  |  | Mongo 4.2 **Patch_9+** | Mongo 4.2 |
|  |  |  |  |  | - API OAuth1 deprecated in 22.1 (Help / Confluence)    - Built-In Authentication pit-stop at 22.1 to reset passwords (Help) |  |  |  |
|  |  |  |  |  |  |  |  | Mongo 4.2 |
|  |  |  |  |  |  |  |  |  |
| **Starting ver** | **25.1** | **24.2** | **24.1** | **23.2** | **23.1** | **22.3** | **22.1_Patch_9+** | **21.4** |
|  |  |  |  | **       ^ ^              ** 22.1.1.9.42691 Patch_9 or higher before upgrading to or through 22.3 to avoid the defect TCPE-1100 (this may not be true [EdP]) | [Old V2V Guide](https://help.alteryx.com/current/en/server/install/install-or-upgrade-server/version-to-version-server-upgrade-guide-unsupported-versions.html)** ** |  |  |  |

# 21.4-

| **Version** |  |  |  |  |  |
| --- | --- | --- | --- | --- | --- |
| **21.4** | Mongo 4.2 |  |  |  |  |
| \| \| \| | **<< Stop at 21.4 and regenerate Controller Token before upgrade to 22.1+ >> **  [Controller Token Length Transition from 21.4 to 22.3](https://alteryx.atlassian.net/wiki/search?text=Controller+Token+Length+Transition+from+21.4+to+22.3)  <https://help.alteryx.com/current/en/server/install/install-or-upgrade-server/version-to-version-server-upgrade-guide-unsupported-versions.html#2021-4:~:text=Internal%20Change%3A%20Controller%20Token%20length%20extended> |  |  |  |  |
| **21.4** |  | Mongo 4.2 | Mongo 4.2 | Mongo 4.2 | Mongo 4.2 |
| **21.3.6+** |  | Mongo 4.2 | Mongo 4.2 | Mongo 4.2 | Mongo 4.2 |
| **21.3.5-** |  |  | Mongo 4.0 | n/a | n/a |
| **21.2** |  |  |  | Mongo 4.0 | Mongo 4.0 |
| **19.3 - 21.1 ** |  |  |  |  | Mongo 4.0 |
|  |  |  |  |  |  |
| **Starting ver** | **21.4** | **21.3.6+** | **23.1.5-** | **21.2** | **19.3 - 21.1** |