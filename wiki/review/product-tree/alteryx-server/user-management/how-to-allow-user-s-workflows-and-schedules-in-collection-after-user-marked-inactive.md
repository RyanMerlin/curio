---
id: b831f95902681d54
title: How to Allow User's Workflows and Schedules in Collection after User marked Inactive
status: review
source:
  kind: confluence_page
  id: confluence-page:1640793095
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/1640793095
  summary: null
category:
- product-tree
- alteryx-server
- user-management
keywords:
- users
- collections
- workflows
- schedules
- inactive
- how-to
created_at: 2026-04-14T13:40:14Z
updated_at: 2026-04-14T13:50:13Z
confidence: 0.82
cross_refs: []
content_hash: sha256:96bd772c3dd3b3f8f38f7b99c2f095db2d12b9498a4776b58f0dfd098af19245
confluence_page_id: null
model_used: claude-sonnet-4-6
---

# Below is in development and should get another run-through on a test Server before going live

When a user is marked Inactive in the Server UI their Workflows and Schedules are removed from Collections and become inaccessible to other Users in those Collections. correct?Red

> **ℹ️ Info**
>
> Tested Server 2022.1, Case 00593606

note This process may not be needed in a future version of Server when Private Studios have been removed and the API can transfer ownership of assets without requiring users be in the same Private Studio.

<https://community.alteryx.com/t5/Alteryx-Server-Knowledge-Base/How-to-move-from-Subscriptions-to-Collections-in-Server/ta-p/1137150> (1137150)

This process may not be needed in a future version of Server when Private Studios have been removed and the API can transfer ownership of assets without requiring users be in the same Private Studio.

<https://community.alteryx.com/t5/Alteryx-Server-Knowledge-Base/How-to-move-from-Subscriptions-to-Collections-in-Server/ta-p/1137150> (1137150)

| Task | Steps |
| --- | --- |
| Ensure users are in same Collection | Ensure the Source User and Destination User are both in the same Collection |
| Add Source User assets to Collection | Add all assets from Source User (workflows and schedules) to the Collection via the Server UI. |
| Remove the Source User from Collection via Mongo | To find which Collections they are in, use the following Mongo query:#E3FCEFdb.getCollection('collections').find({"Users.ActiveDirectoryObject.DisplayName":/FIRST LAST/}).pretty()Example: #E3FCEFdb.getCollection('collections').find({"Users.ActiveDirectoryObject.DisplayName":/Tim Randall/})Note: after Source User is removed from the Collection, other users can still run the Workflows, view results, and Schedule Workflows owned by the Source User. |
| Set Source User to Active: false in Mongo | Set Source User to Active: false in Mongo:#E3FCEFdb.getCollection('users').update({"Email":"EMAIL_ADDRESS"},{$set:{Active:false}})Example: #E3FCEFdb.getCollection('users').update({"Email":"tim.randall@alteryx.com"},{$set:{Active:false}}) |

# Concerns

By circumventing the Server UI process to mark Source User inactive by directly editing Mongo, some steps the Server would take (such as deactivating the Source User’s Schedules) will not occur.  This puts the database a bit out of whack from what the Server would expect if it had deactivated Source User itself (in Server's mind, all Schedules for a User who is Inactive should also be Inactive).

Issues could come up if, in a future release.  For example,

- if Server UI starts checking the user before running or displaying a Schedule and crashes (as it likes to do when confused) if an Active Schedule has an Inactive User, OR
- the Server adds a check in a future release and disables the Schedule b/c the User is inactive.  Then the customer opens a case on why a bunch of schedules were deactivated after upgrade, OR
- everything is fine and this solution works till the end of time!