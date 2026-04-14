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
created_at: 2026-04-14T15:18:28Z
updated_at: 2026-04-14T15:18:28Z
confidence: null
cross_refs: []
content_hash: sha256:a250cb4fe04906d91220f375bb216c973120e7695e1c9ff7d0c0bbfa77bea0be
confluence_page_id: null
model_used: null
---

---

---

> **ℹ️ Info**
>
> SQL DB Schema differs from Mongo but the Mongo pages can help, especially for AlteryxService
> 
> [Mongo Entity-Relationship Diagram / ERD](https://alteryx.atlassian.net/wiki/search?text=Mongo+Entity-Relationship+Diagram+(ERD))

| **Database+Schema names** | Confusing:     - AlteryxGallery is the database, alteryx_server is the SQL schema    - AlteryxService is the database, alteryx_service is the SQL schema  So the full name of a table in SQL DB is:     - AlteryxGallery.alteryx_server.TABLE_NAME    - AlteryxService.alteryx_service.TABLE_NAME |
| --- | --- |
| **SQL DB Schema Version** | SQL DB schema version replaces separate schema versions for AlteryxGallery and AlteryxService  <https://help.alteryx.com/current/en/server/configure/database-management/sql-db-management/sql-db-schema-reference.html> |
| **Help** | <https://help.alteryx.com/current/en/server/configure/database-management/sql-db-management/sql-db-schema-reference/alteryxgallery-sql-db-schema.html>  <https://help.alteryx.com/current/en/server/configure/database-management/sql-db-management/sql-db-schema-reference/alteryxservice-sql-db-schema.html> |