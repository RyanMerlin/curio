---
id: 2228a6ae568e0315
title: "How to use alteryxEngineCmd.exe to queue a job from a \nBAT using the API"
status: intake
source:
  kind: confluence_page
  id: confluence-page:2200241367
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/2200241367
  summary: null
category: []
keywords: []
created_at: 2026-04-14T15:12:39Z
updated_at: 2026-04-14T15:12:39Z
confidence: null
cross_refs: []
content_hash: sha256:7252a2a42503d84ca43492904cf8a7dc90edc99d3d458d419578f0c21cb662b4
confluence_page_id: null
model_used: null
---

> **ℹ️ Info**
>
> This solution uses **alteryxEngineCmd.exe** to immediately run a short workflow that uses the API to queue a resource-intensive target workflow.  Developed for 00688174.

> **⚠️ Warning**
>
> We generally don’t recommend using **alteryxEngineCmd.exe** on Server:
> 
> - The workflow job and results do not appear in Server UI
> - If run from a command-line script it subverts the #Simultaneous setting and runs one more workflow that the Server is sized for. Or more if multiple scripts launch workflows, overwhelming the Server.

|  |  |
| --- | --- |
|  |  |

# Solution

1. BAT file calls AlteryxEngineCmd.exe to immediately run queueWF.yxmd
2. This workflow uses the API to queue the Target workflow and logs the Queue ID the or error that the API returns.

Walk through using the **V3 API Pack**

# Configuration

|  |  |
| --- | --- |
|  |  |
|  |  |
|  |  |
|  |  |
|  |  |
|  |  |
|  |  |