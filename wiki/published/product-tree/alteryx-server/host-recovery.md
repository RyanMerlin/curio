---
id: c3fff7762fbe4739
title: Host Recovery
status: published
source:
  kind: confluence_page
  id: confluence-page:1803420127
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/1803420127
  summary: null
category:
- product-tree
- alteryx-server
keywords:
- tgal
- host
- recovery
- tgaldcfcfffefeeffsystem
- jira
created_at: 2026-04-13T01:55:58Z
updated_at: 2026-04-13T02:02:48Z
confidence: 0.55
cross_refs: []
content_hash: sha256:1f430619bd87c60cacf1494d7b3de1d655814db230965ce5a42aaf7caea25b9c
confluence_page_id: null
model_used: heuristic
---

---

**Host Recovery** is defined as restoring a Mongo database that was backed up on another machine.  This occurs when customers setup a Sandbox using the database from their Prod or Dev Server or if they want to move their Prod environment to a higher-spec server.  They MUST follow the Host Recovery process for this to be successful.

<https://help.alteryx.com/current/en/server/install/server-host-recovery-guide.html>

**Script for Host Recovery** released in 25.2 and is compatible with 24.2+

**It requires the older Flexera license key**, not Alteryx One licensing

Not following the Host Recovery process leads to encrpytion issues with credentials,

- Credentials
- DCM
- Gallery Database Connections

- DCM couldn’t be transferred in a Host Recovery until versions supported the https://help.alteryx.com/current/en/server/install/server-host-recovery-guide/encryption-key-transfer-process.html <== see patch versions on this page

Original fix using command-line led to a new issues (TGAL-8320)

TGAL-744677dcf2c9-72f3-3ff6-8e99-fe88e9f473f1System Jira (TGAL-7446)

Solution Options for TGAL-7256 (TGAL-7256) **> DCME Key Backups** has the procedure that needs to be done for the fix once it's released.  Procedure hasn't changed even though the code for what is done has changed.

==

Defect created for issue found with original fix (TGAL-7446)

TGAL-832077dcf2c9-72f3-3ff6-8e99-fe88e9f473f1System Jira (TGAL-8320)

New story created to address TGAL-8320: TGAL-856977dcf2c9-72f3-3ff6-8e99-fe88e9f473f1System Jira (TGAL-8569)

This is hopefully the final fix.  When the code from this story is released, the procedure in the DCME Key Backups page should address all the issues in all versions this story is released on.

==

This is rolling out in in patches

TGAL-676477dcf2c9-72f3-3ff6-8e99-fe88e9f473f1System Jira (TGAL-6764)