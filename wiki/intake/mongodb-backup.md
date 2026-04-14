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
created_at: 2026-04-14T15:18:28Z
updated_at: 2026-04-14T15:18:28Z
confidence: null
cross_refs: []
content_hash: sha256:13b845642b496b79e4abcde74c0edb62b9f9e131fc81b979f0733e854f563387
confluence_page_id: null
model_used: null
---

---

---

for embedded mongo onlyPurple

> **ℹ️ Info**
>
> How to backup an Embedded MongoDB

| **Key Articles** | [Alteryx Server Backup and Recovery Part 2: Procedures](https://knowledge.alteryx.com/index/s/article/Alteryx-Server-Backup-Recovery-Part-2-Procedures-1583460176762) (KB)                                    <== **includes PowerShell script to automate backups**  <https://help.alteryx.com/current/en/server/install/server-host-recovery-guide/disaster-recovery-preparation.html>  <https://help.alteryx.com/current/en/server/best-practices/backup-best-practices/critical-server-files-and-settings-to-backup.html?lang=en> |
| --- | --- |
| **Alternatives** | When Service is stopped (to ensure you don’t capture Mongo DB mid-update)     - ZIP the Persistence folder    - Snapshot the entire machine |
| **Tool Updates** | Mar-2024 – Mongo backup and Restore tools were updated and backported to older Server versions. The goal was to reduce errors during backup and restore such as memory exhaustion.     - TCPE-95277dcf2c9-72f3-3ff6-8e99-fe88e9f473f1System Jira |
| **Validate backup counts** | [Utility Workflow - Validate_Mongo_Dump_Collection_Counts](https://alteryx.atlassian.net/wiki/search?text=Utility+Workflow+-+Validate_Mongo_Dump_Collection_Counts)    <== Utility compares mongoDump.log counts to the source MongoDB.             Created for a case where backup didn’t backup all records |
| **Jira** | TCPE-95577dcf2c9-72f3-3ff6-8e99-fe88e9f473f1System Jira |
| **Help** | <https://help.alteryx.com/current/server/mongodb-backups> |

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