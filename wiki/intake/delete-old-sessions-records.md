---
id: e93172b88a7858ad
title: Delete old sessions records
status: intake
source:
  kind: confluence_page
  id: confluence-page:1702927558
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/1702927558
  summary: null
category: []
keywords: []
created_at: 2026-04-14T15:06:10Z
updated_at: 2026-04-14T15:06:10Z
confidence: null
cross_refs: []
content_hash: sha256:c84fc8805d0cd7c3b8ed5f2bf66afda5ac2b1346ccf96499d7108d8fef7b953c
confluence_page_id: null
model_used: null
---

> **ℹ️ Info**
>
> This articles explains how to **delete sessions** in MongoDB and/or add an index
> 
> Too many **sessions **records can cause various timeout errors on Server

|  |  |
| --- | --- |
|  |  |
|  |  |
|  |  |

---

db.sessions.count()

Some customers may want to copy the old records before deleting them

Consider keeping **90 days** if the customer’s TAM is planning to run the **Server Health Check** workflow as it examines 90 days of sessions records

db.sessions.remove({UpdateDate:{$lt: new Date(ISODate().getTime() - 1000 * 86400 * **30**)}})

db.getCollection('sessions').createIndex({SessionId: -1})

clear-sessions.bat MONGO_NON_ADMIN_PSWD NUM_DAYS_TO_RETAIN

clear-sessions.bat 92f44226996547188a0b568c3119ef5200485d4bfb90386382bc69130c099aec 7

"C:\Program Files\Alteryx\bin\mongosh.exe" localhost:27018/AlteryxGallery -u user -p %1 --eval "db.sessions.remove({UpdateDate:{$lt: new Date(ISODate().getTime() - 1000 * 86400 * %2)}});"