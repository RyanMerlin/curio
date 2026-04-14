---
id: b2024558325369eb
title: Delete mongod.lock
status: intake
source:
  kind: confluence_page
  id: confluence-page:1702894333
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/1702894333
  summary: null
category: []
keywords: []
created_at: 2026-04-14T13:40:14Z
updated_at: 2026-04-14T13:40:14Z
confidence: null
cross_refs: []
content_hash: sha256:3f1c6a93a4a3064d0d93b432dac0779023093df2e6d1b666341991b7628f8a22
confluence_page_id: null
model_used: null
---

for embedded mongo onlyPurple

> **ℹ️ Info**
>
> This articles explains how to delete the mongod.lock file in MongoDB

**mongod.lock** is in the persistence folder and must be 0 kb to allow the Service to start.  This is a lock file to ensure only one Mongod.exe is using the Mongo folder at a time.  Delete the following file if the Service is not running and the file size is >0KB:

%ProgramData%\Alteryx\Service\Persistence\MongoDB \mongod.lock
For more details see [Alteryx Service will not start Error No suitable servers](https://community.alteryx.com/t5/Alteryx-Server-Knowledge-Base/Alteryx-Service-will-not-start-Error-quot-No-suitable-servers/ta-p/344412) (344412)