---
id: c7b3cd17c65b77ae
title: SPECIFIC SAML CONFIGURATIONS
status: intake
source:
  kind: confluence_page
  id: confluence-page:1665370204
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/1665370204
  summary: null
category: []
keywords: []
created_at: 2026-04-14T13:40:14Z
updated_at: 2026-04-14T13:40:14Z
confidence: null
cross_refs: []
content_hash: sha256:5bc27864a4082748b97d7b192e9cc9f7b65a44d75e9c6093402ac36d77a2fb2f
confluence_page_id: null
model_used: null
---

---

*[Organized section — child pages listed separately]*

---

> **ℹ️ Info**
>
> SAMLis one of the Authentication methods used by Server

| Key Articles | SCIM  <== auto-create Server Customer Groups to match SAML groups |
| --- | --- |

---

---

| Configuration | ADFSConfiguring SAML on Alteryx Server for ADFS (KB)JumpCloudConfigure SAML on Alteryx Server with JumpCloud  (KB)OktaConfiguring SAML on Alteryx Server for Okta (KB)SCIM Provisioning with OKTA Error authenticating Gateway Time-out (KB)OneLoginConfiguring SAML on Alteryx Server for OneLogin (KB)PingOneConfiguring SAML on Alteryx Server for PingOne (KB)PingFederatePingFederate (SAML) |
| --- | --- |
| APOD Setup | How to configure SAML on an APOD |
| What IDPs are compatible? | We should be able to connect if the IDP supports:SAML 2.0 protocolSHA2Sending the claims we require (firstName, lastName, email) |
| Using an IDP with multiple Servers | Most IDPs can only handle ONE Alteryx Server.  So a seperate IPD application needs to be created to support each Alteryx Server.  If setting up multiple Alteryx Servers with SAML, each will likely need to use a unique IDP application.  With so many different IDPs some may be able to handle multiple Servers.Example:  With a Prod and Dev env, they should have two separate IDP applications in most IDPs (federated can behave differently). |