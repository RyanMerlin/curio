---
id: 1e6e4339e2384b59
title: How to Stop / Cancel a Workflow Job running on Server
status: intake
source:
  kind: confluence_page
  id: confluence-page:1778716312
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/1778716312
  summary: null
category: []
keywords: []
created_at: 2026-04-14T15:18:28Z
updated_at: 2026-04-14T15:18:28Z
confidence: null
cross_refs: []
content_hash: sha256:b28334dc848bb6215512dfc1c533adacf6e2084258cb9b53a813ff3619c9be09
confluence_page_id: null
model_used: null
---

> **ℹ️ Info**
>
> There are multiple ways to stop a running workflow job

> **ℹ️ Info**
>
> 24.2+ **Alteryx System Settings > Worker > General > Cancel jobs running longer than** setting applies to both Scheduled and Manual jobs automatically cancelling a job running longer than set.  **Server UI > Admin** allows setting different values for Scheduled vs Manual.
> 
> 24.1 and prior had no limit on Manaul job runs.  A community solution:
> 
> - How to Cancel Manual Jobs in Alteryx Server <== not sure which versions it works on

| **Using** | **Steps** |
| --- | --- |
| **Server UI** | Click the **delete icon **(looks like a do not enter sign) to the right of the Job. |
| **Legacy Scheduler** | Connect to the Server from the Designer **Legacy Scheduler** (**Options > View Schedules**) and stop the job.  The Alteryx Service must be running for this to work (ie, not Stopping).  This tends to work even if the Server UI is failing to display Jobs.  [Stop a Long Running App/Workflow on a Private Gallery](https://knowledge.alteryx.com/index/s/article/Stop-a-Long-Running-App-Workflow-on-a-Private-Gallery-1583459847562)  (KB) |
| **Task Manager** | View **Task Manager > Details **and delete the **AlteryxEngineCMD.exe** instance(s).  Each Workflow has its own AlteryxEngineCMD.exe.  If more than one job is running you'll have to guess which it is.  This could leave the **AS_Queue **record in an odd state and you'll may need to edit it to **Status = Error** and set **CompletionDateTime**.  [Alteryx Service Stuck in Stopping State](https://knowledge.alteryx.com/index/s/article/Alteryx-Service-Stuck-in-Stopping-State-1583461078466) (144673) |
| **AS_Queue** | If it's stuck (not really running) **edit AS_Queue** and set its **Status = Error** and set **CompletionDateTime**.  [This was mentioned elsewhere but not tested] |