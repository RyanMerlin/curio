---
id: cf778d85a0014d18
title: CryptoMigration in 22.3
status: staged
source:
  kind: confluence_page
  id: confluence-page:1640793170
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/1640793170
  summary: null
category:
- by-account
keywords:
- upgrade
- will
- logging
- server
- cryptomigration
created_at: 2026-04-12T20:59:51Z
updated_at: 2026-04-12T20:59:57Z
confidence: 0.55
cross_refs: []
content_hash: sha256:04a84e893a3720f8b8fceda72bff376756f499cdee58aa8e22868f844544e430
confluence_page_id: null
model_used: heuristic
---

---

Upgrade to Server 2022.3 requires a CryptoMigration step to re-encrypt data in Mongo and RuntimeSettings.xml to the AES256 standard (like the FIPs Server) with SHA-256 hashing.

Customers should use the **Migration Prep Tool** to prepare for a 2022.3 Server upgrade and reduce downtime during upgrade.  The Tool will re-encrypt workflows in the background while the Service runs as usual with no downtime.  Or customers can run the Server Upgrade and the CryptoMigration will occur when the Service first starts (not recommended).

The CryptoMigration process was tested for upgrades from 2021.2+.  If coming from an earlier version, upgrade to 2021.2-2021.1 first.  For Built-In Auth, upgrade to 2021.1 first.

Controller > General > Logging  <== Upgrade]]> GCSE-211277dcf2c9-72f3-3ff6-8e99-fe88e9f473f1System Jira

If there is a space in the log folder path the logging will appear in wrong file.  Ex:  **D:\Program Data\Alteryx\Service** will log to a file called **D:\Program** and not rotate.  This is the path set by:

```
Controller > General > Logging]]>
```

Controller > General > Logging]]>