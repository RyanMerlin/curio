---
id: d2ef774371fd5d33
title: How to - Determine the Customer's SSL Cert on LB or Gallery
status: intake
source:
  kind: confluence_page
  id: confluence-page:1930003044
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/1930003044
  summary: null
category: []
keywords: []
created_at: 2026-04-14T15:16:58Z
updated_at: 2026-04-14T15:16:58Z
confidence: null
cross_refs: []
content_hash: sha256:1588329c226523ab66c14e623635c0841e1da96a4046647517b0621c34ca7556
confluence_page_id: null
model_used: null
---

> **ℹ️ Info**
>
> A Server SSL cert must be added in MMC and bound to a specific port to work on a LB or Gallery

---

Multiple certs can be added but only one bound to a specific port

**SAN **shows the FQDNs (with option wildcards) that the cert was created for

**Thumbprint **must match the bound certificate’s **Certificate Hash**

**Certificate Hash** must match certificate’s **Thumbprint **

netsh http show sslcert