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
created_at: 2026-04-14T15:16:58Z
updated_at: 2026-04-14T15:16:58Z
confidence: null
cross_refs: []
content_hash: sha256:13ae16c7f335724445b16c5738a9b9ebd42434f24a60a12b1c4d85a0c271bbee
confluence_page_id: null
model_used: null
---

> **ℹ️ Info**
>
> Each Server upgrade will update the Mongo DB schema by adding or removing fields and collections.
> 
> This process is a common point of failure, leading to the Service failing to start after the upgrade.

| **Key Articles** | <https://help.alteryx.com/current/en/server/configure/mongodb-management/mongodb-schema-reference.html>  <== **matches Server version to Schema** |
| --- | --- |

---

%ProgramData%\Alteryx\Gallery\Logs\**alteryx-migration.csv**

db.versions.find({},{Number:1, _id:0}).limit(1).sort({$natural:-1})

db.versions.find({},{Number:1, _id:0}).limit(1).sort({$natural:-1})
{ "Number" : 31 }
>]]>