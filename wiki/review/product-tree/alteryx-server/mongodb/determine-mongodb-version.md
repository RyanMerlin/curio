---
id: 047c582d46a1497f
title: Determine MongoDB Version
status: review
source:
  kind: confluence_page
  id: confluence-page:1702763681
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/1702763681
  summary: null
category:
- product-tree
- alteryx-server
- mongodb
keywords:
- mongodb
- version
- embedded-mongo
- diagnostic
created_at: 2026-04-14T15:18:28Z
updated_at: 2026-04-14T15:19:35Z
confidence: 0.85
cross_refs: []
content_hash: sha256:29169e9ed437f667d0b10748e2de9bafbaca3a7dbfec630b786442ae814c0664
confluence_page_id: null
model_used: claude-sonnet-4-6
---

for embedded mongo onlyPurple

> **ℹ️ Info**
>
> There appears no simple Mongo command can tell the version of MongoDB a persistece folder contains.  Fortunately this issue doesn’t come up often as customers typically know the Server version each backup or PreUpgrade folder relates to.

The MongoDB version number is stored in the text file, **ASMongoDBVersion.bin**, in the persistence folder as described in [ASMongoDBVersion.bin](https://alteryx.atlassian.net/wiki/search?text=ASMongoDBVersion.bin) .  While typically correct, it can be wrong.

---

**ASMongoDBVersion.bin** is not copied to the Pre_Upgrade folders created during a Mongo upgrade. Therefore, the Pre_Upgrade folder date may be your best clue to understanding what upgrade it relates to.

---

You can try to deduce the MongoDB version by trying to start MongoDB with different versions of mongod.exe.  For example, Mongod3_X can't start a Mongo 4.X database, and vice versa.  However, Mongod4_0 can start of Mongo 4.2 database (and vice versa).  So the best you get is a determination of the major version.

---

> **📝 Note**
>
> The upgrade to Mongo 4.0 created a folder ending in "4_0", however the upgrade to Mongo 4.2 was performed in-place.  Therefore the folder name remained ending in 4_0, despite the database being 4.2.  Don't trust the folder name.

---

The **db.version()** mongo function returns the version of Mongod.exe that was run, not the version of the data.  [How To Get the Version of your MongoDB Database (Embedded MongoDB)](https://knowledge.alteryx.com/index/s/article/how-to-get-mongodb-version) (KB) refers to using db.version().

---

The **admin > System > featureCompatibilityVersion.version** field seems to hold a version #.

---

A likely out-of-date internal article: [MongoDB Version (Embedded) Upgrade Guide/FAQs](https://alteryx.lightning.force.com/kA02R000000CsmOSAS) (KB)

---

A public article on this issue, but still seems to be showing the EXE version, not the data file version.

- https://www.geeksforgeeks.org/mongodb/how-to-find-the-exact-version-of-installed-mongodb/

---

More testing would need to be done with known copies of database folders.