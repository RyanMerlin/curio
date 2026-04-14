---
id: 85a5cc64639539f6
title: LIST OF UTILITY WORKFLOWS
status: review
source:
  kind: confluence_page
  id: confluence-page:2200306970
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/2200306970
  summary: null
category:
- product-tree
- alteryx-server
keywords:
- utility-workflows
- gallery
- tools
- list
created_at: 2026-04-14T15:18:28Z
updated_at: 2026-04-14T15:20:19Z
confidence: 0.75
cross_refs: []
content_hash: sha256:e607fdebc5018e5a7b79c6e05056d2b048f7122746a8cd17cc846256811c1061
confluence_page_id: null
model_used: claude-sonnet-4-6
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

Version Compatibility column definitions

–** ServiceDataParser **– 23.1 or prior containing **__ServiceData** blob field called that was unpacked by a macro, 
                                         [Mongo Input Tool](https://alteryx.atlassian.net/wiki/search?text=Mongo+Input+Tool) 
– **Mongo                   **– accesses Mongo

| **Utility workflow** | **Access** | **Version Compatibility** |
| --- | --- | --- |
| ### Alteryx Server Windows Authentication User Health Check  Reaches out to Windows Active Directory to compare AD record to what’s currently in MongoDB. | [Alteryx Server Windows Authentication User Health Check](https://alteryx.lightning.force.com/kA02R000000CuzySAC)(internal KB)  Test this before giving to a customer as it was developed a few versions ago and is a complex workflow | Mongo |
| ### Calculate size of job output files in Mongo  Identify workflows that are outputting large data files locally and bloating Mongo | [Utility Workflow - Identifying and cleaning up large result files from Mongo](https://alteryx.atlassian.net/wiki/search?text=Utility+Workflow+-+Identifying+and+cleaning+up+large+result+files+from+Mongo) | Mongo |
| ### Check Corrupt Schedules Embedded  Looks for corrupt schedules by getting each schedule from the API and reporting Schedules that don’t return a 200.  Created by Tim R. | [Utility Workflow - Check Corrupt Schedules Embedded](https://alteryx.atlassian.net/wiki/search?text=Utility+Workflow+-+Check+Corrupt+Schedules+Embedded) | Mongo sql db |
| ### Check for Corrupt Schedules  Find corrupt schedules in **AS_Schedules **collection | [Check for Corrupt Schedules.yxmd](http://ayx-gcs-gal-01/gallery/#!app/Check-for-Corrupt-Schedules/63d971e01a47b1c75f4e0543) (Support Gallery) | tbd |
| ### Count User Collection Assets WinAuth \| Count User Collection Assets SAML  Get how many workflows a user has access to via collections. This helps to troubleshoot defect TGAL-6357 | [Count User Collection Assets WinAuth](http://ayx-gcs-gal-01/gallery/#!/app/Count-User-Collection-Assets-WinAuth/65288ab4382bc304191a585c) (Support Gallery) [Count User Collection Assets SAML](http://ayx-gcs-gal-01/gallery/#!/app/Count-User-Collection-Assets-SAML/652875bf382bc3041901e4e2) (Support Gallery)  Defect     - TGAL-6357: Open Workflow From Gallery Window Immediately Exits (Lucene)DONE [fixed 23.1]    - TGAL 6357 Open Workflow From Gallery Window Immediately Exits (KB)    - Gallery Log Error - too many boolean clauses ---> Lucene.Net.Search.BooleanQuery+TooManyClauses: maxClauseCount is set to 1024  > **📝 Note** > > Limitation – if the user has a Collection shared with them based on being a member of an AD Group the workflow won’t know about it (the workflow doesn’t have access to AD).  So it won’t know to “count” the workfows in that collection. | Mongo |
| ### Credentials | List of users for each Credential by Ravi Kuppina  List of workflows using Credentials by London H     - http://ayx-gcs-gal-01/gallery/#!/app/Get-User-Credential-Workflows/6529afc5382bc304196daeb8 |  |
| ### Customer Managed Telemetry Enterprise Utility  Public tool introduced in 23.1 to push Telemetry data into Tableau or PowerBI | [Telemetry](https://alteryx.atlassian.net/wiki/spaces/SupportDesigner/pages?title=Telemetry)  [Utility Workflow - CustomerManagedTelemetry_UserStats (Designer Usage)](https://alteryx.atlassian.net/wiki/search?text=Utility+Workflow+-+CustomerManagedTelemetry_UserStats+(Designer+Usage))  <== option when customer doesn’t have Tableau or PowerBI | 23.1+ |
| ### CustomerManagedTelemetry_UserStats  Summarize Designer Usage Telemetry | [Utility Workflow - CustomerManagedTelemetry_UserStats (Designer Usage)](https://alteryx.atlassian.net/wiki/search?text=Utility+Workflow+-+CustomerManagedTelemetry_UserStats+(Designer+Usage)) | 23.1+ |
| ### Delete_Users_Workflows  Delete all of a user’s active workflows | [Utility Workflow - Delete_Users_Workflows](https://alteryx.atlassian.net/wiki/search?text=Utility+Workflow+-+Delete_Users_Workflows) | Mongo |
| ### Dump_DCME  Dump the 4 DCMe collections and check referential integrity | [Explore DCM collections - Referential Integrity and What's Shared to a User](https://alteryx.atlassian.net/wiki/search?text=Explore+DCM+collections+-+Referential+Integrity+and+What's+Shared+to+a+User) | Mongo |
| ### Find Corrupt Gallery Database Connections  Troubleshoot Gallery Database Connection issues and syncing issues | [Utility Workflow - Find Corrupt Gallery Database Connections](https://alteryx.atlassian.net/wiki/search?text=Utility+Workflow+-+Find+Corrupt+Gallery+Database+Connections) | Mongo API |
| ### Find Orphaned Schedules and Workflows  Identify orphaned workflow schedules. The outputs will return the orphaned schedule ID. Orphaned schedules should be assigned a workflow or removed. | [Find Orphaned Schedules and Workflows.yxmd](http://ayx-gcs-gal-01/gallery/#!/app/Find-Orphaned-Schedules-and-Workflows/645d69601a47b1c75f4f0caa)          <== Support Gallery [Find Orphaned Schedules and Workflows APP.yxwz](http://ayx-gcs-gal-01/gallery/#!/app/Find-Orphaned-Schedules-and-Workflows-APP/645d6c9f1a47b1c75f4f0e3f)   <== Support Gallery | unk |
| ### Get Collection Users AD \|Get Collection Users SAML  Get a list of **Collection Users** | [Utility Workflow - Get Collection Users](https://alteryx.atlassian.net/wiki/search?text=Utility+Workflow+-+Get+Collection+Users)  > **📝 Note** > > Limitation – does not list users from groups added to the Collection | Mongo |
| ### Get Collection Sizes | [Utility Workflow - Get Collection Sizes - mongo_sizes.yxmd](https://alteryx.atlassian.net/wiki/search?text=Utility+Workflow+-+Get+Collection+Sizes+-+mongo_sizes.yxmd)  Gets the size of each collection in the databases | Mongo |
| ### Get_Workflow_Owners_Created_and_Modified_Dates  Get list of all workflows with their owner, created date, and modified date | [Get_Workflow_Owners_Created_and_Modified_Dates](http://ayx-gcs-gal-01/gallery/#!/app/Get_Workflow_Owners_Created_and_Modified_Dates/6598cbc7a43c06b035f79847) (Support Gallery) | Mongo |
| ### How to use the API to change multiple schedules  The Schedule GET/PUT API calls are incompatible, rendering them useless.  Jenn P created a workflow that allows the user to change multiple schedules (ex: move to a different Worker tag) using the API | [How to use the API to update one or multiple schedules, and/or transfer ownership of schedules from one user to another](https://alteryx.atlassian.net/wiki/search?text=How+to+use+the+API+to+update+one+or+multiple+schedules,+and/or+transfer+ownership+of+schedules+from+one+user+to+another) | Mongo |
| ### Job Runtime Details  Show **Job Queue Time** vs **Actual Execution Time** | [Analyze Server Job Runtime Details](https://knowledge.alteryx.com/index/s/article/Alteryx-Server-Job-Runtime-Details) (KB)  <== **Has versions with and without the ServiceDataParser** [Job Runtime Details](http://ayx-gcs-gal-01/gallery/#!/app/Job-Runtime-Details/641c90d21a47b1c75f4ea03c) (Support Gallery) | Mongo |
| ### List_Users_DCM_Connections_Shared_With  Get a list of the users DCM Connections have been shared with | Older versions | Mongo Input Tool |
| ### Role Set Command  Create MongoDB Command to change the Role for a set of user emails | [Utility Workflow - Role Set Command](https://alteryx.atlassian.net/wiki/search?text=Utility+Workflow+-+Role+Set+Command) |  |
| ### SearchServerWorkflowXML  Search for specific text in workfow XML, like Data Connection name.  Update of the classic Server Ripper utility. [by London H] | [Utility Workflow - SearchServerWorkflowXML](https://alteryx.atlassian.net/wiki/search?text=Utility+Workflow+-+SearchServerWorkflowXML) | tbd |
| ### User Usage / Activity Level / Telemetry  Uses auditEvents to determine activity level by day, total activity, and first/last login | [Utility Workflow - Server Usage Telemetry](https://alteryx.atlassian.net/wiki/search?text=Utility+Workflow+-+Server+Usage+Telemetry) | Mongo |
| ### validate_appInfos_AS_ApplicationVersions  Compares versions in appInfos to AS_ApplicationVersions.  Mismatches cause issues. | [Utility Workflow - validate_appInfos_AS_ApplicationVersions](https://alteryx.atlassian.net/wiki/search?text=Utility+Workflow+-+validate_appInfos_AS_ApplicationVersions) | Mongo |
| ### Validate_Mongo_Dump_Collection_Counts  Validate mongoDump.log collection counts against MongoDB collection counts to ensure the dump includes all records from the original database. | [Utility Workflow - Validate_Mongo_Dump_Collection_Counts](https://alteryx.atlassian.net/wiki/search?text=Utility+Workflow+-+Validate_Mongo_Dump_Collection_Counts) | Mongo |
| ### Workflow Size  Get the size of each workflow in the database, including assets | [Utility Workflow - Workflow Size](https://alteryx.atlassian.net/wiki/search?text=Utility+Workflow+-+Workflow+Size) |  |
| ### Workflows Run Count and Last Run Date  Gets a simple list of the run count and last run date for workflows | **23.2+ (no __ServiceData blob)**  Older versions  ---  **23.1- (__ServiceData blob)**     - Workflows Run Count and Last Run Date.yxmd (Support Gallery) | Mongo |