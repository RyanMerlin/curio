---
id: 7b4f3bb2fc3300e9
title: Issue - Service fails to start when ServerUI HTTPS/SSL is selected
status: intake
source:
  kind: confluence_page
  id: confluence-page:2997911697
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/2997911697
  summary: null
category: []
keywords: []
created_at: 2026-04-14T15:09:17Z
updated_at: 2026-04-14T15:09:17Z
confidence: null
cross_refs: []
content_hash: sha256:3630dff84a8bfac62be3ef1c6d69339c8ca84e2e6304edc182f9acf459f2d51e
confluence_page_id: null
model_used: null
---

|  |  |
| --- | --- |
|  |  |
|  |  |

# Troubleshooting

|  |  |  |
| --- | --- | --- |
|  |  |  |
|  |  |  |
|  |  |  |
|  |  |  |

[GCS Operations via Workflows: Card - access it on https://go.skype.com/cards.unsup...](https://teams.microsoft.com/l/message/19:05d469da10bd4f5a99baa1aa0baad90e@thread.skype/1743688069045?tenantId=522f39d9-303d-488f-9deb-a6d77f1eafd8&groupId=d7adbfc0-2b4e-487b-a707-42e478f217b6&parentMessageId=1743688069045&teamName=GRP_Customer%20Support%20-%20Skill%20Teams&channelName=Designer&createdTime=1743688069045)

posted in GRP_Customer Support - Skill Teams / Designer on Thursday, April 3, 2025 9:47 AM

For enabling SSL , only the SSL option to enable at Server UI > General should be enabled. I would skip the Global and Controller SSL options and leave those off

If the service isn't starting check the service logs. Are you seeing a bunch of 400 and/or 404 errors? If so then the cert is not configured properly. Check the certificate itself. Go into the MMC and double click the cert and check it's General tab. Does it have information like this?

If not then it's not trusted and therefore it won't work. If it does have information, I would check the certificate's SAN/DSN names or the certt itself and check to see if it matches the Gallery URL FQDN in the AYX System Settings. Then I would go into the command prompt and check to see that the cert is bound to port 443 on IP 0.0.0.0 by matching the thumbprint of the cert and what shows up in the command prompt.

If all of that is correct, then I would do an nslookup on the FQDN of that Gallery URL and see if the DNS record shows up correctly.