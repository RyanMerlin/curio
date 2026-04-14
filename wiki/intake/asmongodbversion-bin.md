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
created_at: 2026-04-14T15:18:28Z
updated_at: 2026-04-14T15:18:28Z
confidence: null
cross_refs: []
content_hash: sha256:04d5a9da2764c0b1927b4b10e03e808e534ff22420e7f64d70f92b4756cb6065
confluence_page_id: null
model_used: null
---

> **ℹ️ Info**
>
> **ASMongoDBVersion.bin** is a text file in the Mongo database folder.  It's checked during a Server upgrade to determine the Mongo database version and if a Mongo version upgrade is needed.  The first line, with the Mongo version number, is the only line that's needed. Mongo does not create or use this file, it's only used by the Server upgrade process.

| **Key Articles** | [Embedded MongoDB upgrade / migration](https://alteryx.atlassian.net/wiki/search?text=Embedded+MongoDB+upgrade+/+migration)   <== what folders are created during a MongoDB version upgrade |
| --- | --- |
| **File Location** | Found in the **Persistence Folder** as set in the **Alteryx System Settings > Controller > Persistence > Data Folder** |

---

---

# Version Grid

> **ℹ️ Info**
>
> While the original **ASMongoDBVersion.bin** contents may include several lines, the file only needs to contain the version number on a line of its own

| **Content of** **ASMongoDBVersion.bin** (nothing else needed) | **First Server Version** | **Can Upgrade to ** | **Notes** |
| --- | --- | --- | --- |
| #### 8.tbd | 25.2 | tbd |  |
| #### 7.0.9 | 24.2 | 8.tbd |  |
| #### 6.0.5 | 23.2 | 7.0.9 | Upgrade errors for 6.0.5     - Could not finalize mongodb restore the mongodb database failed to start with exit code 100 (KB)    - Alteryx Server Upgrade "Error: Could not start previous version of MongoDB: The MongoDB database failed to start with exit code: 14." (KB) |
| #### 4.2.22 | 22.3 22.1.1_Patch3 21.4.2_Patch5 21.3.8_Stable | 6.05 | We did a minor upgrade to 4.2.22 in several patches, see TGAL-677277dcf2c9-72f3-3ff6-8e99-fe88e9f473f1System Jira |
| #### 4.2.15 | 21.3.6 | 6.05 | 6.05     - Edit ASMongoDBVersion.bin to contain 4.2.22 prior to upgrading to a Mongo 6.0.5 version of Server |
| #### 4.0.10 | 19.3 | 4.2.15 4.2.22 *6.05 w/ chg* | 6.05     - Edit ASMongoDBVersion.bin to contain 4.2.22 prior to upgrading to a Mongo 6.0.5 version of Server |

---

# What if customer unchecks option for MongoDB upgrade during installation?

> **📝 Note**
>
> In Mongo upgrades prior to 7.0, the customer was provided the OPTION to upgrade the Embedded MongoDB version.  Since this is a required step, unchecking the MongoDB upgrade leads to the Service being set to Manually run and refusing to start.

See

- How to upgrade Embedded MongoDB version manually