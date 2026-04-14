---
id: 361c9b5d918bff3b
title: SSL/TLS Commands to View, Bind/Add, and Remove Certificates on Server Ports
status: review
source:
  kind: confluence_page
  id: confluence-page:1744667238
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/1744667238
  summary: null
category:
- product-tree
- alteryx-server
- authentication
keywords:
- ssl
- tls
- certificates
- commands
- netsh
- ports
created_at: 2026-04-14T15:18:28Z
updated_at: 2026-04-14T15:20:47Z
confidence: 0.9
cross_refs: []
content_hash: sha256:690cf0ca8e294b7b9ef4e94a7692b9efb624bbb1fccec2ad35925da4fb47e232
confluence_page_id: null
model_used: claude-sonnet-4-6
---

> **ℹ️ Info**
>
> See also: [Command line Testing Tools for Windows and Server](https://alteryx.atlassian.net/wiki/search?text=Command+line+Testing+Tools+for+Windows+and+Server)

Open command prompt **as Administrator**

| **Task** | **Command** |
| --- | --- |
| **View bound certificates** | netsh http show sslcert |
| **Remove certificate from a port** | netsh HTTP delete sslcert ipport=0.0.0.0:**443** [How to Remove the SSL Certificate from Alteryx Server](https://knowledge.alteryx.com/index/s/article/Removing-SSL-Certificate-from-Alteryx-Server) (KB) <== **REMOVE cert** |
| **Bind certificate to a port** (then restart Service) | netsh http add sslcert ipport=0.0.0.0:**443 **certhash=‎**YOUR_CERT_HASH** appid={eea9431a-a3d4-4c9b-9f9a-b83916c11c67} **appid** doesn’t change and is unique to Alteryx Server  [Configuring Alteryx Server for SSL: Obtaining and Installing Certificates](https://knowledge.alteryx.com/index/s/article/Configuring-Alteryx-Server-for-SSL-Obtaining-and-Installing-Certificates-1583459841225) (KB) <== **ADD cert** |