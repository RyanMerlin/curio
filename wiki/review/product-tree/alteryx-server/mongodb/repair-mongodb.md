---
id: 2f4696bc6d7b69ad
title: Repair MongoDB
status: review
source:
  kind: confluence_page
  id: confluence-page:1702762253
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/1702762253
  summary: null
category:
- product-tree
- alteryx-server
- mongodb
keywords:
- mongodb
- repair
- in-place
- embedded-mongo
- maintenance
created_at: 2026-04-14T13:40:14Z
updated_at: 2026-04-14T13:50:44Z
confidence: 0.85
cross_refs: []
content_hash: sha256:590eaa2c42a91d191afb61f2c10aebc20dc9fab50f33ec4a792a313925ec97b0
confluence_page_id: null
model_used: claude-sonnet-4-6
---

for embedded mongo onlyPurple

> **ℹ️ Info**
>
> Mongo provides and in-place “repair” function that can help if you cant start MongoDB (with errors such as 'item not found ') or experience odd behavior (such as not being able to start the Shell)

| Repair MongoDB | #E3FCEFc:cd %ProgramFiles%\Alteryx\binmongod.exe --dbpath "DRIVE:\PERSISTENCE_FOLDER" --port 27018 --repair |
| --- | --- |
| Troubleshooting | For the following error running --repaircan't start without --journal enabled when journal files are present, terminatingadd  --nojournal flag after --repair#E3FCEFc:cd %ProgramFiles%\Alteryx\binmongod.exe --dbpath "DRIVE:\FOLDER" --port 27018 --repair --nojournal |