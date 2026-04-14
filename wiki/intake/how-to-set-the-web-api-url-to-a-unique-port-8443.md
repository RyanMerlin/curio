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
created_at: 2026-04-14T15:09:17Z
updated_at: 2026-04-14T15:09:17Z
confidence: null
cross_refs: []
content_hash: sha256:d88cc584d3d80b10fa46d9f4030b57c556860428e27f5e9d318e3febebfae09f
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

|  |  |
| --- | --- |
|  |  |
|  |  |

[How to Run Alteryx Server on a port other than 80](https://knowledge.alteryx.com/index/s/article/Running-Alteryx-Server-on-a-port-other-than-80-1583460188680)  <== Old KB with partial explanation