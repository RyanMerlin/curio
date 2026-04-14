---
id: e9ea10ee50e57e2a
title: How to - Validate SSL Cert can be Used for Gallery
status: review
source:
  kind: confluence_page
  id: confluence-page:2201291939
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/2201291939
  summary: null
category:
- product-tree
- alteryx-server
- authentication
keywords:
- ssl
- certificate
- gallery
- validation
- service-startup
created_at: 2026-04-14T15:18:28Z
updated_at: 2026-04-14T15:20:08Z
confidence: 0.87
cross_refs: []
content_hash: sha256:e5e5d5c1933e59139e2d912c428333cd195704ddde397d7908b1851d267f7cf6
confluence_page_id: null
model_used: claude-sonnet-4-6
---

| **Issue** | > **📝 Note** > > Alteryx Service does not start with SSL enabled |
| --- | --- |
| **Screenshot** |  |
| **Related Issues** | Alteryx Service will start with SSL disabled, however when enabled, the service fails to start. |

# Troubleshooting

|  | **Check** | **Steps** |
| --- | --- | --- |
| 1 | **Enhanced Key Usage** | 1. Open MMC, add the Certificated Snap In, go to Personal, locate the SSL cert.    2. Double click the certificate, go to Details, look for “Enhanced Key Usage” field.    3. Inspect the fields  Primarily used in Self-Signed Cert environments, for example PNC employs this procedure. When they ask to get a certificate created, the system asks for it’s purpose, which is then listed in the “Enhanced Key Usage” field. Below is an example of a certificate that is NOT certified to work with Alteryx Gallery. This is for Secure Email and Client Authentication (SAML). |
| 2 |  |  |