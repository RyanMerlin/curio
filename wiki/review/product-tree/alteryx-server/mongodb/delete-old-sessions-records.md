---
id: e93172b88a7858ad
title: Delete old sessions records
status: review
source:
  kind: confluence_page
  id: confluence-page:1702927558
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/1702927558
  summary: null
category:
- product-tree
- alteryx-server
- mongodb
keywords:
- mongodb
- sessions
- delete
- maintenance
- cleanup
created_at: 2026-04-14T15:18:28Z
updated_at: 2026-04-14T15:19:34Z
confidence: 0.85
cross_refs: []
content_hash: sha256:66813ac7082d70e5973a33288d4a68c1bd6390be9e3679291bfc41dc598c23b6
confluence_page_id: null
model_used: claude-sonnet-4-6
---

> **ℹ️ Info**
>
> This articles explains how to **delete sessions** in MongoDB and/or add an index
> 
> Too many **sessions **records can cause various timeout errors on Server

| **Count all sessions** | db.sessions.count() |
| --- | --- |
| **Count sessions > 30 days old** | db.sessions.find({UpdateDate:{$lt: new Date(ISODate().getTime() - 1000 * 86400 * **30**)}}).count() |
| **Delete sessions >30 days old** | db.sessions.remove({UpdateDate:{$lt: new Date(ISODate().getTime() - 1000 * 86400 * **30**)}}) |
| **Add an index to make access faster** | db.getCollection('sessions').createIndex({SessionId: -1}) |

---

db.sessions.count()

Some customers may want to copy the old records before deleting them

Consider keeping **90 days** if the customer’s TAM is planning to run the **Server Health Check** workflow as it examines 90 days of sessions records

db.sessions.remove({UpdateDate:{$lt: new Date(ISODate().getTime() - 1000 * 86400 * **30**)}})

db.getCollection('sessions').createIndex({SessionId: -1})

clear-sessions.bat MONGO_NON_ADMIN_PSWD NUM_DAYS_TO_RETAIN

clear-sessions.bat 92f44226996547188a0b568c3119ef5200485d4bfb90386382bc69130c099aec 7

"C:\Program Files\Alteryx\bin\mongosh.exe" localhost:27018/AlteryxGallery -u user -p %1 --eval "db.sessions.remove({UpdateDate:{$lt: new Date(ISODate().getTime() - 1000 * 86400 * %2)}});"