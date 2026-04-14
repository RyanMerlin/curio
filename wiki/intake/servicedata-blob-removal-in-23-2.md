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
created_at: 2026-04-14T15:12:39Z
updated_at: 2026-04-14T15:12:39Z
confidence: null
cross_refs: []
content_hash: sha256:c8337534189db74bc41689d7579a7f756a3d009929fbe94b16e8e63d4e4ad7cd
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

|  |  |
| --- | --- |
|  |  |