---
id: 05b942ff49281926
title: MongoDB Backup
status: intake
source:
  kind: confluence_page
  id: confluence-page:1997439417
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/1997439417
  summary: null
category: []
keywords: []
created_at: 2026-04-14T15:09:17Z
updated_at: 2026-04-14T15:09:17Z
confidence: null
cross_refs: []
content_hash: sha256:1e787538acb349c35dade8314388a4c7baf960517511bf9915d027f8d2f8ad46
confluence_page_id: null
model_used: null
---

---

---

for embedded mongo onlyPurple

> **ℹ️ Info**
>
> How to backup an Embedded MongoDB

|  |  |
| --- | --- |
|  |  |
|  |  |
|  |  |
|  |  |
|  |  |

---

X: 
cd \FOLDER \Alteryx\bin
net stop alteryxservice
**AlteryxService.exe emongodump=DRIVE:\PATH_BACKUP**

Failed: error writing data for collection `AlteryxService.AS_ResultsFiles.Files` to disk: error reading collection: (CursorNotFound) cursor id 1079540379547815088 not found

Mongodump failed: 2

AlteryxService.exe uses the mongodump.exe and mongorestore.exe files in the bin folder when performing a backup or restore.  It then copies the two BIN files the Alteryx Service needs to have in the persistence folder.

mongodump --host=localhost:27018 -vvvvv  --out=DRIVE:\PATH 2>DRIVE:\PATH\mongoDump.log

mongodump --uri="mongodb://localhost:27017 " -vvvvv  --out DRIVE:\PATH 2>DRIVE:\PATH\mongoDump.log

Failed: error dumping metadata: error creating directory for metadata file ut=E:\BACKUP\AlteryxService: mkdir ut=E:: The filename, directory name, or volume label syntax is incorrect.