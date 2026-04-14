---
id: 40b5ef0ea02c2a56
title: Errors (SAML)
status: intake
source:
  kind: confluence_page
  id: confluence-page:1855128540
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/1855128540
  summary: null
category: []
keywords: []
created_at: 2026-04-14T15:02:18Z
updated_at: 2026-04-14T15:02:18Z
confidence: null
cross_refs: []
content_hash: sha256:a85f7dfb5e1ca48d5a8c2d562262c4cfa2ca2743890a8628d971ac20e2162d6f
confluence_page_id: null
model_used: null
---

# SAML Errors

note For aas log errors, see SAML SSO / AAS Logs

For aas log errors, see [SAML SSO / AAS Logs](/wiki/spaces/SupportServer/pages/1656685042/SAML+SSO+AAS+Logs)

---

- Exception thrown when asserting SAML response from IDP (KB)
- TCPE-59777dcf2c9-72f3-3ff6-8e99-fe88e9f473f1System Jira [22.3.1_Patch6, 23.1.1_Patch4, 23.2-LTS]

- Hard 404 when clicking the Sign In button
- IDP may not have had enough time to start and needs the workaround to give the authentication service more time to start
- TCPE-103677dcf2c9-72f3-3ff6-8e99-fe88e9f473f1System Jira <== request to make default timeout longer, add your case!
- Alteryx Server: Service will not start due to the AlteryxAuthorizationService taking a long time to come up (KB) <== See Solution
- https://alteryx.atlassian.net/wiki/spaces/SupportCseBasics/pages/1999244069/CSU+Tech+Talk#Trend-%E2%80%93-23.2-upgrade-failing-due-to-Mongo-6.0-upgrade-failing-silently

- PingId Access Denied after enabling SAML with x509 (KB)

- SAML Error - TCPE-82077dcf2c9-72f3-3ff6-8e99-fe88e9f473f1System Jira