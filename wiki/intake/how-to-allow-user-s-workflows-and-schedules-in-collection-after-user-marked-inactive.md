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
created_at: 2026-04-14T15:12:39Z
updated_at: 2026-04-14T15:12:39Z
confidence: null
cross_refs: []
content_hash: sha256:7b151f8331abf0321f057f8e8c3d5c69f8736fdbd107f05bd283168a3fe1f13e
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

|  |  |
| --- | --- |
|  |  |
|  |  |
|  |  |
|  |  |

# Concerns

By circumventing the Server UI process to mark Source User inactive by directly editing Mongo, some steps the Server would take (such as deactivating the Source User’s Schedules) will not occur.  This puts the database a bit out of whack from what the Server would expect if it had deactivated Source User itself (in Server's mind, all Schedules for a User who is Inactive should also be Inactive).

Issues could come up if, in a future release.  For example,

- if Server UI starts checking the user before running or displaying a Schedule and crashes (as it likes to do when confused) if an Active Schedule has an Inactive User, OR
- the Server adds a check in a future release and disables the Schedule b/c the User is inactive.  Then the customer opens a case on why a bunch of schedules were deactivated after upgrade, OR
- everything is fine and this solution works till the end of time!