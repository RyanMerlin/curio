---
id: 567092b57260c876
title: 'Example: Alteryx Server Upgrade 22.1 to 23.2 (MongoDB Upgrade: 4.2 to 6.0)'
status: intake
source:
  kind: confluence_page
  id: confluence-page:2639659009
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/2639659009
  summary: null
category: []
keywords: []
created_at: 2026-04-14T15:12:39Z
updated_at: 2026-04-14T15:12:39Z
confidence: null
cross_refs: []
content_hash: sha256:b0fd7f1545430d4772b057115ed8517de2ab9cc439767ac95118036a36df2ca0
confluence_page_id: null
model_used: null
---

> **ℹ️ Info**
>
> **Subject:** Upgrade from 4.2 to 6.0 - The Stages of Migration and Logs
> 
> **Overview:** This document demonstrates the migration process for upgrading Alteryx Server from version **22.1** to **23.2**, including the MongoDB upgrade from **4.2 to 6.0**. Logging has been configured in `D:\ProgramData\Alteryx` to distinguish logs for each stage

Review: Embedded MongoDB upgrade (a.k.a. MongoDB version migration) - Support-Server - Confluence

---

---

### Step 1: Running Migration Prep Tool for upgrade to/through 22.3

The **Migration Prep Tool** is a crucial first step for upgrading from older versions of Alteryx Server (2022.1 and below) due to the need for Crypto Migration. [This step is not needed if migrating from 2022.3 + since the crypto migration is already completed with 2022.3]

**Purpose:**

- Detect potential issues with Alteryx Service DB crypto migration.

**Logs:**

- Location: C:\ProgramData\Alteryx\Service
- File Name: AlteryxServiceMigrator_X.log
- Errors (CryptoMigration Log)

Review this log to confirm no errors or warnings before proceeding.

---

### Step 2: Installing the New Alteryx Version

Post installation, a popup prompts for the MongoDB version upgrade (if migrating from 4.2 → 6.0).

**Key Actions:**

1. Backup Process: When clicked on BeginBackupandMigrationTwo backups are created by AlteryxService.exe: [This process requires 3x free space in Drive and the customer should ensure they do have that space]MongoDB_PreupgradeMongoDB_Backup
   - Two backups are created by AlteryxService.exe: [This process requires 3x free space in Drive and the customer should ensure they do have that space]MongoDB_PreupgradeMongoDB_Backup
      - MongoDB_Preupgrade
      - MongoDB_Backup

2. CryptoMigration Execution:Executable: AlteryxServiceMigrator22_2.exeTask: Updates runtimesettings.xml and begins migrating other Alteryx Service DB collections.Logs: Written to:Default Location: C:\ProgramData\Alteryx\Service [This has the runtimesettings.xml migration info and the rest migrator log will be written in Configured Location]Configured Location: D:\ProgramData\Alteryx\ServiceFile Name: AlteryxServiceMigrator_X.log
   - Executable: AlteryxServiceMigrator22_2.exe
   - Task: Updates runtimesettings.xml and begins migrating other Alteryx Service DB collections.
   - Logs: Written to:Default Location: C:\ProgramData\Alteryx\Service [This has the runtimesettings.xml migration info and the rest migrator log will be written in Configured Location]Configured Location: D:\ProgramData\Alteryx\ServiceFile Name: AlteryxServiceMigrator_X.log
      - Default Location: C:\ProgramData\Alteryx\Service [This has the runtimesettings.xml migration info and the rest migrator log will be written in Configured Location]
      - Configured Location: D:\ProgramData\Alteryx\Service
      - File Name: AlteryxServiceMigrator_X.log

**Validation:**

- Verify successful migration of collections like AS_Runascredentials,As_Schedules,etc.
- Ensure no errors in AlteryxServiceMigrator_X.log.
- Errors (CryptoMigration Log)

---

### Step 3: Alteryx Service Schema Migration

Once the service migration completes, the process transitions to the **Alteryx Server Schema Migration**, monitored by `AlteryxServerMigrator.exe`.

**Key Points:**

- Logs Location:Default: C:\ProgramData\Alteryx\ServiceConfigured: D:\ProgramData\Alteryx\ServiceFile Name: alteryx-migration.csvAlteryxSERVICE Schema Migration Log Messages  (alteryx-migration.csv)
   - Default: C:\ProgramData\Alteryx\Service
   - Configured: D:\ProgramData\Alteryx\Service
   - File Name: alteryx-migration.csv
   - AlteryxSERVICE Schema Migration Log Messages  (alteryx-migration.csv)

