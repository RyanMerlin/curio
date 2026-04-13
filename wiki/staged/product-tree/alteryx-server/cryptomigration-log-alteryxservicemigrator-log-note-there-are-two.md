---
id: 4872a577921a82a4
title: 'CryptoMigration Log (AlteryxServiceMigrator_#.log) - Note: there are TWO'
status: staged
source:
  kind: confluence_page
  id: confluence-page:1640761815
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/1640761815
  summary: null
category:
- product-tree
- alteryx-server
keywords:
- migration
- time
- local
- universal
- coordinated
created_at: 2026-04-13T01:55:58Z
updated_at: 2026-04-13T01:56:05Z
confidence: 0.55
cross_refs: []
content_hash: sha256:c5ce7615fb186c4bb334bf7fcdcdd00f7a566c6eeb41483181c671273ef6d9e4
confluence_page_id: null
model_used: heuristic
---

---

- Errors (CryptoMigration Log)

---

> **ℹ️ Info**
>
> CryptoMigration logs record the MongoDB re-encryption process when upgrading to (or through) 2022.3

| Location | Logs appear in TWO locations AlteryxServiceMigrator_#.logs are found in Controller > General > Logging folder     <== Service start logs here]]>Logs rotate starting with # = 0.  Case 00596989 showed a rotation to “1” during main CryptoMigration, but subsequent Service starts are logging the “All tables already migrated; ending migration“ back in the “0” file.Controller node contains the main log of Collection CryptoMigrations.   Other nodes generate a log when Service first starts for CryptoMigration of RuntimeSettings.xml. |
| --- | --- |
| Customer Request | #E3FCEFPlease send your CryptoMigration logs named AlteryxServiceMigrator_#.log. These can appear in two locations.If you ran the Prep Tool, logs appear in C:\ProgramData\Alteryx\ServiceWhen starting the Service the logs will be created in Controller logging folder set in Alteryx System Settings > Controller > General > Logging |
| Troubleshooting | CryptoMigration in 22.3 Errors (CryptoMigration Log)  <== Log errors |
| Defects | TGAL-730577dcf2c9-72f3-3ff6-8e99-fe88e9f473f1System Jira   <== requesting that only one location be used for the log files |

---

---

# General Information

| Issue | Notes |
| --- | --- |
| Errors in the log | Look for lines containing error status code:;1;;2;;3;   <== most commonMigration failedSee Errors (CryptoMigration Log) |
| Timezone | Entries are in machine’s timezone described on the first line  migration at <2023-01-30 20:50:25 W. Europe Standard Time> local time]]> |
| Encoding of the file | Read with ANSI encoding (more important for Chinese and Japanese character sets) |
| Service logs report CryptoMigration failure | CryptoMigration Error reported in Service Log:3989: 2023-02-03 15:30:33.527000,ERROR,7348,AlteryxService,,,,,,,,,"AlteryxServiceMigrator22_2.exe_Error: Migration returned exit code <4>"3994: 2023-02-03 15:30:33.910000,ERROR,7348,AlteryxService,,,,,,,,,"AlteryxService_SvcReportEvent: App <AlteryxService> message <Migration did not complete successfully. Please see log for more details.>"3995: 2023-02-03 15:30:33.911000,ERROR,7348,AlteryxService,,,,,,,,,"AlteryxService_LogStartupError: There was an error starting the Alteryx Service <Migration did not complete successfully. Please see log for more details.>" |

# Log Status Codes

Each line contains a status code as the second field (fields demarcated with a semi-colon)

