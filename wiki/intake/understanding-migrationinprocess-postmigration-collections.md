---
id: eaee0c5e0d92a988
title: Understanding MigrationInProcess / PostMigration collections
status: intake
source:
  kind: confluence_page
  id: confluence-page:2266399210
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/2266399210
  summary: null
category: []
keywords: []
created_at: 2026-04-14T15:18:28Z
updated_at: 2026-04-14T15:18:28Z
confidence: null
cross_refs: []
content_hash: sha256:fb7fa20ab19d7fe6d6d7409196b1f0a16b1017d729294eef27bc01d3eb933f0d
confluence_page_id: null
model_used: null
---

> **ℹ️ Info**
>
> Explanation of the Miongo collections with names like **usersMigrationInProgress **and **usersPostMigration_45 **and how to use them for rollbacks

| **Logs** | [Schema Migration logs for Gallery and Service (alteryx-XXX-migration.csv)](https://alteryx.atlassian.net/wiki/search?text=Schema+Migration+logs+for+Gallery+and+Service+(alteryx-XXX-migration.csv)) |
| --- | --- |
| **Help** | <https://help.alteryx.com/current/en/server/configure/database-management/mongodb-management/mongodb-schema-reference.html> |

---

---

# Background

During Server Upgrade, developers often update the schema (record definition) of collections in both the **AlteryxGallery **and **AlteryxService **databases to add, adjust. or remove fields to match changes in Server functionality.  This is called **Schema Migration**.

---

# Collections created during Upgrade / Schema Migration

| **Collection** | **What is it** |
| --- | --- |
| **collectionX** | Initial collection |
| **collectionXMigrationInProcess** | Records are being actively migrated from **collectionX **to this collection during upgrade / schema migration.  If schema migration is stuck, It’s safe to rename this collection so schema migration will re-attempt migrating **collectionX**.  If the Service is still starting, this collection will be immediately recreated as schema migration continues. |
| **collectionXPostMigration_##** | After the schema migration completes collections are rotated:     - ORIGINAL collectionX  ==> collectionXPostMigration_##    - collectionXMigrationInProcess  ==>  NEW collectionX  If there is a schema migration error that **collectionXPostMigration_##** already exists, it’s safe to rename this collection.  If the Service is still starting, this collection will soon be recreated as schema migration continues. |

---

# Understanding the ## for collectionXPostMigration_## collections

The naming convention used is confusing and misleading.

These collections are a backup of the original **collectionX** collection **BEFORE** the schema migration to **##.  **A better name would have been **collectionXPreMigration_## **as this represents the collection as it was prior to the schema migration to ##.  They can be used in a few ways:

- Confirm what data looked like just before an upgrade
- Rollback an upgrade in the case there was no backup prior to upgrade
- Use it to try to perform schema migration / upgrade for the individual collection on an APOD

## Example 1 – Schema migrates on each upgrade

Translating these file names to understand what version they came from takes a bit of thought.

Example below: **users** collection for a customer who started with **23.1-LTS** and upgraded individually through all patches and Versions, eventually upgrading to **24.1**.

When they upgraded from **23.1-LTS** TO **23.1_Patch_1** the pre-upgrade **users **collection was renamed **usersPostMigration_45**, therefore **usersPostMigration_45 **is actually **23.1-LTS / schema version 44.**

## Example 2 – Schema doesn’t migrate often

Many collections remain stable during upgrades and do not create PostMigration collections.

Example below: **subscriptions **collection for a customer who started with **21.4-LTS** and upgraded individually through all patches and Versions, eventually upgrading to **24.1**.

In this case, the upgrades to **22.1**, **22.3**, and **23.1** didn’t lead to a schema migration, so no backup **PostMigration## **collection was created for these upgrades.

---

# How to use the PostMigration collections for a rollback

If the customer has no backup and their IT didn’t take snapshots of the Server and they need to roll back an upgrade, you can do this through careful renaming of the PostMigration collections.

This assumes the same version on Mongo.  **tbd **- Can you backup a later Mongo version and restore to an earlier version?  If so, you can do that and also edit the [ASMongoDBVersion.bin](https://alteryx.atlassian.net/wiki/search?text=ASMongoDBVersion.bin) to the earlier Mongo version, this would allow the database to be started with the earlier Mongo versions.

(1) Backup the current database before making changes.

(2) Determine which collections were migrated and therefore created a backup **xxxPostMigration_## **collection.  These would have a ## greater than the schema version prior to upgrade

(3) Rename the primary collection **collectionX** to **collectionX_Bkp_YYYYMMDD**

(4) Rename the FIRST PostMigration collection with a higher schema version to be the next primary collection (**collectionX**)

(5) Delete records for the **versions** collection so the last record matches the schema version you are rolling back to.

(6) Uninstall the current Server version.

(7) Install the rollback version of Server