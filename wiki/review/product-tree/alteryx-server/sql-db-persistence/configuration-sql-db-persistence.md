---
id: 8e77004416c69f7d
title: Configuration (SQL DB Persistence)
status: review
source:
  kind: confluence_page
  id: confluence-page:2650801612
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/2650801612
  summary: null
category:
- product-tree
- alteryx-server
- sql-db-persistence
keywords:
- sql-db
- persistence
- configuration
- hub
created_at: 2026-04-14T15:18:28Z
updated_at: 2026-04-14T15:19:29Z
confidence: 0.8
cross_refs: []
content_hash: sha256:6cfe622f43bf5e7543f835be31ca88d5c4684593d628cf7fa31529139e23a0df
confluence_page_id: null
model_used: claude-sonnet-4-6
---

---

---

| **Key Articles** | [How to stand up a new install of Alteryx Server with user-managed MSSQL persistence with custom database names](https://alteryx.lightning.force.com/kA0Uu0000000OtZKAU) (KB) |
| --- | --- |
| **Change SQL port** | While port 1433 is specified in the Connection string, to change it, you have to add **tcp:** in front of SQL Server name, ex:  Driver={ODBC Driver 17 for SQL Server};Server=**tcp:**sqlserver.example.com,**5352**;UID=MY_USER;PWD=MY_PSWD;Integrated Security=False;Database=AlteryxService;  Server=**tcp:**sqlserver.example.com,**5352**;Database=AlteryxGallery;User ID=MY_USER;Password=MY_PSWD; |
| **APOD Setup** | [Configure APOD - SQL DB Persistence](https://alteryx.atlassian.net/wiki/spaces/SupportCseBasics/pages?title=Configure+APOD+-+SQL+DB+Persistence)  [Configure APOD - SQL DB Persistence - Mongo to SQL Migration](https://alteryx.atlassian.net/wiki/spaces/SupportCseBasics/pages?title=Configure+APOD+-+SQL+DB+Persistence+-+Mongo+to+SQL+Migration) |
| **Help** | <https://help.alteryx.com/current/en/server/configure/database-management/sql-db-management/configure-sql-server.html> |