| Status Code | Meaning |
| --- | --- |
| ;1; | ErrorTGAL-1126877dcf2c9-72f3-3ff6-8e99-fe88e9f473f1System Jira  <== reclassified from ;4; to ;1; |
| ;2; | Error |
| ;3; | Error |
| ;4; | Nothing needed migrating for that object. Example:]]>Starting (Jun-24) to see Errors with the status ;4; as well:;4;Failed to acquire migration lock.E11000 duplicate key error collection: AlteryxService.AlteryxServiceMigrationLock index: lockId_1 dup key: { lockId: "unique_lock" }: generic server errorCryptoMigration Log Error - Failed to acquire migration lock.E11000 duplicate key error collection: AlteryxService.AlteryxServiceMigrationLock index: lockId_1 dup key: { lockId: "unique_lock" } GCSE-233877dcf2c9-72f3-3ff6-8e99-fe88e9f473f1System Jira  <== Posted this as a defect |
| ;5; | Status updates you can use to confirm the CryptoMigration is not stuck.  Example: of <9682>.
Line    66: 2023-02-03 11:11:16.624651;5;App migration is approximately 1% complete. Migrating app <32> of <9682>.
Line    73: 2023-02-03 11:11:51.209371;5;App migration is approximately 1% complete. Migrating app <35> of <9682>.]]>CryptoMigration previously completed.   Each time Service starts it will kick off the CryptoMigration, so this message should appear many times at the end of the file. |
| ;6; | Demarcates the Begin and End of the Migrator running in general and for each type of object being CryptoMigrated.  Indicates the timezone as well.  Example: migration at <2023-02-03 11:10:36 GMT Standard Time> local time

Line     2: 2023-02-03 11:10:36.537821;6;Begining  migration at <2023-02-03 11:10:36 GMT Standard Time> local time
Line 19467: 2023-02-03 12:20:29.972643;6;Ending  migration at <2023-02-03 12:20:29 GMT Standard Time> local time. Total duration was approximately <4193> seconds

Line 19468: 2023-02-03 12:20:29.972678;6;Begining  migration at <2023-02-03 12:20:29 GMT Standard Time> local time
Line 19469: 2023-02-03 12:20:30.051059;6;Ending  migration at <2023-02-03 12:20:30 GMT Standard Time> local time. Total duration was approximately <0> seconds

Line 19470: 2023-02-03 12:20:30.051108;6;Ending  migration at <2023-02-03 12:20:30 GMT Standard Time> local time. Total duration was approximately <4193> seconds]]>Starting (Jun-24) to see Errors with the status ;6; aas well:2024-05-16 13:13:59.280268;6;RunasCredential failed to fetch decrypt key using AES256 encryption. <Server Error: 500 Server Error Internal Error in DoDecrypt: Cannot get decrypt>. Attempting to fetch keys with sha256 encryption (for Version < 22.3).Cryptomigration Log Error - RunasCredential failed to fetch decrypt key using AES256 encryption. <Server Error: 500 Server Error Internal Error in DoDecrypt: Cannot get decrypt>. Attempting to fetch keys with sha256 encryption (for Version < 22.3). GCSE-233877dcf2c9-72f3-3ff6-8e99-fe88e9f473f1System Jira  <== Posted this as a defect |
| ;7; | Lists specific objects being migrated.  Example:
Line     7: 2023-02-03 11:10:41.134810;7;Migrated app id <55f177923519ac178db70b26> successfully]]> |

# Specific Log Entries

> **ℹ️ Info**
>
> The values ;1; through ;7; indicate the type of log entry, with ;3; being an error.

| Log entry | What it means |
| --- | --- |
|  | CryptoMigration previously completed.   Each time Service starts it will kick off the CryptoMigration, so this message should appear many times at the end of the file. |
|  | Prep Tool was run with no parameters or missing the -p flag. |
|  | Check the parameters used to start, they are failing to authenticate |
|  | CryptoMigration Log Error - Error importing keys to Microsoft\Crypto\RSA\MachineKeys\ directory in ProgramData: Bad Data. (-2146893819) . |
|  | When an older, short Controller Token is forcibly lengthened.  Fortunately the new token will match on the Controller and Workers, so they will continue to communicate after CryptoMigration. |
| to  and then  to  as part of migration finalization
;5;Renaming  to  and then  to  as part of migration finalization
;5;Renaming  to  and then  to  as part of migration finalization
;5;Renaming  to  and then  to  as part of migration finalization
;5;Renaming  to  and then  to  as part of migration finalization
;5;Renaming  to  and then  to  as part of migration finalization
;5;Renaming  to  and then  to  as part of migration finalization
;5;Renaming  to  and then  to  as part of migration finalization]]> | Flipping the re-encrypted collections with their pre-re-encryption versions at the very end of the process |
|  | CryptoMigration can’t access Mongo.CryptoMigration Log Error - No suitable servers found (`serverSelectionTryOnce` set):[connection refused calling ismaster on 'localhost:27018']: generic server error |

