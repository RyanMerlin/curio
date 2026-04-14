---
id: cc36d4a37c2ad48b
title: Start MongoDB
status: intake
source:
  kind: confluence_page
  id: confluence-page:1702893171
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/1702893171
  summary: null
category: []
keywords: []
created_at: 2026-04-14T15:16:58Z
updated_at: 2026-04-14T15:16:58Z
confidence: null
cross_refs: []
content_hash: sha256:045975427e9b5c2f69f3ea6df13bb7817c8ba4c2cbd5fb99836e1af2c19d24d8
confluence_page_id: null
model_used: null
---

for embedded mongo onlyPurple

---

---

> **ℹ️ Info**
>
> How to start the Mongo database if the Service isn’t running so you can access it from Robo3T or Mongo Shell

| **Start Mongo DB** | Open a Command Prompt As Administrator and navigate to **\Alteryx\bin** folder, then:  X:  cd \FOLDER \Alteryx\bin **mongod.exe --dbpath "DRIVE:\PERSISTENCE_FOLDER" --port 27018** A successful start will show the following in one of the last few lines  If the database doesn't start, review the messages to determine a cause and see Repair MongoDB.  Type **Ctrl-C** to stop the database.  > **📝 Note** > > Remember to stop the database before re-starting the Service so the Service can start Mongo itself  [Based on [How to Start Mongo Manually Using the Mongo Executable (Daemon) mongod.exe](https://community.alteryx.com/t5/Product-Resources/How-to-Start-Mongo-Manually-Using-the-Mongo-Executable-Daemon/ta-p/630951) (630951)] |
| --- | --- |
| **--auth flag** | Adding **--auth** flag to the command to start MongoDB will require Roboo3T or MongoDB Shell to supply a user and password.  Without --auth, you can     - Connect Robo3T to all three databases by unchecking Perform authentication when setting up the connection    - Start Mongo Shell without either the -u or -p parameters  **Without auth**:     - mongod.exe --dbpath "DRIVE:\PERSISTENCE_FOLDER" --port 27018  **With auth:**     - mongod.exe --dbpath "DRIVE:\PERSISTENCE_FOLDER" --port 27018 --auth |
| **Errors** | > **⚠️ Warning** > > 2024-01-12T23:03:47.650+0000 I  STORAGE  [initandlisten] exception in initAndListen: DBPathInUse: **Unable to create/open the lock **[file: C:\ProgramData\Alteryx\Service\Persistence\MongoDB_07\mongod.loc](#)k (The process cannot access the file because it is being used by another process.). Ensure the user executing mongod is the owner of the lock file and has the appropriate permissions. Also make sure that another mongod instance is not already running on the C:\ProgramData\Alteryx\Service\Persistence\MongoDB_07 directory, terminating  The Service or another instance of Mongo is already running.  Stop them or simply start the mongo shell since Mongo is running. |
| **Older version of mongod.exe** | An older version of mongod.exe is in the BIN folder as well, to be able to manually start an older version database folder.  For more details and troubleshooting see [How to Start Mongo Manually](https://community.alteryx.com/t5/Product-Resources/How-to-Start-Mongo-Manually-Using-the-Mongo-Executable-Daemon/ta-p/630951) (630951) |