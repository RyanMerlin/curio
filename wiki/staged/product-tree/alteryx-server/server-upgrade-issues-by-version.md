---
id: 76ab286cdda87a35
title: Server Upgrade Issues-by-Version
status: staged
source:
  kind: confluence_page
  id: confluence-page:2650999118
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/2650999118
  summary: null
category:
- product-tree
- alteryx-server
keywords:
- error
- upgrade
- version
- jira
- migration
created_at: 2026-04-13T01:55:58Z
updated_at: 2026-04-13T01:56:18Z
confidence: 0.55
cross_refs: []
content_hash: sha256:16b345ab03f2b34295f59c027cb50c3a86ffe34398393f090c2d9b338f07b820
confluence_page_id: null
model_used: heuristic
---

> **ℹ️ Info**
>
> This is a high-level list of commonly experienced upgrade issues by version linking to the relevant articles with resolutions

---

---

|  | Issue |
| --- | --- |
|  |  |
| 25.1 |  |
|  |  |
| 25.1 | activeRed PreventGreen (25.1) Unhandled Exception when Starting Designer |
| Prevent | If you have Copilot installed, you need to uninstall or install the latest version BEFORE upgrading to 25.1 https://help.alteryx.com/aac/en/alteryx-copilot.html#uninstall-alteryx-copilot > Uninstallhttps://marketplace.alteryx.com/en-US/apps/476096/alteryx-copilot  <== latest version |
| Jira | https://alteryx.atlassian.net/issues/TDES-14553?jql=textfields%20~%20%22Spike%3A%20Unhandled%20Exception%20occurred%20popup%20dialog%20appears%20after%20upgrading%20to%2025.1%20from%20a%20previous%20version%20with%20Copilot%20installed%22 |
| Release Notes | Release notes call out this issuehttps://help.alteryx.com/release-notes/en/release-notes/designer-release-notes/designer-2025-1-release-notes.html#copilot-designer-compatibility |
| Confluence | General Error - An Unhandled Exception occurred > Upgrade to 25.1? Copilot |
|  |  |
| 25.1 | activeRed (25.1-tbd) Error publishing with a credential - Invalid username or password. |
| Issue | Publishing to 25.1 from older Desogner will get the error below |
| Confluence | File > Save Error - Invalid username or password |
| Jira | GCSE-358077dcf2c9-72f3-3ff6-8e99-fe88e9f473f1System Jira |
|  |  |
|  |  |
| 24.2Mongo 7.0 upgradeNew AMP Recommendations (which can be applied to old versions) |  |
|  |  |
| 24.2 | activeRed PreventGreen (24.2-tbd Mongo 7.0) Embedded MongoDB upgrade no longer backs up the data for rollback |
| Prevent | Ensure customer snapshots Server with the Service stopped. |
| Issue | You can no longer rely on the Mongo_Backup folder to use in a rollback since it’s not created,  The Mongo_PreUpgrade folder is used to upgrade the database to 7.0 during the backup process, so it can’t be used in a rollback. |
| Confluence | MongoDB Upgrade Folder Structure |
|  |  |
| 24.2 | activeRed PreventGreen (24.2) Alteryx License Server 24.2 Admin Command Error “Server responded with a 401“ |
| Prevent | Be aware of the issue and use the post-upgrade workaround to reset the password |
| Issue | After upgrading Alteryx License Server to version 2024.2 reverts to default password even though Admin password is specified in installation wizard |
| Error | When running an Admin command using the Admin password, it fails with the error:Server responded with a 401 and an error message of:key='glsErr.userAuthFailed'message='Authorization attempt at uri=/api/1.0/instances/~/authorize failed for user admin (error BadCredentialsException(Bad credentials))'arguments=[uri=/api/1.0/instances/~/authorize, admin, BadCredentialsException, Bad credentials] |
| Cause | Defect |
| Jira | TCPE-149877dcf2c9-72f3-3ff6-8e99-fe88e9f473f1System Jira |
| Resolution | See Workaround on the defect |
| Versions | 24.2 after upgrade |
|  |  |
| 24.2 | activeRed PreventGreen (24.2 - tbd) MongoDB won’t upgrade to 7.0 because it thinks it’s on 4.2 |
| Prevent | (1) Ensure the planned upgrade includes only one MongoDB version upgrade, seeServer Upgrade Version Paths - What version can upgrade to what versions? (2) If starting from a Server version using Mongo6.0, confirm ASMongoDBVersion.bin in the persistence folder contains the correct value for Mongo 6.0:  “6.0.5” |
| Issue | The step to upgrade the Embeded MongoDB to 7.0 failsYou are upgrading from a version of Server that utilizes MongoDB version older than 6.0. You need to upgrade to Server version 2023.2 before moving forward. |
| Cause | OPTION 1 – The user is attempting to upgrade from a version of Server that uses Mongo 4.XOPTION 2 – The file that indicates the current MongoDB version, ASMongoDBVersion.bin, is inaccurately showing version 4.x |
| Resolution | OPTION 1 – Rollback and upgrade Server one Embeddd MongoDB upgrade at a time.  First to a version with Mongo 6.0, then a version with Mongo 7.0.OPTION 2 – Edit ASMongoDBVersion.bin to contain simply. “6.0.5” so the upgrade software will see the Persistence folder as Mongo 6.0.  Run a manual Mongo upgrade to 7.0 and restart the Server. |
| Confluence | Mongo Database Upgrade Error - You are upgrading from a version of Server that utilizes MongoDB version older than 6.0 |
| Versions | 24.2 - tbdAffects the Embedded Mongo upgrade to Mongo7.0.  Similar issue occurs with different error messages fro each Embedded Mongo DB update. |
|  |  |
| 24.2 | activeRed (24.2 - tbd) Data loss during MongoDB Version Upgrade |
| Issue | The MongoDB Upgrade process does not report some errors that can result in silent data loss during MongoDB version upgrade |
| Logs | You can review the MongoDB version upgrade log in the folder created next to the Persistence folder during upgrade:\xxx_PreUpgrade\migration.logUnfortunately, the log includes several errors that look bad but are expected.  The workflow provided in the Confluence link will parse the migration.log and ignore the expected errors. |
| Versions | Upgrades to 24.2 that include a MongoDB version upgrade |
| Confluence | migration.log (embedded Mongo version upgrade)  <== diagnostic workflow to filter out expected errors in migration.logCSU Tech Talk 25 |
| Jira | GCSE-322477dcf2c9-72f3-3ff6-8e99-fe88e9f473f1System Jira |
|  |  |
| 24.2 | 24.2.1.1.41 _Patch_1Blue Gallery stops responding/fails as CPU utilization builds slowly over time |
| Issue | After upgrading to 24.2.1.14, Gallery encounters intermittent outage due to excessive CPU load which increases slowly over time until Gallery failure |
| Logs | Gallery Logs repeat2024-11-09 00:37:07.237346,INFO,39,AlteryxServerHost,ActionTrigger<IScheduleOperations>,Work,,,,CST-DEMO-GAL-1,,,,,,"Performing Alteryx.Server.Models.Operations.ScheduleOperations, self-triggered by timeout.",Gallery Log Info - Performing Alteryx.Server.Models.Operations.ScheduleOperations, self-triggered by timeout. |
| Cause | At minimum, a moderate usage of the Server API and API tokens generated will compound the number of processes created leading to an overload of CPU utilization |
| Workaround | From Jira cardIf increased resource usage is gradual enough, scheduling a periodic restart of the AlteryxService on any gallery nodesIf happening frequently:Minimize Server API use in workflowsTemporarily disable Server API use by setting it to a different portRollback |
| Jira | TGAL-1213877dcf2c9-72f3-3ff6-8e99-fe88e9f473f1System Jira [25.1-LTS, 24.2.1_Patch1] |
|  |  |
|  |  |
| 24.1Timestamps moved to UTCPython upgrade to 3.10appInfos.PublishedRevision merged into Revisions array |  |
|  |  |
| 24.1 | Action Req'dYellow Python version upgrade requires update of all Connectors |
| Issue | Python version upgrade requiring update of all Connectors and workflows using Python |
| Errors | tbd |
| Help | Python Version Upgrades |
|  |  |
| 24.1 | activeRed PreventGreen (24.1 - tbd) Upgrade from 21.4 early patches (LTS-p4) is unstable |
| Prevent | Patch to the latest 21.4 before upgrading |
| Issue | Upgrade will hit a Gallery Schema Migration error on migration 34.03:2024-08-28 17:10:31.281899,FATAL,1,AlteryxServerWebApiHost,migrationLogger,DoMigrateDatabase,"Migration failed with error: An error occurred while deserializing the Users property of class Alteryx.Server.Models.BaseModels.DataConnection: Expected a nested document representing the serialized form of a Alteryx.Server.Models.BaseModels.DataConnectionUser value, but found a value of type String instead."," |
| Versions | 24.1+ |
| Jira | TGAL-1176777dcf2c9-72f3-3ff6-8e99-fe88e9f473f1System Jira |
|  |  |
| 24.1 | activeRed PreventGreen (24.1 - tbd) Manual Run Counts show lower after upgrade |
| Prevent | The issue can’t be prevented, but it can be addressed immediately after upgrade (BEFORE workflows are manually run as that loses the original run count).Upgrade to experience the issueRun the workflow attached to the Jira ticketRun the Mongo update queries generated by the workflow |
| Issue | Schema migration for 24.1 is setting the most recent revision’s manual run count to 0.  This leads the total manual run count to appear lower after upgrade.  Note: scheduled and API job runs are not included in this count. |
| Versions | 24.1, 24.2 |
| Jira | TGAL-1240977dcf2c9-72f3-3ff6-8e99-fe88e9f473f1System Jira[not a problem in 25.1 upgrade] |
|  |  |
| 24.1 | activeRed PreventGreen (24.1 - tbd) Run Mode reverts to Safe |
| Prevent | For Customers who have:Server default is Safe or Semi-SafeAdmins manually set workflow settings to Semi-Safe or UnrestrictedMongo query can be developed to set the appInfos.PublishedRevsion.ExecutionMode to the value of Revisions[0].ExecutionMode BEFORE upgrade to avoid the workflows reverting to Safe mode.  Note: not sure how to handle the “published” version not being the latest version. |
| Issue | If the Server default of Run Mode = Safe or Semi-Safe and admins indiivaully set Workflows to Semi-Safe or Unrestricted after the workflow is published will experience workflows reverting to Safe mode after upgrade to/through 24.1 |
| Versions | Upgrade to/through 24.1 where the schema migration merges the appInfos.PublishedRevision into the Revisions array. |
| Jira | GCSE-340177dcf2c9-72f3-3ff6-8e99-fe88e9f473f1System Jira |
|  |  |
| 24.1 | activeRed PreventGreen (24.1 - 24.2) Service Schema Migration 3 fails – Year number is out of range 1400..9999: '0001-01-01T00:00:00' |
| Prevent | Run the queries listed in the Jira ticket |
| Issue | Dates in a variety of collections have the value 0001-01-01T00:00:00, which confuses the AlteryxService Schema Migration as it converts all times from Server timezone to UTC. |
| Errors | Service Schema Migration (alteryx-migration.csv)2024-09-19 03:49:43.680292,INFO,1,AlteryxServerMigrator,migrationLogger,MoveNext,Starting Service Migration: 3,2024-09-19 03:49:44.391287,FATAL,1,AlteryxServerMigrator,migrationLogger,DoMigrateDatabase,Migration failed with error: Year number is out of range 1400..9999: '0001-01-01T00:00:00',Service Log2024-09-19 03:49:43.680292,ERROR,7884,AlteryxService,,,,,,,,,"S:\Alteryx\Service\AlteryxService\src\AlteryxServiceManager.cpp: 1156. Server Migrator failed with exit code <3762504530>. See Migrator Logs for detailed error message. This error must be addressed in order for the service to start." |
| Versions | Not expressly tested in 24.2 but expecting it will be an issue since upgrades that go through 24.1 will include the service Schema 3 upgrade |
| Resolution | Service Schema Migration Log Fatal - Migration failed with error: Year number is out of range 1400..9999: '0001-01-01T00:00:00' |
|  |  |
| 24.1 | activeRed PreventGreen (24.1) Custom site colors are removed |
| Prevent | See defect workaround to reapply the colors after upgrade |
| Issue | Custom site colors are lost in upgrade |
| Versions | Affects upgrade to 24.1 |
| Jira | TPRI-634877dcf2c9-72f3-3ff6-8e99-fe88e9f473f1System Jira [24.2-LTS] |
|  |  |
| 24.1 | 24.1.1_P7Blue (24.1) Schedule times off by # of hours user is off from UTC after upgrade to 24.1 |
| Issue | The page listing Schedules will show the wrong Schedule Frequency time after upgrade even though Schedules run as expected and Last/Next times appear correctly.The issue is that the Server is thinking the Frequency time in the DB is in UTC and adjusting it by how many hours they are from UTC when displaying.  But the Frequency is still stored in the original Server timezone.  So if Frequency was “Daily at 10:00pm” the 10p in the Frequency field will be interpreted as UTC and the shown as “6:00p” to the user for a Server machine set to EDT.Editing the Schedule will not correct the issue.  New Schedules appear with correct Frequency. |
| Versions | Affects24.1 (experienced with 24.1_P6 but may occur in earlier patches) |
| Notes | Applying 24.1_P7 after the problem has been experienced in an upgrade to 24.1 does NOT update the Frequency display field in the DB, so Schedule page still shows the wrong time.  Users must edit the Schedule and re-se to update the Frequency field display. |
| Jira | TGAL-1186977dcf2c9-72f3-3ff6-8e99-fe88e9f473f1System Jira [24.1.1_Patch7] |
|  |  |
| 24.1 | 24.2.1_Patch2 , 24.1.1_Patch7Blue (24.1 - 24.2) Gallery Schema migration error - 40.02 - An error occurred while deserializing the CustomCss property |
| Log | Galley Schema Migration alteryx-migration.csv2024-11-19 09:16:53.433478,INFO,1,AlteryxServerHost,migrationLogger,MoveNext,Starting Migration: 40.02,2024-11-19 09:16:53.471474,FATAL,1,AlteryxServerHost,migrationLogger,DoMigrateDatabase,"Migration failed with error: An error occurred while deserializing the CustomCss property of class Alteryx.Server.Models.BaseModels.Configuration: An error occurred while deserializing the Sections property of class Alteryx.Server.Models.BaseModels.CustomCss: An error occurred while deserializing the Styles property of class Alteryx.Server.Models.BaseModels.CssSection: An error occurred while deserializing the Properties property of class Alteryx.Server.Models.BaseModels.CssSelector: An error occurred while deserializing the Names property of class Alteryx.Server.Models.BaseModels.CssProperty: Expected a nested document representing the serialized form of a Alteryx.Server.Models.BaseModels.CssProperty+CssProperyName value, but found a value of type String instead." |
| Versions | Affects24.2-LTS24.1.1_Patch4 |
| JIra | TGAL-1212877dcf2c9-72f3-3ff6-8e99-fe88e9f473f1System JiraFixed25.1-LTS24.2.1_Patch224.1.1_Patch7 |
|  |  |
| 24.1 | 24.1_Patch_5, 24.2_Patch_1Blue (24.1 - 24.2) Gallery Schema migration error - 40.01 - An error occurred while deserializing the CustomCss property |
| Log | Galley Schema Migration alteryx-migration.csv |
| Versions | 24.1, 24.2 prior to patches |
| JIra | TGAL-1189177dcf2c9-72f3-3ff6-8e99-fe88e9f473f1System Jira [24.1.1_Patch5, 24.2.1_Patch1, 25.1-LTS] |
|  |  |
| 24.1 | 24.1.1._patch_4Blue Gallery Schema migration error - Migration 49 failed for appInfos |
| Gallery Schema Migration Log | {timestamp},FATAL,1,AlteryxServerHost,migrationLogger,DoMigrateDatabase,Migration failed with error: Migration to version 49 failed: Renaming collection from 'appInfos' to 'appInfosMigrationInProcess_49'->Done renaming collection->Mirgation 49 failed for appInfos. Exception: Element 'Messages' not found.->Aborting migration->Removing collection: appInfosMigrationInProcess_49->Done Aborting migration->,{timestamp},FATAL,1,AlteryxServerHost,migrationLogger,DoMigrateDatabase,Migration failed with error: Migration to version 49 failed: Renaming collection from 'appInfos' to 'appInfosMigrationInProcess_49'->Done renaming collection->Mirgation 49 failed for appInfos. Exception: Element 'DatasetMessages' not found.->Aborting migration->Removing collection: appInfosMigrationInProcess_49->Done Aborting migration->,{timestamp},FATAL,1,AlteryxServerWebApiHost,migrationLogger,DoMigrateDatabase,Migration failed with error: Migration to version 49 failed: Renaming collection from 'appInfos' to 'appInfosMigrationInProcess_49'->Done renaming collection->Mirgation 49 failed for appInfos. Exception: Unable to cast object of type 'MongoDB.Bson.BsonNull' to type 'MongoDB.Bson.BsonString'.->Aborting migration->Removing collection: appInfosMigrationInProcess_49->Done Aborting migration->,This error occurs when the Messages, DatasetMessages, or TagIds arrays affected by schema migration 49 contain null values. |
| Jira | TGAL-1145877dcf2c9-72f3-3ff6-8e99-fe88e9f473f1System Jira |
|  |  |
| 24.1 | 24.2-LTS, 24.1.1_Patch4Blue (24.1) Schema Migration 35 Failing with error Migration failed with error: An error occurred while deserializing the CustomCss |
| Issue | Gallery Schema Migration 35 for fails upgrading to 24.1 |
| Gallery Log | {timestamp},FATAL,1,AlteryxServerHost,migrationLogger,DoMigrateDatabase,"Migration failed with error: An error occurred while deserializing the CustomCss property of class Alteryx.Server.Models.BaseModels.Configuration: An error occurred while deserializing the Sections property of class Alteryx.Server.Models.BaseModels.CustomCss: An error occurred while deserializing the Styles property of class Alteryx.Server.Models.BaseModels.CssSection: An error occurred while deserializing the Properties property of class Alteryx.Server.Models.BaseModels.CssSelector: An error occurred while deserializing the Names property of class Alteryx.Server.Models.BaseModels.CssProperty: Expected a nested document representing the serialized form of a Alteryx.Server.Models.BaseModels.CssProperty+CssProperyName value, but found a value of type String instead.", |
| Versions | Jira states this affects versions 24.1-LTS and 24.2-LTS but the 24.1 fix is patch4 and the schema migration is for 22.1 |
| Jira | TGAL-1145977dcf2c9-72f3-3ff6-8e99-fe88e9f473f1System Jira [24.2-LTS, 24.1.1_Patch4] |
|  |  |
|  |  |
| 23.2Mongo 6.0 upgrade__ServiceData blob removedWhy was the __ServiceData field removed and how can I retrieve data contained in the __ServiceData blob? (KB)A few BLOBs remain using a different field an,e  theycan still be unpacked with ServiceDataParser.yxmc, Mongo Input Tool |  |
|  |  |
| 23.2 | activeRed preventGreen Mongo upgrade 6.0 - Unexpected '4.0.10' in ASMongoDBVersion.bin |
| Prevent | Both checks are required as we see that ASMongoDBVersion.bin will hold on to an old setting (4.0.10) regardless of the Alteryx Server versionConfirm we ARE upgrading from a 4.2.X version of Mongo based on the Server version   ASMongoDBVersion.bin    ASMongoDBVersion.bin Confirm the content of ASMongoDBVersion.bin is correctly set to either    4.2.15    4.2.22 |
| Error | Occurs in last step of the upgrade installation, before attempting to start the ServiceAlteryx Server Database MigrationCould not start previous version of MongoDB: The MongoDB database failed to start with exit code: 14. |
| Cause | Mongo upgrade to 6.0 fails due to the wrong version (“4.0.10”) appearing in ASMongoDBVersion.bin file in Persistence Folder |
| KB | Alteryx Server Upgrade "Error: Could not start previous version of MongoDB: The MongoDB database failed to start with exit code: 14." (KB) |
|  |  |
| 23.2 | activeRed Mongo upgrade 6.0 - Missing ASMongoDBVersion.bin |
| Error | Occurs in last step of the upgrade installation, before attempting to start the ServiceAlteryx Server Database MigrationCould not finalize MongoDB restore. The MongoDB database failed to start with exit code: 100. |
| Cause | Mongo upgrade to 6.0 fails due to missing ASMongoDBVersion.bin file in Persistence Folder.  The file was in the Persistence folder before upgrade begins but the upgrade is misplacing the file. |
| Resolution | Recreate missing ASMongoDBVersion.binCould not finalize mongodb restore the mongodb database failed to start with exit code 100 (KB) |
| Jira | GCSE-285577dcf2c9-72f3-3ff6-8e99-fe88e9f473f1System Jira |
|  |  |
| 23.2 | activeRed (23.2 - 24.1) Service Schema Migration 2 fails – AS_Queue index |
| Issue | Service Schema Migration 2 fails |
| Logs | Service Scheme Migration – alteryx-migration.csv from Service log directory2024-09-03 15:17:51.478226,INFO,1,AlteryxServerMigrator,migrationLogger,MoveNext,Starting Service Migration: 2,2024-09-03 15:17:55.477498,FATAL,1,AlteryxServerMigrator,migrationLogger,DoMigrateDatabase,Migration failed with error: One or more errors occurred.,Service Log2024-09-03 15:17:57.935000,ERROR,7848,AlteryxService,,,,,,,,,"S:\Alteryx\Service\AlteryxService_Client\src\RunAlteryxServerMigrator.cpp: 135. Server Migrator failed with exit code <3762504530>. See Migrator Logs for detailed error message. This error must be addressed in order for the service to start." |
| Versions | 23.2, 24.1 |
| Jira | TGAL-1180077dcf2c9-72f3-3ff6-8e99-fe88e9f473f1System Jira [24.2] |
|  |  |
| 23.2 | activeRed  (23.2+) Version numbers of existing workflows all display as 1 |
| Issue |  |
| KB | Alteryx Workflow Version History Discrepancy with Alteryx Server 2023.2 (KB)  <== workaround |
| Confluence | Issues (User > My Workspace)  <== generic cross-link, no additional information |
| Jira | TCPE-110077dcf2c9-72f3-3ff6-8e99-fe88e9f473f1System Jira[Cancelled]Jira was Cancelled, not Done, due to a script that can be run in Mongo to fix the issue after it occurs, per commentsOnly affects workflows that existed before patch 22.1.1.42691, per Jira |
|  |  |
| 23.2 | activeRed (23.2 - 24.1) Data loss during MongoDB Version Upgrade |
| Issue | The MongoDB Upgrade process does not report some errors that can result in silent data loss during MongoDB version upgrade |
| Logs | You can review the MongoDB version upgrade log in the folder created next to the Persistence folder during upgrade:\xxx_PreUpgrade\migration.logUnfortunately, the log includes several errors that look bad but are expected.  The workflow provided in the Confluence link will parse the migration.log and ignore the expected errors. |
| Versions | Upgrades to 23.2 and 24.1 that include a MongoDB version upgrade. |
| Confluence | migration.log (embedded Mongo version upgrade)  <== diagnostic workflow to filter out expected errors in migration.logCSU Tech Talk 25 |
| Jira | GCSE-322477dcf2c9-72f3-3ff6-8e99-fe88e9f473f1System Jira |
|  |  |
| 23.2 | not doneRed  (23.2) Revision numbers of existing workflows all displaying as 1 after upgrade to 2023.2 |
| Workaround | Script in Jira |
| Jira | TCPE-110077dcf2c9-72f3-3ff6-8e99-fe88e9f473f1System Jira |
|  |  |
|  |  |
| 23.1JS Framework ReplacedLucene Replaced |  |
|  |  |
| 23.1 | activeRed preventGreen (23.1-23.2) Upgrade failure due to missing AS_Versions collection |
| Prevent | When upgrading from 22.3 some customers will be missing AlteryxService.AS_Versions.  You can check for this collection and create it as described in the Confluence page below if it’s missing. |
| Issue | When upgrading from a NEW 22.3, 23.1, 23.2 installs (already CryptoMigrated) some customers will be missing AlteryxService.AS_Versions (which is the indicator to the upgrader that the data is CryptoMigrated).  This leads the upgrade to beleive that CryptoMigration needs to be performed.  However, the 22.3+ database and RuntimeSettings.xml are already CryptoMigrated and will error towards the end of the attempt to CryptoMigrate them again.This issue applies to upgrades of NEW installs of 22.3 (Mongo 4.2) to 23.1 (Mongo 4.2), 23.2 (Mongo 6.0), 24.1 (Mongo 6.0)23.1 (Mongo 4.2) to 23.2 (Mongo 6.0), 24.1 (Mongo 6.0)23.2 (Mongo 6.0) to 24.1 (Mongo 6.0), 24.2 (Mongo 7.0), any version that uses Mongo 7.0 |
| Patching before upgrade | User needs to upgrade their version to the Patch before upgrading to new version, so this works differently than most patches where upgrading TO the Patch version corrects the issue.  Ex: upgrade 23.1_LTS to 23.1.392_Patch_7 before upgrading to 23.2 to prevent this issue23.1.392_Patch_7  <== upgrade to this patch (or later) to percent the error when upgrading to a later version23.2.173_Patch_4  <== upgrade to this patch (or later) to percent the error when upgrading to a later version24.1_LTS                <== issue no longer occurs for users start with a NEW install of 24.2 |
| Service Log | Service LogTBD |
| CryptoMigration Log | Cryptomigration – AlteryxServiceMigrator_#.log2024-06-08 14:37:41.902258;3;Error during key initialization. <Error importing keys to Microsoft\Crypto\RSA\MachineKeys\ directory in ProgramData: Bad Version of provider. (-2146893817)>CryptoMigration Log Error - Error during key initialization. <Error importing keys to Microsoft\Crypto\RSA\MachineKeys\ directory in ProgramData: Bad Version of provider. (-2146893817)> |
| Additional possible errors | From the DefectThe error that appears in in AlteryxServiceMigrator_X.log appears to be dependent on what the pre-upgrade version was. When upgrading from 2023.1.1.361 to 2023.2.1.173, the error at appeared is:2024-05-17 19:47:28.654117;3;Error during key initialization. <Error importing keys to Microsoft\Crypto\RSA\MachineKeys\ directory in ProgramData: Bad Version of provider. (-2146893817)>Examples of upgrades and error message received:Got signal 222023.2.1.89 > 2024.1.1.17Bad Version of provider2022.3.1.597 > 2023.1.1.3922022.3.1.597 > 2024.1.1.172023.1.1.361 > 2023.2.1.173LastStartupError.txt and AlteryxServiceMigrator_X.log files from following replication tests attached:2023.2.1.89 > 2024.1.1.172022.3.1.597 > 2024.1.1.17 |
| Verions | 23.1, 23.2 |
| Jira | TGAL-1118577dcf2c9-72f3-3ff6-8e99-fe88e9f473f1System Jira |
| Release Notes | https://help.alteryx.com/release-notes/en/release-notes/server-release-notes/server-2024-1-release-notes.html##:~:text=Known-,TGAL%2D11185,-GCSE%2D2288 |
|  |  |
| 23.1 | activeRed Lucene indexing replaced |
| Issue | Indexing errors due to the indexing system being replaced cause failures after Service starts or the appearance of missing users or other dataReindex MongoDB |
|  |  |
| 23.1 | activeRed Embedded R upgrade |
| Issue | Embedded R version upgrades from 4.1.3 to 4.2.3 in RInstaller_2023.1.1.200.exe+ requiring customers to update R codeR Tool + |
|  |  |
| 23.1 | patchBlue JS Framework replaced |
| Issue | UI Framework replaced leading to numerous Analytic App interface defects Issues (Analytic Apps) These were corrected in 23.1 patches so won’t apply in most recent patch version or upgrade |
|  |  |
| 22.3CryptoMigration |  |
|  |  |
| 22.3 | activeRed preventGreen CryptoMigration |
| Prevent | The CryptoMigration Prep Tool Pre Flight checks can identify some Cryptomigration issues before attempting the upgradehttps://help.alteryx.com/20223/en/server/install/install-or-upgrade-server/migration-prep-tool/run-the-migration-prep-tool.html The PreFlight checks REQUIRE a 64-bit Controller token or it will error with:CryptoMigration Log Pre Flight Error - RunasCredential failed while Decrypting storage key. <Server Error: 500 Server Error The Authentication Key is not valid, client <xxx.xxx.xxx.xx>> |
| Issue | CryptoMigration to AES256 standard generated a large number of errors |
| Error | Numerous error can occur when running the CryptoMigration Prep Tool, see Confluence page below. |
| Confluence | CryptoMigration in 22.3 Errors (CryptoMigration Log) |
| Help | https://help.alteryx.com/current/en/server/install/install-or-upgrade-server/migration-prep-tool.html |
|  |  |
| 22.3 | patchBlue  SAML Okta Login Leads to Please Sign In page |
| Error | Please Sign InThe page you are trying to reach requires you to sign in before you can view it. |
| KB | Exception thrown when asserting SAML response from IDP (KB) |
| Jira | TCPE-59777dcf2c9-72f3-3ff6-8e99-fe88e9f473f1System Jira [22.3.1_Patch6, 23.1.1_Patch4, 23.2-LTS] |
|  |  |
| 22.3 | patchBlue (22.3 - 23.2) SAML URL case change |
| Issue | SAML authentication URL case change prevents login after upgrade |
| Error | Page Not FoundThe page you are trying to reach does not exist. |
| Log | AAS/SSO log  
  Saml2 Status Code: Requester->  Saml2 Status Message: UnknownAssertionConsumerServiceURL https://{FQDN}/webapi/saml2/acs->  
  Saml2 Second Level Status: ->   at Sustainsys.Saml2.Saml2P.Saml2Response.]]> |
