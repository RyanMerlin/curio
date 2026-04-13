---
id: 4f393b55d44f4f89
title: Server Upgrade Issues by Version - 22.3 and 22.1
status: published
source:
  kind: confluence_page
  id: confluence-page:2650999118
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/2650999118
  summary: null
category:
- product-tree
- alteryx-server
keywords:
- upgrade
- version
- 22.3
- 22.1
- controller-token
created_at: 2026-04-13T23:20:00Z
updated_at: 2026-04-13T23:20:00Z
confidence: 0.82
cross_refs:
- published/product-tree/alteryx-server/server-upgrade-issues-by-version.md
- published/product-tree/alteryx-server/host-recovery-encryption-key-transfer-process.md
- published/product-tree/alteryx-server/faq-help-cryptomigration.md
content_hash: sha256:d37b4759e674c29504334ab37b5eed445fecf7899e99f2ede3c65d9e69deb3b5
confluence_page_id: null
model_used: codex-curation
---

> **ℹ️ Info**
>
> Focused issue inventory for the 22.3 and 22.1 Server upgrade families.

# 22.3

| Item | Notes |
| --- | --- |
| CryptoMigration | Use the Prep Tool first; many failure modes originate here. |
| SAML Okta login leads to Please Sign In | Patch-level known issue with KB guidance available. |
| SAML URL case change | Lowercase ACS endpoint handling is a known upgrade trap. |

# 22.1

| Item | Notes |
| --- | --- |
| Encryption Key Transfer impacts host recovery | New process requirements can break DCM and shared connections if ignored. |
| Gallery Schema Migration 40.03 failures on `PasswordSecured` nulls | Pre-clean null values before upgrade or rollback recovery. |
| Controller Token length transition breaks host recovery | Validate and regenerate tokens when still at 40 characters. |
| Gallery migration failures on `CustomCss` and Schema 41 | Patch guidance and Jira references exist, but these are still common prep checks. |

## Related Pages

- [Server Upgrade Issues-by-Version](server-upgrade-issues-by-version.md)
- [Host Recovery Encryption Key Transfer Process](host-recovery-encryption-key-transfer-process.md)
- [FAQ / Help - CryptoMigration](faq-help-cryptomigration.md)
