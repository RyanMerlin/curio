---
id: 7d406c80ea5f7835
title: How to Change User's Domain
status: intake
source:
  kind: confluence_page
  id: confluence-page:2736455738
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/2736455738
  summary: null
category: []
keywords: []
created_at: 2026-04-14T15:10:53Z
updated_at: 2026-04-14T15:10:53Z
confidence: null
cross_refs: []
content_hash: sha256:f40b9f7f2c2994f9b61bcf43f9131105ebeefda1e207f74332132c2539b31368
confluence_page_id: null
model_used: null
---

> **ℹ️ Info**
>
> Individual users or entire groups may have their domain changed.  This page describes how the database can be updated to use the new domain.

> **ℹ️ Info**
>
> **What happens when an existing user logs in with a new domain?**
> 
> The user’s email address is used during login to find their **AlteryxGallery.users** collection record.  If a user logs in with an email address in a new domain, Server will consider them a NEW user and create NEW **users** and **subscriptions** collection records.  The user will no longer see the workflows they previously published using their original address since they are now a NEW account.
> 
> To see their previous workflows/schedules/collections etc, the database needs to be updated so their original **users** collection record has their new email (and for AD, possibly a new **AD SID**).

> **📝 Note**
>
> **For AD Authentication**
> 
> - If the user’s new domain email address has a new AD SID, additional effort is needed to update the AD SID in the original user record, see:Updating a User's AD SID when it has changed
>    - Updating a User's AD SID when it has changed
> 
> - The Server connects to a single AD Domain Controller and will need to be able to validate the new domain.  If the Server can only validate the ol;d domain, their IT needs to become involved to determine the AD DC the Server is connecting to and why that AD DC can’t validate email addresses using the new domain (this is NOT something we can fix with an Alteryx Server setting).
> - See Multiple domain environments requirements for Server:https://help.alteryx.com/current/en/server/configure/configure-alteryx-server-authentication.html#idp412701_body > Set Up Integrated Windows Authentication
>    - https://help.alteryx.com/current/en/server/configure/configure-alteryx-server-authentication.html#idp412701_body > Set Up Integrated Windows Authentication

|  |  |
| --- | --- |

---

---

# Update Email Domain ONLY – any Auth type

> **ℹ️ Info**
>
> This section is applicable for
> 
> - SAML / Built-in
> - AD / AD+Kerberos when the user’s AD SID does NOT change

> **📝 Note**
>
> **Default Admin is lost**
> 
> Ensure you update the **Alteryx System Settings **default Admin with the new domain, otherwise, the Admin may lose their curator role (if it wasn’t explicitly set on their **users **record).

> **⚠️ Warning**
>
> **AD Auth**
> 
> Do not use this section for **AD Auth** if the user’s **AD SID **changes as it will break the database as the **AD SID** will be old/wrong in the AlteryxGallery collections:  **users **and** collections. ** Thiis will prevent the user from being able to log in.

## Option 1 - Mongo UpdateMany() Query

> **📝 Note**
>
> If users have already logged into the Server with the new domain then they have TWO **users **collection records for the same base name:
> 
> - ed.phelps@newDomain.com  <== NEW record (we don’t want this)
> - ed.phelps@origDomain.com  <== ORIGINAL record (we want to update this to the new domain since it has access to workflows the user uploaded and the Collections they belong to)
> 
> **That’s a problem!**  If we simply update their **ORIGINAL **address with the new domain we’ll have two records with the same email address, which will confuse Server.  We need to obscure the **NEW **records (by adding “_old” to the end), then update the **ORIGINAL **emails with the new domain.

|  |  |
| --- | --- |
|  |  |
|  |  |
|  |  |

## Option 2 - Analytic App

App developed by Tim R.  Currently this isn’t doing more than the update queries above.  But it will eventually help update AD SID for AD Authentication Servers where the AD SID for the user is also changed.

- <== locked version is ok to send to customers              <== don’t send to customer
- Older versions

---

# SAML / Built-in Authentication

> **ℹ️ Info**
>
> This is the easiest since the **users.email** is the only element that needs to be updated

See **Update Email Domain ONLY**above

---

# AD / AD+Kerberos Authentication

## Update Email Domain ONLY

See **Update Email Domain ONLY**above

## Update Email Domain AND AD SID

> **ℹ️ Info**
>
> If both the email domain and the **AD SID **are changing for a user then updates must be made in:
> 
> - users.WindowsIdentity
> - collections.Users.ActiveDirectoryObject

**NOTE TO CSE**:

- Let's create an automated process for the next customer who undergoes a Domain change that also changes AD SIDs. We’ll need a lookup table of the old SID and the new information to update the highlighted sections of the two records above. Reach out to Ed Phelps, and we’ll put this together with a workflow to turn the lookup table into the necessary Mongo queries. It's not too hard, and then we’ll have a tool for future users.

The following KB talks through manually updating the AD SID information highlighted below.

- How To Manually Change A Domain User To New Domain (KB)

**users record**

The WindowsIdenity must be updated with new AD SID information

> **📝 Note**
>
> **collections record**
> 
> Collection membership is based on **AD SID **rather than **users collection IDs (**as are used everywhere else in the database).
> 
> - Why make this terrible choice?  To help Admins “pre-load” Collections with new AD users who would then get email that they were added to the Collection with a link.