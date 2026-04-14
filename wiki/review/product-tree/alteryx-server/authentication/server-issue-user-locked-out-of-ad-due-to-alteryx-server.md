---
id: 2860ea9e2d263572
title: Server Issue - User locked out of AD due to Alteryx Server
status: review
source:
  kind: confluence_page
  id: confluence-page:3406233691
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/3406233691
  summary: null
category:
- product-tree
- alteryx-server
- authentication
keywords:
- authentication
- active-directory
- locked-out
- windows-auth
- issue
created_at: 2026-04-14T15:18:28Z
updated_at: 2026-04-14T15:20:38Z
confidence: 0.88
cross_refs: []
content_hash: sha256:6f4943d3904ee3c0121840fd6fd46f75c421c6dd567ca04bf57777f79f5bff15
confluence_page_id: null
model_used: claude-sonnet-4-6
---

| **Issue** | > **📝 Note** > > User  locked out of AD for a brief period (~15min) due to Alteryx Server |
| --- | --- |
| **Screenshot** |  |

# Troubleshooting

|  | **Check** | **Steps** |
| --- | --- | --- |
| 1 | **Have IT determine the cause of the lock out**  Bad password | **Cause**     - Too many attempts with a bad password  **Resolution**     - Republish workflows that were published with user’s credentials    - Reschedule Schedules the user scheduled with their credentials    - Update DCM Credentials with new password and sync to Server    - Update the user’s profile default credentials. If they appear, remove or update them even if the user says they don’t use them. These DO get used if the user attempts to manually enter their credentials subsitituing the correct password the user types with the bad password stored in the User Profile Default Credential).    - Update Admin > Credentials if the user’s credential was added and shared with others.  Note: it would be better practiceto only share Service Accts with passwords that don’t expire.  **Workaround**     - IT can relax the threshold for bad auth attempts locking the user’s AD account.  **Cases**     - 00783839 |
| 2 | **Have IT determine the cause of the lock out**  Too many NTLM requests | **Cause**     - Server makes a lot of NTLM authentication requests as the user browses Server UI or runs workflows.  Some A/Vs view this as suspicious and temporarily lock the account.  **Resolution**     - IT needs to dial down the A/V |
| 3 | **Final things to try** | > **📝 Note** > > This is not a confirmed resolution since mutiple things were done at the same time, and none should have corrected the issue.  But customer was no longer locked out of AD when running workflows on Server after this.  **Possible Resolution from 00808117**     - A minimal workflow was tested and led to user AD lockout.  It’s unclear why the workflow was triggering AD checks to run a minimal workflow.    - Steps the customer says were followedCustomer removed and re-added Gellery Connection in DesignerCustomer re-synced DCM connections (should they sync if local Designer and Server were already synced?).Server was also likely rebooted       - Customer removed and re-added Gellery Connection in Designer       - Customer re-synced DCM connections (should they sync if local Designer and Server were already synced?).       - Server was also likely rebooted |