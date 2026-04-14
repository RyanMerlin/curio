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
created_at: 2026-04-14T15:18:28Z
updated_at: 2026-04-14T15:18:28Z
confidence: null
cross_refs: []
content_hash: sha256:a9adaf5baf5fa9b724aae829e8831dc95a207bba6a80c1a209916b495e673350
confluence_page_id: null
model_used: null
---

> **ℹ️ Info**
>
> The **Mongo Input Tool** can be used to explore an Embedded or a non-TLS User-managed database. For a TLS User-managed database you can use the [Mongo ODBC Driver](https://alteryx.atlassian.net/wiki/spaces/SupportDesigner/pages?title=Mongo+ODBC+Driver).

> **📝 Note**
>
> The **MongoDB Tools** were replaced with [Mongo ODBC Driver](https://alteryx.atlassian.net/wiki/spaces/SupportDesigner/pages?title=Mongo+ODBC+Driver) in 23.1 release
> 
> To use MongoDB Input in 23.1+, **right-click tool ribbon > Show Deprecated Tools**

> **⚠️ Warning**
>
> Do NOT use the **MongoDB Output Tool **to write the Alteryx Server, it will corrupt the system

| **Schemas** | <https://help.alteryx.com/current/server/alteryxgallery-mongodb-schema>  <https://help.alteryx.com/current/server/alteryxservice-mongodb-schema>  [Mongo Collections and Entity-Relationship Diagram (ERD)](https://alteryx.atlassian.net/wiki/search?text=Mongo+Entity-Relationship+Diagram+(ERD)) |
| --- | --- |

---

REGEX_Replace([_id], '^.*:\s\"(.*)\".*', '$1')

**__ServiceData blob was removed in 23.2** and the fields it contained now appear as normal fields.  A few blobs remain that have been renamed but can still be unpacked with the macro below.

[Starting with Server 2023.2, ServiceData Blob Removed from MongoDB](https://knowledge.alteryx.com/index/s/article/Starting-with-Server-20232-ServiceData-Blob-Removed-from-MongoDB)  (KB)

TWI-146077dcf2c9-72f3-3ff6-8e99-fe88e9f473f1System Jira

Plugins cannot be loaded from a DLL path with upward references: 'EngineDll=”..\AlteryxService_Client.dll”