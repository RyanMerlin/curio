---
id: 08096e21c47ad4e8
title: How to Set the Web API URL to a Unique Port (8443)
status: intake
source:
  kind: confluence_page
  id: confluence-page:1766330084
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/1766330084
  summary: null
category: []
keywords: []
created_at: 2026-04-14T15:16:58Z
updated_at: 2026-04-14T15:16:58Z
confidence: null
cross_refs: []
content_hash: sha256:df43e6bf249e62523ea93faeaa798e93a0548fb154e070f79d76cd421a623ca6
confluence_page_id: null
model_used: null
---

> **ℹ️ Info**
>
> In some installations, the Web API URL must be set to a unique port.

> **📝 Note**
>
> If the **Base Address does NOT end with /gallery **(or another word), the **Web API** must be set to a unique port.
> 
> [Michael Spoula] To remove /gallery Server needs to be running on TLS (SSL) or you have to change the serviceport manually.  However, this does not work with SAML (no workaround to my knowledge) and requires WebAPI be set to another port as described in this article.

# Steps

Choose an available port.  In the example below we assume SSL is enabled and (arbitrarily) choose 8443.

| 1 | Set the WebAPI URL to use port 8443 (this is an arbitrary port number)  <https://my.domain.com>:**8443**/webapi/ |
| --- | --- |
| 2 | If using SSL, the certificate will need to be bound to port 8443  View the current certificate  netsh http show sslcert Bind the certificate to port 8443  netsh http add sslcert ipport=0.0.0.0:**8443 **certhash=‎**YOUR_CERT_HASH** appid={eea9431a-a3d4-4c9b-9f9a-b83916c11c67} |
| 3 | Restart the Service |

[How to Run Alteryx Server on a port other than 80](https://knowledge.alteryx.com/index/s/article/Running-Alteryx-Server-on-a-port-other-than-80-1583460188680)  <== Old KB with partial explanation