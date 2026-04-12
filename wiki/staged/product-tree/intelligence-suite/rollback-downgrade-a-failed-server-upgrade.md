---
id: 564f90efd2a29dbe
title: Rollback / Downgrade a Failed Server Upgrade
status: staged
source:
  kind: confluence_page
  id: confluence-page:1709050604
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/1709050604
  summary: null
category:
- product-tree
- intelligence-suite
keywords:
- upgrade
- version
- server
- mongodb
- will
created_at: 2026-04-12T20:59:51Z
updated_at: 2026-04-12T21:05:15Z
confidence: 0.55
cross_refs: []
content_hash: sha256:6c647c42758ecc4c4ffcafcb6ea7cdb1c2f56fd101e6f1cf44a2e48e40ddbfe9
confluence_page_id: null
model_used: heuristic
---

> **ℹ️ Info**
>
> Rolling back a failed Server upgrade is typically quick and successful

| Key Articles | https://help.alteryx.com/current/en/server/install/downgrade-alteryx-server.html (Help)How To: Downgrade Alteryx Server (KB) |
| --- | --- |

---

---

# How to Rollback a Server Upgrade

|  | Task | Steps |
| --- | --- | --- |
| 1 | Prepare | Review the Questions section below to prepare for edge cases |
| 2 | Get a copy of logs for review for why an upgrade failed | Service logGallery logGallery schema migration log (alteryx-migration.csv from Gallery log folder)Service schema migration log  (alteryx-migration.csv from service log folder)Embedded Mongo DB version upgrade log, if one occurred as part of the upgrade (migration.log in PreUpgrade sister folder to the Persistence folder)If embedded Mongo was restored from another machine, grab the mongoDump.log and mongoRestore.log from backup and restore folders, respectivelyCryptomigration logs (AlteryxServiceMigrator_#.log)C:\ProgramData\Alteryx\Service                                                       <== Prep Tool logs hereAlteryx System Settings > Controller > General > Logging folder                                                        <== Service start logs hereMore log info Logs and Traces |
| 3 | Stop the Service | Order:  Workers … Gallery … Controller [... user-managed Mongo] |
| 4 | Uninstall Server | Run the Add/Remove Programs app to uninstall Server, Predictive Tools, Intelligence Suite |
| 5 | Get the previous version’s installer | See below > https://alteryx.atlassian.net/wiki/spaces/SupportServer/pages/1709050604#Q.-Does-the-customer-have-the-installation-file-for-that-version%3F   Match the main version number, ex 2022.1, don't worry about sub-version/patches (exception: 2021.3.6 MongoDB upgraded from 4.0 to 4.2) |
| 6 | Install old version | Run installer As Adminstrator and choose the installation folder that matches their previous installation. |
| 7 | Set Service Log On user | If used, re-apply the Service Log on User, Services app > Alteryx Service > right-click > Properties > Log On tab > This account |
| 8 | If upgrade was to or through 2022.3, restore RuntimeSettings.xml | Upgrades to or through 2022.3 CryptoMigrated C:\ProgramData\RuntimeSettings.xml.  Replace RuntimeSettings.xml with RuntimeSettings.22_2_legacy.xml (the pre-CryptoMigration backup of RuntimeSettings.xml) |
| 9 | Restore MongoDB | This isn’t necessary if upgrade failed on 2022.3 CryptoMigration because original collections will be untouched)Confirm available disk space.If restoring from the Pre_Upgrade folder, recreate ASMongoDBVersion.bin with a single line containing the mongo version matching the Server version before restore, see:  ASMongoDBVersion.bin  Restore MongoDB backup with the following commands (adjust drive and directory based on where they installed Server)#E3FCEFc:cd %ProgramFiles%\Alteryx\binAlteryxService.exe emongorestore=DRIVE:\PATH_BACKUP,DRIVE:\PATH_RESTORE |
| 10 | Point Alteryx System Settings to the restored MongoDB | Point Alteryx System Settings > Controller > Persistence to the restored MongoDB folder. |

# Questions to Prepare for Rollback

