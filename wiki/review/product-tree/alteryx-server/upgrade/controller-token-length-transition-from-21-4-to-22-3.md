---
id: d54a8c85f7706f28
title: Controller Token Length Transition from 21.4 to 22.3
status: review
source:
  kind: confluence_page
  id: confluence-page:1778614404
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/1778614404
  summary: null
category:
- product-tree
- alteryx-server
- upgrade
keywords:
- upgrade
- controller-token
- '21.4'
- '22.3'
- token-length
- migration
created_at: 2026-04-14T15:18:28Z
updated_at: 2026-04-14T15:19:30Z
confidence: 0.87
cross_refs: []
content_hash: sha256:3112299704543de941251db0e9b1f922ccc0b180b8aceedfbd1aad1f45b62156
confluence_page_id: null
model_used: claude-sonnet-4-6
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
> | **Ver** | **Upgrade** | **New install** | **Host Recovery requires a token of length** | **Notes** |
> | --- | --- | --- | --- | --- |
> |  |  |  |  |  |
> | **21.3** | 40 | 40 | 40 | All good! |
> |  |  |  |  |  |
> | **21.4** | **40** | 64 | 64 | Host recovery fails if a 40-char token is used |
> | **22.1** | **40** or 64 | 64 | 64 | Host recovery fails if a 40-char token is used |
> |  |  |  |  |  |
> | **22.3** | 64 | 64 | 64 | All good!  ---  In an upgrade that lengthens the token from 40 to 64, the Controller and Workers will lengthen in the same way and still connect after upgrade.  Admin should update their Runbook with the new token.  The upgrade will backup RuntimeSettings.xml before re-encrypting it.     - RuntimeSettings.22_2_legacy.xml – the original, pre-upgrade version    - RuntimeSettings.22_2_migration.xml – the re-encrypted version    - RuntimeSettings.xml – a copy of the re-encrypted untimeSettings.22_2_migration.xml  See [FAQ / Help - CryptoMigration](https://alteryx.atlassian.net/wiki/search?text=FAQ+/+Help+-+CryptoMigration) |
> |  |  |  |  |  |