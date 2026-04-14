---
id: bc722dd840b197b8
title: Get MongoDB Non-Admin Password
status: intake
source:
  kind: confluence_page
  id: confluence-page:1702763820
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/1702763820
  summary: null
category: []
keywords: []
created_at: 2026-04-14T15:12:39Z
updated_at: 2026-04-14T15:12:39Z
confidence: null
cross_refs: []
content_hash: sha256:4102f363f3967f8421e75ee582439996949eb12b1ba2243b9b0936e4c86d6bfc
confluence_page_id: null
model_used: null
---

for embedded mongo onlyPurple

> **ℹ️ Info**
>
> The Non-Admin password is used when accessing the database from Robo3T, the MongoDB Shell, or running the Reindesing workflow.

# Option - Alteryx System Settings

**Alteryx System Settings > Controller > Persistence > Password** (do not use the Admin password)

# Option - Command line

Open a Command Line As Administrator

c: 
cd \%ProgramFiles% \Alteryx\bin
AlteryxService.exe getemongopassword
This decrypts the Admin and Non-Admin passwords from the ASCredentials.bin file.