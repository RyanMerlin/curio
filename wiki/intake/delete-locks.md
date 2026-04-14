---
id: 84aebb6e90f09d0d
title: Delete locks
status: intake
source:
  kind: confluence_page
  id: confluence-page:1702927446
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/1702927446
  summary: null
category: []
keywords: []
created_at: 2026-04-14T15:10:53Z
updated_at: 2026-04-14T15:10:53Z
confidence: null
cross_refs: []
content_hash: sha256:fbd6c5ff1318b7026dd25a72aea481d79f9b04e2d1529d7ee52ece5fa2851ce5
confluence_page_id: null
model_used: null
---

> **ℹ️ Info**
>
> This articles explains how to “delete locks” in MongoDB

As part of a [Server Host Recovery](https://help.alteryx.com/current/server/server-host-recovery-guide) or Reindex you need to "delete locks".

You'll delete all records from the **AlteryxGallery.locks** and **AlteryxGallery_Lucene.luceneFs.locks** collections.  These locks contain the machine name of the machine that wrote them and can't be deleted by another machine. Because of this, these locks will block the Service from accessing Mongo if the Server's machine name was changed or the Mongo database was restored from another machine without following the [Server Host Recovery](https://help.alteryx.com/current/server/server-host-recovery-guide).

You can delete locks without stopping the Service.

For Embedded Mongo, open a Command Prompt as the Administrator and enter the following (adjusted for location of BIN folder)

c: 
cd %ProgramFiles% \Alteryx\bin
**For 23.1+**

mongo -u user -p MONGO_USER_PASSWORD -host localhost:27018 **AlteryxGallery**
db.locks.deleteMany({})
db.searchLocks.deleteMany({})
exit
**For 22.3 and prior only** – You're returned to the command line, now enter

mongo -u user -p MONGO_USER_PASSWORD  -host localhost:27018 **AlteryxGallery**
db.locks.deleteMany({})
exit
Press **Enter**

**For 22.3 and prior only** – You're returned to the command line, now enter

mongo -u user -p MONGO_USER_PASSWORD -host localhost:27018 **AlteryxGallery_Lucene**
db.luceneFs.locks.deleteMany({})
exit
Press **Enter**