---
id: a28543d7b010e27a
title: Understanding DCM in Failover Servers
status: intake
source:
  kind: confluence_page
  id: confluence-page:2011236267
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/2011236267
  summary: null
category: []
keywords: []
created_at: 2026-04-14T15:10:53Z
updated_at: 2026-04-14T15:10:53Z
confidence: null
cross_refs: []
content_hash: sha256:c6ddd01d803f2cc4c89f33abc4bed54347e683cd3454f166ae17e908f1703228
confluence_page_id: null
model_used: null
---

> **ℹ️ Info**
>
> In following the HA Server Setup process in Help or Confluence
> 
> - https://help.alteryx.com/current/en/server/best-practices/high-availability-best-practices.html
> - How to Setup Microsoft failover cluster service in AWS
> 
> you would have been directed to follow steps from the [Server Host Recovery Guide](https://help.alteryx.com/current/en/server/install/server-host-recovery-guide.html) including the<https://help.alteryx.com/current/en/server/install/server-host-recovery-guide/encryption-key-transfer-process.html>
> 
> This page explains what’s happening with the RuntimeSettings.xml **<DCMSecretEncrypted>** during the **Encryption Key Transfer Process** to further your understanding.

> **ℹ️ Info**
>
> HA Failover Servers need to have the HA Primary Controller’s DCM key transferred to them before failover to be able to decrypt both DCM and Shared Database Connection passwords.  Customers who followed Help will have already done this.

# Background

- RuntimeSettings.xml <DCMSecretEncrypted> is used to decrypt both DCM Credentials and Shared Gallery Connections.
- <DCMSecretEncrypted> encryption is machine-specific, therefore the <DCMSecretEncrypted> value will appear differently in each HA Controller’s RuntimeSettings.xml after completing the transfer. Because you can’t visually compare the encrypted values in the RuntimeSettings.xml file, you must ensure you follow the transfer steps and, as always, perform disaster recovery drills periodically to validate the setup.
- This Encryption Key Transfer Process only needs to be done once. However, if the HA Primary Controller’s RuntimeSettings.xml is ever reset, the server Host Recovery steps will need to be performed again to transfer the following from the HA Primary Server to the HA Failover Servers:Controller Token (<ServerSecretEncrypted)<StorageKeysEncrypted>Encryption Key (<DCMSecretEncrypted>
   - Controller Token (<ServerSecretEncrypted)
   - <StorageKeysEncrypted>
   - Encryption Key (<DCMSecretEncrypted>