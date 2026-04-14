---
id: cb7c556a7dc01feb
title: Example MongoDB Queries / Commands
status: intake
source:
  kind: confluence_page
  id: confluence-page:1702828808
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/1702828808
  summary: null
category: []
keywords: []
created_at: 2026-04-14T15:06:10Z
updated_at: 2026-04-14T15:06:10Z
confidence: null
cross_refs: []
content_hash: sha256:1c79e9431b7c7658d354dc947c2254191d895aaadb1653e8b6323755eb5b306e
confluence_page_id: null
model_used: null
---

|  |  |
| --- | --- |
|  |  |

---

---

|  |  |  |
| --- | --- | --- |
|  |  |  |
|  |  |  |
|  |  |  |
|  |  |  |

AlteryxGalleryappinfosMark a workflow IsDeleted.  Get the WorkfowID from the URL when viewing the workflow in Server UI. The workflow may still appear in the user's Studio but present a "Page Not Found" error when clicked on.  If this occurs, you'll need to Reindex MongoDB Search across ALL elements of the Revisions array for a DateCreated that’s nullFind all “deleted” workflowsTagIds array queries23.1+ Mongo appInfos indexing can fail due to bad TagIds values, see:  How to Troubleshoot a Failed 23.1/23.2 Reindex Find records where the TagIds array length is greater than 1 1" })]]>Find TagIds arrays that contain ““ (empty string)Remove empty tags from the TagIds arrayFind TagIds arrays that are not empty and do not contain ““ (empty string) value  0"},{  TagIds : {$nin:[""]} } ] })]]>Replace a bad TagId array value (STRING_FROM_ERROR) with the correct tags.objectID (TAGS_COLLECTION_OBJECTID)auditEventsFind audit events on a user recordFind oldest auditEvent record:db.getCollection("auditEvents").find().sort({ "Timestamp" : 1 }).limit(1)collectionsFind a Collection based on the URL collecton ID (which is NOT the collection ObjectID)Find all Collections containing a specific Application (grab the ID from the URL when viewing the Workflow)dataConnectionsFind data connections with long encrypted connection strings= 1024" })]]>sessionsDelete old sessions records Delete sessions over 30 days oldsubscriptionsSet null expiration dates to a future dateGet list of user’s in a Subscription (per ChatGPT)usersFind nullsUpdate nullsFind specific user, note: searches are case senstive, use slashes and “i” to make them case-insensitiveFind duplicate emailsFind users in a specific Private StudioUpdate user’s Private StudioUpdate a user’s AD SIDHow to understand SID and How to replace SID Update NULL timezones with a specific timezone copied from another record, replace Amnerica/Chicago with a timezone value another user has set in their user recordGet a list of all user emailsReturn only specific fields (if getting data in a JSON object, use quotes, ex: “WindowsIdentity,Sid”)Set Role to NoAccess for users whose last login was 2024 or earlierSet Active flag to FALSE for users whose last login was 2024 or earlier.       Note:  Reindex after running this, otherwise the users will still appear in Admin > Users, Reindex MongoDB Investigate reasons preventing a user from logging inFind locked accounts in Built-in AuthFind inactive users (once marked InActive they won’t appear in Server UI but the user will be blocked from logging in)Find IsDeleted users, these will have had PII removed and will not be blocking a user from logging in, however you’d need to resurrect the record to get the user back to their original studio and ownership of workflowsversionsDetermine current Schema versionAlteryxServiceAS_Results23.2 changes the schema and replace __ServiceData blob with OutputLog. blob.  The latter can also be unpacked with the ServiceDataParser.yxmc. see: Mongo Input Tool AS_RunAsCredentialsRemove records from a set of IDsAS_QueueAS_Queue represents both completed and queued jobs.A Schedule will not queue another Job if the previous Job hasn’t completed.[23.1]  The Next Run Time will only update when the Job completes.  Until then it will simply show the same time as te Last Run.  This is confusing when the Job is queued, since the L:ast Run is showing the time the Job was queued, not when it last ran since it’s still waiting to run.It’s ok to delete AS_Queue records, no other collections rely on these records.Remove all queued Jobs (ie, not Running or Complete)Find old jobs stuck initializing (it’s best to remove them based on their _id as removing all just Initializiang could catch a new job that just came in,Remove old Jobs that errored.  These are NOT removed by Persistence Settings and build up forever. Change 2020-01-01 to the date before which you want to remove Errored AS_Queue records.AS_SchedulesIt’s ok to delete AS_Schedules records, no other collections rely on these records.Find and remove corrupt schedules.  You can copy the result of the find() command so you know what schedules are being removed.  As with all MongoDB changes, a recent backup should exist.Delete all Schedules (useful if setting up a Sandbox from a Prod Mongo and you don’t want any Schedules to run (alternatively you could set all Schedules as disabled)Enable or disable selected Schedules. This command targets the binary value in AlteryxService > AS_Schedules > __ServiceData. Confirmed it still works on Server 2023.1.1.247 (Patch 2). Starting from 23.2 __ServiceData is deprecated, so this query won’t work db.AS_Schedules.update({ _id: "OBJECT_ID" }, { $set: { __ServiceData: BinData(0,"[__ServiceData]") } }, { multi: true })   <== ONEdb.AS_Schedules.updateMany({}, { $set: { __ServiceData: BinData(0,"[__ServiceData]") } }, { multi: true })                 <== ALLSee “ReadAllSchedules” App:  List all CollectionsFind String anywhere in the databaseFrom John Motlagh.To useEnter your content for the script variables:  searchString, saveDirectory, filenameStart Start MongoDB Shell for the database to searchPaste the entire script into the shellFind what collection an ID is fromSome errors will show an ID but it may not be clear what collection it’s from.  This command will find the collection using that ID.Determine size of each collectionMongo Command that works in Studio3tCommanddb.getCollectionNames().forEach(function (collName) {    var stats = db.getCollection(collName).stats();    print(collName + ": " + stats.size + " bytes (" + (stats.size / (1024 * 1024)).toFixed(2) + " MB)");});See alsoUtility Workflow - Get Collection Sizes - mongo_sizes.yxmd