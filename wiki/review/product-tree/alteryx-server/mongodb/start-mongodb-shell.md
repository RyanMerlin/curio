---
id: 4ccfefdfbe06ecd1
title: Start MongoDB Shell
status: review
source:
  kind: confluence_page
  id: confluence-page:1702828761
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/1702828761
  summary: null
category:
- product-tree
- alteryx-server
- mongodb
keywords:
- mongodb
- shell
- mongo-shell
- embedded-mongo
- diagnostic
created_at: 2026-04-14T13:40:14Z
updated_at: 2026-04-14T13:50:58Z
confidence: 0.8
cross_refs: []
content_hash: sha256:de7cff60d662cb3a833922571e49ef2ea352dc7bdfcfdfec17c4728178885556
confluence_page_id: null
model_used: claude-sonnet-4-6
---

for embedded mongo onlyPurple

---

*[Organized section — child pages listed separately]*

---

> **ℹ️ Info**
>
> The MongoDB Shell provides command-line access to the currently running MongoDB instance.

---

| Start MongoDB Shell | Get the Alteryx System Settings > Controller > Persistence > Non-Admin PasswordIf the Service isn’t running, then Start MongoDB.Open a Command Prompt As AdministratorNavigate to the \Alteryx\bin folderEnter commands below for the AlteryxGallery database#E3FCEF[ for 23.2+ ]mongosh -u user -p USER_PSWD -host localhost:27018 AlteryxGallery[ for 23.1 and prior ] mongo -u user -p USER_PSWD -host localhost:27018 AlteryxGalleryOr open the AlteryxService database#E3FCEF[ for 23.2+ ]mongosh -u user -p USER_PSWD -host localhost:27018 AlteryxService[ for 23.1 and prior ]mongo -u user -p USER_PSWD -host localhost:27018 AlteryxServiceYou may now enter commands, see Example MongoDB Queries / Commands Note:  If you started MongoDB manually withOUT the --auth flag you do need to include the following to start the Shell#E3FCEF-u user -p USER_PSWDType Ctrl-C to exit the Shell.How to connect to MongoDB from the Command Line (KB) |
| --- | --- |
| Troubleshooting | Error: couldn't connect to server localhost:27018, connection attempt failed: SocketException: Error connecting to localhost:27018 (127.0.0.1:27018) :: caused by :: No connection could be made because the target machine actively refused it.The above error indicates the Mongo DB isn’t running, see Start MongoDB |
| Older version of mongo.exe | An older version of mongo.exe is in the BIN folder as well, to be used if you used an older version of mongod.exe to start the Database. |