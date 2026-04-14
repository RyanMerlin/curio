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
created_at: 2026-04-14T15:16:58Z
updated_at: 2026-04-14T15:16:58Z
confidence: null
cross_refs: []
content_hash: sha256:ca27c256e23271b859fa361ee379620cb868b54da82d0998985fdc5ada1eaeec
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

| **Access** | <== **uses Control Containers (23.1+)**                    **TODO - remove dependency on V3 API pack,** see:                    <https://alteryx.atlassian.net/wiki/spaces/SupportServer/pages/1778616339/How+to+call+Server+API+endoints+with+the+Download+Tool#Full-Example>  ---  <== **doesn’t use Control Containers**  ---  <== **POST /user/v2/workflows/{appId}/job for older Servers**  ---  <== Appends logging to TXT file, more flexible than XLSX as it handles  responses with different columns .                      **TODO - merge this with the Containers solution above** |
| --- | --- |
| **Older versions** |  |

# Solution

1. BAT file calls AlteryxEngineCmd.exe to immediately run queueWF.yxmd
2. This workflow uses the API to queue the Target workflow and logs the Queue ID the or error that the API returns.

Walk through using the **V3 API Pack**

# Configuration

| **File** | **Configuration Steps** |
| --- | --- |
| ## queueWF.bat      - Called from the customer's script    - Uses AlteryxEngineCmd.exe to immediately run queueWF.yxmd    - Logs the results and timestamp to queueLog.txt (logging looks like the Results pane for queueWF.yxmd) | 1. Update path to AlteryxEngineCmd.exe |
| ## queueWF.yxmd     - Calls API to queue targetWF.yxmd in Gallery    - Logs AS_Queue.id and timestamp to XLSX file by appending rows | 1. Install V3 API Pack (faster) or Server API Tool (slower)How to use the V3 API Pack Server API Tool +       1. How to use the V3 API Pack       2. Server API Tool +     2. Configure tools for yourServer URLWorkflow ID you want to queue (from URL when viewing workflow in Server UI)API Token and Secret       1. Server URL       2. Workflow ID you want to queue (from URL when viewing workflow in Server UI)       3. API Token and Secret |
| ## targetWF.yxmd     - Resource-intensive workflow that’s queued by this process | n/a |
| ## queueLog.txt     - Logs queueWF.yxmd run results from the queueWF.bat | n/a |
| ## queueLog_Success.xlsx     - Logs successful API calls for V3 Macro Pack option | File must already exist as it’s appended to |
| ## queueLog_Failure.xlsx     - Logs failing API calls for V3 Macro Pack option | File must already exist as it’s appended to |
| ## queueLog_ServerApiTool.xlsx     - Logs successful calls for Server API Tool option.  When the Server API Tool fails, there is no logging | File must already exist as it’s appended to |