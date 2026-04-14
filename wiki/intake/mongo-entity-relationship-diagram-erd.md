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
created_at: 2026-04-14T15:02:19Z
updated_at: 2026-04-14T15:02:19Z
confidence: null
cross_refs: []
content_hash: sha256:6d8be1b470524124084bf6ca54abc283490055e7dea26e5e29edc3a1642701e4
confluence_page_id: null
model_used: null
---

> **ℹ️ Info**
>
> The Dev Team does not maintain and ERD for Mongo.  Below is what Support has developed.

| Entity-Relation Diagram | London Hanson has carried forward a project Sophia F started to create an ERDERD (Lucid Chart)For an ERD of version 23.1+ (w/o Lucene), scroll to the right  as of Jul-2025 |
| --- | --- |
| Schema | https://help.alteryx.com/current/server/alteryxgallery-mongodb-schema https://help.alteryx.com/current/server/alteryxservice-mongodb-schema Example MongoDB Queries / Commands  <== some info on specific collectionsSPECIFIC COLLECTIONS |
| Collections created during upgrade | Schema MigrationXxxMigrationInProcess  <== temp collection while migrating schemaXxxPostMigration_##     <== backup BEFORE schema migrationUnderstanding MigrationInProcess / PostMigration collections  CryptoMigrationAS_Xxx.22.3                     <== temp collection while CryptoMigrationAS_Xxx.Pre22.2               <== backup BEFORE CrytpoMigrationFAQ / Help - CryptoMigration FAQ / Help - CryptoMigration |
| Collection Hierarchy Navigation | Collection hierarchy for Appinfos seems to beappinfos  > AS_Applications    > AS_ApplicationVersions (and .Files)      > AS_PackageDefinitions (and .Files)        > AS_AppChunks (and .Files)          > Tenth Circle of HellMatt H workflow that connects from appinfos to AS_AppChunk level |