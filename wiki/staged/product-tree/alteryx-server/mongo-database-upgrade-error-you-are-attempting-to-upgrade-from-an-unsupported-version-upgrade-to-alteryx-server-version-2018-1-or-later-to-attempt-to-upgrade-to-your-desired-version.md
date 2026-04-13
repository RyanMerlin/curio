---
id: 8cb2a8060c1e486a
title: Mongo Database Upgrade Error - You are attempting to upgrade from an unsupported version. Upgrade to Alteryx Server version 2018.1 or later to attempt to upgrade to your desired version.
status: published
source:
  kind: confluence_page
  id: confluence-page:2730393723
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/2730393723
  summary: null
category:
- product-tree
- alteryx-server
keywords:
- version
- upgrade
- mongodb
- server
- versions
created_at: 2026-04-13T01:55:58Z
updated_at: 2026-04-13T01:58:58Z
confidence: 0.55
cross_refs: []
content_hash: sha256:54b63d828bbb4dbd42cf6efc91bc4348855806a41e90db85e33221ed773583a4
confluence_page_id: null
model_used: heuristic
---

| Context |  |
| --- | --- |
| Error | You are attempting to upgrade from an unsupported version. Upgrade to Alteryx Server version 2018.1 or later to attempt to upgrade to your desired version. |
| Screenshot |  |
| Related Errors | Mongo Database Upgrade Error - You are upgrading from a version of Server that utilizes MongoDB version older than 6.0 <== Mongo 7.0 upgrade |
| Versions |  |

# Troubleshooting

|  | Check | Steps |
| --- | --- | --- |
| 1 | MongoDB versions if the pre and post upgrade versionsTrying to upgrade more than one version of MongoDB | TroubleshootingLook at the mongoDB versions for the pre and post upgrade Server versions to confirm they are only attempting to upgrade ONE version jump in Mongo.https://help.alteryx.com/current/en/server/configure/database-management/mongodb-management/mongodb-schema-reference.html CauseUpgrade can’t upgrade more than one version of MongoDBResolutionRollbackChoose a leser upgrade version that only includes one MongoDB version upgradeConf pagesServer Upgrade Version Paths - What version can upgrade to what versions? |
| 2 | ASMongoDBVersion.binFile is missing or has unexpected vaue | TroubleshootingReview ASMongoDBVersion.bin in their Persistence folderIt should contain the version of MongoDB of their PRE-upgrade Serverhttps://help.alteryx.com/current/en/server/configure/database-management/mongodb-management/mongodb-schema-reference.html  <== MongoDB version for each Server versionASMongoDBVersion.bin  <== explains this text file and what the file shold contain  for each MongoDB versionExample:  Pre-upgrade 23.2 ASMongoDBVersion.bin should contain 6.0.5  (NOTE: the file can have other text, we only care about the version number)Upgrade to 24.2 MongoDB v7.0 will check ASMongoDBVersion.bin to confirm it contains 6.0.5If it contains the older value 4.2.22 it means the DB is from a Server older than 23.2, or that the previous upgrade to 23.2 didn’t properly update this file to 6.0.5CauseIf ASMongoDBVersion.bin is missing or does NOT contain the expected value, the error at the top of this page will occur.ResolutionEdit ASMongoDBVersion.bin and enter the expecte MongoDB version of the PRE-upgrade Server found in the first column of the table on page:ASMongoDBVersion.bin Stop Service (if running)Manually upgrade the databaseHow to upgrade Embedded MongoDB version manually Check the log created in the backup folder the upgrade creates.  This log name seems to be bouncing around from version to version but ends with .LOG. |