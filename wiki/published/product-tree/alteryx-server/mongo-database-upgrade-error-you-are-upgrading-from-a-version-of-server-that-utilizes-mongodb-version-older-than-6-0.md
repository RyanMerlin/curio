---
id: 8036d36f5b34bfa8
title: Mongo Database Upgrade Error - You are upgrading from a version of Server that utilizes MongoDB version older than 6.0
status: published
source:
  kind: confluence_page
  id: confluence-page:2807562241
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/2807562241
  summary: null
category:
- product-tree
- alteryx-server
keywords:
- version
- upgrade
- mongo
- server
- mongodb
created_at: 2026-04-12T20:59:51Z
updated_at: 2026-04-12T21:05:57Z
confidence: 0.55
cross_refs: []
content_hash: sha256:0efc3b4239727d5be57b6eb0fd6cd22801c309ca90650e7462b58c929cb55ad3
confluence_page_id: null
model_used: heuristic
---

| Context | Upgrading Server to a version using MongoDB 7.0 from a version using an earlier database |
| --- | --- |
| Error | You are upgrading from a version of Server that utilizes MongoDB version older than 6.0. You need to upgrade to Server version 2023.2 before moving forward. |
| Screenshot |  |
| Related Errors | Mongo Database Upgrade Error - You are attempting to upgrade from an unsupported version. Upgrade to Alteryx Server version 2018.1 or later to attempt to upgrade to your desired version.                        <== Mongo 6.0 upgrade |
| Versions | Upgrade to a version that uses Mongo 7.0 |

# Troubleshooting

|  | Check | Steps |
| --- | --- | --- |
| 1 | Are they upgrading from a Mongo 6.0 version? | CauseThey are upgrading from a Server version using Embedded Mongo version prior to 6.0ResolutionRollback and upgrade to a version of Server that includes only ONE Mongo upgrade at a time.  Refer to:https://help.alteryx.com/current/en/server/configure/database-management/mongodb-management/mongodb-schema-reference.html Cases00742822 |
| 2 | ASMongoDBVersion.bin | Pre-ConditionASMongoDBVersion.bin in the Persistence folder shows a version prior to Mongo 6.0.  For example:CauseIf they were running a version of Server with MongoDB version 6.0 then the issue may be that the content of ASMongoDBVersion.bin is incorrect and is telling the step that performs the MongoDB upgrade the wrong Mongo version.  For example:  4.2.22 instead of 6.0.Resolutionif you are certain they were running a version of Sevver with MongoDB 6.0, directly edit ASMongoDBVersion.bin to contain onlyMore information:  ASMongoDBVersion.bin Re-attempt the Mongo version upgrade manually: How to upgrade Embedded MongoDB version manually Start the Service to begin the Service Schema Migration followed by the Gallery Schema Migration Cases00742314 |