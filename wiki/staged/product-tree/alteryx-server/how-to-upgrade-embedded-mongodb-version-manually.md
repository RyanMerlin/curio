---
id: 9c496c5d7999b435
title: How to upgrade Embedded MongoDB version manually
status: staged
source:
  kind: confluence_page
  id: confluence-page:2438889832
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/2438889832
  summary: null
category:
- product-tree
- alteryx-server
keywords:
- upgrade
- mongodb
- version
- service
- embedded
created_at: 2026-04-12T20:59:51Z
updated_at: 2026-04-12T21:02:30Z
confidence: 0.55
cross_refs: []
content_hash: sha256:7ee1e4f99ca2ca8490c46b9cde8647cdf7eab44acba2df9b73eccb37e973ec2b
confluence_page_id: null
model_used: heuristic
---

> **📝 Note**
>
> In Mongo upgrades prior to 7.0, the customer was provided the OPTION to upgrade the Embedded MongoDB version.  Since this is a required step, unchecking the MongoDB upgrade leads to the Service being set to **Manual** start and refusing to start.
> 
> In this state, the customer must manually upgrade the Embedded MongoDB version and set the Service to **Automatic (Delayed Start) **in the Service app.

| Access | #E3FCEF\Alteryx\bin\MongoDbUpgrade.exe  <== later versions\Alteryx\bin\MongoDbUpgradeTo##.exe  <== earleir versionsYou can run this from File Explorer or the command line.  There are no parameters, it will use the RuntimeSettings.xml Persistence folder setting as the source folder for the upgrade. |
| --- | --- |
| Key Articles | Embedded MongoDB upgrade / migration ASMongoDBVersion.bin  Error in Service logs when upgrading to Server version 23.2: Invalid feature compatibility version value, expected '5.0' or '5.3' or '6.0' (KB) |