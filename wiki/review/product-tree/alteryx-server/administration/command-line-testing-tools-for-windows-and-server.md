---
id: 311d01f429737a79
title: Command line Testing Tools for Windows and Server
status: review
source:
  kind: confluence_page
  id: confluence-page:1720391652
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/1720391652
  summary: null
category:
- product-tree
- alteryx-server
- administration
keywords:
- command-line
- testing
- diagnostic
- windows
- tools
created_at: 2026-04-14T15:18:28Z
updated_at: 2026-04-14T15:19:28Z
confidence: 0.88
cross_refs: []
content_hash: sha256:cdc3c35343e90fffe2c41178a82c7c2d777588b4ca6419b0e04afcd73e4038f3
confluence_page_id: null
model_used: claude-sonnet-4-6
---

> **ℹ️ Info**
>
> Common testing tools in a Server environment

| **Key Articles** | [Diagnosing Network Connection Issues](https://knowledge.alteryx.com/index/s/article/Diagnosing-Network-Connection-Issues) (KB) |
| --- | --- |

---

AlteryxService test

Get-ADPrincipalgroupmembership -Identity ‘FIRST.LAST ’

hostname

ipconfig

netsh http show sslcert

netsh winhttp show proxy

netsh winhttp dump > proxy.txt

netsh http show urlacl

netstat -an | find ":**80** " | find "LISTENING"

netstat -aon

Faled to register Service URL (5).

Please contact your Systems Administrator to ensure that port 443 is unused and open for inbound connections, and verify that the Alteryx Service is not running on this system

ipconfig /flushdns

nltest /dsgetdc:DOMAIN_NAME

nltest /dclist:DOMAIN_NAME

nltest /Server:CLIENT_COMPUTER_NAME /SC_RESET:DOMAIN_NAME \DOMAIN_CONTROLLER_NAME

nslookup alteryx.com

ping **GALLERY.HOST.COM**

Test-NetConnection -ComputerName "CONTROLLER_NAME " -Port 80

C:\temp\latencyOutpu2.txt ]]> whoami /user

wmic memorychip get capacity && wmic cpu get name,numberofcores,numberoflogicalprocessors

wmic computersystem get domain

wmic useraccount where name="**FIRST.LAST**" get sid

wmic useraccount where sid='**S-1-5-21-1777081478-1322062499-644039835-1808318**'