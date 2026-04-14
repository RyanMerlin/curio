---
id: a9db67734d8ed998
title: How to monitor/ping Gallery to ensure it's running?
status: intake
source:
  kind: confluence_page
  id: confluence-page:2481291417
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/2481291417
  summary: null
category: []
keywords: []
created_at: 2026-04-14T15:18:28Z
updated_at: 2026-04-14T15:18:28Z
confidence: null
cross_refs: []
content_hash: sha256:5ca23de4e9e38b28e3648041a9c9bcaedb67e6c79cf54bd89ee75a7372c62f62
confluence_page_id: null
model_used: null
---

> **ℹ️ Info**
>
> Customer wants to monitor the status of an Alteryx Gallery and Service to check they are up so they can report if service is down / not responding.
> 
> This is often referred to as checking for a heartbeat.  <https://en.wikipedia.org/wiki/Heartbeat_(computing)>

| **Monitor Gallery** | /gallery/api/status/ping/ Returns **200 **when all is good.  From:  [Requirements for Configuring Alteryx Server with a Load Balancer (or Reverse Proxy/VIP)](https://knowledge.alteryx.com/index/s/article/Requirements-for-Configuring-Alteryx-Server-with-a-Load-Balancer-or-Reverse-Proxy-1628116360935) (KB) |
| --- | --- |
| **MonitorService** | /alteryxservice/status |