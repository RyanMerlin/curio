---
id: 326c3e215c588136
title: How to Transfer/Change Ownership of Workflow / Move Workflow to Another Studio
status: intake
source:
  kind: confluence_page
  id: confluence-page:1695484187
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/1695484187
  summary: null
category: []
keywords: []
created_at: 2026-04-14T15:18:28Z
updated_at: 2026-04-14T15:18:28Z
confidence: null
cross_refs: []
content_hash: sha256:ab4ce25a7a1abd4e5b456499f7faefa901b06a0fee94ee2e83d27c0a0a476273
confluence_page_id: null
model_used: null
---

---

---

> **ℹ️ Info**
>
> This page discussed changing ownership of assets:
> 
> - workflows
> - schedules
> - collection owner
> 
> DCM Connection ownership can’t be changed

---

---

# Transfer Ownership in 24.1+

> **ℹ️ Info**
>
> Transferring ownership of workflows and schedules can be done through the UI or API
> 
> **Encourage customers to upgrade to 24.1+ if they are asking about transferring ownership.**

> **📝 Note**
>
> Past runs are NOT shared in the transfer

> **📝 Note**
>
> Other elements may need to be shared with the new owner to run the workflow:
> 
> - DCM Connection
> - Shared Gallery Database Connection
> - Shared Gallery Credential

> **📝 Note**
>
> Workflow will move to new owner’s studio, affecting access for all other users in the source or destination studio

> **📝 Note**
>
> When transferring Schedules, the new owner must have the permission to schedule workflows

> **📝 Note**
>
> **Shared Gallery Connection **– The user who PUBLISHED the workflow is the **Revisions.0. AuthorId** and will need to continue to have the Credential shared with them for the workflow to run.  Otherwise, error:
> 
> - The user supplied does not have access to the given Run As Account
> - TCPE-169877dcf2c9-72f3-3ff6-8e99-fe88e9f473f1System Jira

| **Requirements** | - New owner must have access to DCM or Shared Gallery Connection the workflow relies on |
| --- | --- |
| **Help** | **UI**     - https://help.alteryx.com/current/en/server/administer-alteryx-server/workflows--admin-interface/transfer-workflow-ownership.html#transfer-workflow-ownership    - https://help.alteryx.com/current/en/server/administer-alteryx-server/schedules--admin-interface/transfer-schedule-ownership.html  **API**     - PUT /v3/users/{userId}/assetTransfer    - PUT /v3/workflows/transfer/{workflowId} |

---

# Transfer Ownership in 23.2 and prior

> **⚠️ Warning**
>
> 23.2 and prior did not have an easy process for transferring ownership of workflows.
> 
> **Encourage customers to upgrade to 24.1+ if they are asking about transferring ownership.  Some of the options below are roundabout, take serveral steps, or are not really transferring ownership.  CUSTOMERS NEED TO BE UPGRADING!**

> **ℹ️ Info**
>
> **Workflows are primarily owned by STUDIOS**, the UI misleading lists the user as the owner, but think of them as the creator.
> 
> **Schedules are owned by USERS**, the API can transfer Schedule User ownership

> **ℹ️ Info**
>
> See also:
> 
> [How to Allow User's Workflows and Schedules in Collection after User marked Inactive](https://alteryx.atlassian.net/wiki/search?text=How+to+Allow+User's+Workflows+and+Schedules+in+Collection+after+User+marked+Inactive)
> [How to Transition from Subscriptions to Collections](https://alteryx.atlassian.net/wiki/search?text=How+to+Transition+from+Subscriptions+to+Collections)

## Options

| **Option**  Share via a Collection | Share the workflow and/or schedule to a Collection.  Other users can then run, schedule, and edit the workflows or schedule (depending on their permissions to the Collection).  Ownership can be ignored. |
| --- | --- |
| **Option**  Re-publish | Download User_A’s workflows and have User_B upload them.  User_B will then be the owner of the newly published workflows.  Version history is lost. |
| **Option**  Move all workflows to a different studio, then change ownership | Use the process below to truly move workflows to another Studio.  This helps if a user leaves an organization.  Internal KB     - https://knowledge.alteryx.com/index/s/article/How-to-move-from-Subscriptions-to-Collections-in-Server      > Move Workflows Between Studio  Public KB was shut down and Ed’s trying to get it back as of Sep-05-2024     - https://community.alteryx.com/t5/Alteryx-Server-Knowledge-Base/How-to-move-from-Subscriptions-to-Collections-in-Server/ta-p/1137150#:~:text=to%20studio%20B.-,Move%20Workflows%20Between%20Studios,-Now%20let%E2%80%99s%20review         > Move Workflows Between Studios  Old KB that seems to be doing the same thing as Mariah’s KB     - Workflows move to different Private Studio (KB) **Step 1** – Ensure OLD user is the **ONLY **user in their Studio.  **Step 2 **- Edit OLD user and enter the Subscription Key for the destination Subscription.  This will move the OLD user and all of their workflows to the destination Subscription.  **Step 3** – See option below for using the API to change ownership |
| **Option**  Use API to change ownership | > **📝 Note** > > Ensure User_A and User_B are in the same Studio >  > This option is not helpful if users are all in their own Studios  Use the **GET /v3/workflows/{workflowId** endpoint to get details on a workflow owned by the OLD user. Get the Workflow ID from the URL when viewing the workflow.     - GET /v3/workflows/{workflowId}  Edit this JSON to match the **PUT /v3/workflows/{workflowId}** contract, changing the OwnerID from OLD user to NEW user.  Get the OwnerID from the URL when editing the user in the Admin module.     - PUT /v3/workflows/{workflowId} |