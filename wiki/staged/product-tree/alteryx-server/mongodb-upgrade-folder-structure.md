---
id: 58848bbf6262533b
title: MongoDB Upgrade Folder Structure
status: staged
source:
  kind: confluence_page
  id: confluence-page:2979693181
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/2979693181
  summary: null
category:
- product-tree
- alteryx-server
keywords:
- folder
- mongodb
- mongo
- upgrade
- version
created_at: 2026-04-12T20:59:51Z
updated_at: 2026-04-12T21:02:31Z
confidence: 0.55
cross_refs: []
content_hash: sha256:e5b3a5831b4ca1d6977101937af7f238f1fee60475851ee4e0a4f2af78ccba14
confluence_page_id: null
model_used: heuristic
---

---

---

> **ℹ️ Info**
>
> If the MongoDB version is upgraded during a Server upgrade backup folders are created.  These have differed for different versions but are detailed below.
> 
> These backups can be used if the customer needed to rollback their upgrade and had not taken a backup of their own,

---

---

## Upgrade to Mongo 7.0  (upgrading to Server 24.2 - 25.1)

> **ℹ️ Info**
>
> During the embedded MongoDB version upgrade to 7.0, ONE folder is created
> 
> Upgrade to Mongo 7.0 is in place on the MongoDB folder per comments in TGAL-1252677dcf2c9-72f3-3ff6-8e99-fe88e9f473f1System Jira

| Folder | Description | MongoDB version |
| --- | --- | --- |
| MongoDB | Original Persistence folder upgraded to Mongo 6.0 | Mongo 7.0 |
| MongoDB_PreUpgrade | MongoDB backup of pre-upgrade Persistence folder | Mongo 7.0Per Mohsen’s testing, the upgrade occurs in the backup.Based on this, there’s no _Backup folder to use for rollback. |

---

## Upgrade to Mongo 6.0  (upgrading to Server 23.2 - 24.1)

> **ℹ️ Info**
>
> During the embedded MongoDB version upgrade TWO folders are created
> 
> Upgrade to Mongo 6.0 is done in a mongo restore of **MongoDB_PreUpgrade** to **MongoDB**

| Folder | Description | MongoDB version |
| --- | --- | --- |
| MongoDB | Original Persistence folder upgraded to Mongo 6.0 | Mongo 6.0 |
| MongoDB_Backup | Original Persistence folder is renamed _Backup at the start of the MongoDB version upgradeIs this folder missing?  Is the persistence folder under the root of a drive? If so, permissions may have prevented the creation of this folder during DB upgrade.  [Mohsen] | Mongo 4.0/4.2 |
| MongoDB_PreUpgrade | MongoDB backup of pre-upgrade Persistence folder | Mongo 6.0The upgrade to 6.0 occurs as the db is backed up into this folder. |

---

## Upgrade to Mongo 4.2  (upgrading to Server 21.3.6 - 23.1)

> **ℹ️ Info**
>
> During the embedded MongoDB version upgrade ONE folder is created

| Folder | Description | MongoDB version |
| --- | --- | --- |
| MongoDB | Original Persistence folder upgraded to Mongo 4.2 | Mongo 4.2 |
| MongoDB_PreUpgrade | MongoDB backup of pre-upgrade Persistence folder | Mongo 4.0 |

---

## Upgrade to Mongo 4.0  (upgrading to Server 19.3 - 21.3.5)

> **ℹ️ Info**
>
> During the embedded MongoDB version upgrade TWO folders are created

| Folder | Description | MongoDB version |
| --- | --- | --- |
| MongoDB_40 | New folder with upgrade to 4.0 | Mongo 4.0 |
| MongoDB | Untouched, original Persistence folder | Mongo 3.4 |
| MongoDB_PreUpgrade | MongoDB backup of pre-upgrade Persistence folder | Mongo 3.4 |