# Example Logs

2022.1 to 2022.3 Prep Tool Run

2023-02-23 03:10:47.641660;6;Begining <Migration to 22.3> migration at <2023-02-23 03:10:47 Coordinated Universal Time> local time
2023-02-23 03:10:47.642155;6;Begining <AppChunk migration> migration at <2023-02-23 03:10:47 Coordinated Universal Time> local time
2023-02-23 03:10:50.671188;5;App migration is approximately 0% complete. Migrating app <1> of <3>.
2023-02-23 03:10:51.917079;5;App migration is approximately 67% complete. Migrating app <3> of <3>.
2023-02-23 03:10:52.004390;6;Ending <AppChunk migration> migration at <2023-02-23 03:10:52 Coordinated Universal Time> local time. Total duration was approximately <4> seconds
2023-02-23 03:10:52.004761;6;Ending <Migration to 22.3> migration at <2023-02-23 03:10:52 Coordinated Universal Time> local time. Total duration was approximately <4> seconds
**** This log is missing mention of RuntimeSettings.xml updates despite the fact RuntimeSettings.xml was backed up and CryptoMigrated**

2023-02-23 03:35:56.936217;6;Begining <Migration to 22.3> migration at <2023-02-23 03:35:56 Coordinated Universal Time> local time
2023-02-23 03:35:56.936274;6;Begining <AppChunk migration> migration at <2023-02-23 03:35:56 Coordinated Universal Time> local time
2023-02-23 03:35:59.261651;4;Nothing to migrate for <AS_PackageDefinitions>
2023-02-23 03:35:59.262367;6;Ending <AppChunk migration> migration at <2023-02-23 03:35:59 Coordinated Universal Time> local time. Total duration was approximately <2> seconds
2023-02-23 03:35:59.262401;6;Begining <RunAs migration> migration at <2023-02-23 03:35:59 Coordinated Universal Time> local time
2023-02-23 03:35:59.540855;6;Begining <AS_RunAsCredentials> migration at <2023-02-23 03:35:59 Coordinated Universal Time> local time
2023-02-23 03:35:59.971678;5;AS_RunAsCredentials migration is approximately 0% complete. Migrating AS_RunAsCredentials <1> of <2>.
2023-02-23 03:35:59.971791;7;Migrating AS_RunAsCredentials id <63f6d746474f00006e000658>
2023-02-23 03:36:01.033719;7;Migrated AS_RunAsCredentials id <63f6d746474f00006e000658> successfully
2023-02-23 03:36:01.033775;5;AS_RunAsCredentials migration is approximately 50% complete. Migrating AS_RunAsCredentials <2> of <2>.
2023-02-23 03:36:01.033798;7;Migrating AS_RunAsCredentials id <63f6d7a2474f00006e00065f>
2023-02-23 03:36:01.042268;7;Migrated AS_RunAsCredentials id <63f6d7a2474f00006e00065f> successfully
2023-02-23 03:36:01.042326;6;Ending <AS_RunAsCredentials> migration at <2023-02-23 03:36:01 Coordinated Universal Time> local time. Total duration was approximately <1> seconds
2023-02-23 03:36:01.042368;6;Begining <AS_Queue> migration at <2023-02-23 03:36:01 Coordinated Universal Time> local time
2023-02-23 03:36:01.676346;5;AS_Queue migration is approximately 0% complete. Migrating AS_Queue <1> of <4>.
2023-02-23 03:36:01.676399;7;Migrating AS_Queue id <63f6d781474f00006e00065d>
2023-02-23 03:36:01.700057;7;Migrated AS_Queue id <63f6d781474f00006e00065d> successfully
2023-02-23 03:36:01.700118;7;Migrating AS_Queue id <63f6d7a5474f00006e000664>
2023-02-23 03:36:01.706665;7;Migrated AS_Queue id <63f6d7a5474f00006e000664> successfully
2023-02-23 03:36:01.706708;7;Migrating AS_Queue id <63f6d7bd474f00006e00066a>
2023-02-23 03:36:01.708725;7;Migrated AS_Queue id <63f6d7bd474f00006e00066a> successfully
2023-02-23 03:36:01.708763;5;AS_Queue migration is approximately 75% complete. Migrating AS_Queue <4> of <4>.
2023-02-23 03:36:01.708775;7;Migrating AS_Queue id <63f6d7d4474f00006e00066c>
2023-02-23 03:36:01.710669;7;Migrated AS_Queue id <63f6d7d4474f00006e00066c> successfully
2023-02-23 03:36:01.710723;6;Ending <AS_Queue> migration at <2023-02-23 03:36:01 Coordinated Universal Time> local time. Total duration was approximately <0> seconds
2023-02-23 03:36:01.710752;6;Begining <AS_Schedules> migration at <2023-02-23 03:36:01 Coordinated Universal Time> local time
2023-02-23 03:36:01.713742;4;Nothing to migrate for <AS_Schedules>
2023-02-23 03:36:01.713783;6;Ending <AS_Schedules> migration at <2023-02-23 03:36:01 Coordinated Universal Time> local time. Total duration was approximately <0> seconds
2023-02-23 03:36:01.714150;6;Ending <RunAs migration> migration at <2023-02-23 03:36:01 Coordinated Universal Time> local time. Total duration was approximately <2> seconds
2023-02-23 03:36:01.714168;7;All migrations successful
2023-02-23 03:36:01.714178;6;Begining <Migration finalization> migration at <2023-02-23 03:36:01 Coordinated Universal Time> local time
2023-02-23 03:36:01.982389;5;Renaming <AS_PackageDefinitions> to <Backup_AS_PackageDefinitions.Pre22.2> and then <AS_PackageDefinitions.22.3> to <AS_PackageDefinitions> as part of migration finalization
2023-02-23 03:36:01.985905;5;Renaming <AS_PackageDefinitions.Files> to <Backup_AS_PackageDefinitions.Pre22.2.Files> and then <AS_PackageDefinitions.22.3.Files> to <AS_PackageDefinitions.Files> as part of migration finalization
2023-02-23 03:36:02.019723;5;Renaming <AS_App_Chunks> to <Backup_AS_App_Chunks.Pre22.2> and then <AS_App_Chunks.22.3> to <AS_App_Chunks> as part of migration finalization
2023-02-23 03:36:02.022552;5;Renaming <AS_App_Chunks.Files> to <Backup_AS_App_Chunks.Pre22.2.Files> and then <AS_App_Chunks.22.3.Files> to <AS_App_Chunks.Files> as part of migration finalization
2023-02-23 03:36:02.027788;5;Renaming <AS_Queue> to <Backup_AS_Queue.Pre22.2> and then <AS_Queue.22.3> to <AS_Queue> as part of migration finalization
2023-02-23 03:36:02.030995;5;Renaming <AS_Queue.Files> to <Backup_AS_Queue.Pre22.2.Files> and then <AS_Queue.22.3.Files> to <AS_Queue.Files> as part of migration finalization
2023-02-23 03:36:02.036093;5;Renaming <AS_RunAsCredentials> to <Backup_AS_RunAsCredentials.Pre22.2> and then <AS_RunAsCredentials.22.3> to <AS_RunAsCredentials> as part of migration finalization
2023-02-23 03:36:02.039164;5;Renaming <AS_RunAsCredentials.Files> to <Backup_AS_RunAsCredentials.Pre22.2.Files> and then <AS_RunAsCredentials.22.3.Files> to <AS_RunAsCredentials.Files> as part of migration finalization
2023-02-23 03:36:02.045511;6;Ending <Migration finalization> migration at <2023-02-23 03:36:02 Coordinated Universal Time> local time. Total duration was approximately <0> seconds
2023-02-23 03:36:02.045542;6;Ending <Migration to 22.3> migration at <2023-02-23 03:36:02 Coordinated Universal Time> local time. Total duration was approximately <5> seconds
2021.3 to 2022.3 Service start (NO Prep Tool), note the Controller Token was lengthened from to the new 64-char standard

