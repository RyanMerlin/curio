---
id: 9d22c624772bb143
title: Host Recovery Encryption Key Transfer Process
status: staged
source:
  kind: confluence_page
  id: confluence-page:2971599161
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/2971599161
  summary: null
category:
- product-tree
- alteryx-server
keywords:
- transfer
- encryption
- patch
- process
- source
created_at: 2026-04-12T20:59:51Z
updated_at: 2026-04-12T21:05:12Z
confidence: 0.55
cross_refs: []
content_hash: sha256:9dee756716df5a597fc70dd1347dcbdaceb80aea363714e0315a4f2b37aaf1f1
confluence_page_id: null
model_used: heuristic
---

---

---

> **ℹ️ Info**
>
> **Encryption Key Transfer** moves the Source **RuntimeSettings.xml <DCMSecretEncrypted>** to the Host Recovery machine.  This is required to decrypt both
> 
> - DCM
> - Gallery Database Connections
> 
> <https://help.alteryx.com/current/en/server/install/server-host-recovery-guide/encryption-key-transfer-process.html>

| Key Articles | https://help.alteryx.com/current/en/server/install/server-host-recovery-guide/encryption-key-transfer-process.html <== linked to in Server Host Recovery Guide |
| --- | --- |
| Versions | The Encryption Key Transfer Process was added in the following versions.  Apply at least the patch level listed below on the Source Server before attempting the Encryption Key Transfer Process.21.4.2 Patch 1122.1 Patch 922.3 Patch 623.1 Patch 2 23.2+ |
| Background | TGAL-7446 / TGAL-7447 created the Encryption Key Transfer Process that willGet the decrypted DCM Secret from the Source RuntimeSettings.xml <DCMSecretEncrypted>Transfer it to the Target ServerMachine-encrypt it to the Target’s RuntimeSettings.xmlNote:  While the DCM Secret will now be the SAME on both the Source and Target Servers, the value is machine-encrypted, so <DCMSecretEncrypted> will appear differently on the two machines. |
| Tutorial | CSU - Host Recovery Encryption Key Transfer Process - 2024-02-22 - Alan Yim |