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
created_at: 2026-04-14T15:16:58Z
updated_at: 2026-04-14T15:16:58Z
confidence: null
cross_refs: []
content_hash: sha256:2de87f6a46a3fff42ccc94d19ced78571a0d372b090bff43d36e3cb1bf83ea82
confluence_page_id: null
model_used: null
---

---

---

> **ℹ️ Info**
>
> SAMLis one of the Authentication methods used by Server

| **Key Articles** | SCIM  <== auto-create Server Customer Groups to match SAML groups |
| --- | --- |

---

---

| #### Configuration | ##### ADFS     - Configuring SAML on Alteryx Server for ADFS (KB)  ---  ##### JumpCloud     - Configure SAML on Alteryx Server with JumpCloud  (KB)  ---  ##### Okta     - Configuring SAML on Alteryx Server for Okta (KB)     - SCIM Provisioning with OKTA Error authenticating Gateway Time-out (KB)  ---  ##### OneLogin     - Configuring SAML on Alteryx Server for OneLogin (KB)  ---  ##### PingOne     - Configuring SAML on Alteryx Server for PingOne (KB)  ---  ##### PingFederate     - PingFederate (SAML)  --- |
| --- | --- |
| **APOD Setup** | How to configure SAML on an APOD |
| #### What IDPs are compatible? | We should be able to connect if the IDP supports:     - SAML 2.0 protocol    - SHA2    - Sending the claims we require (firstName, lastName, email) |
| #### Using an IDP with multiple Servers | > **📝 Note** > > Most IDPs can only handle ONE Alteryx Server.  So a seperate IPD application needs to be created to support each Alteryx Server.  If setting up multiple Alteryx Servers with SAML, each will likely need to use a unique IDP application.  With so many different IDPs some may be able to handle multiple Servers.  Example:  With a Prod and Dev env, they should have two separate IDP applications in most IDPs (federated can behave differently). |