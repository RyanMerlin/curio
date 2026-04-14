---
id: 5572620671e964e1
title: MongoDB to SQL DB Migration
status: intake
source:
  kind: confluence_page
  id: confluence-page:2188215414
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/2188215414
  summary: null
category: []
keywords: []
created_at: 2026-04-14T15:02:19Z
updated_at: 2026-04-14T15:02:19Z
confidence: null
cross_refs: []
content_hash: sha256:45b373dadceec566a3b68d08eb93addde44acfc711a002a84179e835c36cf411
confluence_page_id: null
model_used: null
---

---

---

note Asked Michael P where to place this error.  If it’s the Service Log it would be good to have the lead-in to show the full error messages.  And since this error refers to looking into the “Migrator” logs, it would be good to understand which of the many migration logs this is and get that error as well (likely the Service Schema migration).

- Server Migrator failed with exit code 2993

Asked Michael P where to place this error.  If it’s the Service Log it would be good to have the lead-in to show the full error messages.  And since this error refers to looking into the “Migrator” logs, it would be good to understand which of the many migration logs this is and get that error as well (likely the Service Schema migration).

- Server Migrator failed with exit code 2993

---

> **ℹ️ Info**
>
> 24.1+ Server Mongo to SQL DB Migration process.

> **📝 Note**
>
> After upgrade, Service must be started to perform Mongo Schema Migration before attempting Mongo to SQL DB Migration

| Access | Migration workflowhttps://us1.alteryxcloud.com/license-portal/ > Server > VERSION >        Workflow to migrate from MongoDB to SQLMicrosoft ODBC Driver 18 for SQL Server (x64)https://learn.microsoft.com/en-us/sql/connect/odbc/download-odbc-driver-for-sql-server?view=sql-server-ver16 Simba MongoDB 2.3.22.1024 64-bit Driverhttps://us1.alteryxcloud.com/license-portal/ > Drivers > MongoDB >        Simba MongoDB 2.3.22.1024 64-bitSimba SchemaUnpack Migration Workflow YXZP for \Resources\MongoDB_Schema.json needed to configure the Simba driver |
| --- | --- |
| Tutorials | https://youtu.be/RUytFrg5Bcc?si=fYT3h-sIKoCT6gbs   16m   <== Jarrod’s walk-through of configuring the SQL DB migration |
| Help | https://help.alteryx.com/current/en/server/configure/database-management/sql-db-management/mongo-to-sql-migration-guide.html https://help.alteryx.com/current/en/server/configure/database-management/sql-db-management/server-sql-db-customer-faq.html#mongodb-to-mssql-migrator-faq > MongoDB to MSSQL Migrator FAQ |