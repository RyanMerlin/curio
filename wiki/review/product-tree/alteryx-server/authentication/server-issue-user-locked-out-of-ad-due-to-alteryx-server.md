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
created_at: 2026-04-14T13:40:14Z
updated_at: 2026-04-14T13:50:48Z
confidence: 0.88
cross_refs: []
content_hash: sha256:f2d6f7a0c76c279e1dac44a7f2a5e223cea973f9562b716a538d80992481ab4c
confluence_page_id: null
model_used: claude-sonnet-4-6
---

| Issue | User  locked out of AD for a brief period (~15min) due to Alteryx Server |
| --- | --- |
| Screenshot |  |

# Troubleshooting

|  | Check | Steps |
| --- | --- | --- |
| 1 | Have IT determine the cause of the lock outBad password | CauseToo many attempts with a bad passwordResolutionRepublish workflows that were published with user’s credentialsReschedule Schedules the user scheduled with their credentialsUpdate DCM Credentials with new password and sync to ServerUpdate the user’s profile default credentials. If they appear, remove or update them even if the user says they don’t use them. These DO get used if the user attempts to manually enter their credentials subsitituing the correct password the user types with the bad password stored in the User Profile Default Credential).Update Admin > Credentials if the user’s credential was added and shared with others.  Note: it would be better practiceto only share Service Accts with passwords that don’t expire.WorkaroundIT can relax the threshold for bad auth attempts locking the user’s AD account.Cases00783839 |
| 2 | Have IT determine the cause of the lock outToo many NTLM requests | CauseServer makes a lot of NTLM authentication requests as the user browses Server UI or runs workflows.  Some A/Vs view this as suspicious and temporarily lock the account.ResolutionIT needs to dial down the A/V |
| 3 | Final things to try | This is not a confirmed resolution since mutiple things were done at the same time, and none should have corrected the issue.  But customer was no longer locked out of AD when running workflows on Server after this.Possible Resolution from 00808117A minimal workflow was tested and led to user AD lockout.  It’s unclear why the workflow was triggering AD checks to run a minimal workflow.Steps the customer says were followedCustomer removed and re-added Gellery Connection in DesignerCustomer re-synced DCM connections (should they sync if local Designer and Server were already synced?).Server was also likely rebooted |