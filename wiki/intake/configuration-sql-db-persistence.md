---
id: 8e77004416c69f7d
title: Configuration (SQL DB Persistence)
status: intake
source:
  kind: confluence_page
  id: confluence-page:2650801612
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/2650801612
  summary: null
category: []
keywords: []
created_at: 2026-04-14T15:02:19Z
updated_at: 2026-04-14T15:02:19Z
confidence: null
cross_refs: []
content_hash: sha256:c42113c2f4021e51d5036e23ea27c09b516100a5ca3d751086ecd57f15a72a92
confluence_page_id: null
model_used: null
---

---

---

| Key Articles | How to stand up a new install of Alteryx Server with user-managed MSSQL persistence with custom database names (KB) |
| --- | --- |
| Change SQL port | While port 1433 is specified in the Connection string, to change it, you have to add tcp: in front of SQL Server name, ex:#E3FCEFDriver={ODBC Driver 17 for SQL Server};Server=tcp:sqlserver.example.com,5352;UID=MY_USER;PWD=MY_PSWD;Integrated Security=False;Database=AlteryxService;Server=tcp:sqlserver.example.com,5352;Database=AlteryxGallery;User ID=MY_USER;Password=MY_PSWD; |
| APOD Setup | Configure APOD - SQL DB Persistence Configure APOD - SQL DB Persistence - Mongo to SQL Migration |
| Help | https://help.alteryx.com/current/en/server/configure/database-management/sql-db-management/configure-sql-server.html |