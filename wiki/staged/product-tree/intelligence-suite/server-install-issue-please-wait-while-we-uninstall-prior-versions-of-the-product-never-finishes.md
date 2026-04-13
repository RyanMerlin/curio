---
id: 81ca3ed36782940d
title: Server Install Issue - "Please wait while we uninstall prior versions of the product" never finishes
status: staged
source:
  kind: confluence_page
  id: confluence-page:1679491882
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/1679491882
  summary: null
category:
- product-tree
- intelligence-suite
keywords:
- uninstall
- server
- prior
- versions
- issue
created_at: 2026-04-13T01:55:58Z
updated_at: 2026-04-13T01:56:17Z
confidence: 0.55
cross_refs: []
content_hash: sha256:6328abc5f435e9f4605240461227bfa187babf37ac12e711ee43311839e3d817
confluence_page_id: null
model_used: heuristic
---

| Issue | "Please wait while we uninstall prior versions of the product" never finishes |
| --- | --- |
| Screenshot |  |
| Versions | 2022.3 |

# Troubleshooting

|  | Check | Steps |
| --- | --- | --- |
| 1 | Reboot and try again |  |
| 2 | Try uninstalling from the Add or remove programs App | Uninstall other Alteryx products first, like Predictive Tools or Intelligence Suite |
| 3 | Try command-line Uninstall for 2022.2 and prior | 2022.2 and prior included a REMOVE command-line optionAdjust based on the install exe you have (this must be the original installer, not a patch installer) |
| 4 | Ensure user has Administratice rights on the machine | Confirm with IT that the account being used to login to the Server machine has administrator rights on the Server. |
| 5 | Try a reduced set of steps from the Complete Uninstall process | WARNING - do NOT delete Mongo or RuntimeSettings.xml or probably anything in ProgramData, adjust the process below and ensure you have a backup of Mongo, RuntimeSettings.xml, and Controller Token first.Removing the Registry entries may be the step that solves the problem.https://community.alteryx.com/t5/Alteryx-Server-Knowledge-Base/Alteryx-Server-Silent-Uninstallation/ta-p/1028776 (1028776) <<== this mostly seems like the Complete Uninstall process with the command-line REMOVE=TRUE step above rather than uninstalling from the Add / Remove Applications app.https://community.alteryx.com/t5/Alteryx-Designer-Knowledge-Base/Complete-Uninstall-of-Alteryx-Designer/ta-p/402897 (402897) |

# Resolution

|  | Cause | Resolution |
| --- | --- | --- |
| 1 |  |  |

# Workarounds

|  | Workaround | Steps |
| --- | --- | --- |
| 1 |  |  |

# Research

| 1 | 00603102 - Ed P - in progress |
| --- | --- |
| 2 | 00544448 - Issue was with Designer and customer was presented the article to fully uninstall designer.  This should NOT be done for Server as it would delete the Mongo database and RuntimeSettings.xml |
| 3 | 00598430 - Darine - Designer that was still failing after Full Uninstall.  Asked him how he solved on a call. |