---
id: 4ccfefdfbe06ecd1
title: Start MongoDB Shell
status: intake
source:
  kind: confluence_page
  id: confluence-page:1702828761
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/1702828761
  summary: null
category: []
keywords: []
created_at: 2026-04-14T15:18:28Z
updated_at: 2026-04-14T15:18:28Z
confidence: null
cross_refs: []
content_hash: sha256:6c7a4276444c514b23e6427a0743bdb9c505d4935d1bf418d84ed6ddabeb1f5e
confluence_page_id: null
model_used: null
---

for embedded mongo onlyPurple

---

---

> **ℹ️ Info**
>
> The MongoDB Shell provides command-line access to the currently running MongoDB instance.

---

| #### Start MongoDB Shell | 1. Get the Alteryx System Settings > Controller > Persistence > Non-Admin Password    2. If the Service isn’t running, then Start MongoDB.    3. Open a Command Prompt As Administrator    4. Navigate to the \Alteryx\bin folder    5. Enter commands below for the AlteryxGallery database  **[ for 23.2+ ]** mongosh -u user -p USER_PSWD  -host localhost:27018 AlteryxGallery  **[ for 23.1 and prior ] ** mongo -u user -p USER_PSWD  -host localhost:27018 AlteryxGallery Or open the AlteryxService database  **[ for 23.2+ ]** mongosh -u user -p USER_PSWD -host localhost:27018 AlteryxService  **[ for 23.1 and prior ]** mongo -u user -p USER_PSWD -host localhost:27018 AlteryxService You may now enter commands, see     - Example MongoDB Queries / Commands  ---  Note:  If you started MongoDB manually withOUT the **--auth **flag you do need to include the following to start the Shell  -u user -p USER_PSWD ---  Type **Ctrl-C** to exit the Shell.  [How to connect to MongoDB from the Command Line](https://knowledge.alteryx.com/index/s/article/How-to-connect-to-MongoDB-from-command-line) (KB) |
| --- | --- |
| #### Troubleshooting | > **⚠️ Warning** > > Error: couldn't connect to server localhost:27018, connection attempt failed: SocketException: Error connecting to localhost:27018 (127.0.0.1:27018) :: caused by :: **No connection could be made because the target machine actively refused it.**  The above error indicates the Mongo DB isn’t running, see [Start MongoDB](https://alteryx.atlassian.net/wiki/search?text=Start+MongoDB) |
| #### Older version of mongo.exe | An older version of mongo.exe is in the BIN folder as well, to be used if you used an older version of mongod.exe to start the Database. |