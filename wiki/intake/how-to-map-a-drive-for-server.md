---
id: 623122ffdaf6924e
title: How to Map a Drive for Server
status: intake
source:
  kind: confluence_page
  id: confluence-page:2999026275
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/2999026275
  summary: null
category: []
keywords: []
created_at: 2026-04-14T13:40:14Z
updated_at: 2026-04-14T13:40:14Z
confidence: null
cross_refs: []
content_hash: sha256:cb0a98187d638772cd69efd5d1776ae596f4ce26bfe28de379d1aeea518bf399
confluence_page_id: null
model_used: null
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

How to Setup UNC Filepath on an APOD

# BAT file

---

# Task

Run **Task Scheduler** > **Create Task… **(on right pane)

This will run when the system reboots and runs as the SYSTEM rather than a specific user.

Click **Change User or Group…** to set **NT AUTHORITY\SYSTEM** in dialog aboveabove

This where the BAT file is located: