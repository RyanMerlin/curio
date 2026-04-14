---
id: 3293eeb89f96002e
title: How to Chain Workflows and Analytic Apps on Server (aka Workflow Orchestration)
status: intake
source:
  kind: confluence_page
  id: confluence-page:1640793044
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/1640793044
  summary: null
category: []
keywords: []
created_at: 2026-04-14T15:06:10Z
updated_at: 2026-04-14T15:06:10Z
confidence: null
cross_refs: []
content_hash: sha256:c2e9f15f8288c27924f70ba8591b382356a7555ccd10457c76ede7e140b165b3
confluence_page_id: null
model_used: null
---

> **ℹ️ Info**
>
> Chaining allows a workflow or analytic app to trigger a next workflow or analytic app to run

note 2a4ebf0c-cefc-46f8-b90f-f9682a02f914 **For Designer-only chaining, see:**

How to Chain Workflows and Analytic Apps in Designer (aka Workflow Orchestration)

**For Designer-only chaining, see:**

[How to Chain Workflows and Analytic Apps in Designer (aka Workflow Orchestration)](/wiki/spaces/SupportDesigner/pages/1765015871/How+to+Chain+Workflows+and+Analytic+Apps+in+Designer+aka+Workflow+Orchestration)

---

Chainiing Workflows on a Server should be a built-in feature, VOC Feature Reqeusdt:

- Ability to chain workflows on the Server
- Old Ideas Board:  https://community.alteryx.com/t5/Alteryx-Server-Ideas/Workflow-chaining-based-on-alteryx-workflows-available-in/idi-p/613473 (613473)

More recent (23.1+) versions allow Scheduling an Analytic App, but will only run the FIRST App in the chain, then stop.

The Job Results of 2nd+ chained workflow using this method will not appear in the Server UI and “local” output isn’t supported (output must be to a UNC path)

The Job Results of 2nd+ chained workflow will not appear in the Server UI and “local” output isn’t supported (output must be to a UNC path)

The Job Results of 2nd+ chained workflow will not appear in the Server UI and “local” output isn’t supported (output must be to a UNC path)

This call leads to permanent Mongo bloat [need citation ]

Help indicates this is not supported for Server: <https://help.alteryx.com/current/en/server/install/alteryxservice-commands.html#idm45052117816768:~:text=%23Not%20supported%20on%20Alteryx%20Server>

We generally don’t recommend using **alteryxEngineCmd.exe** on Server:

- The workflow job and results do not appear in Server UI
- If run from a command-line script it subverts the #Simultaneous setting and runs one more workflow that the Server is sized for. Or more if multiple scripts launch workflows, overwhelming the Server.

**Scheduling **Apps is difficult as of 24.2.  The Schedule button is available but will only run the first app in the chain but not ask questions, using the tool configurations like a normal workflow.

- GS-29277dcf2c9-72f3-3ff6-8e99-fe88e9f473f1System Jira                      <== 25.1 feature to ask questions, but still not chain
- TPRI-640877dcf2c9-72f3-3ff6-8e99-fe88e9f473f1System Jira                      <== [24.2-LTS, 24.1.1_Patch4]

**On Success - Show Results to User** option may not work in some versions of Gallery, the results are not shown