|  | Question | Details |
| --- | --- | --- |
| 1 | Q. What is the current Server’s installation folder?  Ex: D:\Alteryx | Customers often install to D: drive. To find the installation folder, right-click Designer desktop icon > Properties > Target. |
| 2 | Q. Is upgrade failing on CryptoMigration in 2022.3? | If so, MongoDB does NOT need to be restored since the original Collections have not changed (ie, no Schema Migration has occurred).  The CryptoMigration simply created staging Collections of CryptoMigrated data and these will be ignored by the original Server version. |
| 3 | Q. Does the customer have a MongoDB backup prior to upgrade? | If not:  Does their IT do snapshot backups of the Server?  Great!  Have IT restore the snapshot and you do not need to do anything else in this article.noteRed If the snapshot was taken while the Service was Running (ie not Stopped) there is a small chance the Snapshot caught MongoDB while it was in the process of writing data and will restore a corrupt database.Did the upgrade include a MongoDB version upgrade?  If so, the last step would have asked the customer to upgrade MongoDB and would have created a Pre_Upgrade MongoDB backup folder before upgrading.  Use this after recreating ASMongoDBVersion.bin with a single line containing the mongo version matching the rollback Server version, see:  ASMongoDBVersion.bin  If there is no Mongo backup then, a Rollback requires renaming schema migration collections in AlteryxGallery. Request assistance from E3T. |
| 4 | Q. Does the customer know the Server version they upgraded from? | If not:  Review the schema migration log to determine what schema the migration moved to in the last upgrade or what schema migration it started on for this upgrade and use the chart in https://help.alteryx.com/current/server/mongodb-schema-reference  to determine the pre-upgrade version. |
| 5 | Q. Does the customer have the installation file for that version? | If not:  Customer may have the original installer download.If not, Customer can download it from https://downloads.alteryx.com.  For 2021.3 the sub-version is VERY important since the MongoDB version upgraded in 2021.3.6If the customer's version is no longer available for download you can find it in Artifactory.  CSEs are generally prohibited from providing Designer or Server installation files directly to Customers to ensure we don't export our software to a country the US Government forbids export (contact Fulfillment to provide installation files).  However, common sense can prevail if you can see that they have Server and you can’t engage Fulfillment.  To confirm they are geo-validated (ie, not in North Korea)  have them visit the Downloads and Licenses page and confirm they have the ability to download more recent versions of Server. |
| 6 | Q. Did the customer Regenerate the Controller Token when trying to make the upgrade work? | If so: are we rolling back from a 2021.4+ upgrade to 2021.3 or prior?We have a problem.  The 2021.4+ Controller Token is 64-char while the 2021.3 and prior Controller Token is 40-char and will not understand a 64-char Controller Token. Can we recover the original RuntimeSettings.xml?Did they upgrade to or through 2022.3?  If so, the original RuntimeSettings.xml was backed up to RuntimeSettings.22_2_legacy.xml.Did they back up RuntimeSettings.xml prior to regenerating the Token? Did IT snapshot the machine prior to the Token being regenerated?If you recovered the original RuntimeSettings.xml the steps below are not needed.If you can’t recover the original RuntimeSettings.xml then:Upgrade was to 2022.3+Delete the values for both RuntimeSettings.xml <ServerSecretEncrypted> and <StorageKeysEncrypted> and restart the Service.  They will both regenerate.  This will cause all existing Credentials to fail because they won’t be able to be decrypted.  New Credentials will be able to be used.  Users will need to update all Shared Credentials and republish all workflows using Credentials.2022.3 Crypto Migration errors are expected for AS_RunAsCredentials and AS_Queue records using these older Credentials.  Running a workflow using one of the old Credentials that can no longer be decrypted creates the error:AlteryxService_RetrieveSecureData failedUpgrade was to 2021.4 or 2021.1Delete the value from RuntimeSettings.xml <ServerSecret> and restart the Service.  The Controller Token will regenerate and will be compatible with the <StorageKeysEncryped> because you’re on the same machine. |