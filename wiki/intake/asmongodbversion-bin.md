---
id: a6c6a58589511219
title: ASMongoDBVersion.bin
status: intake
source:
  kind: confluence_page
  id: confluence-page:2652471356
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/2652471356
  summary: null
category: []
keywords: []
created_at: 2026-04-14T15:09:17Z
updated_at: 2026-04-14T15:09:17Z
confidence: null
cross_refs: []
content_hash: sha256:f574d1230646dc81b2b9bb229ba62132d3604417a2a87e33fd9980519aa5a118
confluence_page_id: null
model_used: null
---

> **ℹ️ Info**
>
> **ASMongoDBVersion.bin** is a text file in the Mongo database folder.  It's checked during a Server upgrade to determine the Mongo database version and if a Mongo version upgrade is needed.  The first line, with the Mongo version number, is the only line that's needed. Mongo does not create or use this file, it's only used by the Server upgrade process.

|  |  |
| --- | --- |
|  |  |

---

---

# Version Grid

> **ℹ️ Info**
>
> While the original **ASMongoDBVersion.bin** contents may include several lines, the file only needs to contain the version number on a line of its own

|  |  |  |  |
| --- | --- | --- | --- |
|  |  |  |  |
|  |  |  |  |
|  |  |  |  |
|  |  |  |  |
|  |  |  |  |
|  |  |  |  |

---

# What if customer unchecks option for MongoDB upgrade during installation?

> **📝 Note**
>
> In Mongo upgrades prior to 7.0, the customer was provided the OPTION to upgrade the Embedded MongoDB version.  Since this is a required step, unchecking the MongoDB upgrade leads to the Service being set to Manually run and refusing to start.

See

- How to upgrade Embedded MongoDB version manually