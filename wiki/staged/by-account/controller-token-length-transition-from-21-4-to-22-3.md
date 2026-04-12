---
id: d54a8c85f7706f28
title: Controller Token Length Transition from 21.4 to 22.3
status: staged
source:
  kind: confluence_page
  id: confluence-page:1778614404
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/1778614404
  summary: null
category:
- by-account
keywords:
- token
- controller
- length
- upgrade
- recovery
created_at: 2026-04-12T20:59:51Z
updated_at: 2026-04-12T20:59:57Z
confidence: 0.55
cross_refs: []
content_hash: sha256:c4ab83977e12084996e671b27dae7a8dd4ae717650b64dcd3f04997b4f042e58
confluence_page_id: null
model_used: heuristic
---

> **ℹ️ Info**
>
> The Controller Token length length changes in different versions

> **⚠️ Warning**
>
> **Note**
> 
> If regenerating the Controller Tolen, save a copy of the
> 
> - Controller Token
> - C:\ProgramData\Alteryx\RuntimeSettings.xml
> 
> These are needed if the customer needs to restore a previous Mongo backup.  Regenerating the Controller Token will also change RuntimeSettings.xml <**StorageKeysEncrypted**>
> 
> # Controller Token Length and Impact on Host Recovery
> 
> | Ver | Upgrade | New install | Host Recovery requires a token of length | Notes |
> | --- | --- | --- | --- | --- |
> |  |  |  |  |  |
> | 21.3 | 40 | 40 | 40 | All good! |
> |  |  |  |  |  |
> | 21.4 | 40 | 64 | 64 | Host recovery fails if a 40-char token is used |
> | 22.1 | 40 or 64 | 64 | 64 | Host recovery fails if a 40-char token is used |
> |  |  |  |  |  |
> | 22.3 | 64 | 64 | 64 | All good!In an upgrade that lengthens the token from 40 to 64, the Controller and Workers will lengthen in the same way and still connect after upgrade.  Admin should update their Runbook with the new token.The upgrade will backup RuntimeSettings.xml before re-encrypting it.RuntimeSettings.22_2_legacy.xml – the original, pre-upgrade versionRuntimeSettings.22_2_migration.xml – the re-encrypted versionRuntimeSettings.xml – a copy of the re-encrypted untimeSettings.22_2_migration.xmlSee FAQ / Help - CryptoMigration |
> |  |  |  |  |  |