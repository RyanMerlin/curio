---
id: 530650b89d4905c6
title: Troubleshooting  (SAML)
status: intake
source:
  kind: confluence_page
  id: confluence-page:1671266865
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/1671266865
  summary: null
category: []
keywords: []
created_at: 2026-04-14T15:18:28Z
updated_at: 2026-04-14T15:18:28Z
confidence: null
cross_refs: []
content_hash: sha256:57dce9f5523f014fda696d0dc10ba443e41c55a8a06d361259bdfba467c66e5c
confluence_page_id: null
model_used: null
---

| #### Logs | [SAML SSO / AAS Logs](https://alteryx.atlassian.net/wiki/search?text=SAML+SSO+/+AAS+Logs) |
| --- | --- |
| #### SAML Tracer file during login process | <https://chromewebstore.google.com/detail/saml-tracer/mpdajninpobndbfcldcmbpnnbhibjmch?hl=en&pli=1> |
| #### Decode SAML tokens | <https://samltool.io/>                                                    <== **decode SAML messages** |
| #### .har file during login if SAML Tracer not available | [How to generate HAR file on the Gallery and Designer Cloud](https://knowledge.alteryx.com/index/s/article/How-to-generate-HAR-file-on-the-Gallery-and-Designer-Cloud) (KB)                                                   <== **How to generate a .har file ** |
| #### Troubleshooting | [Checklist-for-working-SAML-cases](https://alteryx.lightning.force.com/kA02R000000Q6PaSAK) (Internal KB)  <https://help.alteryx.com/current/server/configure-alteryx-server-authentication>                                                     <== **expand SAML section at end**  **Letter casing matters** when setting up SAML (vendor dependant) including the webapi address in the AYX System Settings. Specifically, if the Gallery Base/Server UI address has all caps, but the Web API URL has lowercase, a "401" error is received when signing in (we used Okta). Might have to do with verifying the HTTP requests match.  From: [Today I Learned](https://teams.microsoft.com/l/message/19:e60a6805a0e946259e9feb3525bd0234@thread.skype/1692204582408?tenantId=522f39d9-303d-488f-9deb-a6d77f1eafd8&groupId=688c106c-2fcb-4f03-b2b5-1f6b7e0b39ae&parentMessageId=1692204582408&teamName=GRP_Customer%20Support&channelName=Today%20I%20Learned&createdTime=1692204582408&allowXTenantAccess=false)  ---  **Troubleshooting Tools**     - Fiddler trace for Designer connection issues    - https://chrome.google.com/webstore/detail/saml-tracer/mpdajninpobndbfcldcmbpnnbhibjmch?hl=en plug-in for Chrome for Gallery connection issues    - Designer CEF Debugger doesn’t help |