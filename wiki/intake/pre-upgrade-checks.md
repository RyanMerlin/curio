---
id: 3ace91393a47ef5d
title: Pre-Upgrade Checks
status: intake
source:
  kind: confluence_page
  id: confluence-page:1831112050
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/1831112050
  summary: null
category: []
keywords: []
created_at: 2026-04-13T01:55:58Z
updated_at: 2026-04-13T01:55:58Z
confidence: null
cross_refs: []
content_hash: sha256:f820c46d999615444317e23b31bab6ff96964d61b0f5a72ed042a7c0bd8071f0
confluence_page_id: null
model_used: null
---

---

The **Pre-Upgrade Checks workflow** is critical to run prior to upgrading Server.

During an upgrade the Mongo Schema (the definition of the collections/tables and fields) is updated.  This process is a frequent source of upgrade failures.  The **Pre-Upgrade Checks** identify records likely to fail during the so they can be corrected BEFORE the upgrade.