---
id: 2dafa898f910632a
title: 'Upgrade Error - Failed to open XML file C:\ProgramData\Alteryx\Engine\202#.#\InstallInfo.xml, system error: -2147024786'
status: intake
source:
  kind: confluence_page
  id: confluence-page:2386460678
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/2386460678
  summary: null
category: []
keywords: []
created_at: 2026-04-12T20:59:51Z
updated_at: 2026-04-12T20:59:51Z
confidence: null
cross_refs: []
content_hash: sha256:0a8190253289db6c3a9c25a07a218aac7031883db5d0d30d35559bdab5a18a1e
confluence_page_id: null
model_used: null
---

| Context |  |
| --- | --- |
| Error | Failed to open XML fileC:\ProgramData\Alteryx\Engine\202#.#\InstallInfo.xml,system error: -2147024786 |
| Screenshot |  |
| Related Errors |  |
| Versions |  |

# Troubleshooting

|  | Check | Steps |
| --- | --- | --- |
| 1 | Is the installer trying to upgrade Server despite Server not being installed? | Case 00715588 – Customer have installed / uninstalled / upgraded a couple of versions and gotten the machine off track such that the installer thought it was upgrading (despite Server not being installed) and therefore expected this file to exist.The screenshot of the error shows that it thinks it’s “Updating” the Server as well as language in the dialogs below.The Windows Add/Remove Programs did not list Alteryx Server, so we couldn’t use that to uninstall.We followed procedures for a Full Uninstall and issue persisted..ResolutionPlaced the file InstallInfo.xml provided from one of our test Servers in the folder to address the errorRestarted the 23.1 installer and installation completed, however many files were missing and neither Designer nor Alteryx System Settings would start or even present an error.  Since the system thought it was upgrading 23.1 it cut some corners and din’t install all files.Downloaded 23.2 and upgraded hoping it would replace everything with a working set of Server files (which it did)Confirmed 23.2 was workingUsed the Windows Uninstaller to uninstall ServerWe renamed C:\ProgramData\Alteryx and customer's E: drive Persistence folder so these would start freshWe installed 23.1 and confirmed it was working |