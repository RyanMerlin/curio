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
created_at: 2026-04-14T15:10:53Z
updated_at: 2026-04-14T15:10:53Z
confidence: null
cross_refs: []
content_hash: sha256:a34f86f4d0b1d88e43f3f5ac7e2b399dd6a24348d407bed21e13f3a81de242e6
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

|  |  |
| --- | --- |
|  |  |

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
> How to Allow User's Workflows and Schedules in Collection after User marked Inactive
> How to Transition from Subscriptions to Collections

## Options

|  |  |
| --- | --- |
|  |  |
|  |  |
|  |  |