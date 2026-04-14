---
id: 3353b8d9772dc136
title: How to - Search Certificates in MMC Using Thumbprint/Hash Value
status: intake
source:
  kind: confluence_page
  id: confluence-page:2036629823
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/2036629823
  summary: null
category: []
keywords: []
created_at: 2026-04-14T15:06:10Z
updated_at: 2026-04-14T15:06:10Z
confidence: null
cross_refs: []
content_hash: sha256:bbccdbe0f2b23b3a4d966a0ef239c34e34197e74919d88199ce5821ee5e65889
confluence_page_id: null
model_used: null
---

> **ℹ️ Info**
>
> Find a cert mentioned in a Windows Event Viewer message

If there are errors in the Windows Event Viewer about a particular certificate, for example

> **⚠️ Warning**
>
> Certificate for local system with Thumbprint 1e bc e3 74 20 98 2d 21 dc 4d 46 7b 4d c8 6b aa 6e 00 72 18 is about to expire or already expired."),

You can search for the certificate by

1. Right-click any of the certificate folders (Personal, Trusted, or Root)
2. Select Find Certificates
3. Ensure the Find in dropdown stays as All certificate stores
4. Paste the hash/thumbprint value in the Contains textbo
5. In the Look in field section, select SHA-1 Hash
6. Click Find Now

This should return the cert in the lower dialog box!