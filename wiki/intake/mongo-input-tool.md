---
id: 00f28039e7316a01
title: Mongo Input Tool
status: intake
source:
  kind: confluence_page
  id: confluence-page:1702763531
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/1702763531
  summary: null
category: []
keywords: []
created_at: 2026-04-14T15:12:39Z
updated_at: 2026-04-14T15:12:39Z
confidence: null
cross_refs: []
content_hash: sha256:21b6dfccf4b7ed4925dcaa6b9af5dcccee05b13f72161c0e18ca8a3ef1ae1126
confluence_page_id: null
model_used: null
---

> **ℹ️ Info**
>
> The **Mongo Input Tool** can be used to explore an Embedded or a non-TLS User-managed database. For a TLS User-managed database you can use the Mongo ODBC Driver.

> **📝 Note**
>
> The **MongoDB Tools** were replaced with Mongo ODBC Driver in 23.1 release
> 
> To use MongoDB Input in 23.1+, **right-click tool ribbon > Show Deprecated Tools**

> **⚠️ Warning**
>
> Do NOT use the **MongoDB Output Tool **to write the Alteryx Server, it will corrupt the system

|  |  |
| --- | --- |

---

REGEX_Replace([_id], '^.*:\s\"(.*)\".*', '$1')

**__ServiceData blob was removed in 23.2** and the fields it contained now appear as normal fields.  A few blobs remain that have been renamed but can still be unpacked with the macro below.

[Starting with Server 2023.2, ServiceData Blob Removed from MongoDB](https://knowledge.alteryx.com/index/s/article/Starting-with-Server-20232-ServiceData-Blob-Removed-from-MongoDB)  (KB)

TWI-146077dcf2c9-72f3-3ff6-8e99-fe88e9f473f1System Jira

Plugins cannot be loaded from a DLL path with upward references: 'EngineDll=”..\AlteryxService_Client.dll”