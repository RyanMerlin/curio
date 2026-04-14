---
id: dd354a6069320a10
title: ServiceData Blob Removal in 23.2
status: review
source:
  kind: confluence_page
  id: confluence-page:1944160523
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/1944160523
  summary: null
category:
- product-tree
- alteryx-server
- upgrade
keywords:
- upgrade
- '23.2'
- servicedata
- blob
- migration
created_at: 2026-04-14T15:18:28Z
updated_at: 2026-04-14T15:20:43Z
confidence: 0.87
cross_refs: []
content_hash: sha256:2f1728bf2d567eb4d9be8346224002d315258434e64db03e6f70ee3f39bce3c8
confluence_page_id: null
model_used: claude-sonnet-4-6
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
| **Log** | [Schema Migration log](https://alteryx.atlassian.net/wiki/search?text=Schema+Migration+logs+for+Gallery+and+Service+(alteryx-XXX-migration.csv))  <== **schema migration errors here** |