| KB | Alteryx SAML auth error after upgrade to 2023 isUnauthorized=true (KB) |
| Veraions | 22.3,  23.2 |
| Jira | TCPE-94077dcf2c9-72f3-3ff6-8e99-fe88e9f473f1System Jira [22.3.1_Patch10, 23.1.1_Patch7, 23.2.1_Patch4, 24.1] |
|  |  |
|  |  |
| 22.1Encryption Key Transfer |  |
|  |  |
| 22.1 | Action Req'dYellow Encryption Key Transfer Process Impacts Server Host Recovery |
| Issue | Customers are required to follow a new process to transfer an Encryption Key during a Host Recovery for DCM and Shared Gallery Connection. The requires line of site between new and older Controllers that isn’t always possible in customer environments leading to broken DCM and Shared Gallery Connections |
| Errors | [tbd, link to DCM and Shared Gallery Connection errors that occur when Encryption Key is not xfered] |
| Help | https://help.alteryx.com/current/en/server/install/server-host-recovery-guide/encryption-key-transfer-process.html https://help.alteryx.com/current/en/server/install/server-host-recovery-guide/disaster-recovery-preparation.html |
|  |  |
| 22.1Patch_3 | ActiveRed PreventGreen (22.1_Patch3 - 24.1) Migration to version 40.03 failed: Renaming collection from 'dataConnections' to 'dataConnectionsMigrationInProcess_40.03 |
| Prevent | Run the following query to update dataConnections.PasswordSecured NULLs to empty strings |
| Issue | Gallery Schema Migration 40.03 fails due to NULL dataConnections.PasswordSecured values.  This occurs in the schema migration applied by 22.1_Patch3. |
| Log Error | Gallery Migration log alteryx-migration.csv2024-12-20 20:37:06.863334, INFO, 1, AlteryxServerWebApiHost, migrationLogger, MoveNext, Starting Migration: 40.032024-12-20 20:37:07.345314, FATAL, 1, AlteryxServerWebApiHost, migrationLogger, DoMigrateDatabase, Migration failed with error: Unable to cast object of type 'MongoDB.Bson.BsonNull' to type 'MongoDB.Bson.BsonString'. |
| Resolution | If still trying to upgrade, change dataConnectionsInProcess.PasswordSecured NULL values to ““ (empty string) with the following queryIf rolled back, dataConnections.PasswordSecured NULL values to ““ (empty string) |
| Jira | GCSE-290277dcf2c9-72f3-3ff6-8e99-fe88e9f473f1System Jira  [25.1-LTS]GCSE-301177dcf2c9-72f3-3ff6-8e99-fe88e9f473f1System Jira  <== possibly a dupe |
| Confluence | Gallery Schema Migration Log Error - Migration to version 40.03 failed: Renaming collection from 'dataConnections' to 'dataConnectionsMigrationInProcess_40.03 Gallery Schema Migration Log Fatal - Migration failed with error: Unable to cast object of type 'MongoDB.Bson.BsonNull' to type 'MongoDB.Bson.BsonString'.  <== 40.03 Gallery Schema Migration Log Fatal - Migraration failed with error: Unable to cast object of type 'MongoDB.Bson.BsonNull' to type 'MongoDB.Bson.BsonString'.  <== 40.03  <== possibly a dupe |
| Versions | 22.1_Patch3 - 24.1 (limited by Mongo 4.2 to 6.0 upgrade).  Additionally, defect was corrected in 25.1-LTS.  But this doesn’t matter for Embedded Mongo since the user would have already experienced it upgrading to a Mongo 6.0 version and had to change the NULLs to ““ already. |
|  |  |
| 22.1Patch_3 | ActiveRed PreventGreen (22.1_Patch3 - 24.1) Gallery Schema Migration 40.03 fails with error: Unable to cast object of type 'MongoDB.Bson.BsonNull' to type 'MongoDB.Bson.BsonString'. |
| Same as | This resolved in the same was as Migration to version 40.03 failed: Renaming collection from 'dataConnections' to 'dataConnectionsMigrationInProcess_40.03 above |
| Confluence | Gallery Schema Migration Log Fatal - Migration failed with error: Unable to cast object of type 'MongoDB.Bson.BsonNull' to type 'MongoDB.Bson.BsonString'.  <== 40.03 |
| Jira | GCSE-290277dcf2c9-72f3-3ff6-8e99-fe88e9f473f1System Jira  [25.1-LTS] |
|  |  |
| 22.1 | patchBlue Action Req'dYellow Controller Token Length Change Breaks Host Recovery |
| Issue | Controller token changes from 40- to 64-char for new installs, but upgrades were leaving it at 40-charThis breaks Server Host Recovery and requires the token to be regenerated as a 64-char before Host Recovery can be performedNewer upgrades are lengthening the Token to 64-char in a way that keeps Controller and Gallery/Worker nodes in syncIt’s not clear if a patch will lengthen the token |
| Action | Check the Controller Token length, if 40-char use Regenerate to make it 64-char.  The new Controller Token needs to be updated to other nodes. |
| Error | [tbd SHRG error with 40-char token] |
| Confluence | Controller Token Length Transition from 21.4 to 22.3 |
|  |  |
| 22.1 | patch ???Blue (22.1 - 24.2) Gallery migration fails on 40.01 with error deserializing the CustomCss property |
| Issue | 40.01 Gallery Schema migration for 22.1 fails |
| Gallery Log | 2024-09-15 13:44:22.108304,FATAL,1,AlteryxServerWebApiHost,migrationLogger,DoMigrateDatabase,"Migration failed with error: An error occurred while deserializing the CustomCss property of class Alteryx.Server.Models.BaseModels.Configuration: An error occurred while deserializing the Sections property of class Alteryx.Server.Models.BaseModels.CustomCss: An error occurred while deserializing the Styles property of class Alteryx.Server.Models.BaseModels.CssSection: An error occurred while deserializing the Properties property of class Alteryx.Server.Models.BaseModels.CssSelector: An error occurred while deserializing the Names property of class Alteryx.Server.Models.BaseModels.CssProperty: Expected a nested document representing the serialized form of a Alteryx.Server.Models.BaseModels.CssProperty+CssProperyName value, but found a value of type String instead.", |
| Versions | Jira states this only affects 24.1.1_Patch4, 24.2-LTS but doesn’t list a fix version in 24.2.  As the schema migration is for 22.1 it seems this issue would have started to occur in 22.1. |
| Jira | TGAL-1189177dcf2c9-72f3-3ff6-8e99-fe88e9f473f1System Jira [24.1.1_Patch5, 25.1-LTS] |
|  |  |
| 22.1 | 24.1-LTSBlue (22.1-23.2) Schema Migration 41 Failing with error Migration failed with error: An error occurred while deserializing the CustomCss property |
| Issue | Gallery Schema Migration 41 for 22.1 fails |
| Gallery Log | 2024-03-22 22:47:45.307692,FATAL,1,AlteryxServerWebApiHost,migrationLogger,DoMigrateDatabase,"Migration failed with error: An error occurred while deserializing the CustomCss property of class Alteryx.Server.Models.BaseModels.Configuration: An error occurred while deserializing the Sections property of class Alteryx.Server.Models.BaseModels.CustomCss: An error occurred while deserializing the Styles property of class Alteryx.Server.Models.BaseModels.CssSection: An error occurred while deserializing the Properties property of class Alteryx.Server.Models.BaseModels.CssSelector: An error occurred while deserializing the Names property of class Alteryx.Server.Models.BaseModels.CssProperty: Expected a nested document representing the serialized form of a Alteryx.Server.Models.BaseModels.CssProperty+CssProperyName value, but found a value of type String instead.", |
| Versions | Jira card states this affects version 24.1.0.07337 and is fixed in 24.1-LTS.  Since schema 41 is for 22.1 Server is would seem to be an issue starting with this version |
| Jira | TGAL-1060977dcf2c9-72f3-3ff6-8e99-fe88e9f473f1System Jira [24.1-LTS] |