---
id: 964bbd0f66ece1d4
title: How to build remove() queries from CryptoMigration logs
status: published
source:
  kind: confluence_page
  id: confluence-page:1950515201
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/1950515201
  summary: null
category:
- product-tree
- alteryx-server
keywords:
- error
- workflow
- remove
- httpsalteryxatlassiannetwikispacessupportserverpages
- backported
created_at: 2026-04-13T01:55:58Z
updated_at: 2026-04-13T02:02:49Z
confidence: 0.55
cross_refs: []
content_hash: sha256:70f9bdfd37279c5c2b6c8235f9b925868e84a8986d7da4c4ed529c713a8fbf25
confluence_page_id: null
model_used: heuristic
---

> **ℹ️ Info**
>
> 22.3 CryptoMigration errors often require deleting records from AS_RunAsCredentials, AS_Queue, etc.  If there are more than a handful, use the workflow below created by Daniel Barber to parse the CryptoMigation log and build the necessary Mongo remove() queries.

note 24.1 changes the error messages and may impact this workflow.  It’s unclear if the error message changes will be backported to 22.3, 23.1, 23.2.

24.1 changes the error messages and may impact this workflow.  It’s unclear if the error message changes will be backported to 22.3, 23.1, 23.2.

| Access | Updated 10/04/24noteUPDATES NEEDEDmongo deprecated remove() in favor of deleteMany().It doesn’t appear the workflow is generating queries for:CryptoMigration Log Error - Error Migrating PackageDefinitionMigration <XXX> Error: Unable to make staging directory. https://alteryx.atlassian.net/wiki/spaces/SupportServer/pages/2204174300 A few errors were updated by Product in 24.1 and possibly backported to earlier versions. The change in the error message isn’t being caught by the current filters in the workflow:CryptoMigration Log Error - Error Unpackaging app: <XXX> Error: Error in validating workflow chunks. Please check workflow is valid in Server UI. - RENAMED 3x https://alteryx.atlassian.net/wiki/spaces/SupportServer/pages/1640696624 
UPDATES NEEDEDmongo deprecated remove() in favor of deleteMany().It doesn’t appear the workflow is generating queries for:CryptoMigration Log Error - Error Migrating PackageDefinitionMigration <XXX> Error: Unable to make staging directory. https://alteryx.atlassian.net/wiki/spaces/SupportServer/pages/2204174300 A few errors were updated by Product in 24.1 and possibly backported to earlier versions. The change in the error message isn’t being caught by the current filters in the workflow:CryptoMigration Log Error - Error Unpackaging app: <XXX> Error: Error in validating workflow chunks. Please check workflow is valid in Server UI. - RENAMED 3x https://alteryx.atlassian.net/wiki/spaces/SupportServer/pages/1640696624 |
| --- | --- |
| Questions? | Daniel Barber |
| What does the workflow do? | This workflow Reads the customer’s AlteryxServiceMigrator_#.log Parses for CryptoMigration ErrorsBuilds the necessary Mongo remove() commands The Browse Tools will have the complete query and you can right-click and copy without headers |
| Past versions |  |