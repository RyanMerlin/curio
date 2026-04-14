---
id: c7b3cd17c65b77ae
title: SPECIFIC SAML CONFIGURATIONS
status: review
source:
  kind: confluence_page
  id: confluence-page:1665370204
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/1665370204
  summary: null
category:
- product-tree
- alteryx-server
- authentication
keywords:
- saml
- configuration
- idp
- sso
created_at: 2026-04-14T15:18:28Z
updated_at: 2026-04-14T15:20:45Z
confidence: 0.78
cross_refs: []
content_hash: sha256:c31616bd86425d2f6180c8c4bbc06e985a642fe7c955cdf72d1740622cb073b6
confluence_page_id: null
model_used: claude-sonnet-4-6
---

---

---

> **ℹ️ Info**
>
> SAMLis one of the Authentication methods used by Server

| **Key Articles** | [SCIM](https://alteryx.atlassian.net/wiki/search?text=SCIM)  <== auto-create Server Customer Groups to match SAML groups |
| --- | --- |

---

---

| #### Configuration | ##### ADFS     - Configuring SAML on Alteryx Server for ADFS (KB)  ---  ##### JumpCloud     - Configure SAML on Alteryx Server with JumpCloud  (KB)  ---  ##### Okta     - Configuring SAML on Alteryx Server for Okta (KB)     - SCIM Provisioning with OKTA Error authenticating Gateway Time-out (KB)  ---  ##### OneLogin     - Configuring SAML on Alteryx Server for OneLogin (KB)  ---  ##### PingOne     - Configuring SAML on Alteryx Server for PingOne (KB)  ---  ##### PingFederate     - PingFederate (SAML)  --- |
| --- | --- |
| **APOD Setup** | [How to configure SAML on an APOD](https://alteryx.atlassian.net/wiki/spaces/SupportCseBasics/pages?title=How+to+configure+SAML+on+an+APOD) |
| #### What IDPs are compatible? | We should be able to connect if the IDP supports:     - SAML 2.0 protocol    - SHA2    - Sending the claims we require (firstName, lastName, email) |
| #### Using an IDP with multiple Servers | > **📝 Note** > > Most IDPs can only handle ONE Alteryx Server.  So a seperate IPD application needs to be created to support each Alteryx Server.  If setting up multiple Alteryx Servers with SAML, each will likely need to use a unique IDP application.  With so many different IDPs some may be able to handle multiple Servers.  Example:  With a Prod and Dev env, they should have two separate IDP applications in most IDPs (federated can behave differently). |