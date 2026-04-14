---
id: a20e5abee1a3d4ec
title: How to reset a user's Built-in Auth password if the Server can't send email
status: intake
source:
  kind: confluence_page
  id: confluence-page:2310013006
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/2310013006
  summary: null
category: []
keywords: []
created_at: 2026-04-14T13:40:14Z
updated_at: 2026-04-14T13:40:14Z
confidence: null
cross_refs: []
content_hash: sha256:5bc9a4a4f173fb3eef451a63c83ae0b3253b89d05a8467ceb2d5eccbfff228b9
confluence_page_id: null
model_used: null
---

> **ℹ️ Info**
>
> **Built-in Auth** requires that Alteryx System Settings is configured to send emails to change a user’s password if they’ve forgotten it.
> 
> Because the settings application doesn’t use DCM, it can’t connect to modern email systems using more advanced email authentication (OAuth2 / Azure AD).
> 
> The page below explains how to copy the password from a newly created user to a user who forgot their password.

---

---

| Add a NEW user | Login to the Server as a new user with a new password |
| --- | --- |
| Install Studio3T Free | https://studio3t.com/download-studio3t-free/ |
| Configure Studio 3T | You’ll configure using the Non-Admin Monog password found in Alteryx System Settings > Persistence > PasswordIn Studio 3T:localhost above works when on the Controller using Embedded Mongo, for other situations: Example Mongo Connection Strings |
| Find the NEW user | Run the query:#E3FCEFdb.getCollection("users").find({"Email":"DummyUser@myCompany.com"})Right-click > Document View |
| Copy the NEW Password fields |  |
| Find the EXISTING user who needs to update their password updated | Perform the same search as for the NEW user, but with the EXISTING user’s email address.Right-click > Document > Edit |
| Paste the NEW Password | Replace the SecurityInfo section of the EXISTING user with the SecurityInfo copied from the NEW user.Then click Validate and Update |
| Have EXISITING user reattempt login with new password | The password should be updated immediately, not need to restart the Service. |