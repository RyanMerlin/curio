---
id: 85a5cc64639539f6
title: LIST OF UTILITY WORKFLOWS
status: intake
source:
  kind: confluence_page
  id: confluence-page:2200306970
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/2200306970
  summary: null
category: []
keywords: []
created_at: 2026-04-14T15:02:19Z
updated_at: 2026-04-14T15:02:19Z
confidence: null
cross_refs: []
content_hash: sha256:36bfcbc8c737a79cfd48a0753991a0b8f60ff12127719629e576b6d87bd40cf1
confluence_page_id: null
model_used: null
---

> **ℹ️ Info**
>
> List of some  key Utility workflows.
> 
> See the **Customer Support Gallery** for additional Support Workflows
> 
> - http://ayx-gcs-gal-01/gallery          <== Support Gallery
> - 00 GCS Gallery Workflow List.yxmd <== XLSX of all workflows w/ descriptions

---

---

note 4a0100a8-7f61-4bcb-8c94-0791127f7a32 Version Compatibility column definitions

–** ServiceDataParser **– 23.1 or prior containing **__ServiceData** blob field called that was unpacked by a macro, 
                                         Mongo Input Tool 
– **Mongo                   **– accesses Mongo

Version Compatibility column definitions

–** ServiceDataParser **– 23.1 or prior containing **__ServiceData** blob field called that was unpacked by a macro, 
                                         [Mongo Input Tool](/wiki/spaces/SupportServer/pages/1702763531/Mongo+Input+Tool#%E2%80%A6unpack-the-__ServiceData-/-ServiceData-blob) 
– **Mongo                   **– accesses Mongo

| Utility workflow | Access | Version Compatibility |
| --- | --- | --- |
| Alteryx Server Windows Authentication User Health CheckReaches out to Windows Active Directory to compare AD record to what’s currently in MongoDB. | Alteryx Server Windows Authentication User Health Check (internal KB)Test this before giving to a customer as it was developed a few versions ago and is a complex workflow | Mongo |
| Calculate size of job output files in MongoIdentify workflows that are outputting large data files locally and bloating Mongo | Utility Workflow - Identifying and cleaning up large result files from Mongo | Mongo |
| Check Corrupt Schedules EmbeddedLooks for corrupt schedules by getting each schedule from the API and reporting Schedules that don’t return a 200.  Created by Tim R. | Utility Workflow - Check Corrupt Schedules Embedded | Mongosql db |
| Check for Corrupt SchedulesFind corrupt schedules in AS_Schedules collection | Check for Corrupt Schedules.yxmd (Support Gallery) | tbd |
| Count User Collection Assets WinAuth \| Count User Collection Assets SAMLGet how many workflows a user has access to via collections. This helps to troubleshoot defect TGAL-6357 | Count User Collection Assets WinAuth (Support Gallery)Count User Collection Assets SAML (Support Gallery)DefectTGAL-6357: Open Workflow From Gallery Window Immediately Exits (Lucene)DONE [fixed 23.1]TGAL 6357 Open Workflow From Gallery Window Immediately Exits (KB)Gallery Log Error - too many boolean clauses ---> Lucene.Net.Search.BooleanQuery+TooManyClauses: maxClauseCount is set to 1024Limitation – if the user has a Collection shared with them based on being a member of an AD Group the workflow won’t know about it (the workflow doesn’t have access to AD).  So it won’t know to “count” the workfows in that collection. | Mongo |
| Credentials | List of users for each Credential by Ravi Kuppina List of workflows using Credentials by London Hhttp://ayx-gcs-gal-01/gallery/#!/app/Get-User-Credential-Workflows/6529afc5382bc304196daeb8 |  |
| Customer Managed Telemetry Enterprise UtilityPublic tool introduced in 23.1 to push Telemetry data into Tableau or PowerBI | Telemetry Utility Workflow - CustomerManagedTelemetry_UserStats (Designer Usage)  <== option when customer doesn’t have Tableau or PowerBI | 23.1+ |
| CustomerManagedTelemetry_UserStatsSummarize Designer Usage Telemetry | Utility Workflow - CustomerManagedTelemetry_UserStats (Designer Usage) | 23.1+ |
| Delete_Users_WorkflowsDelete all of a user’s active workflows | Utility Workflow - Delete_Users_Workflows | Mongo |
| Dump_DCMEDump the 4 DCMe collections and check referential integrity | Explore DCM collections - Referential Integrity and What's Shared to a User | Mongo |
| Find Corrupt Gallery Database ConnectionsTroubleshoot Gallery Database Connection issues and syncing issues | Utility Workflow - Find Corrupt Gallery Database Connections | MongoAPI |
| Find Orphaned Schedules and WorkflowsIdentify orphaned workflow schedules. The outputs will return the orphaned schedule ID. Orphaned schedules should be assigned a workflow or removed. | Find Orphaned Schedules and Workflows.yxmd          <== Support GalleryFind Orphaned Schedules and Workflows APP.yxwz   <== Support Gallery | unk |
| Get Collection Users AD \|Get Collection Users SAMLGet a list of Collection Users | Utility Workflow - Get Collection Users Limitation – does not list users from groups added to the Collection | Mongo |
| Get Collection Sizes | Utility Workflow - Get Collection Sizes - mongo_sizes.yxmd Gets the size of each collection in the databases | Mongo |
| Get_Workflow_Owners_Created_and_Modified_DatesGet list of all workflows with their owner, created date, and modified date | Get_Workflow_Owners_Created_and_Modified_Dates (Support Gallery) | Mongo |
| How to use the API to change multiple schedulesThe Schedule GET/PUT API calls are incompatible, rendering them useless.  Jenn P created a workflow that allows the user to change multiple schedules (ex: move to a different Worker tag) using the API | How to use the API to update one or multiple schedules, and/or transfer ownership of schedules from one user to another | Mongo |
| Job Runtime DetailsShow Job Queue Time vs Actual Execution Time | Analyze Server Job Runtime Details (KB)  <== Has versions with and without the ServiceDataParserJob Runtime Details (Support Gallery) | Mongo |
| List_Users_DCM_Connections_Shared_WithGet a list of the users DCM Connections have been shared with | Older versions | Mongo Input Tool |
| Role Set CommandCreate MongoDB Command to change the Role for a set of user emails | Utility Workflow - Role Set Command |  |
| SearchServerWorkflowXMLSearch for specific text in workfow XML, like Data Connection name.  Update of the classic Server Ripper utility. [by London H] | Utility Workflow - SearchServerWorkflowXML | tbd |
| User Usage / Activity Level / TelemetryUses auditEvents to determine activity level by day, total activity, and first/last login | Utility Workflow - Server Usage Telemetry | Mongo |
| validate_appInfos_AS_ApplicationVersionsCompares versions in appInfos to AS_ApplicationVersions.  Mismatches cause issues. | Utility Workflow - validate_appInfos_AS_ApplicationVersions | Mongo |
| Validate_Mongo_Dump_Collection_CountsValidate mongoDump.log collection counts against MongoDB collection counts to ensure the dump includes all records from the original database. | Utility Workflow - Validate_Mongo_Dump_Collection_Counts | Mongo |
| Workflow SizeGet the size of each workflow in the database, including assets | Utility Workflow - Workflow Size |  |
| Workflows Run Count and Last Run DateGets a simple list of the run count and last run date for workflows | 23.2+ (no __ServiceData blob) Older versions 23.1- (__ServiceData blob)Workflows Run Count and Last Run Date.yxmd (Support Gallery) | Mongo |