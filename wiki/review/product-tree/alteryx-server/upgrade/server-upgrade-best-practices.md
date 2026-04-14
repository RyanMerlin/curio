---
id: c1609b7c480436b5
title: Server Upgrade Best Practices
status: review
source:
  kind: confluence_page
  id: confluence-page:3208446355
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/3208446355
  summary: null
category:
- product-tree
- alteryx-server
- upgrade
keywords:
- upgrade
- best-practices
- planning
- procedure
created_at: 2026-04-14T13:40:14Z
updated_at: 2026-04-14T13:50:50Z
confidence: 0.85
cross_refs: []
content_hash: sha256:23e97f6f21bc3c10f021e76cc01fe2d469c7d1de2de92c3ea95d040188cbe7a3
confluence_page_id: null
model_used: claude-sonnet-4-6
---

## Redesign Help page

<https://help.alteryx.com/current/en/server/best-practices/server-upgrade-best-practices.html#review-the-release-notes>

---

### Status: In Review

| Published Date | Published Link |
| --- | --- |
|  |  |

(This box is invisible)

**Status Key:**

- Draft - Documentation is being drafted
- In Review - Documentation is being reviewed and edited
- Ready - Documentation has been signed off and is ready to be published
- Published (Add link & Date) - Documentation is published. Please add a link to the top of the page
| POC | Sign-off Status | Sign-off Required by |
| --- | --- | --- |
| PM -   or | 34
82d88a62-73bc-4aae-8c6f-a51b62007ae3
complete
PM Signoff

35
f3ed277e-857b-499f-89f2-255c2c95b58b
complete |  |
---

Although upgrading from one version of Alteryx Server to another is a straightforward process, there are several considerations and preparation steps that can help ensure a smooth upgrade. This page will provide a topical overview of the process, including links to useful documentation, and a step-by-step approach to consider when planning your upgrade.

Not every step or recommendation in this document is applicable to every environment or installation. Your plan might differ.

In general, the upgrade process should consist of the following high-level steps:

1. Document Your Environment
2. Perform a Server Health Check (Optional)
3. Select a Target Version or Versions
4. Download the Software
5. Perform a Sandbox Upgrade or Blue-Green Deployment
6. Schedule the Upgrade
7. Perform the Upgrade

