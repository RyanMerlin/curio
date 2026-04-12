---
id: dd354a6069320a10
title: ServiceData Blob Removal in 23.2
status: staged
source:
  kind: confluence_page
  id: confluence-page:1944160523
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/1944160523
  summary: null
category:
- product-tree
- alteryx-designer
keywords:
- servicedata
- blob
- removal
- schema
- issues
created_at: 2026-04-12T20:59:51Z
updated_at: 2026-04-12T21:02:33Z
confidence: 0.55
cross_refs: []
content_hash: sha256:b33c860f0d493d84217a8eb75dea5e099d8f21672c3f653606e6ae1c9f71e89d
confluence_page_id: null
model_used: heuristic
---

> **ℹ️ Info**
>
> The 23.2 upgrade removes the **__ServiceData blob **field and expands the data into seperate fields

> **📝 Note**
>
> This will break Administative workflows that used the **ServiceDataParser macro** and will need to be refactored to remove the macro and access the fields directly** **

note This page tracks issues related to the removal of the **ServiceData blob**

This page tracks issues related to the removal of the **ServiceData blob**

| Key Articles | https://alteryx.atlassian.net/wiki/spaces/SupportServer/pages/1702763531/Mongo+Input+Tool#%E2%80%A6-unpack-the-__ServiceData-%2F-ServiceData-blob |
| --- | --- |
| Log | Schema Migration log  <== schema migration errors here |