---
id: 1e0735c3c873cc22
title: Determine MongoDB AlteryxGallery Schema
status: intake
source:
  kind: confluence_page
  id: confluence-page:1702894190
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/1702894190
  summary: null
category: []
keywords: []
created_at: 2026-04-14T15:09:17Z
updated_at: 2026-04-14T15:09:17Z
confidence: null
cross_refs: []
content_hash: sha256:c7d7131e38e3833c4599ec674745bf36170f35502eb0ae434ed0ac812a66b8d0
confluence_page_id: null
model_used: null
---

> **ℹ️ Info**
>
> Each Server upgrade will update the Mongo DB schema by adding or removing fields and collections.
> 
> This process is a common point of failure, leading to the Service failing to start after the upgrade.

|  |  |
| --- | --- |

---

%ProgramData%\Alteryx\Gallery\Logs\**alteryx-migration.csv**

db.versions.find({},{Number:1, _id:0}).limit(1).sort({$natural:-1})

db.versions.find({},{Number:1, _id:0}).limit(1).sort({$natural:-1})
{ "Number" : 31 }
>]]>