The following sections describe these steps and add narrative to help plan your work. Links to detailed instructions, where they exist, will be shown inline, and an aggregated list of all the links provided in this document can be found in the [Guides and Help Articles](https://help.alteryx.com/current/en/server/best-practices/server-upgrade-best-practices.html#guides-and-help-articles) section.

Find a checklist of these steps in the [Server Upgrade Checklist](https://help.alteryx.com/current/en/server/install/install-or-upgrade-server/server-upgrade-checklist.html).

Alteryx and its partners are available to assist in planning and executing an upgrade. Speak to your Account Executive if you need assistance with this process.

**New Terminology**

With the release of Server 2022.3, the term Gallery has been deprecated in favor of Server UI. Although the legacy term still exists in the software and documentation at the time of this writing, this document uses Server UI to refer to the applicable services, nodes, configurations, and others.

## Section 1. Document Your Environment

### Capture Your Architecture and Configuration

It is necessary to have complete understanding (and documentation) of your environment. At a minimum, you need to know:

- How many servers are installed, and what are their functions?How many each of Controllers, Server UI instances, Workers, MongoDB and SQL nodes do you have in each of your environments (Dev/Test/Production)?Do you run a High Availability (HA) Environment?
   - How many each of Controllers, Server UI instances, Workers, MongoDB and SQL nodes do you have in each of your environments (Dev/Test/Production)?
   - Do you run a High Availability (HA) Environment?

- Is there an architectural diagram which visualizes the environment? If not, this is a good opportunity to create one.
- What software version of Alteryx Server is running in your environment?
- What additional software has been installed?Custom R librariesCustom Python librariesThird-party utilitiesConnectorsNot every connector needs to be upgraded, and some might not have an upgrade available.
   - Custom R libraries
   - Custom Python libraries
   - Third-party utilities
   - ConnectorsNot every connector needs to be upgraded, and some might not have an upgrade available.
      - Not every connector needs to be upgraded, and some might not have an upgrade available.

- Data packs: Location Insights, Business Insights, Intelligence Suite, and so on.Best practice is to install matching versions of these add-ons during the upgrade, if available.Custom tools designed by your users or downloaded from Community, other purchased or free third-party tools and connectors. Make a list of these, along with versions.
   - Best practice is to install matching versions of these add-ons during the upgrade, if available.
   - Custom tools designed by your users or downloaded from Community, other purchased or free third-party tools and connectors. Make a list of these, along with versions.

- The configuration options which have been set via the Alteryx System Settings configuration tool, including but not limited to:WorkspacesLogging DirectoriesScheduler and Engine enablement settingsPersistence settings, including:Database typeData folderRetention optionsServer UI settings (URLs and security)Authentication methods and IDP informationSMTPRun-as UserNoteThese configuration options (such as Scheduler, Engine, Persistence, Server UI, SMTP settings, Run-as User, and more) are captured in C:\ProgramData\Alteryx\RuntimeSettings.xml, so an admin doesn’t need to separately record settings. A copy of RuntimeSettings.xml provides all of these in a plain text XML file.
   - Workspaces
   - Logging Directories
   - Scheduler and Engine enablement settings
   - Persistence settings, including:Database typeData folderRetention options
      - Database type
      - Data folder
      - Retention options

   - Server UI settings (URLs and security)
   - Authentication methods and IDP information
   - SMTP
   - Run-as UserNoteThese configuration options (such as Scheduler, Engine, Persistence, Server UI, SMTP settings, Run-as User, and more) are captured in C:\ProgramData\Alteryx\RuntimeSettings.xml, so an admin doesn’t need to separately record settings. A copy of RuntimeSettings.xml provides all of these in a plain text XML file.

- Service Log-On User
- Physical and virtual server specificationsCores and memoryOS versionFor more information, go to Server System Requirements.
   - Cores and memory
   - OS version
   - For more information, go to Server System Requirements.

- MongoDB Databases and Python versionsUser-managed MongoDB: User-managed MongoDB version is independent of a Server upgrade. You might need to separately upgrade your User-managed MongoDB in parallel to a Server upgrade. In the case of the user-managed instance, Alteryx doesn't provide support. For more information, go to Version Support Policy.Embedded MongoDB: Embedded Mongo and Python versions follow from the Server version and don’t need to be separately noted. For more information about embedded MongoDB versions, go to MongoDB Schema Reference or Version Support Policy.User-managed SQL DB: User-managed SQL DB version is independent of a Server upgrade. You might need to separately upgrade your User-managed SQL DB in parallel to a Server upgrade. In the case of the user-managed instance, Alteryx doesn't provide support. For more information, go to Version Support Policy.NoteIf you are using the Python tool, please check the Python Tool Environment in Server Upgrades before upgrade.NoteYour list of scheduled jobs, collections, workflows, and memberships is part of the MongoDB and is not lost during the upgrade.
   - User-managed MongoDB: User-managed MongoDB version is independent of a Server upgrade. You might need to separately upgrade your User-managed MongoDB in parallel to a Server upgrade. In the case of the user-managed instance, Alteryx doesn't provide support. For more information, go to Version Support Policy.
   - Embedded MongoDB: Embedded Mongo and Python versions follow from the Server version and don’t need to be separately noted. For more information about embedded MongoDB versions, go to MongoDB Schema Reference or Version Support Policy.
   - User-managed SQL DB: User-managed SQL DB version is independent of a Server upgrade. You might need to separately upgrade your User-managed SQL DB in parallel to a Server upgrade. In the case of the user-managed instance, Alteryx doesn't provide support. For more information, go to Version Support Policy.

We have prepared a [Configuration and Architecture Checklist](https://help.alteryx.com/downloads/server/Configuration_Architecture_Checklist_v01.pdf) to make this step easier for you. Completing this checklist will give you an overview of your infrastructure and configuration.

### Identify Business Critical Workflows

An important part of planning your upgrade is to identify business critical workflows that you want to protect and test as part of the upgrade process. These are generally workflows that are run on a schedule, act as dependencies to downstream work (inside or outside of Alteryx), and/or provide critical data/output to key stakeholders in the company. Essentially, you want to identify any workflow which, if unavailable for any significant amount of time, will have a deleterious effect on your business.

Identifying critical workflows can help you choose your target version. If the critical workflow contains tools or connectors that are not compatible with a particular version, you’ll want to take that into consideration when selecting your target version (see [Section 3](https://help.alteryx.com/current/en/server/best-practices/server-upgrade-best-practices.html#section-3--select-a-target-version-or-versions) below for the [Version-to-Version Server Upgrade Guide: Supported Versions](https://help.alteryx.com/current/en/server/install/install-or-upgrade-server/version-to-version-server-upgrade-guide-supported-versions.html)). These workflows can also be modified for [post-upgrade testing](https://help.alteryx.com/current/en/server/best-practices/server-upgrade-best-practices.html#4--test-qc-the-upgrade) (below) and included in your QA plan.

### Create Test Versions of Business-Critical Workflows

When you test critical flows during your post-upgrade quality control, you’ll need to disable or edit any outputs that write data to other production systems, or produce outputs that will reside in the production environment.

A common methodology here is to create dedicated test versions of these flows which still access the target systems and directories, but will not overwrite data in production files and tables.

- For data operations, change the outputs to write to dedicated test versions of tables.
- For file operations, write files with a different file naming convention, or to a testing subfolder.

This allows you to do end-to-end testing that does not impact production. These test workflows should be used in production testing for the same reason.

### Additional Considerations

Plan and schedule your upgrade activities to minimize disruption to your organization’s ongoing operations. Schedule the upgrade window outside of normal business hours (if possible), and during periods of "lighter" utilization. For example, do not schedule during fiscal year-end close, quarter-end processing, monthly audits, and so on.

Some clients use an upgrade window to revisit Operating System (OS) versions on their Alteryx machines. Please work with your IT Department if this is something you wish to do and review the [System Requirements](https://help.alteryx.com/current/en/server/system-requirements.html). Be sure to document in your upgrade plan whether you are upgrading the OS at the same time. If problems occur after the Alteryx upgrade, support engineers will be made aware of them.

## Section 2. Perform a Server Health Check (Optional)

An Alteryx Server Health Check is a valuable resource for understanding usage patterns of an Alteryx Server environment. It analyzes historical usage patterns to determine how busy the Server environment is, what kind of optimization activities might be needed, and whether the environment is sized appropriately.

If you’d like to learn more, contact your Alteryx Account Executive.

## Section 3. Select a Target Version or Versions

Choose the Server version you plan to upgrade to. Depending on your organization’s upgrade cycle, you may be several versions behind the latest release. The number of versions between your current and target version affects your upgrade approach, so plan accordingly.

Many organizations don’t upgrade with every release, especially since multiple versions can be released each year. However, staying reasonably current is important. Regular upgrades provide critical security updates, performance improvements, and new features.

A common strategy is to standardize on the latest release or the previous one to balance stability with timely access to enhancements.

Select the target version of the Server software. Depending on your internal upgrade cadence, you might be one to several versions behind the current software release. There are considerations that apply depending on the number of versions between your current version and your target. Most clients do not upgrade each time a new version is released (there can be multiple releases in any given year), nor do they always run the current release; many clients opt for current-release-m inus-1.

**Note**

All major releases are supported for 24 months. If your organization has adopted infrequent updates, this should be critical to your decision.

### Review the Release Notes

The first step in the selection process is to read the [Release Notes](https://help.alteryx.com/document/preview/6553144#UUID-28bacd4e-baa7-668c-2b44-df710f9c19ff). These detail new features and programming changes in the potential target versions, and detail bug fixes and known issues.

### Understand the Upgrade Path - Where You Are Now to Where You Want to Go

Some releases have additional considerations if coming from certain prior versions, and some versions are not appropriate for all customers. For example, you might be required to upgrade your MongoDB in order to traverse between versions, or in the case of version 2022.3, you will need to upgrade Server and Designer together because of data encryption enhancements in the software that make that version of Server incompatible with prior versions.

The Alteryx Help & Support site has a [Version-to-Version Server Upgrade Guide: Supported Versions](https://help.alteryx.com/current/en/server/install/install-or-upgrade-server/version-to-version-server-upgrade-guide-supported-versions.html) (for unsupported versions, go to [Version-to-Version Server Upgrade Guide: Unsupported Versions](https://help.alteryx.com/current/en/server/install/install-or-upgrade-server/version-to-version-server-upgrade-guide-unsupported-versions.html)), which highlights tasks and considerations you need to be aware of when upgrading through versions of Alteryx Server. The guide is especially helpful if you are upgrading through multiple versions at once, such as migrating from 2019.1 to 2022.1. To ensure a smooth upgrade, you might need to take some incremental steps.

### Select Your Version

Now that you have educated yourself on the various available versions and the special considerations that line your upgrade path, you are ready to select your target version. From here, proceed to the [Downloads](https://alteryx.flexnetoperations.com/flexnet/operationsportal/logon.do)site.

## Section 4. Download the Software

Visit the Alteryx Licensing portal. You need an account to visit the site. Once there, you’ll find all the downloads available to you, including but not limited to:

- Alteryx Server (current and previous versions)
- Alteryx Designer
- Alteryx Intelligence Suite and Insights Data
- Alteryx supported Database Drivers

Download all the software you need and continue to the next step in the process. For help downloading, visit the [Download and Install a Product](https://help.alteryx.com/current/en/license-and-activate/license-and-activate-with-license-keys/install/download-and-install-a-product.html) help page.

**Version Parity**

In general, it is best practice to keep Server and Designer on the same version. So, downloading the matching Designer installer at this time makes the most sense. However, since upgrading Designer across a large user base requires additional planning and resources, you might not wish to complete the upgrade at the same time as the Server upgrade.

| NoteAs of version 2025.2, Alteryx has decoupled Designer and Server updates to modernize the update experience and reduce IT overhead. You can now upgrade Designer independently of Server. |
| --- |

Server is generally backwards compatible with older versions of Designer, with the caveat that new features supported on the target version of Server won’t be available in older versions of Designer.

In the case of Server and Designer 2022.3, this backward compatibility does not exist due to data encryption enhancements across the platform. If you are planning to upgrade to or beyond this version, Designer must be updated at the same time. There are special instructions found on the [Migration Prep Tool](https://help.alteryx.com/current/en/server/install/install-or-upgrade-server/migration-prep-tool.html) help page for preparing an upgrade to this version. If you are trying to download an older version no longer available on the [Downloads](https://alteryx.flexnetoperations.com/flexnet/operationsportal/logon.do)page, please contact [Fulfillment](mailto:fulfillment@alteryx.com).

## Section 5. Perform a Sandbox Upgrade or Blue-Green Deployment

Directly upgrading your Production Server can lead to downtime if an issue is encountered during the upgrade or in post-upgrade testing.  Failure to have a backup to enable rollback can leave your Production environment down until the upgrade issue is resolved.

Better practices include

- Sandbox Upgrade – Testing the upgrade in a non-production environment and documenting your process steps prior to upgrading your production Server helps ensure the process will run smoothly in your production environment, and that your business-critical workflows and third-party tools continue to run as expected. In the event they do not, it provides an opportunity to explore and remediate these issues and add these remediation steps to your production upgrade plan. The steps you follow in this test upgrade, plus any additional remediation steps you add in the quality control phase, will become your upgrade “script” for production.  Contact your Account Team to request a Sandbox license.
- Blue-Green Deployment – A Sandbox server upgrade becomes the new Production environment after validation.  This eliminates the risk of your Production Server being down for an indeterminate amount of time as it is not upgraded in place. The original Production environment is blue, and the new/Sandbox environment is green, where updates can be deployed and tested.  Blue-Green deployment validates that the Server environment and required database drivers, DSNs, Connectors and other settings are fully understood as they must be set up on the Sandbox for validation.

The table below shows a tiered list of minimum to best-case recommendations when upgrading Server:

| Embedded Mongo Backup | Minimum | Better | Best | Benefit |
| --- | --- | --- | --- | --- |
| Backup MongoDB | 1
2e7b022f-0f84-48b1-b8ba-425d78e6a12e
complete | 2
a573f5e7-9d4d-4375-8847-5ce7beb1c710
incomplete | 3
066c5577-b1c6-43ae-9981-c72e6e908685
incomplete | Preserves database information for rollback in the event of errors during the upgrade. Rollback may include manual steps and may require the assistance of Alteryx Support. |
| IT Snapshot of the Windows Server machine | 7
7a217ef8-2fee-4d8e-8580-f798ae2de12e
incomplete | 8
cc74266e-eff1-4c0d-bf4f-5bf8053672df
complete | 9
a719c662-1bbe-4bf4-94c0-1a43fb4a7826
incomplete | Provides quick and easy rollback with no manual steps.Steps: Your IT team will back up a complete instance of the Alteryx Server machine.NOTE:  The Service must be stopped so the Snapshot doesn’t capture the database while it’s being updated, as this could leave it in a non-functional state when restored. |
| Blue-Green Deployment | 19
dee069ed-1a71-4cb1-b002-26e4bf31ae61
incomplete | 20
5e419a38-a471-4f8b-85c4-d9c0a4642da2
incomplete | 21
2e1d2fc1-f791-4e0c-ab70-a126274f2dd4
complete | A database backup is needed in a Blue-Green Deployment to migrate host machines, but an IT snapshot is not necessary. |
| User-Managed Mongo or SQL Server Backup | Minimum | Better | Best | Benefit |
| Back up the database | 22
97ac1b1e-2105-4dd3-8b91-69cf937331e3
incomplete | 23
794bc948-7a00-455c-8868-dd6eb3a59d5a
incomplete | 24
02ecff4b-24e2-4450-873b-f7aa4c8f413f
complete | Upgrades perform schema migrations on database collections.  The original database is required to be able to roll back the upgrade. |
| Upgrade | Minimum | Better | Best | Benefit |
| In-place upgrade of Production Server | 25
7112af73-6630-4c90-94a0-d669f9e52be9
complete | 26
9887df13-04fc-4f79-b7f0-aea3e247c5f2
incomplete | 27
9346d789-1f14-4237-ba6c-6bafb5ec55f4
incomplete | Easiest to perform, but highest risk of unpredictable Production Server downtime in the event of an upgrade issue. |
| Test upgrade on a Sandbox server | 10
7e72394f-2777-47c4-8ab9-2441f4ba1bf2
incomplete | 11
b77d80bc-ddee-40d8-b80e-e5c7a84d9dbc
complete | 12
bba82cdf-0553-4b44-99d0-56d38eeeaff5
incomplete | Flush out upgrade issues and test critical workflows and Connectors in the upgraded environment without Production downtime.Steps:  Perform a Host Recovery to a Sandbox environment:Embedded Mongo - copy the database to the SandboxUser-Managed Database - point Sandbox to a duplicate of the databaseAfter validation of upgrade process, perform an in-place upgrade of Production Server. |
| Blue-Green deployment | 28
defdcd30-6da4-4494-8866-cf770bbd7d1b
incomplete | 29
fcaf6000-0189-4036-b8ed-11ecc62914f5
incomplete | 30
0b8fdcbc-fee1-4cc2-8be4-cb72c4a123c3
complete | Blue-Green deployment eliminates risk and potential downtime of an in-place upgrade as the original Production environment is not upgraded.Steps:  Perform a Sandbox upgrade, validate critical workflows, then switch the URL of your Production environment to the Sandbox (making it the new Production environment). |

Ideally, start with the same-version Sandbox/Dev/Test Server and upgrade it. See the [Alteryx Server Sandbox Environment Community](https://knowledge.alteryx.com/index/s/article/Alteryx-Server-Sandbox-Environment) article for more information on Sandbox environments.

If you have a **multi-node environment**, testing is still effective on a single machine that runs Controller + Server UI + Worker. Similarly, if you have a User-Managed MongoDB, restoring a database backup to the test machine's embedded MongoDB can help validate the upgrade. Contact your Account Executive for information on a Sandbox license.

At a bare minimum, you should install the target version of Designer on a user's machine to test critical workflows in the new version. Instructions can be found on the [Install Two Versions of Designer on the Same Machine](https://help.alteryx.com/current/en/license-and-activate/license-and-activate-with-license-keys/install/install-two-versions-of-designer-on-the-same-machine.html) help page.

### 1. Perform a Backup

Perform a backup for:

- Embedded MongoDB
- User-Managed MongoDB and SQL(Note: Creating a backup is the responsibility of the customer and is not supported by Alteryx.)
   - (Note: Creating a backup is the responsibility of the customer and is not supported by Alteryx.)

- RuntimeSettings.xml
- Controller Token saved to a TXT file

### 2. Complete Pre-Upgrade Checks

- You can avoid many Server upgrade problems by performing the pre-upgrade checks/workflow found in the Alteryx Server: Pre-Upgrade Checks Community article. This procedure addresses the most common issues a client will face performing an upgrade and lists the recommended solutions/steps for each.It’s important to run the pre-upgrade checks in each of your environments before performing the upgrade. For example, you are testing on a development machine, then you’ll want to rerun the checks on your prod environment and take the indicated steps before completing that upgrade.
- rollbackAs part of pre-upgrade planning, agree with business stakeholders on how long the upgrade will be allowed to run before triggering a rollback.
- For best results, avoid combining a Server upgrade with a host migration. Perform one operation at a time, allowing for a full validation period after completion before starting the next.

**Disable Scheduler on Worker Nodes During Upgrades**

By default, schedules that should have run while the Server was being upgraded will pick up as soon as the Server and nodes restart. Keep this in mind when running the test upgrade on your Sandbox, as you likely don’t want to have workflows kick off and impact your production systems.

We recommend to disable all schedules prior to upgrade and determine what should run on an individual basis.

If you do not want schedules to run when the Service starts:

1. Run Alteryx System Settings on each Worker (and main Server node).
2. Deselect Worker > General > Run unassigned jobs.
3. Give the Worker a unique Job tag (for instance “UPGRADETESTING”).

Alternatively, contact [Customer Support](https://community.alteryx.com/t5/Support/bd-p/SupportPage) for assistance in deleting all schedules.

### 3. Perform the Upgrade

Performing the upgrade is a straightforward process if you are upgrading in place. There are different steps if you are performing a fresh installation of the new version on a target machine, which include the application of licenses which is not part of the upgrade path; existing active licenses continue to work on upgraded machines without intervention. The general upgrade steps are shown on the [Install or Upgrade Server](https://help.alteryx.com/current/en/server/install/install-or-upgrade-server.html).

The different instructions for new installations and upgrades-in-place are detailed, and the document includes links to associated help files/articles on licensing, system requirements, preparatory checklists, MongoDB upgrades, and more. Many of these are included in the [Guides and Help Articles](https://help.alteryx.com/current/en/server/best-practices/server-upgrade-best-practices.html#guides-and-help-articles) section at the end of this document.

Please note that Predictive Tools should be upgraded with the main installation. If you had a Service Log-On User set, you need to set it again after the upgrade; upgrades remove and reinstall the Alteryx Service.

**Upgrading a Multi-Node Environment**

In multi-node environments, all nodes should be upgraded to the same version and nodes must be shut down in the order shown in the Shutdown section of the document in the [How to Restart the Services in a Multi-Node Alteryx Server](https://knowledge.alteryx.com/index/s/article/How-to-restart-the-services-in-a-multi-node-Alteryx-Server) Community article.

After you upgrade all nodes, follow the proper restart order listed in the Startup section of the same document.

Once everything is up and running, upgrade any connectors, data packs, drivers, add-ons (such as Intelligence Suite), and third-party tools that need to be.

### 4. Test the Upgrade

Now that the Server software and any applicable connectors have been upgraded, it is time to start testing.

#### Alteryx Services

The first tests are basic and you can find them in the Test section of the [Server Upgrade Checklist](https://help.alteryx.com/current/en/server/install/install-or-upgrade-server/server-upgrade-checklist.html#test).

Can you:

- Is the Alteryx Service running?
- Can you:Access the Server URL?Move around Admin pages and view Users, Collections, and so on?Publish a workflow from Designer to the Server?Run the workflow?If your configuration allows, save and run a workflow specifying your credentials?
   - Access the Server URL?
   - Move around Admin pages and view Users, Collections, and so on?
   - Publish a workflow from Designer to the Server?
   - Run the workflow?
   - If your configuration allows, save and run a workflow specifying your credentials?

#### Configuration Options

Next, examine the configuration options in the [Alteryx System Settings](https://help.alteryx.com/current/en/server/configure/system-settings.html) configuration tool to ensure that no settings have been lost. These settings were documented in the [Document Your Environment](https://help.alteryx.com/current/en/server/best-practices/server-upgrade-best-practices.html#section-1--document-your-environment) section. If there are any changes you need to make, such as persistence settings, SMTP configurations, etc., now is a time to make them. Also, make a note of these changes to reuse them in your upgraded production environment.

**Note**

Some settings are actively changed across some of the upgrades. For example, 2022.1 set AMP on for the Server and changed the number of workflows allowed to run simultaneously.

Always check the [Release Notes](https://help.alteryx.com/document/preview/6553144#UUID-28bacd4e-baa7-668c-2b44-df710f9c19ff) for more information.

#### Connectors and Drivers

The next step is to test your connectors and drivers to critical systems, such as SharePoint and O365 connectors, and ODBC/OleDB connectors to SQL Server, Snowflake, Databricks, and so on. Make sure you can connect, read, and write data.

#### Critical Workflows

Now, test your business-critical flows and flows that utilize the connectors also documented in the [Document Your Environment](https://help.alteryx.com/current/en/server/best-practices/server-upgrade-best-practices.html#section-1--document-your-environment) section. This set of tests will use the test versions of the workflows created in the [Create Test Versions of Business-Critical Workflows](https://help.alteryx.com/current/en/server/best-practices/server-upgrade-best-practices.html#UUID-cf459da5-0bc8-a982-cd40-acaa7b2326f0_N1696851842778) section of this guide. If you run unmodified productions versions of these flows, then your production destinations will be impacted as if these flows were running normally.

#### Scheduler and Server UI

Finally, if you are running Scheduler and Server UI, test these as well:

- Can a workflow be scheduled, and does it run?
- Do analytic apps run correctly?

**Important**

Make sure that any apps you publish and schedule/run in this environment are not production versions. If you run unmodified productions versions of these flows, then your production destinations will be impacted as if these flows were running normally.

#### 5. Note Any Errors and Get Help

Catalog any problems your testing uncovers, such as:

- Services that don’t start or report errors.
- MongoDB Schema or crypto migrations that fail.
- Workflows that don’t run or run with unexpected results or errors.
- Connectors that don’t work.
- Errors in MongoDB.

The [Server Upgrade Checklist](https://help.alteryx.com/current/en/server/install/install-or-upgrade-server/server-upgrade-checklist.html) includes some common troubleshooting steps in the last section. Customer Support can assist if you experienced an error in the upgrade process and are unable to resolve it with the common troubleshooting steps shown in the guide. Your Account Executive can provide options if you would like assistance planning or executing an upgrade.

#### 6. Perform a Rollback/Restore

If you were unable to resolve issues that were uncovered during the Test and Quality Control phase, it’s time to do a rollback or restore. Prior to rolling back or restoring, you might wish to gather log files from the Server machines to provide to Customer Support or for internal review prior to the next upgrade attempt. If you have a snapshot/backup, you can revert to it now, and plan your next upgrade attempt. If a snapshot methodology was not possible, then you can follow the conventional rollback methodology shown in the [How To: Downgrade Alteryx Server Community article](https://community.alteryx.com/t5/Alteryx-Server-Knowledge-Base/How-To-Downgrade-Alteryx-Server/ta-p/796943).

## Section 6. Schedule the Production Upgrade

Once you have successfully tested the upgrade in your non-production environment, and have your documented upgrade process, it is time to plan your production environment upgrade.

**Note**

Your production upgrade should follow the “script” you created in your test environment, with changes specific to any architectural differences between the environments. For example, if your tested environment was a single node architecture, but your production environment has separate nodes for Workers and Server UI, then the production environment will have additional installation steps. Be aware of this as you plan.

**Pro tip**: Use [Server Notifications](https://help.alteryx.com/current/en/server/administer-alteryx-server/notifications.html#add-a-system-message) via Alteryx Server UI as an additional communication channel to inform users about pending upgrades. You can also post upgrade information in your internal Alteryx community (for example, SharePoint, Confluence, Yammer, Teams, and so on).

You’ll need to schedule an appropriate amount of downtime and inform the users that workflows on Server will not be running during the upgrade. For business-critical flows, users can run them in your newly upgraded test environment, run them locally, or simply plan for the outage and inform affected downstream audiences about the delay.

If you are planning to upgrade Designer as well, whether via packaging/automation methods or manual installation process, plan for the extra time and resources needed to complete the installations, and be sure to inform your user base as well. Remember that Server is backward-compatible with Designer, up until version 2022.3, but newer versions of Designer do not work with older versions of Server. So, Server upgrades must always precede Designer upgrades.

**Important**

Remember to plan your upgrade at a time that minimizes disruptions to your business. Refer to [Additional Considerations](https://help.alteryx.com/current/en/server/best-practices/server-upgrade-best-practices.html#UUID-cf459da5-0bc8-a982-cd40-acaa7b2326f0_N1696852195442) for more detail and recommendations.

## Section 7. Perform the Production Upgrade

### Upgrade Server

The high level steps in this section are a mirror to steps 1-6 in [Section 5](https://help.alteryx.com/current/en/server/best-practices/server-upgrade-best-practices.html#section-5--perform-a-sandbox-dev-server-upgrade-and-test-the-results). Refer to the appropriate section for additional details and help links.

- Back up the environment.
- Complete pre-upgrade checks.
- Perform the upgrade, including connectors, data packs, add-ons, third-party tools, etc.
- Test/Quality Control the upgrade.Check that scheduled jobs, collections, memberships, and published workflows still exist and work as expected (where testable).
   - Check that scheduled jobs, collections, memberships, and published workflows still exist and work as expected (where testable).

- Note any errors and get help from Customer Support.
- Perform a rollback/restore if necessary.Restore from backups or follow the process in the How to: Downgrade Alteryx Server Community article.
   - Restore from backups or follow the process in the How to: Downgrade Alteryx Server Community article.

### Upgrade Designer (optional)

Once your production environment is up and running, you can upgrade your Designer installs if they are part of your plan. Remember that Designer cannot be a greater version than what has been installed on Server, and this upgrade affects users’ machines directly. Steps for upgrading Designer can be found in [Upgrade Designer](https://help.alteryx.com/current/en/designer/get-started/activate-designer/upgrade-designer.html).

**Designer and Server Compatibility**

Designer version needs to be equal to or older than the Server version it connects to. **The exception is Server 2022.3 (or later) which requires at least Designer 2022.3 due to changes in encryption**.

Designer version can NOT be newer than the Server it connects to.

Only the version (Year.Release) needs to match, not the specific patch.

Similar to a Server upgrade, upgraded Designer versions should be tested to ensure that workflows continue to run, and that connections to Server can still be made. As with the best practices for Server upgrades, plan to test your Designer upgrade on a small subset of user machines.

## What to Do if the Upgrade Fails

If the Server upgrade fails or encounters critical issues, do not repeatedly restart the upgrade process. Instead, follow the steps below to collect diagnostic information and engage Alteryx Customer Support for assistance.

#### 1. Stop All Alteryx Services

Before collecting logs, stop all Alteryx services to ensure log integrity.

1. Open Services.msc.
2. Locate and stop AlteryxService.

#### 1. Submit a Support Ticket

Open a case with Alteryx Customer Support and include the following details:

- The Server version you upgraded from and to.
- A brief description of what failed (for example, during installation, after service start, or during schema migration).
- Environment type: Is this a single-node or multi-node configuration?
- Was a host migration attempted at the same time as the upgrade?
- The timestamp of failure and any visible error messages.
- Have any troubleshooting steps already been taken? (for example, service restart, system reboot, or manual rollback attempt)
- Do you have a current backup of your Alteryx Server and database data?

Include the log files listed below from the past 24 hours (or the 2–3 most recent log files) for troubleshooting:

- Server (Gallery) LogsLocation: \%ProgramData%\Alteryx\Gallery\Logs
- Service LogsLocation: \%ProgramData%\Alteryx\Service\AlteryxServiceLog.logInclude multiple .log files if available.
- RuntimeSettings.xmlLocation: \%ProgramData%\Alteryx\RuntimeSettings.xml
- AlteryxGallery schema migration log configured in Alteryx System Settings > Server UI > Logging Directory.Versions 2025.1 and later: C:\ProgramData\Alteryx\Service\alteryx-gallery-migration.csvVersions 2024.2 and earlier: C:\ProgramData\Alteryx\Gallery\Logs\alteryx-migration.csv
   - Versions 2025.1 and later: C:\ProgramData\Alteryx\Service\alteryx-gallery-migration.csv
   - Versions 2024.2 and earlier: C:\ProgramData\Alteryx\Gallery\Logs\alteryx-migration.csv

- AlteryxService schema migration log configured in Alteryx System Settings > Controller > Logging.Versions 2025.1 and later: C:\ProgramData\Alteryx\Service\alteryx-service-migration.csvVersions 2024.2 and earlier: C:\ProgramData\Alteryx\Service\alteryx-migration.csv
   - Versions 2025.1 and later: C:\ProgramData\Alteryx\Service\alteryx-service-migration.csv
   - Versions 2024.2 and earlier: C:\ProgramData\Alteryx\Service\alteryx-migration.csv

- AlteryxServiceMigrator_#.logs (required when upgrading to or through version 2022.3), found in:Prep Tool logs: C:\ProgramData\Alteryx\ServiceService start logs: Alteryx System Settings > Controller > General > Logging folder
   - Prep Tool logs: C:\ProgramData\Alteryx\Service
   - Service start logs: Alteryx System Settings > Controller > General > Logging folder

#### 2. Additional Information (Optional)

If possible, also include:

- Screenshots of any error messages.
- Installer log files (typically in %TEMP% or %LOCALAPPDATA%\Temp).
- Output from any Migration Prep Tool used before the upgrade.

#### 3. Next Steps

- Do not uninstall or roll back unless directed by Alteryx Support.
- Wait for Support to analyze the logs and provide remediation guidance.

## Guides and Help Articles

On this list you can find links to all resources mentioned in this document, as well as additional resources which can be helpful in the Server upgrade process.

### Preparing for an Upgrade

- Release Notes
- Server System Requirements
- Server System Settings
- Software and Licensing Portal
- Download and Install a Product
- Version-to-Version Server Upgrade Guide: Supported Versions
- Version-to-Version Server Upgrade Guide: Unsupported Versions
- Alteryx Server: Pre-Upgrade Checks
- Migration Prep Tool for Upgrading to Version 2023.2

### Backing Up and Restoring Your Environment

- Critical Server Files and Settings to Backup
- MongoDB Backups
- Backup & Recovery Best Practices Part 1
- Backup & Recovery Best Practices Part 2
- Restoring Your Environment After a Failed Upgrade

### Performing an Upgrade

- Install or Upgrade Server
- Installing a Test Environment on a Designer Machine
- Server Upgrade Checklist
- How to Stop and Restart the Services in a Multi-Node Alteryx Server
- Add a System Message for Users
- Upgrade Designer

### Additional Resources

- Alteryx Server Sandbox Environment
- Alteryx Server Upgrade Guidelines - Checklist to upgrade Alteryx Server (Previous version)
- Alteryx Support Portal
- Alteryx Server Architectures
- Alteryx Server Workload Management
- Alteryx Server 101 Introduction
- Alteryx Server 101 Admin
- Alteryx Automated Deployment