---
id: e9ea10ee50e57e2a
title: How to - Validate SSL Cert can be Used for Gallery
status: intake
source:
  kind: confluence_page
  id: confluence-page:2201291939
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/2201291939
  summary: null
category: []
keywords: []
created_at: 2026-04-14T15:02:19Z
updated_at: 2026-04-14T15:02:19Z
confidence: null
cross_refs: []
content_hash: sha256:a06fd17cdc0a4223988bdcd38c526ad71d5237976a9694a8aa5c563e2bcb1c12
confluence_page_id: null
model_used: null
---

| Issue | Alteryx Service does not start with SSL enabled |
| --- | --- |
| Screenshot |  |
| Related Issues | Alteryx Service will start with SSL disabled, however when enabled, the service fails to start. |

# Troubleshooting

|  | Check | Steps |
| --- | --- | --- |
| 1 | Enhanced Key Usage | Open MMC, add the Certificated Snap In, go to Personal, locate the SSL cert. Double click the certificate, go to Details, look for “Enhanced Key Usage” field. Inspect the fieldsPrimarily used in Self-Signed Cert environments, for example PNC employs this procedure. When they ask to get a certificate created, the system asks for it’s purpose, which is then listed in the “Enhanced Key Usage” field. Below is an example of a certificate that is NOT certified to work with Alteryx Gallery. This is for Secure Email and Client Authentication (SAML).Another example that won't allow the service to start as it only allows Client Authentication (No Server Authentication)Click to see an example of a cert that is only allowed to use for SAML and Secure Email. |
| 2 |  |  |