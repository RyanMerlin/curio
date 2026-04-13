---
id: d8b45be104aeaaf8
title: Errors (CryptoMigration Log)
status: published
source:
  kind: confluence_page
  id: confluence-page:1640760350
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/1640760350
  summary: null
category:
- product-tree
- alteryx-server
keywords:
- errors
- cryptomigration
- error
- more
- page
created_at: 2026-04-13T01:55:58Z
updated_at: 2026-04-13T01:58:50Z
confidence: 0.55
cross_refs: []
content_hash: sha256:f64d8463939d7f05c48257f2bcd8eaa4f5eb963f850c52c5b73b52e92bb222d9
confluence_page_id: null
model_used: heuristic
---

# CryptoMigration Errors

> **ℹ️ Info**
>
> Errors in **AlteryxServiceMigrator_#.log** generally contain the status code **;3;, ;2;**, or** ;1;  **Starting in Jun-24 we’re seeing code **;4;** (but this should change to **;1; **soon, see [TGAL-11268](https://alteryx.atlassian.net/browse/TGAL-11268) )also reporting errors and issues, more details:
> 
> - https://alteryx.atlassian.net/wiki/spaces/SupportServer/pages/1640761815/CryptoMigration+Log+Files+-+there+are+TWO#Log-Status-Codes

> **ℹ️ Info**
>
> Several error messages were improved in **24.1** and may be backported.  See bottom of this page for more details.

---

---

## Project to improve the CryptoMigration errors, first released in 24.1 with possible backports coming

- Crypto Migration Error Improvements  <== maps old to new errors
- GS-321877dcf2c9-72f3-3ff6-8e99-fe88e9f473f1System Jira
- E3T VoC (Salesforce)                                      <= E3T VOC request to improve these errors with additional context
- https://alteryx.atlassian.net/wiki/spaces/SupportCseBasics/pages/1999244069/CSU+Tech+Talk+24+Q1#Improved-CryptoMigration-Errors
- Dev improved some of the more cryptic CryptoMigration errors in 24.1 and will likely backport to older versions
- Most add more detail at the end and are listed on the existing Confluence page

- In one case, three different errors were replaced with a new error. A new CryptoMigration error page was created so you can find this error. The esiting pages remain so you can find both the old and new messages. The resolutions from the three seperate errors will be merged.