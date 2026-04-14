---
id: 57ce97232ed25a1e
title: How to Set a Gallery URL loopback in the Server hosts file
status: intake
source:
  kind: confluence_page
  id: confluence-page:1745944926
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/1745944926
  summary: null
category: []
keywords: []
created_at: 2026-04-14T15:10:53Z
updated_at: 2026-04-14T15:10:53Z
confidence: null
cross_refs: []
content_hash: sha256:75d1356ac278b5e09f6736023dae86c325c542a48ef2784815b834bc665937d9
confluence_page_id: null
model_used: null
---

> **ℹ️ Info**
>
> Some Servers experience intermittent or consistent problems if the Server is going outside of itself to resolve its own FQDN name.  This is especially an issue with load balancers.
> 
> Adding an entry to the **hosts **file allows requests made from the Service to find the Gallery, bypassing any DNS/network entity that may be causing an issue.

Another issue also referred to as “loopback” <https://docs.google.com/document/d/1Kj0UBGaUhgw8wX0K2izOiEZbKs8yzzxQ718dXSf08H4/edit?usp=sharing> (Google)

|  |  |
| --- | --- |

# Edit the hosts files

1 –  Edit the file

2 – Add a line with 127.0.0.1 and the Base Address of the Gallery (don’t include /gallery).

3 – Restart the Service