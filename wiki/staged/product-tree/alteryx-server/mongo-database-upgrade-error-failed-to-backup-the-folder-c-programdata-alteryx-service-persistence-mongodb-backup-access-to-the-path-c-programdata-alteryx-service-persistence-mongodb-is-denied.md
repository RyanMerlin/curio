---
id: 91337d4e259e2352
title: Mongo Database Upgrade Error - Failed to backup the folder:C:\ProgramData\Alteryx\Service\Persistence\MongoDB_Backup. Access to the path 'C:\ProgramData\Alteryx\Service\Persistence\MongoDB' is denied
status: staged
source:
  kind: confluence_page
  id: confluence-page:2660237315
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/2660237315
  summary: null
category:
- product-tree
- alteryx-server
keywords:
- antivirus
- upgrade
- mongo
- error
- folder
created_at: 2026-04-13T01:55:58Z
updated_at: 2026-04-13T01:56:15Z
confidence: 0.55
cross_refs: []
content_hash: sha256:b2fd50975c9aea35f629df23a35c274dfeece5b5f767b15902a74c46fbaef074
confluence_page_id: null
model_used: heuristic
---

| Context | MongoDB Migration when performing an upgrade |
| --- | --- |
| Error | Failed to backup the folder:C:\ProgramData\Alteryx\Service\Persistence\MongoDB_Backup. Access to the path 'C:\ProgramData\Alteryx\Service\Persistence\MongoDB' is denied |
| Screenshot |  |
| Related Errors |  |
| Versions |  |

# Troubleshooting

|  | Check | Steps |
| --- | --- | --- |
| 1 | Antivirus or Permissions | This error occurs when there is permission issues to the Mongo Folder or if the Folder is locked during the process due to Antivirus interfernceAs per the procmon logs, we can see the Macafee Antirus is inspecting the Alteryxc folders, which locking the files.ResolutionCheck if the Mongo Folder has enough permission for the Service account or the account looged inDisable the Antivirus temporarily. |

# Research

| 00737322in progressRed Customer disabled the Macafee Antivirus temporarily and the upgrade completed without any issues |
| --- |