---
id: b831f95902681d54
title: How to Allow User's Workflows and Schedules in Collection after User marked Inactive
status: intake
source:
  kind: confluence_page
  id: confluence-page:1640793095
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/1640793095
  summary: null
category: []
keywords: []
created_at: 2026-04-14T15:18:28Z
updated_at: 2026-04-14T15:18:28Z
confidence: null
cross_refs: []
content_hash: sha256:55a286adea960a18a7a8cbb00edc493ca38d396d4d8dbc9059a73daa3477b14a
confluence_page_id: null
model_used: null
---

# Below is in development and should get another run-through on a test Server before going live

When a user is marked Inactive in the Server UI their Workflows and Schedules are removed from Collections and become inaccessible to other Users in those Collections. correct?Red

> **ℹ️ Info**
>
> Tested Server 2022.1, Case 00593606

This process may not be needed in a future version of Server when Private Studios have been removed and the API can transfer ownership of assets without requiring users be in the same Private Studio.

<https://community.alteryx.com/t5/Alteryx-Server-Knowledge-Base/How-to-move-from-Subscriptions-to-Collections-in-Server/ta-p/1137150> (1137150)

| **Task** | **Steps** |
| --- | --- |
| **Ensure users are in same Collection** | Ensure the Source User and Destination User are both in the same Collection |
| **Add Source User assets to Collection** | Add all assets from Source User (workflows and schedules) to the Collection via the Server UI. |
| **Remove the Source User from Collection via Mongo ** | To find which Collections they are in, use the following Mongo query:  db.getCollection('collections').find({"Users.ActiveDirectoryObject.DisplayName":/FIRST LAST /}).pretty() Example:  db.getCollection('collections').find({"Users.ActiveDirectoryObject.DisplayName":/Tim Randall /}) Note: after Source User is removed from the Collection, other users can still run the Workflows, view results, and Schedule Workflows owned by the Source User. |
| **Set Source User to Active: false in Mongo** | Set Source User to **Active: false** in Mongo:  db.getCollection('users').update({"Email":"EMAIL_ADDRESS "},{$set:{Active:false}}) Example:  db.getCollection('users').update({"Email":"[tim.randall@alteryx.com](mailto:tim.randall@alteryx.com)"},{$set:{Active:false}}) |

# Concerns

By circumventing the Server UI process to mark Source User inactive by directly editing Mongo, some steps the Server would take (such as deactivating the Source User’s Schedules) will not occur.  This puts the database a bit out of whack from what the Server would expect if it had deactivated Source User itself (in Server's mind, all Schedules for a User who is Inactive should also be Inactive).

Issues could come up if, in a future release.  For example,

- if Server UI starts checking the user before running or displaying a Schedule and crashes (as it likes to do when confused) if an Active Schedule has an Inactive User, OR
- the Server adds a check in a future release and disables the Schedule b/c the User is inactive.  Then the customer opens a case on why a bunch of schedules were deactivated after upgrade, OR
- everything is fine and this solution works till the end of time!