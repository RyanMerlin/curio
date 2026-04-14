---
id: e36bdc5f3d94406f
title: MongoDB Restore
status: review
source:
  kind: confluence_page
  id: confluence-page:1997472512
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/1997472512
  summary: null
category:
- product-tree
- alteryx-server
- mongodb
keywords:
- mongodb
- restore
- backup
- embedded-mongo
- recovery
created_at: 2026-04-14T15:18:28Z
updated_at: 2026-04-14T15:20:25Z
confidence: 0.87
cross_refs: []
content_hash: sha256:5916b3b02fe26ddaa22d8260ed249d7e6f920135c00f213c8d3a3862fe89d5d0
confluence_page_id: null
model_used: claude-sonnet-4-6
---

for embedded mongo onlyPurple

> **ℹ️ Info**
>
> How to restore a MongoDB backup

> **ℹ️ Info**
>
> **AlteryxService.exe** calls the Mongo-supplied **mongorestore.exe** in \Ateryx\bin to restore the database.  **AlteryxService.exe** also copies other files the Server needs to run that are not used by Mongo.
> 
> - ASCredentials.bin
> - ASMongoDBVersion.bin

| **Key Articles** | [Alteryx Server Backup and Recovery Part 2: Procedures](https://knowledge.alteryx.com/index/s/article/Alteryx-Server-Backup-Recovery-Part-2-Procedures-1583460176762) (KB)                                    <== **includes PowerShell script ot automate backups** |
| --- | --- |
| **Restore** | X:  cd \FOLDER \Alteryx\bin net stop alteryxservice **AlteryxService.exe emongorestore=”X:\BKP”,”X:\REST”,10** **,10** is not needed in modern versions of Server (24+).  It was used to minimize memory use as large restore could blow out memory.  Newer versions of MongoDB manage memory better w/o the need for .**10.** |
| **Log** | In the restore folder  A quick scan will review any exceptions thrown during the restore as they appear with very different formatting (lots of whitespace and indented sections). |
| **Tool Updates** | Mar-2024 – Mongo backup and Restore tools were updated and backported to older Server versions. The goal was to reduce errors during backup and restore such as memory exhaustion.     - TCPE-95277dcf2c9-72f3-3ff6-8e99-fe88e9f473f1System Jira |
| **Jira** | TCPE-95277dcf2c9-72f3-3ff6-8e99-fe88e9f473f1System Jira  TCPE-95577dcf2c9-72f3-3ff6-8e99-fe88e9f473f1System Jira |
| **Help** | <https://help.alteryx.com/current/server/mongodb-backups> |

# Standard Restore

|  | **Action** | **Steps** |
| --- | --- | --- |
| 1 | #### Prepare APOD | If you will be restoring a large customer database, start an APOD with large C: and D: drives.  Set a large swap file on C: (explained below) and do the restore to D: (see [Mongo Error - fatal error: out of memory](https://alteryx.atlassian.net/wiki/search?text=Mongo+Error+-+fatal+error:+out+of+memory)) |
| 2 | #### Stop Service | Stop the Alteryx Service |
| 3 | #### Host Recovery? | noteRed  If restoring a database from a backup made on another machine, you MUST follow the <https://help.alteryx.com/current/en/server/install/server-host-recovery-guide.html> |
| 4 | #### Restore from pre_upgrade folder? | If restoring from a Pre_Upgrade folder you’ll need to manually create **AS_MongoDBVersion.bin** to contain the MongoDB version that matches the Server install version before restoring, see   > [ASCredentials.bin](https://alteryx.atlassian.net/wiki/search?text=ASCredentials.bin)   > <https://help.alteryx.com/current/server/mongodb-schema-reference> |
| 5 | #### Restore | Open **Command Prompt** as **Administrator**.  (*) If restoring from a **Pre_Upgrade **folder, see section “**Restore from pre_upgrade folder?**“  X:  cd \FOLDER \Alteryx\bin net stop alteryxservice **AlteryxService.exe emongorestore=”X:\BKP”,”X:\REST”,10** The “10” at the end sets the mongo batchSize to try to avoid the [Mongo Error - fatal error: out of memory](https://alteryx.atlassian.net/wiki/search?text=Mongo+Error+-+fatal+error:+out+of+memory) error.  While more reliable, this may slow the restore down. |
| 6 | #### Success? | **Confirm restore was 100% successful**  Check the **mongoRestore.log **in the folder you restored Mongo to  > **⚠️ Warning** > > **Mongo Restore FAILS SILENTLY and frequently failes for larger restores on APODS** >  > [Ed P] Should we consider requesting ZIP files of the persistence folder instead on Mongo backups for larger databases? I know there is lore that ZIPs aren’t reliable, but they seem quite reliable for all other data so why would a Mongo folder be different?  Example of silent failure – the restore appears successful:  But the **mongoRestore.log** (in the restore folder) shows that it failed  or  Despite failing, you can still start the MongoDB without error.  You may be able to start the Service but data will be missing (for example, all of the Shared Gallery Connections in a case Jon L worked).  Other times the Service will fail to start with an error that the “user” user is missing.  To correct **out of memory** issue, see: [Mongo Error - fatal error: out of memory](https://alteryx.atlassian.net/wiki/search?text=Mongo+Error+-+fatal+error:+out+of+memory) |
| 7 | #### Point to restored folder | Run **Alteryx System Settings** and adjust the **Controller > Persistence folder** to the restored location. |