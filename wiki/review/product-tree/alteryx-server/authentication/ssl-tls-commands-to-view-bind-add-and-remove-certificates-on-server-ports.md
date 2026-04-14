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
created_at: 2026-04-14T13:40:14Z
updated_at: 2026-04-14T13:50:56Z
confidence: 0.9
cross_refs: []
content_hash: sha256:bc4b14fd83f96580acd430a038b19f81639ffe931f743dfa6cb9da0897a1f6cd
confluence_page_id: null
model_used: claude-sonnet-4-6
---

> **ℹ️ Info**
>
> See also: Command line Testing Tools for Windows and Server

note Open command prompt **as Administrator**

Open command prompt **as Administrator**

| Task | Command |
| --- | --- |
| View bound certificates | #E3FCEFnetsh http show sslcert |
| Remove certificate from a port | #E3FCEFnetsh HTTP delete sslcert ipport=0.0.0.0:443How to Remove the SSL Certificate from Alteryx Server (KB) <== REMOVE cert |
| Bind certificate to a port(then restart Service) | #E3FCEFnetsh http add sslcert ipport=0.0.0.0:443 certhash=‎YOUR_CERT_HASH appid={eea9431a-a3d4-4c9b-9f9a-b83916c11c67}appid doesn’t change and is unique to Alteryx ServerConfiguring Alteryx Server for SSL: Obtaining and Installing Certificates (KB) <== ADD cert |