2023-02-06 00:55:13.178217;6;Begining <Migration to 22.3> migration at <2023-02-06 00:55:13 Coordinated Universal Time> local time
2023-02-06 00:55:13.178292;6;Begining <RuntimeSettings migration> migration at <2023-02-06 00:55:13 Coordinated Universal Time> local time
2023-02-06 00:55:13.179828;7;Loaded <D:\Alteryx\bin\RuntimeData\RuntimeSettings.xml> and <C:\ProgramData\Alteryx\RuntimeSettings.xml> for migration
2023-02-06 00:55:13.179893;6;Begining <RuntimeSettings> migration at <2023-02-06 00:55:13 Coordinated Universal Time> local time
2023-02-06 00:55:13.406169;7;Updating encryption for [Controller:ServerSecretEncrypted](#)
2023-02-06 00:55:13.406215;6;Skipping Controller:MongoDBPasswordEncrypted due to empty value
2023-02-06 00:55:13.406225;6;Skipping Controller:EmbeddedMongoDBPasswordEncrypted due to empty value
2023-02-06 00:55:13.406234;6;Skipping Controller:AdvancedMongoConnectionEncrypted due to empty value
2023-02-06 00:55:13.406245;6;Skipping Worker:ServerSecretEncrypted due to empty value
2023-02-06 00:55:13.406271;6;Skipping Gallery:MongoDBWebPasswordEncrypted due to empty value
2023-02-06 00:55:13.406281;6;Skipping Gallery:MongoDBSearchPasswordEncrypted due to empty value
2023-02-06 00:55:13.406292;6;Skipping Gallery:MongoDBSearchConnection due to empty value
2023-02-06 00:55:13.406325;6;Skipping Gallery:MongoDBWebConnection due to empty value
2023-02-06 00:55:13.406334;6;Skipping Gallery:SmtpPasswordEncrypted due to empty value
2023-02-06 00:55:13.406341;6;Skipping Worker:ExecutePasswordEncrypted due to empty value
**2023-02-06 00:55:13.418718;6;Generating a new controller token (Controller ServerSecretEncrypted) because existing controller token is too short.**
2023-02-06 00:55:13.419611;6;Empty field: (Worker ServerSecretEncrypted)
2023-02-06 00:55:13.628614;7;Updating [Controller:StorageKeysEncrypted](#)
2023-02-06 00:55:13.628656;7;Updating [Service:MigrationVersionNumber](#) to <1>
2023-02-06 00:55:13.628669;7;Writing updated settings to <C:\ProgramData\Alteryx\RuntimeSettings.22_2_migration.xml>
2023-02-06 00:55:13.629937;6;Ending <RuntimeSettings> migration at <2023-02-06 00:55:13 Coordinated Universal Time> local time. Total duration was approximately <0> seconds
2023-02-06 00:55:13.629968;6;Begining <Final RuntimeSettings> migration at <2023-02-06 00:55:13 Coordinated Universal Time> local time
2023-02-06 00:55:13.630250;5;Moving existing settings file <C:\ProgramData\Alteryx\RuntimeSettings.xml> to <C:\ProgramData\Alteryx\RuntimeSettings.22_2_legacy.xml>
2023-02-06 00:55:13.631122;5;Moving new settings file <C:\ProgramData\Alteryx\RuntimeSettings.22_2_migration.xml> to <C:\ProgramData\Alteryx\RuntimeSettings.xml>
2023-02-06 00:55:13.646861;6;Ending <Final RuntimeSettings> migration at <2023-02-06 00:55:13 Coordinated Universal Time> local time. Total duration was approximately <0> seconds
2023-02-06 00:55:13.646994;6;Ending <RuntimeSettings migration> migration at <2023-02-06 00:55:13 Coordinated Universal Time> local time. Total duration was approximately <0> seconds
2023-02-06 00:55:13.647003;7;All migrations successful
2023-02-06 00:55:13.647013;6;Ending <Migration to 22.3> migration at <2023-02-06 00:55:13 Coordinated Universal Time> local time. Total duration was approximately <0> seconds
2023-02-06 00:55:25.209693;6;Begining <Migration to 22.3> migration at <2023-02-06 00:55:25 Coordinated Universal Time> local time
2023-02-06 00:55:25.209769;6;Begining <AppChunk migration> migration at <2023-02-06 00:55:25 Coordinated Universal Time> local time
2023-02-06 00:55:27.799832;5;App migration is approximately 0% complete. Migrating app <1> of <3>.
2023-02-06 00:55:27.799886;7;Migrating app id <63e04a17085e00001b005a39>
2023-02-06 00:55:28.900392;7;Migrated app id <63e04a17085e00001b005a39> successfully
2023-02-06 00:55:28.901928;7;Migrating app id <63e04a49085e00001b005a40>
2023-02-06 00:55:28.938681;7;Migrated app id <63e04a49085e00001b005a40> successfully
2023-02-06 00:55:28.940081;5;App migration is approximately 67% complete. Migrating app <3> of <3>.
2023-02-06 00:55:28.940114;7;Migrating app id <63e04a67085e00001b005a45>
2023-02-06 00:55:28.969948;7;Migrated app id <63e04a67085e00001b005a45> successfully
2023-02-06 00:55:28.972057;6;Ending <AppChunk migration> migration at <2023-02-06 00:55:28 Coordinated Universal Time> local time. Total duration was approximately <3> seconds
2023-02-06 00:55:28.972101;6;Begining <RunAs migration> migration at <2023-02-06 00:55:28 Coordinated Universal Time> local time
2023-02-06 00:55:29.250018;6;Begining <AS_RunAsCredentials> migration at <2023-02-06 00:55:29 Coordinated Universal Time> local time
2023-02-06 00:55:29.630785;5;AS_RunAsCredentials migration is approximately 0% complete. Migrating AS_RunAsCredentials <1> of <2>.
2023-02-06 00:55:29.630837;7;Migrating AS_RunAsCredentials id <63e04a08085e00001b005a37>
2023-02-06 00:55:29.658831;7;Migrated AS_RunAsCredentials id <63e04a08085e00001b005a37> successfully
2023-02-06 00:55:29.658874;5;AS_RunAsCredentials migration is approximately 50% complete. Migrating AS_RunAsCredentials <2> of <2>.
2023-02-06 00:55:29.658886;7;Migrating AS_RunAsCredentials id <63e04a46085e00001b005a3e>
2023-02-06 00:55:29.664534;7;Migrated AS_RunAsCredentials id <63e04a46085e00001b005a3e> successfully
2023-02-06 00:55:29.664581;6;Ending <AS_RunAsCredentials> migration at <2023-02-06 00:55:29 Coordinated Universal Time> local time. Total duration was approximately <0> seconds
2023-02-06 00:55:29.664622;6;Begining <AS_Queue> migration at <2023-02-06 00:55:29 Coordinated Universal Time> local time
2023-02-06 00:55:30.117032;5;AS_Queue migration is approximately 0% complete. Migrating AS_Queue <1> of <3>.
2023-02-06 00:55:30.117082;7;Migrating AS_Queue id <63e04a17085e00001b005a3c>
2023-02-06 00:55:30.141128;7;Migrated AS_Queue id <63e04a17085e00001b005a3c> successfully
2023-02-06 00:55:30.141181;7;Migrating AS_Queue id <63e04a49085e00001b005a43>
2023-02-06 00:55:30.146966;7;Migrated AS_Queue id <63e04a49085e00001b005a43> successfully
2023-02-06 00:55:30.147027;5;AS_Queue migration is approximately 67% complete. Migrating AS_Queue <3> of <3>.
2023-02-06 00:55:30.147043;7;Migrating AS_Queue id <63e04a7f085e00001b005a46>
2023-02-06 00:55:30.153020;7;Migrated AS_Queue id <63e04a7f085e00001b005a46> successfully
2023-02-06 00:55:30.153085;6;Ending <AS_Queue> migration at <2023-02-06 00:55:30 Coordinated Universal Time> local time. Total duration was approximately <0> seconds
2023-02-06 00:55:30.153115;6;Begining <AS_Schedules> migration at <2023-02-06 00:55:30 Coordinated Universal Time> local time
2023-02-06 00:55:30.154120;4;Nothing to migrate for <AS_Schedules>
2023-02-06 00:55:30.154168;6;Ending <AS_Schedules> migration at <2023-02-06 00:55:30 Coordinated Universal Time> local time. Total duration was approximately <0> seconds
2023-02-06 00:55:30.154556;6;Ending <RunAs migration> migration at <2023-02-06 00:55:30 Coordinated Universal Time> local time. Total duration was approximately <1> seconds
2023-02-06 00:55:30.154582;7;All migrations successful
2023-02-06 00:55:30.154593;6;Begining <Migration finalization> migration at <2023-02-06 00:55:30 Coordinated Universal Time> local time
2023-02-06 00:55:30.419238;5;Renaming <AS_PackageDefinitions> to <Backup_AS_PackageDefinitions.Pre22.2> and then <AS_PackageDefinitions.22.3> to <AS_PackageDefinitions> as part of migration finalization
2023-02-06 00:55:30.422440;5;Renaming <AS_PackageDefinitions.Files> to <Backup_AS_PackageDefinitions.Pre22.2.Files> and then <AS_PackageDefinitions.22.3.Files> to <AS_PackageDefinitions.Files> as part of migration finalization
2023-02-06 00:55:30.449201;5;Renaming <AS_App_Chunks> to <Backup_AS_App_Chunks.Pre22.2> and then <AS_App_Chunks.22.3> to <AS_App_Chunks> as part of migration finalization
2023-02-06 00:55:30.452224;5;Renaming <AS_App_Chunks.Files> to <Backup_AS_App_Chunks.Pre22.2.Files> and then <AS_App_Chunks.22.3.Files> to <AS_App_Chunks.Files> as part of migration finalization
2023-02-06 00:55:30.457507;5;Renaming <AS_Queue> to <Backup_AS_Queue.Pre22.2> and then <AS_Queue.22.3> to <AS_Queue> as part of migration finalization
2023-02-06 00:55:30.460920;5;Renaming <AS_Queue.Files> to <Backup_AS_Queue.Pre22.2.Files> and then <AS_Queue.22.3.Files> to <AS_Queue.Files> as part of migration finalization
2023-02-06 00:55:30.466171;5;Renaming <AS_RunAsCredentials> to <Backup_AS_RunAsCredentials.Pre22.2> and then <AS_RunAsCredentials.22.3> to <AS_RunAsCredentials> as part of migration finalization
2023-02-06 00:55:30.469294;5;Renaming <AS_RunAsCredentials.Files> to <Backup_AS_RunAsCredentials.Pre22.2.Files> and then <AS_RunAsCredentials.22.3.Files> to <AS_RunAsCredentials.Files> as part of migration finalization
2023-02-06 00:55:30.475722;6;Ending <Migration finalization> migration at <2023-02-06 00:55:30 Coordinated Universal Time> local time. Total duration was approximately <0> seconds
2023-02-06 00:55:30.475759;6;Ending <Migration to 22.3> migration at <2023-02-06 00:55:30 Coordinated Universal Time> local time. Total duration was approximately <5> seconds