---
id: 2228a6ae568e0315
title: "How to use alteryxEngineCmd.exe to queue a job from a \nBAT using the API"
status: review
source:
  kind: confluence_page
  id: confluence-page:2200241367
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/2200241367
  summary: null
category:
- product-tree
- alteryx-server
- user-management
keywords:
- alteryxenginecmd
- api
- bat
- job
- queue
- automation
created_at: 2026-04-14T13:40:14Z
updated_at: 2026-04-14T13:50:25Z
confidence: 0.85
cross_refs: []
content_hash: sha256:8651c1d6aaed137f8d7435b4aa36e095718e80d60644ea11cd3f57bccd3e45c8
confluence_page_id: null
model_used: claude-sonnet-4-6
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

| Access | <== uses Control Containers (23.1+)                   TODO - remove dependency on V3 API pack, see:                   https://alteryx.atlassian.net/wiki/spaces/SupportServer/pages/1778616339/How+to+call+Server+API+endoints+with+the+Download+Tool#Full-Example             <== doesn’t use Control Containers           <== POST /user/v2/workflows/{appId}/job for older Servers           <== Appends logging to TXT file, more flexible than XLSX as it handles                    responses with different columns.                     TODO - merge this with the Containers solution above |
| --- | --- |
| Older versions |  |

# Solution

1. BAT file calls AlteryxEngineCmd.exe to immediately run queueWF.yxmd
2. This workflow uses the API to queue the Target workflow and logs the Queue ID the or error that the API returns.

Walk through using the **V3 API Pack**

# Configuration

| File | Configuration Steps |
| --- | --- |
| queueWF.bat Called from the customer's scriptUses AlteryxEngineCmd.exe to immediately run queueWF.yxmdLogs the results and timestamp to queueLog.txt (logging looks like the Results pane for queueWF.yxmd) | Update path to AlteryxEngineCmd.exe |
| queueWF.yxmdCalls API to queue targetWF.yxmd in GalleryLogs AS_Queue.id and timestamp to XLSX file by appending rows | Install V3 API Pack (faster) or Server API Tool (slower)How to use the V3 API Pack Server API Tool + Configure tools for yourServer URLWorkflow ID you want to queue (from URL when viewing workflow in Server UI)API Token and Secret |
| targetWF.yxmdResource-intensive workflow that’s queued by this process | n/a |
| queueLog.txtLogs queueWF.yxmd run results from the queueWF.bat | n/a |
| queueLog_Success.xlsxLogs successful API calls for V3 Macro Pack option | File must already exist as it’s appended to |
| queueLog_Failure.xlsxLogs failing API calls for V3 Macro Pack option | File must already exist as it’s appended to |
| queueLog_ServerApiTool.xlsxLogs successful calls for Server API Tool option.  When the Server API Tool fails, there is no logging | File must already exist as it’s appended to |