- Migration Details:2022.3 → Migration 0: Quick with no DB changes.2023.1 → Migration 1 & 2023.2 → Migration 2: Time-intensive depending on DB size.Creates intermediate collections like AS_XXXXPostMigration_2.
   - 2022.3 → Migration 0: Quick with no DB changes.
   - 2023.1 → Migration 1 & 2023.2 → Migration 2: Time-intensive depending on DB size.
   - Creates intermediate collections like AS_XXXXPostMigration_2.

**Note:**

During this stage:

- AlteryxServerMigrator.exe may exhibit low CPU usage.
- Alteryx Service will remain in a "starting" state until schema migration completes.

**Sample Logs:**

- 2024-11-14 22:25:13.899584,INFO,1,AlteryxServerMigrator,migrationLogger,UseParsedArguments,Alteryx Server Migrator is initialized.,
- 2024-11-14 22:25:14.025583,INFO,1,AlteryxServerMigrator,migrationLogger,UseParsedArguments,Server Mongo Database migrations will not be run via ServerMigrator.,
- 2024-11-14 22:25:15.635592,INFO,1,AlteryxServerMigrator,migrationLogger,UseParsedArguments,Beginning Service Mongo Db Migrations,
- 2024-11-14 22:25:19.505612,FATAL,1,AlteryxServerMigrator,migrationLogger,DoMigrateDatabase,Database requires migration from 0 to 2.  Attempting to obtain lock...,
- 2024-11-14 22:25:19.742607,FATAL,1,AlteryxServerMigrator,migrationLogger,DoMigrateDatabase,"...lock obtained, performing migration.",
- 2024-11-14 22:25:19.742607,INFO,1,AlteryxServerMigrator,migrationLogger,MoveNext,Starting Service Migration: 1,
- 2024-11-14 22:25:19.767620,INFO,1,AlteryxServerMigrator,migrationLogger,MoveNext,,
- 2024-11-14 22:25:19.891610,INFO,1,AlteryxServerMigrator,migrationLogger,MoveNext,Service Migration 1 Completed.,
- 2024-11-14 22:25:19.891610,INFO,1,AlteryxServerMigrator,migrationLogger,DoMigrateDatabase,--Migration 1 took 00:00:00.1490028,
- 2024-11-14 22:25:19.918627,INFO,1,AlteryxServerMigrator,migrationLogger,MoveNext,Starting Service Migration: 2,
- 2024-11-14 22:34:57.720013,INFO,1,AlteryxServerMigrator,migrationLogger,MoveNext,Migrating records from AS_Results.->,
- 2024-11-14 22:34:57.726011,INFO,1,AlteryxServerMigrator,migrationLogger,MoveNext,Migrating records from AS_Applications.->,
- 2024-11-14 22:34:57.726011,INFO,1,AlteryxServerMigrator,migrationLogger,MoveNext,Renaming collection from 'AS_Results' to 'AS_ResultsMigrationInProcess_2'->,
- 2024-11-14 22:34:57.726011,INFO,1,AlteryxServerMigrator,migrationLogger,MoveNext,Renaming collection from 'AS_Applications' to 'AS_ApplicationsMigrationInProcess_2'->,

**Troubleshooting:**

- Wait for service failure to diagnose issues.
- Monitor MongoDB logs to track collection updates.

---

### Step 4: Alteryx Gallery Schema Migration

After completing the Service DB schema migration, the process moves to the **Gallery Schema Migration**.

**Logs Location:**

- Default: C:\ProgramData\Alteryx\Gallery
- Configured: D:\ProgramData\Alteryx\Gallery
- File Name: alteryx-migration.csv
- AlteryxGALLERY Schema Migration Log Messages (alteryx-migration.csv)

**Troubleshooting:**

- If the Gallery becomes inaccessible, check the logs for migration errors or stuck processes.

---

### Summary of Logs by Stage

|  |  |  |
| --- | --- | --- |
|  |  |  |
|  |  |  |
|  |  |  |
|  |  |  |
|  |  |  |
|  |  |  |

---

### Additional Notes

- MongoDB migration can vary in time based on database size.
- Monitor both Alteryx and MongoDB logs for real-time updates during migration.

For further assistance, contact the Alteryx support team or refer to the official documentation.