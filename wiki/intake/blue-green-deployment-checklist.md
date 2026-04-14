---
id: 6b531530a731bd65
title: Blue-Green Deployment Checklist
status: intake
source:
  kind: confluence_page
  id: confluence-page:3221160357
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/3221160357
  summary: null
category: []
keywords: []
created_at: 2026-04-14T15:18:28Z
updated_at: 2026-04-14T15:18:28Z
confidence: null
cross_refs: []
content_hash: sha256:f62639c43cc0e69487d56e68535b4c307e077b7755823d6371da95f781ba040d
confluence_page_id: null
model_used: null
---

Can build off of the current [Server Upgrade Checklist](https://help.alteryx.com/current/en/server/install/install-or-upgrade-server/server-upgrade-checklist.html#upgrade-7047039), with additional considerations (edits shown in yellow highlight,  removed portions in magenta highlight with strikethrough ):

# [EDITED] Blue-Green Deployment Server Upgrade Checklist

Your Server configuration is unique and upgrading it is a project that requires planning and preparatory work to be successful. This checklist ensures you consider all tasks that might be needed for your Blue-Green  upgrade and directs you to Help and Knowledge Base articles for detailed step-by-step procedures.

If you would like help preparing or executing your upgrade, please speak with your Account Executive for options.

## What is a Blue-Green deployment?

A Sandbox server upgrade becomes the new Production environment after validation. This eliminates the risk of your Production Server being down for an indeterminate amount of time as it is not upgraded in place. Blue-Green deployment validates that the Server environment and required database drivers, DSNs, Connectors and other settings are fully understood as they must be set up on the Sandbox for validation.

| [PLAN](https://help.alteryx.com/current/en/server/install/install-or-upgrade-server/server-upgrade-checklist.html#plan-7047039) | [PREP WORK](https://help.alteryx.com/current/en/server/install/install-or-upgrade-server/server-upgrade-checklist.html#prep-work-7047039) | [MIGRATE](https://help.alteryx.com/current/en/server/install/install-or-upgrade-server/server-upgrade-checklist.html#upgrade-7047039:~:text=Perform%20a%20Server%20Host%20Recovery%20to%20new%20or%20test%20Server) | [UPGRADE](https://help.alteryx.com/current/en/server/install/install-or-upgrade-server/server-upgrade-checklist.html#upgrade-7047039) | [TEST](https://help.alteryx.com/current/en/server/install/install-or-upgrade-server/server-upgrade-checklist.html#test) | **SWITCH TRAFFIC/GO LIVE** | [TROUBLESHOOT](https://help.alteryx.com/current/en/server/install/install-or-upgrade-server/server-upgrade-checklist.html#troubleshoot) |
| --- | --- | --- | --- | --- | --- | --- |
| - Determine target version    - Identify critical workflows for validation | - Pre-upgrade check workflow (critical)    - Backup MongoDB    - Backup key files  **Advanced**     - Evaluate Connectors    - Save Python and R environments | - Perform a Server Host Recovery to migrate Production database to a Test/Sandbox machine | - Upgrade in Test environment    - Upgrade live environment  **Advanced**     - Upgrade Connectors    - Restore Python and R environments | - Review upgrade logs    - Access Server UI pages    - Publish from Designer    - Validate critical workflows | - Update the Production URL to direct to the new machine and test    - Decommission the previous Production Server | - Common issues and resolutions    - Rollback    - Customer Support |

## Server Upgrade Overview

Testing your upgrade process prior to upgrading your Production server is the **best way to ensure your Server upgrade process will run smoothly in your production environment**.

Ideally, start with a same-version Sandbox/Dev/Test Server and upgrade it, see [Alteryx Server Sandbox Environment](https://knowledge.alteryx.com/index/s/article/Alteryx-Server-Sandbox-Environment). If you have a multinode environment, testing is still effective on a single machine that runs Controller + Server UI + Worker. Similarly, if you have User-Managed MongoDB, restoring a database backup to the test machine's embedded Mongo can help validate the upgrade. Contact your Account Executive for information on a Sandbox license.

**At a bare minimum**, you should install the target version of Designer on a user's machine to test critical workflows in the new version. For more information, go to [Install Two Versions of Designer on the Same Machine](https://help.alteryx.com/current/en/license-and-activate/install/install-two-versions-of-designer-on-the-same-machine.html).

**Ideal process:**

## Server Upgrade Process

### Plan

| Questions/Steps | Consideration/Links |
| --- | --- |
| Choose your target Server  version to upgrade to . | Version-to-Version Server Upgrade Guide - specific items you should be aware of when upgrading.     - Version-to-Version Server Upgrade Guide: Supported Versions - specific items you should be aware of when upgrading. For unsupported versions, see Version-to-Version Server Upgrade Guide: Unsupported Versions.    - Alteryx Version Support Policy |
| Know your current version for rollback. | You can find your current version:     - Private Studio in a browser > select your name in upper right > My Profile > Version.    - Run Designer on Server, Help > About. |
| Confirm sufficient free space.  **Important**  Upgrade will fail for lack of space. | If you use **Embedded MongoDB** and the [Version-to-Version Server Upgrade Guide](https://help.alteryx.com/current/en/server/install/install-or-upgrade-server/version-to-version-server-upgrade-guide-supported-versions.html) indicates the MongoDB Version will upgrade, confirm sufficient free space:     - MongoDB in Server Upgrades - Best Practices |
| Identify validation workflows. | Identify workflows to validate the upgrade. These are:     - Critical workflows that must run on the Server    - Workflows that:Input/output to a network UNC locationInput/output to a databaseUse Connector ToolsUse Location or Business Insights DatasetsUse Python ToolUse R Tool       - Input/output to a network UNC location       - Input/output to a database       - Use Connector Tools       - Use Location or Business Insights Datasets       - Use Python Tool       - Use R Tool |
| Plan how to manage scheduled workflow during your upgrade. | By default, schedules that should have run while the Server was being upgraded will pick up as soon as the Server and nodes restart. You can suspend all schedules and determine what should run on an individual basis (described in the [UPGRADE section](https://help.alteryx.com/current/en/server/install/install-or-upgrade-server/server-upgrade-checklist.html#upgrade-7047039)). |
| **Advanced issues that might not apply to your upgrade:** |  |
| Do workflows use Connector Tools? | Connectors are installed independently and must be compatible with your new Server version. You can determine the Connectors and versions installed by their folder names under **%ProgramData%\Alteryx\Tools**.  Review each Connector to ensure compatibility with your new Server version to determine Server+Connector version compatibility:     - Designer Compatibility with Data Connectors    - Alteryx Marketplace    - Community Gallery (see Technology Partner section)  If the Python version is upgraded as part of your Server upgrade, all Python-based connectors must be reinstalled. The YXI file takes the current Python version into account during the installation process (so the same YXI file will perform a different installation when run in versions of Designer/Server that use a different Python version). View the Python versions used by Server versions in [Alteryx Embedded Python](https://help.alteryx.com/current/en/release-notes/alteryx-embedded-python.html). |
| Do workflows use the Python Tool? | - Python Tool Environment in Server Upgrades |
| Is your Server configured to use a Macro Repository? | Sandbox or Blue-Green Deployment     - Run Designer of Server as Administrator and review the settings Options > User > Settings > Tools > Macros for any Macro Repository folders.    - These need to be set on the Sandbox Server. |
| Is your organization required to maintain vendor support from MongoDB? | - MongoDB Support Policy Lifecycles    - MongoDB Schema Reference |
| Is your MongoDB User-Managed? | - MongoDB in Server Upgrades - Best Practices |
| Are you changing MongoDB between Embedded and User-Managed? | Don't perform a Server Upgrade and MongoDB migration together, these are separate projects.     - Migrate between Embedded and User-Managed Mongo |
| Are you moving from on-prem to cloud? | Don't perform a Server Upgrade and cloud migration together, these are separate projects.     - Azure and Azure White Paper    - Amazon AWS (gated white paper) and Alteryx Server on AWS |
| Do you use the Connect product? | Upgrade Connect to the same version as Server. For more information, go to [Connect](https://help.alteryx.com/current/en/connect/install/upgrade-connect.html) and [Loaders](https://help.alteryx.com/current/en/connect/administer-connect/load-metadata/load-metadata-from-an-alteryx-server-gallery.html). |

### Prep Work

| Questions/Steps | Considerations/Links |
| --- | --- |
| Deploy a separate, second Server machine environment. | Ensure there are equivalent or greater resources available when compared to the Production environment (available memory, processor speed, cores, etc.) |
| Run pre-upgrade checks.  **Important**  Skipping this step is the cause of most server upgrade failures. | - Alteryx Server: Pre-Upgrade Checks |
| Run Crypto Migration Prep Tool if upgrading to or through 2022.3. | - Migration Prep Tool |
| **Stop Server and backup MongoDB and other critical information.** |  |
| Stop Server. | [Order](https://knowledge.alteryx.com/index/s/article/How-to-restart-the-services-in-a-multi-node-Alteryx-Server): **Workers **(wait for jobs to finish) … **Server UI** … **Controller **… [user-managed MongoDB] |
| Backup Mongo database.  **Important**  A server snapshot is not sufficient as it can restore a corrupt MongoDB if the Service was running when the snapshot was taken. | Perform a MongoDB backup from the command line (adjust for your folder structure).  `C:\Program Files\Alteryx\bin\AlteryxService.exe emongodump=C:\BKP_DIR`     - MongoDB Backups |
| Backup RuntimeSettings.xml, Controller Token, and Service Log On user. | 1. Run Alteryx System Settings > Controller > General > Controller Token > View and copy the Token to a safe location.    2. Make a backup copy ofC:\ProgramData\Alteryx\RuntimeSettings.xml    3. Note the Services App > AlteryxService > Properties > Log On settings. |
| Optionally backup other settings. | - Critical Server Files and Settings to Backup |
| Optionally perform a snapshot backup. | Stop the **AlteryxService**prior to the snapshot. If rollback is needed, you can try using the snapshot, with the MongoDB backup above being your failsafe. |

### Upgrade

| Questions/Steps | Considerations/Links |
| --- | --- |
| **If moving to a new Server or testing the upgrade on a test Server:** |  |
| **Blue-Green deployment to migrate Production to a validated environment:** |  |
| Perform a Server Host Recovery to new or test Server | - Server Host Recovery Guide |
| Test the Host Recovery  **before **upgrading.  **Warning**  Don't skip this step. | Follow the [TEST section](https://help.alteryx.com/current/en/server/install/install-or-upgrade-server/server-upgrade-checklist.html#test) below to ensure the migration was successful **before **upgrading to make troubleshooting easier. |
| **If upgrading in-place (on the same machine):** |  |
| Do you want to suspend Schedules after the upgrade? | If you do not want Schedules to run when the Service starts: Run Alteryx System Settings on each **Worker**, deselect **Worker **> **General **> **Run unassigned jobs**, and give the Worker a unique **Job tag**. Alternatively, contact Customer Support for assistance in deleting all schedules. |
| Stop Server | [Order](https://knowledge.alteryx.com/index/s/article/How-to-restart-the-services-in-a-multi-node-Alteryx-Server): **Workers **(wait for jobs to finish) … **Server UI** … **Controller **… [user-managed Mongo] |
| Upgrade | - Download new version from downloads.alteryx.com.    - Right-click and run the installer As Administrator.    - Choose the same installation path as your old version.    - Choose Migrate Mongo Database if the option is presented.  **Tip**  Save the installer in case you need to roll back to this version after a future upgrade. |
| Did you have a **Service Log On User** in the [PREP WORK section](https://help.alteryx.com/current/en/server/install/install-or-upgrade-server/server-upgrade-checklist.html#prep-work-7047039)? | Reset Service Log On User after upgrade:  **Windows Services** app > right-click **AlteryxService **> **Properties **> **Log On**  **Tip**  In the future consider using the **Alteryx System Settings **> **Worker **> **Run As** user instead as it is not lost during an upgrade. |
| Perform version-specific tasks | - Version-to-Version Server Upgrade Guide: Supported Versions - specific items you should be aware of when upgrading. For unsupported versions, see Version-to-Version Server Upgrade Guide: Unsupported Versions. |
| **Advanced issues that might not apply to your upgrade:**  [Note on reworking this document:  We may want to merge this section into the Planning Step so we’re not repeating here.  Sp here,] |  |
| Do you have a multi-node environment? | All nodes must be upgraded to the same version.  [Restart order](https://knowledge.alteryx.com/index/s/article/How-to-restart-the-services-in-a-multi-node-Alteryx-Server): [user-managed Mongo] … **Controller **… S**erver UI **… **Workers** |
| Do workflows use Connector Tools? | If Connectors need to be upgraded to remain compatible with the new Server version, install upgraded versions of Connectors and delete incompatible Connector folders.  When a Connector version is removed from the Server, existing workflows using that version will stop running with the error message "Error: Unable to resolve plugin Python 'XXXXX\main.py' (Tool Id: X)".  Users need to:     1. Install a version of the Connector that matches what's available on the Server.    2. Delete the old version (simply delete the old version's folder underC:\Users\USER_NAME\AppData\Roaming\Alteryx\Tools    3. Open the workflow, edit the Connector, and re-authenticate it.Alternatively: Delete the Connector and re-add it.    4. Test the workflow is functioning with the new version (some versions change the Tool's UI).    5. Republish the workflow to the Server.    6. Verify the workflow runs on the Server as expected. |
| Do In-DB connections need to be migrated? | Copy file from original Server:  `C:\ProgramData\Alteryx\Engine\SystemConnections.xml`  Note: this will only transfer the connection details, but the drivers would still need to be installed on the new Server machine. |
| Are there ODBC drivers that need to be installed? | Check the drivers installed on the previous Server machine by searching and selecting “ODBC Data Sources (64-bit)” in Windows, then select the “Drivers” tab. The drivers can be downloaded from the [Downloads Portal](https://downloads.alteryx.com/)and installed to the new Server environment. |
| Is there a proxy in place? | This can be checked through various methods ( [How to check if a proxy is setup](https://knowledge.alteryx.com/index/s/article/How-to-check-if-a-proxy-is-setup)). IT team involvement may be necessary. Configure the proxy in the new environment. |

### Test

| Questions/Steps | Considerations/Links |
| --- | --- |
| If upgrading, review MongoDB Schema Migration File | Confirm the schema migrated to the version expected for your new Server version  `%ProgramData%\Alteryx\Gallery\Logs\alteryx-migration.csv`     - MongoDB Schema Reference  Look for a line near the end with a number matching the expected schema  `INFO,1,migrationLogger,MoveNext,Migration 31 Completed.,`  See [TROUBLESHOOT section](https://help.alteryx.com/current/en/server/install/install-or-upgrade-server/server-upgrade-checklist.html#troubleshoot) if the migration didn't complete to the expected level. |
| Basic Server Testing | Is the **AlteryxService **running?  Can you:     - Access the Server URL?    - Move around Admin pages and view Users, Collections, etc.?    - Publish a workflow from Designer to the Server?    - Run the workflow?    - If your configuration allows, save and run a workflow specifying your credentials. |
| Test validation workflows | Test the validation workflows identified in the [PLAN section](https://help.alteryx.com/current/en/server/install/install-or-upgrade-server/server-upgrade-checklist.html#plan-7047039). Confirm these don't output data to production locations or databases if this would produce duplicate records or cause other data integrity issues for your organization. |

**Switch Traffic/Go Live**

| Questions/Steps | Considerations/Links |
| --- | --- |
| Update the URL of the Production environment to the newly upgraded machine | May require IT team involvement. See [Changing the Gallery URL on Alteryx Server](https://knowledge.alteryx.com/index/s/article/Changing-the-Gallery-URL-on-Alteryx-Server-1583460631447)for more information. |
| Decommission the original Production Server environment  **Warning**  Do not destroy the original Production environment until all testing of the upgrade has been completed on the newly upgraded environment | Refer to the Test steps to ensure the original environment is no longer needed. |

### Troubleshoot

| Issues | Troubleshooting |
| --- | --- |
| **Mongo Schema Migration** didn't complete or has an error. | The most common reason for this is that the Pre-Upgrade Checks workflow wasn't run or the issues found weren't corrected.     - How to Run Pre-Upgrade Checks When Gallery Won't Start |
| **UNC Network Locations** Error in workflow accessing UNC location. | Ensure the **Run As User** or **Service Log On User** are properly set and have rights to access the network location. |
| **ODBC / DSNs **Error accessing a database using DSN. | Compare the ODBC driver versions and ODBC System DSNs from your old machine. Look for version or spelling differences. [Download supported drivers](https://downloads.alteryx.com/). |
| **In-DB Connection** Error accessing an In-DB connection. | Copy file from original Server:  `C:\ProgramData\Alteryx\Engine\SystemConnections.xml` |
| **Connector Tool Errors** | See [Connectors Troubleshooting](https://knowledge.alteryx.com/index/s/article/Connectors-Troubleshooting-Landing-Page). |
| **Rollback** | Rollback is quick in a Blue-Green approach, as the original Production environment would still be available. Simply change the URL to the original non-upgraded machine.  If you need to rollback, see [Downgrade Alteryx Server](https://help.alteryx.com/current/en/server/install/downgrade-alteryx-server.html). |
| **Customer Support Assistance** | Customer Support can assist if you experienced an error in the upgrade process and are unable to resolve it with the common troubleshooting articles above. Your Account Executive can provide options if you would like assistance planning or executing an upgrade.  **Case Prioritization**  Criteria 1:     - Sev 1 - Production Server is completely down    - Sev 2 - Sandbox/Dev Server is down or Production Server isn't fully functional  Criteria 2: Paid Support Tier     - Alteryx Support Guidelines    - Support Policy and Guidelines |
| **What to provide to Customer Support?** | To ensure Customer Support can start troubleshooting immediately, please include in your support request:     1. Is this your Dev/Sandbox or Production environment?    2. Is the Server down completely?    3. Version upgrading from and to.    4. Is this a multi-node environment?    5. Description and screenshot of the error you're receiving.    6. The following files, adjusting the location based on your installation:Server UI NodeC:\ProgramData\Alteryx\Gallery\Logs\alteryx-migration.csvC:\ProgramData\Alteryx\Gallery\Logs (past 48 hours)All NodesC:\ProgramData\Alteryx\RuntimeSettings.xmlC:\ProgramData\Alteryx\Service (past 48 hours)    7. When upgrading to or through 2022.3, provide CryptoMigration logs (AlteryxServiceMigrator_#.log). |