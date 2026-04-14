---
id: 5418b254c430c0d5
title: Reindex MongoDB
status: intake
source:
  kind: confluence_page
  id: confluence-page:1702893899
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/1702893899
  summary: null
category: []
keywords: []
created_at: 2026-04-14T15:06:10Z
updated_at: 2026-04-14T15:06:10Z
confidence: null
cross_refs: []
content_hash: sha256:3c2c9800198e042c4cc59f0c67430710d1d6aede59f14d10c93ed17c6e9c83a5
confluence_page_id: null
model_used: null
---

---

**24.1+**

- 24.1+ Mongo Reindexing Overview
- How to Troubleshoot a Failed 24.1+ Reindex
- Gallery Logging of 24.1+ MongoDB Reindexing
- Errors (Reindex MongoDB) <== 23.1+ indexing errors

**23.1 / 23.2**

- 23.1/23.2 Mongo Reindexing Overview
- How to Troubleshoot a Failed 23.1/23.2 Reindex
- Gallery Logging of 23.1/23.2 MongoDB Reindexing
- Errors (Reindex MongoDB) <== 23.1+ indexing errors

**22.3 and prior**

- Gallery Logging of 22.3 and prior MongoDB Reindexing

---

> **ℹ️ Info**
>
> Mongo indexes are used to list elements such as **Workflows **in your **Workspace **or **Users **when adding them to a **Shared Gallery Connection**.
> 
> If the indexes become out of sync it can lead to assets (Workflows or Users in the examples above) to no longer appear in the Server UI despite existing in the database.
> 
> Reindexing will rebuild the indexes with the current data and restore Server UI functionality

> **ℹ️ Info**
>
> **23.1** moved from Lucene Indexes to Index collections we manage in AlteryxGallery.  The AlteryxGallery_Lucene database can be deleted in 23.1+.

|  |  |
| --- | --- |
|  |  |
|  |  |
|  |  |

---

# Sections of the Google Reindexing doc not moved to the new KB Aug-2024

The two docs:

- https://docs.google.com/document/d/1Oqh0SqKTRfBsJF695286xmyBs7K4Y_g3tTJq7wSFxRY/edit?usp=sharing <== old Google doc
- How to and When to Run a Re-index (Manual) (KB) <== new KB

## Multi-node considerations

1. After performing the MongoDB backup, don't restart the multinode environment.

It's safest to ensure there is no interaction with the database while we perform Steps 3 to delete records from three collections.

1. Start mongo directly from a command-line on the Controller machine (which has the mongo persistence directory).  Adjust paths to the mongod.exe and the persistence folder (found in Alteryx System Settings > Controller > Persistence > Data Folder).  This is a SINGLE command line, edit the paths as needed:

`"C:\Program Files\Alteryx\bin\mongod.exe" --dbpath "C:\ProgramData\Alteryx\Service\Persistence\MongoDB" --port 27018 --bind_ip_all --auth`

1. Open a second command-line window and perform steps Step 3.
2. In the command window that started mongodb it's imperative you stop MongoDB otherwise it will block the Controller from accessing the database:

-** Ctrl-C** or enter the command **exit**

- ensure you are out of the mongo environment

- close the command window

1. Carry on with the remaining steps above.

## User-Managed Mongo Considerations

The above applies to built-in Mongo.  For user-managed, please see <https://help.alteryx.com/current/server/mongodb-advanced-connection-strings>

If you have trouble starting MongoDB, contact Support and refer them to the following internal articles:

Internal - [How to and When to Run a Re-index (Manual)](https://alteryx.lightning.force.com/kA02R0000000v4LSAQ) 
Internal - [Using command line to rebuild indexes when you cant use API](https://alteryx.lightning.force.com/kA02R0000000uySSAQ)

The above article (660998) doesn't explain in detail how to get the user-managed mongo database location information from the Alteryx System Settings > Gallery > Persistence screen and how to use that information to update the command-line call to reindex.  The screenshot below shows a user-managed mongo with replica sets and how the information would be used to build the command-line call described in the article.

## Command-line reindexing

The command-line reindexing can be used in the following situations:

- Using MongoDB Atas Database or a TLS-enabled User-Managed Mongo DB
- Gallery API can't be used because of network restrictions, a reverse proxy, or a load balancer.

Internal - [Using command line to rebuild indexes when you cant use API](https://alteryx.lightning.force.com/kA02R0000000uySSAQ)

User-managed mongo REINDEX[23.1+]AlteryxServerHost.exe --rebuild -mongoconnection:mongodb://user:MONGO_NON_ADMIN_PASSWORD @HOST_NAME:27017/AlteryxGallery?connectTimeoutMS=25000[22.3 and prior]AlteryxServerHost.exe --rebuild -mongoconnection:mongodb://user:MONGO_NON_ADMIN_PASSWORD @HOST_NAME:27017/AlteryxGallery?connectTimeoutMS=25000 -luceneconnection:mongodb://user:MONGO_NON_ADMIN_PASSWORD @HOST_NAME:27017/AlteryxGallery_Lucene?connectTimeoutMS=25000 -searchProvider:Lucene