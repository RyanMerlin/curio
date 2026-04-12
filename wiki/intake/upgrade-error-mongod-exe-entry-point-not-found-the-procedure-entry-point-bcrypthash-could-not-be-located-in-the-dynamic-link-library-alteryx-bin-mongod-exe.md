---
id: 591975c0cc4d7d3f
title: Upgrade Error - mongod.exe - Entry Point Not Found | The procedure entry point BCryptHash could not be located in the dynamic link library \Alteryx\bin\mongod.exe
status: intake
source:
  kind: confluence_page
  id: confluence-page:2160460995
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/2160460995
  summary: null
category: []
keywords: []
created_at: 2026-04-12T20:59:51Z
updated_at: 2026-04-12T20:59:51Z
confidence: null
cross_refs: []
content_hash: sha256:2308103a9c48f943861db0b4ffaeea0ce1f2a72aac2e3706db37db3a171ba9e8
confluence_page_id: null
model_used: null
---

| Context | Upgrading Server |
| --- | --- |
| Error | mongod.exe - Entry Point Not FoundThe procedure entry point BCryptHash could not be located in the dynamic link library \Alteryx\bin\mongod.exe |
| Screenshot |  |
| Related Errors |  |
| Versions |  |

# Troubleshooting

|  | Check | Steps |
| --- | --- | --- |
| 1 | Server Version | This error occurs when the OS is too old for Mongo.  Supporting references:https://learn.microsoft.com/en-us/windows/win32/api/bcrypt/nf-bcrypt-bcrypthash  <== Server 2016 min requirementshttps://stackoverflow.com/questions/67860512/the-procedure-entry-point-bcrypthash-could-not-be-located-in-the-dynamic-link-li <== the error on this page is due to using an old OSResolutionUpgrade the Server OS to a version supported by the Server you’re upgrading to.Upgrade failure, error: mongod.exe - Entry Point Not Found \| The procedure entry point BCryptHash could not be located in the dynamic link library \Alteryx\bin\mongod.exe (KB)https://help.alteryx.com/release-notes/en/release-notes/server-release-notes/server-2023-2-release-notes.html##:~:text=%C2%A0help%20page.-,Windows%20Server%202012%20End%20of%20Support,-As%20of%20Server |

# Research

| 00694335 in progressRed Customer is on Win Server 2012 and getting error when upgrading to 23.2.  This is the same as Karen’s customer in XXXX.  Karen’s theory is that this error is due to being on unsupported Win Server version. |
| --- |