---
id: 1e0735c3c873cc22
title: Determine MongoDB AlteryxGallery Schema
status: review
source:
  kind: confluence_page
  id: confluence-page:1702894190
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/1702894190
  summary: null
category:
- product-tree
- alteryx-server
- mongodb
keywords:
- mongodb
- schema
- alteryxgallery
- upgrade
- migration
created_at: 2026-04-14T13:40:14Z
updated_at: 2026-04-14T13:49:59Z
confidence: 0.87
cross_refs: []
content_hash: sha256:70701edb3dc26f514023b6fd00223b4cf4572a74c94cc6174c0cb614d0fc4b2e
confluence_page_id: null
model_used: claude-sonnet-4-6
---

> **ℹ️ Info**
>
> Each Server upgrade will update the Mongo DB schema by adding or removing fields and collections.
> 
> This process is a common point of failure, leading to the Service failing to start after the upgrade.

| Key Articles | https://help.alteryx.com/current/en/server/configure/mongodb-management/mongodb-schema-reference.html  <== matches Server version to Schema |
| --- | --- |

---

%ProgramData%\Alteryx\Gallery\Logs\**alteryx-migration.csv**

db.versions.find({},{Number:1, _id:0}).limit(1).sort({$natural:-1})

db.versions.find({},{Number:1, _id:0}).limit(1).sort({$natural:-1})
{ "Number" : 31 }
>]]>