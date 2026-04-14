---
id: 99312377edaa9ff6
title: Mongo Entity-Relationship Diagram (ERD)
status: intake
source:
  kind: confluence_page
  id: confluence-page:1776681530
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/1776681530
  summary: null
category: []
keywords: []
created_at: 2026-04-14T15:18:28Z
updated_at: 2026-04-14T15:18:28Z
confidence: null
cross_refs: []
content_hash: sha256:d19a9e85533177f1474d8501d5c14e335795e2ff6291351fa9866c4413906be6
confluence_page_id: null
model_used: null
---

> **ℹ️ Info**
>
> The Dev Team does not maintain and ERD for Mongo.  Below is what Support has developed.

| #### Entity-Relation Diagram | London Hanson has carried forward a project Sophia F started to create an ERD     - ERD (Lucid Chart)For an ERD of version 23.1+ (w/o Lucene), scroll to the right       - For an ERD of version 23.1+ (w/o Lucene), scroll to the right     - as of Jul-2025 |
| --- | --- |
| #### Schema | <https://help.alteryx.com/current/server/alteryxgallery-mongodb-schema>  <https://help.alteryx.com/current/server/alteryxservice-mongodb-schema>  [Example MongoDB Queries / Commands](https://alteryx.atlassian.net/wiki/search?text=Example+MongoDB+Queries+/+Commands)  <== **some info on specific collections**  [SPECIFIC COLLECTIONS](https://alteryx.atlassian.net/wiki/search?text=SPECIFIC+COLLECTIONS) |
| **Collections created during upgrade** | **Schema Migration**     - XxxMigrationInProcess  <== temp collection while migrating schema    - XxxPostMigration_##     <== backup BEFORE schema migration    - Understanding MigrationInProcess / PostMigration collections  **CryptoMigration**     - AS_Xxx.22.3                     <== temp collection while CryptoMigration    - AS_Xxx.Pre22.2               <== backup BEFORE CrytpoMigration    - FAQ / Help - CryptoMigration    - FAQ / Help - CryptoMigration |
| #### Collection Hierarchy Navigation | **Collection hierarchy for Appinfos seems to be**     - appinfos  > AS_Applications    > AS_ApplicationVersions (and .Files)      > AS_PackageDefinitions (and .Files)        > AS_AppChunks (and .Files)          > Tenth Circle of Hell  **Matt H workflow that connects from appinfos to AS_AppChunk level** |