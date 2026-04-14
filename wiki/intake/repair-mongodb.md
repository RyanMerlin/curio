---
id: 2f4696bc6d7b69ad
title: Repair MongoDB
status: intake
source:
  kind: confluence_page
  id: confluence-page:1702762253
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/1702762253
  summary: null
category: []
keywords: []
created_at: 2026-04-14T15:16:58Z
updated_at: 2026-04-14T15:16:58Z
confidence: null
cross_refs: []
content_hash: sha256:be99742ad3443b8b8a09ac374dd9d8c44eb5db8ee2e3afd1064e80016b6bbc24
confluence_page_id: null
model_used: null
---

for embedded mongo onlyPurple

> **ℹ️ Info**
>
> Mongo provides and in-place “repair” function that can help if you cant start MongoDB (with errors such as 'item not found ') or experience odd behavior (such as not being able to start the Shell)

| **Repair MongoDB** | c:  cd %ProgramFiles%\Alteryx\bin  mongod.exe --dbpath "DRIVE:\PERSISTENCE_FOLDER " --port 27018 **--repair** |
| --- | --- |
| **Troubleshooting** | For the following error running --repair  > **⚠️ Warning** > > can't start without --journal enabled when journal files are present, terminating  add  **--nojournal** flag after **--repair**  c:  cd %ProgramFiles%\Alteryx\bin  mongod.exe --dbpath "DRIVE:\FOLDER " --port 27018 --repair **--nojournal** |