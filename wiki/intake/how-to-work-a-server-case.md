---
id: a3829912693b5d99
title: How to Work a Server Case
status: intake
source:
  kind: confluence_page
  id: confluence-page:2947253108
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/2947253108
  summary: null
category: []
keywords: []
created_at: 2026-04-14T15:18:28Z
updated_at: 2026-04-14T15:18:28Z
confidence: null
cross_refs: []
content_hash: sha256:c44ddb3cf5f51d93ebf53f81d129b5f78902639f7c301c2461fd184481d4e1cd
confluence_page_id: null
model_used: null
---

| **Questions** | **Steps/Notes** |
| --- | --- |
| What version of Server are they using? | Important to know the full version of Server. With the full version of Server you can determine the patch they are on from the [Release notes for that version](https://help.alteryx.com/release-notes/en/release-notes/server-release-notes/server-2024-2-release-notes.html). |
| What is the architecture of of the Server environment? | Single-node or multi-node environment? If multi-node, please describe the role for each node. |
| What Server authentication is currently setup? | Built-In, SAML, Integrated Windows Auth, Integrated Windows Auth with Keberos. Can be found in Alteryx System Setting > Server UI > Authentication. |
| What is the scope of the issue? | - When did the issue first start occurring and were there any recent changes or updates to the Server environment?    - Is it happening for one user or all users?    - What are the steps to recreate the issue?    - Is the issue occurring in Server UI/Gallery?Is the issue with a workflow failing? Does it run successfully from Designer on the Server Machine or is it only failing on a specific worker?       - Is the issue with a workflow failing? Does it run successfully from Designer on the Server Machine or is it only failing on a specific worker? |
| Request logs to review. | [Logs and Traces](https://alteryx.atlassian.net/wiki/search?text=Logs+and+Traces) and [How To: Attach Server Log Files](https://alteryx.lightning.force.com/kA02R0000000veISAQ) - Know which logs to request depending on the customer issue. Be sure to request logs at outset of case ownership during initial response. |

| **Testing Tools** |  |
| --- | --- |
| [HAR Trace](https://alteryx.atlassian.net/wiki/search?text=HAR+Trace) | HAR trace can be helpful when end user is experiencing issue in Server UI |
| [Fiddler](https://alteryx.atlassian.net/wiki/spaces/SupportCseBasics/pages?title=Fiddler) and/or [Wireshark](https://alteryx.atlassian.net/wiki/spaces/SupportCseBasics/pages?title=Wireshark) | A Fiddler/Wireshark trace from Designer on the Sever machine can help identify network or permission issues with job runs. |
| [Notepad++](https://alteryx.atlassian.net/wiki/spaces/SupportCseBasics/pages?title=Notepad++) | Notepad ++ plugin can help identify pertinent errors in Server and Gallery logs. |
| Legacy Scheduler [Options > View Schedules](https://alteryx.atlassian.net/wiki/spaces/SupportDesigner/pages?title=Options+>+View+Schedules) | If you’re seeing a job fail with a generic error, it can sometimes help to view the job output from the legacy scheduler from Designer on the Server/Controller machine. |

[Server Upgrade Version Paths - What version can upgrade to what versions?](https://alteryx.atlassian.net/wiki/search?text=Server+Upgrade+Version+Paths+-+What+version+can+upgrade+to+what+versions?)