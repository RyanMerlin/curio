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
created_at: 2026-04-14T15:16:58Z
updated_at: 2026-04-14T15:16:58Z
confidence: null
cross_refs: []
content_hash: sha256:e9c153b3b53eafb8bdcdae0911e9fb6e2a8b91f6c2d5be63610ed87c91a65d2e
confluence_page_id: null
model_used: null
---

> **ℹ️ Info**
>
> Some Servers experience intermittent or consistent problems if the Server is going outside of itself to resolve its own FQDN name.  This is especially an issue with load balancers.
> 
> Adding an entry to the **hosts **file allows requests made from the Service to find the Gallery, bypassing any DNS/network entity that may be causing an issue.

Another issue also referred to as “loopback” <https://docs.google.com/document/d/1Kj0UBGaUhgw8wX0K2izOiEZbKs8yzzxQ718dXSf08H4/edit?usp=sharing> (Google)

| **Key Articles** | [How and Why to do a Hosts file modification](https://knowledge.alteryx.com/index/s/article/How-and-Why-to-do-a-Hosts-file-modification) (KB) [Requirements for Configuring Alteryx Server with a Load Balancer (or Reverse Proxy)](https://knowledge.alteryx.com/index/s/article/Requirements-for-Configuring-Alteryx-Server-with-a-Load-Balancer-or-Reverse-Proxy-1628116360935) (KB) |
| --- | --- |

# Edit the hosts files

1 –  Edit the file

2 – Add a line with 127.0.0.1 and the Base Address of the Gallery (don’t include /gallery).

3 – Restart the Service