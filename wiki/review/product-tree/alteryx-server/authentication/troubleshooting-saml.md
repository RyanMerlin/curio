---
id: 530650b89d4905c6
title: Troubleshooting  (SAML)
status: review
source:
  kind: confluence_page
  id: confluence-page:1671266865
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/1671266865
  summary: null
category:
- product-tree
- alteryx-server
- authentication
keywords:
- saml
- troubleshooting
- logs
- aas-logs
- sso
created_at: 2026-04-14T13:40:14Z
updated_at: 2026-04-14T13:51:01Z
confidence: 0.88
cross_refs: []
content_hash: sha256:bd9a696dfc9e10de2df9e8fdc650b9a79b9886136d7db13a5fac56fa3ba50022
confluence_page_id: null
model_used: claude-sonnet-4-6
---

| Logs | SAML SSO / AAS Logs |
| --- | --- |
| SAML Tracer file during login process | https://chromewebstore.google.com/detail/saml-tracer/mpdajninpobndbfcldcmbpnnbhibjmch?hl=en&pli=1 |
| Decode SAML tokens | https://samltool.io/                                                   <== decode SAML messages |
| .har file during login if SAML Tracer not available | How to generate HAR file on the Gallery and Designer Cloud (KB)                                                  <== How to generate a .har file |
| Troubleshooting | Checklist-for-working-SAML-cases (Internal KB)https://help.alteryx.com/current/server/configure-alteryx-server-authentication                                                    <== expand SAML section at endLetter casing matters when setting up SAML (vendor dependant) including the webapi address in the AYX System Settings. Specifically, if the Gallery Base/Server UI address has all caps, but the Web API URL has lowercase, a "401" error is received when signing in (we used Okta). Might have to do with verifying the HTTP requests match.  From: Today I LearnedTroubleshooting ToolsFiddler trace for Designer connection issueshttps://chrome.google.com/webstore/detail/saml-tracer/mpdajninpobndbfcldcmbpnnbhibjmch?hl=en plug-in for Chrome for Gallery connection issuesDesigner CEF Debugger doesn’t help |