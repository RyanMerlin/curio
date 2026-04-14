---
id: 7cf73ca07c306831
title: SQL DB Schema and Collation
status: review
source:
  kind: confluence_page
  id: confluence-page:2192900191
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/2192900191
  summary: null
category:
- product-tree
- alteryx-server
- sql-db-persistence
keywords:
- sql-db
- schema
- collation
- database
- configuration
created_at: 2026-04-14T13:40:14Z
updated_at: 2026-04-14T13:50:55Z
confidence: 0.78
cross_refs: []
content_hash: sha256:b7f18f01a5dd58a19577e20e8be86157d5617ada45934161b8312b7da4ac67e2
confluence_page_id: null
model_used: claude-sonnet-4-6
---

---

*[Organized section — child pages listed separately]*

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