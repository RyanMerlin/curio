---
id: 7cf73ca07c306831
title: SQL DB Schema and Collation
status: intake
source:
  kind: confluence_page
  id: confluence-page:2192900191
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/2192900191
  summary: null
category: []
keywords: []
created_at: 2026-04-14T15:02:19Z
updated_at: 2026-04-14T15:02:19Z
confidence: null
cross_refs: []
content_hash: sha256:99079de7bcb6978a4fd2b16f5370aa93fdde7dd442ca1cf398cf8295819d9040
confluence_page_id: null
model_used: null
---

---

---

> **ℹ️ Info**
>
> SQL DB Schema differs from Mongo but the Mongo pages can help, especially for AlteryxService
> 
> Mongo Entity-Relationship Diagram / ERD

| Database+Schema names | Confusing:AlteryxGallery is the database, alteryx_server is the SQL schemaAlteryxService is the database, alteryx_service is the SQL schemaSo the full name of a table in SQL DB is:AlteryxGallery.alteryx_server.TABLE_NAMEAlteryxService.alteryx_service.TABLE_NAME |
| --- | --- |
| SQL DB Schema Version | SQL DB schema version replaces separate schema versions for AlteryxGallery and AlteryxServicehttps://help.alteryx.com/current/en/server/configure/database-management/sql-db-management/sql-db-schema-reference.html |
| Help | https://help.alteryx.com/current/en/server/configure/database-management/sql-db-management/sql-db-schema-reference/alteryxgallery-sql-db-schema.html https://help.alteryx.com/current/en/server/configure/database-management/sql-db-management/sql-db-schema-reference/alteryxservice-sql-db-schema.html |