---
id: bd5cb6609fa82250
title: FAQ / Help - CryptoMigration
status: published
source:
  kind: confluence_page
  id: confluence-page:1640793183
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/1640793183
  summary: null
category:
- product-tree
- alteryx-server
keywords:
- cryptomigration
- tool
- prep
- upgrade
- will
created_at: 2026-04-13T01:55:58Z
updated_at: 2026-04-13T01:58:53Z
confidence: 0.55
cross_refs: []
content_hash: sha256:a4fea0557829743ef9cd3e9caa6dd0dccead375893cad2e8af728b7d227298e7
confluence_page_id: null
model_used: heuristic
---

# Overview

The upgrade to Server 2022.3 requires a CryptoMigration step to re-encrypt data in Mongo and RuntimeSettings.xml to the AES256 standard with SHA-256 hashing (like the FIPs Server).

Ideally, customers will use the **Migration Prep Tool** to prepare for a 2022.3 Server upgrade and reduce downtime during the actual upgrade. Otherwise, all re-encryption will occur when the Service starts for the first time and the Service will remain in the “Starting” status for minutes…hours…or days, depending on the size of the database.

The CryptoMigration was tested for upgrades from 2021.2+.  If coming from earlier versions, upgrade to 2021.2-2022.1 first.  For Built-In Auth, upgrade to 2022.1 first.

# FAQs

## BEFORE

---

### Best practices?

Back up Mongo, then run the **Migration Prep Tool** to *successful *completion prior to attempting the Server Upgrade.  This minimizes Server downtime as the time-consuming work of re-encrypting workflows is completed (and successful) before starting the Server upgrade.  It reduces the risk that a CryptoMigration failure will cause an Upgrade to fail and necessitate rollback.

Point customers to <https://help.alteryx.com/20223/server/migration-prep-tool> and recommend using the **Migration Prep Tool**.

---

### How long does it take?

Minutes, hours, or days, depending on the database size.

Each version  of each workflow will be re-encrypted at a possible rate of 10-15sec/workflow version = 4-6 versions/min = 240-360 versions/hr = 5760-8640/day.

Slow disk I/O or using Atlas will increase this time.  The Product Team and documentation do not provide clear guidance around speed.

One TAM’s experience

The Migration Prep Tool took 20min for a 13GB database and another 20min for the Server Upgrade.

---

### Disk space needed?

The Migration Prep Tool presents an estimate of the space it will need to create staging collections of re-encrypted workflows.  This can almost double the size of the Mongo database.

However, the total disk space could be up to 8x the original database for the complete upgrade.  For example:

- 100GB - Initial DB
- 100GB - Recommended initial database backup before starting the Migration Prep Tool
- 200GB - DB after CryptoMigration (it won’t quite double the database, but can come close)
- 200GB - Recommended database backup before Server Upgrade (not needed if initial backup is recent)
- 200GB - Mongo 4.0 PreUpgrade backup folder automatically created for upgrade from pre-2021.3.6
- ---------
- 800GB = up to 8x in the worst case

After the upgrade, customers can regain some of this space by deleting pre-re-encryption collections as described in How to Cleanup Server to Recover Disk Space.

---

## DURING

---

### How to run Prep Tool?

The Prep Tool installs to its own folder:

Example for embedded Mongo:

AlteryxServiceMigrator22_2.exe -p -c "mongodb://user:NON_ADMIN_MONGO_PASSWOR D@localhost:27018/AlteryxService?authSource=AlteryxService" -i HOST/IP_ADDRESS  -t CONTROLLER_TOKEN
**OLD example **for embedded Mongo (pre-June 2024 when the Prep Tool was updated and backported, rendering the following obsolete:

AlteryxServiceMigrator22_2.exe -p -c "mongodb://user:NON_ADMIN_MONGO_PASSWORD @localhost:27018/AlteryxService?authSource=AlteryxService"
Help page – <https://help.alteryx.com/20223/en/server/install/install-or-upgrade-server/migration-prep-tool/run-the-migration-prep-tool.html>

---

### What does the Prep Tool CryptoMigrate?

Workflow Collections are read and re-encrypted into new Collections with **22.3** added to their name.  If the Migration Prep Tool or Service are restarted they pick up where they left off.

The **Prep Tool** creates the following CryptoMigrated collections

- AS_App_Chunks.22.3
- AS_App_Chunks.22.3.Files
- AS_PackageDefinitions.22.3
- AS_PackageDefinitions.22.3.Files

As of the latest Migration Prep tools released (>= version 2022.3), the **Prep Tool** also creates the following CryptoMigrated collections:

- AS_Queue.22.3
- AS_Queue.22.3.Files
- AS_RunAsCredentials.22.3
- AS_RunAsCredentials.22.3.Files
- AS_Schedules.22.3
- AS_Schedules.22.3.Files

---

### What happens during Server Upgrade?

When the upgraded Service first starts it kicks off a full CryptoMigration.

**If CryptoMigration Prep Tool was run**
The Service CryptoMigrates any workflows or other assets (Schedules, Credentials, etc.) added since the Prep Tool ran

**If CryptoMigration Prep Tool was NOT run**
The Service CryptoMigrates the collections the Prep Tool would have CryptoMigrated

- AS_App_Chunks.22.3
- AS_App_Chunks.22.3.Files
- AS_PackageDefinitions.22.3
- AS_PackageDefinitions.Files.22.3

**Additional Collections are CryptoMigrated**
Regardless of whether the Migration Prep Tool was ran previously or not, the Service CryptoMigrates (possibly a repeat of the previous Migration Prep Tool) additional collections, creating the following

- AS_Queue.22.3
- AS_Queue.22.3.Files
- AS_RunAsCredentials.22.3
- AS_RunAsCredentials.22.3.Files
- AS_Schedules.22.3
- AS_Schedules.22.3.Files

**RuntimeSettings.xml file is re-encrypted**
This process leaves three versions of RuntimeSettings.xml

- RuntimeSettings.22_2_legacy.xml – the original, pre-upgrade version
- RuntimeSettings.22_2_migration.xml – the re-encrypted version
- RuntimeSettings.xml – a copy of the re-encrypted RuntimeSettings.22_2_migration.xml

If the **Controller Token** was the older, shorter length (40-char), it will be regenerated to the new 64-char length.  The same lengthening happens on Workers so they produce the same new Controller Token value and will be able to connect to the Controller after upgrade.

A new section is added to RuntimeSettings.xml

- 1 ]]>

**On successful completion of all steps above, CryptoMigrated Collections are renamed**

- The original (pre-upgrade) collections are renamed to backups, ex Backup_AS_Queue.Pre.22.2
- The CryptoMigrated collections take their place, ex: AS_Queue.22.3 is renamed AS_Queue

---

### Is CryptoMigration stuck?

What’s normal?  The Service can remain in “starting” status for a long time.  TAM experience [Tom D]:

Server Upgrade (Service “starting”) took over an hour with nothing seeming to be happening  despite having run the CryptoMigration Pep Tool prior.  The CryptoMigration log showed the remaining collections (that only migrate during Server upgrade) were ‘complete’, but the **AlteryxServiceMigrator22_2.exe** stayed in TaskManager with minimal CPU (<5, but changing) with memory use increasing slowly for an hour before the Service started and the Server Service log started logging.

Powershell can stop displaying progress.  TAM experience [James H]

Powershell buffering stopped at n% due to the logging being so long .  From that point monitor the CPU usage on the migrator in Task Manager until it gets to 0.  Can also periodically open the migration log and see what % it is at.  When it reaches 100% and complete you can press ctrl + c in powershell and the rest of the logging messages will complete.

**Option to view progress - **View the CryptoMigration log in a way that doesn’t block the CryptoMigration Prep Tool from writing to it, then view it again after some time to see if it’s adding progress updates.

- Open with Notepad++, or
- Copy it and open in Notepad

**Option to ensure the process isn’t hung - **View the **AlteryxServiceMigrator22_2.exe** in Task Manager and ensure it’s using some CPU cycles.

---

## AFTER

---

### CryptoMigration Log Location

The log is written in two locations.  This was submitted as a defect: GCSE-116077dcf2c9-72f3-3ff6-8e99-fe88e9f473f1System Jira but was deemed WAD:

| Location 1 | The Prep Tool has a fixed locationC:\ProgramData\Alteryx\Service\AlteryxServiceMigrator_#.log |
| --- | --- |
| Location 2 | When the Service starts it will log to the location from Alteryx System Settings (which defaults to C:\ProgramData\Alteryx\Service)Alteryx System Settings > Controller > General > LoggingGCSE-211277dcf2c9-72f3-3ff6-8e99-fe88e9f473f1System JiraIf there is a space in the log folder path the logging will appear in wrong file.  Ex:  D:\Program Data\Alteryx\Service will log to a file called D:\Program and not rotate.  This is the path set by: Controller > General > Logging]]> |

Logs rotate starting with # = 0

Controller node contains the main log with collection migrations that can be created by the Prep Tool or when Service first starts after upgrade.

Other nodes generate a log when Service first starts for CryptoMigration of RuntimeSettings.xml.

See <https://alteryx.atlassian.net/wiki/spaces/SupportServer/pages/edit-v2/1640761815>

---

### Did CryptoMigration succeed?

The CryptoMigration log summarizes errors at the end.  Some errors will prevent the Server upgrade from succeeding while others will not.  At this time we don’t have enough information to determine which are which.

**The last lines do NOT indicate success or failure**, look through the log for **Migration failed**.  To find lines with errors, search for **;1;**, **;2;**, and **;3;**  (as of Jun-24, also review **;4;**, but this is moving to **;1;** soon per

- TGAL-1126877dcf2c9-72f3-3ff6-8e99-fe88e9f473f1System Jira )

See <https://alteryx.atlassian.net/wiki/spaces/SupportServer/pages/edit-v2/1640761815>

---

### If CryptoMigration stopped or Service won’t start?

Re-run the Migration Prep Tool or restart the Service a few times.  The CryptoMigration process will pick up where it left off.

---

### Fix CryptoMigration Issues

See Errors (CryptoMigration Log)

If a rollback is needed, the original RuntimeSettings.xml has to be restored.  The CryptoMigration Tool renamed the original to **RuntimeSettings.22_2_legacy.xml**.

If upgrading to Server 2024.1+, a possible last resort is to use the Command Line to force CryptoMigration to skip over the records that failed to migrate and allow Alteryx Service to start: How To: Run the Hidden CLI command for Crypto Migration . Note that the underlying issue(s) will still persist after the upgrade.

---

### How to Rollback?

1. Rename RuntimeSettings.22_2_legacy.xml to RuntimeSettings.xml
2. CollectCrytpo Migration logsPre-migration Mongo backupIf they won’t provide the entire Mongo, try to capture the JSON for an example record that failed CryptoMigration.Pre-migration RuntimeSettings.xml (ie, RuntimeSettings.22_2_legacy.xml)Controller Token in a text file
   1. Crytpo Migration logs
   2. Pre-migration Mongo backupIf they won’t provide the entire Mongo, try to capture the JSON for an example record that failed CryptoMigration.
      1. If they won’t provide the entire Mongo, try to capture the JSON for an example record that failed CryptoMigration.

   3. Pre-migration RuntimeSettings.xml (ie, RuntimeSettings.22_2_legacy.xml)
   4. Controller Token in a text file

3. Rollback the upgrade in the normal way
4. With the above, we can do a Host Recovery and attempt the CryptoMigration Tool to document the issue in Jira

---

### How to clean up the pre-re-encryption collections bloating Mongo?

After the 2022.3 CryptoMigration, the original (un-re-encrypted) Collections are left in the Mongo database.  These can be deleted:

- Perform a MongoDB backup
- Drop AlteryxService collections starting with "Backup_".  This can be done in Robo3T by right-clicking each collection and choosing drop or in Mongo Shell with the following commands (tip: copy and paste all commands together):

db.getCollection("Backup_AS_App_Chunks.Pre22.2").drop()
db.getCollection("Backup_AS_App_Chunks.Pre22.2.Files").drop()
db.getCollection("Backup_AS_PackageDefinitions.Pre22.2").drop()
db.getCollection("Backup_AS_PackageDefinitions.Pre22.2.Files").drop()
db.getCollection("Backup_AS_Queue.Pre22.2").drop()
db.getCollection("Backup_AS_Queue.Pre22.2.Files").drop()
db.getCollection("Backup_AS_RunAsCredentials.Pre22.2").drop()
db.getCollection("Backup_AS_RunAsCredentials.Pre22.2.Files").drop()

- Perform a Mongo backup and restore to compress the size of the Mongo database