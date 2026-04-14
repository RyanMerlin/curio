---
id: c9b979627008a0fd
title: Logging (SCIM)
status: review
source:
  kind: confluence_page
  id: confluence-page:2675835371
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/2675835371
  summary: null
category:
- product-tree
- alteryx-server
- authentication
keywords:
- scim
- logging
- provisioning
- diagnostic
created_at: 2026-04-14T13:40:14Z
updated_at: 2026-04-14T13:50:35Z
confidence: 0.85
cross_refs: []
content_hash: sha256:683c61d56a5b15b0291c12ad5615d245544a4367478491e2996d7cf877900738
confluence_page_id: null
model_used: claude-sonnet-4-6
---

---

*[Organized section — child pages listed separately]*

---

| Access | Provisioning Agent logsIt’s on the customer to find and review theseGallery logsSearch for scim\ |
| --- | --- |
| Logs | There are two “logs”:Provisioning Agent Log – In general, customers will need to review the EntraID Provision Application SCIM logs on their own since the Provisioning app is out of scope of support as it’s not our product.  However, we can track errors seen and how they are reolved under this page.Errors (SCIM Provisioning Agent log) Gallery log – To allow SCIM to create, add, and update users and groups, several “SCIM” API endpoints were added to Server and the calls are logged in Gallery logs (like other API endpoints)Gallery Log Messages |
|  |  |