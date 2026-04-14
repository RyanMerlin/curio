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
created_at: 2026-04-14T15:16:58Z
updated_at: 2026-04-14T15:16:58Z
confidence: null
cross_refs: []
content_hash: sha256:09126a7b179ec7bce721cf76a751de9ae34c2e6a7dda99d61a9515aa783bb188
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

| **Database+Schema names** | Confusing:     - AlteryxGallery is the database, alteryx_server is the SQL schema    - AlteryxService is the database, alteryx_service is the SQL schema  So the full name of a table in SQL DB is:     - AlteryxGallery.alteryx_server.TABLE_NAME    - AlteryxService.alteryx_service.TABLE_NAME |
| --- | --- |
| **SQL DB Schema Version** | SQL DB schema version replaces separate schema versions for AlteryxGallery and AlteryxService  <https://help.alteryx.com/current/en/server/configure/database-management/sql-db-management/sql-db-schema-reference.html> |
| **Help** | <https://help.alteryx.com/current/en/server/configure/database-management/sql-db-management/sql-db-schema-reference/alteryxgallery-sql-db-schema.html>  <https://help.alteryx.com/current/en/server/configure/database-management/sql-db-management/sql-db-schema-reference/alteryxservice-sql-db-schema.html> |