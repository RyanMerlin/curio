---
id: 623122ffdaf6924e
title: How to Map a Drive for Server
status: review
source:
  kind: confluence_page
  id: confluence-page:2999026275
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/2999026275
  summary: null
category:
- product-tree
- alteryx-server
- administration
keywords:
- drive-mapping
- network
- configuration
- apod
created_at: 2026-04-14T15:18:28Z
updated_at: 2026-04-14T15:19:58Z
confidence: 0.82
cross_refs: []
content_hash: sha256:a3486b3acd850df42c81579a59e1178efc5ac5465975b01b2550824a97752ff1
confluence_page_id: null
model_used: claude-sonnet-4-6
---

> **ℹ️ Info**
>
> Matt H was able to get a drive letter mapped on an APOD that worked with Server.

> **📝 Note**
>
> We want cusomers using UNC paths for workflows on Server, not a drive letter.
> 
> Microsoft also suggests NOT using drive letters on Servers.  They don’t say it doesn’t work, they just say don’t do it:
> 
> - https://learn.microsoft.com/en-us/windows/win32/services/services-and-redirected-drives

---

---

# Setup a Share on your APOD

[How to Setup UNC Filepath on an APOD](https://alteryx.atlassian.net/wiki/spaces/SupportCseBasics/pages?title=How+to+Setup+UNC+Filepath+on+an+APOD)

# BAT file

---

# Task

Run **Task Scheduler** > **Create Task… **(on right pane)

This will run when the system reboots and runs as the SYSTEM rather than a specific user.

Click **Change User or Group…** to set **NT AUTHORITY\SYSTEM** in dialog aboveabove

This where the BAT file is located: