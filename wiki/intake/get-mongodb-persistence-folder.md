---
id: b2f7db4e1e9f8b6d
title: Get MongoDB Persistence Folder
status: intake
source:
  kind: confluence_page
  id: confluence-page:1702894595
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/1702894595
  summary: null
category: []
keywords: []
created_at: 2026-04-14T15:10:53Z
updated_at: 2026-04-14T15:10:53Z
confidence: null
cross_refs: []
content_hash: sha256:9f5e6a291e425055a187038cb2ba037cb5f926bca1473c26ee314abda187bbdc
confluence_page_id: null
model_used: null
---

for embedded mongo onlyPurple

> **ℹ️ Info**
>
> Determine the current MongoDB Persistence Folder for Embedded Mongo.

# Location

The MongoDB Persistence Folder is set in **Alteryx System Settings > Controller > Persistence > Data Folder**.  The default location is

In RuntimeSettings.xml it appears as

```
True C:\ProgramData\Alteryx\Service\Persistence\MongoDB ]]>
```

# Create a new database

Changing to a NEW folder name (the folder need not exist) will create a NEW database.  This is a great way to test a different auth type or determine if the Service isn’t starting due to a database issue or a configuration/environmental issue.

When restoring a Mongo database, change the Persistence Folder setting to point to the restored folder.