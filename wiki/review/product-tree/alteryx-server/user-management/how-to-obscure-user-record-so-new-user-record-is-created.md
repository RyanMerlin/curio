---
id: 7b6e31ca7aaffb14
title: How to Obscure User Record so New User Record is Created
status: review
source:
  kind: confluence_page
  id: confluence-page:3662053718
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/3662053718
  summary: null
category:
- product-tree
- alteryx-server
- user-management
keywords:
- users
- records
- obscure
- privacy
- account-management
created_at: 2026-04-14T15:18:28Z
updated_at: 2026-04-14T15:19:59Z
confidence: 0.85
cross_refs: []
content_hash: sha256:05df6fe926e554c55b07c5e1629c0c39cabcabcfa54866688d9b228db5e285c6
confluence_page_id: null
model_used: claude-sonnet-4-6
---

> **ℹ️ Info**
>
> You can obscure a user record so Server creates a new user record and studio upon next login.  While not common, there have been issues where a user’s record is not performing correctly – ex:  00788934, SQL DB, user could not create or edit Schedules.
> 
> The user will lose:
> 
> - Access to their workflows (you can move the new user to the old Studio to regain access)
> - Assets shared with them:Collection membershipOther users' DCMShared Gallery connections
>    - Collection membership
>    - Other users' DCM
>    - Shared Gallery connections
> 
> 
> DCM
> 
> - I’m not sure what happens with thier DCM.  They can sync up to Server from their new user record, but this will lead to DCM with the same IDs in the Server database for two user IDs. [EdP]

|  |  |
| --- | --- |
| **Change last name** | Add **_old** to the user’s last name. This ensures it’s clear that this is the “old” user record. |
| **Change email** | Add **_old** after their last name in their email. |
| **Change Studio name** | Add **_old **to the Studio name |
| **For AD Auth** | **MongoDB**     - Edit AlteryxGallery.users.WindowsIndentity.Sid to have all 9’s to ensure it doesn’t match any other user  **SQL DB**     - Edit alteyx_server.UserWindowsIdentiites.Sid to have all 9’s to ensure it doesn’t match any other user |
| **User opens new tab and login** | They will be prompted for their timezone, which indicates a new user record is being added. |