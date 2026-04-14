---
id: dd354a6069320a10
title: ServiceData Blob Removal in 23.2
status: intake
source:
  kind: confluence_page
  id: confluence-page:1944160523
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/1944160523
  summary: null
category: []
keywords: []
created_at: 2026-04-14T15:16:58Z
updated_at: 2026-04-14T15:16:58Z
confidence: null
cross_refs: []
content_hash: sha256:2edbaa9d0045b211a3bd8d6ae3ff8f35e1b09a9bda4901c83115161aa140bd05
confluence_page_id: null
model_used: null
---

> **ℹ️ Info**
>
> The 23.2 upgrade removes the **__ServiceData blob **field and expands the data into seperate fields

> **📝 Note**
>
> This will break Administative workflows that used the **ServiceDataParser macro** and will need to be refactored to remove the macro and access the fields directly** **

This page tracks issues related to the removal of the **ServiceData blob**

| **Key Articles** | <https://alteryx.atlassian.net/wiki/spaces/SupportServer/pages/1702763531/Mongo+Input+Tool#%E2%80%A6-unpack-the-__ServiceData-%2F-ServiceData-blob> |
| --- | --- |
| **Log** | Schema Migration log  <== **schema migration errors here** |