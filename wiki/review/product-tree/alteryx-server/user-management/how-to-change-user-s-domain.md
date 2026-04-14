---
id: 7d406c80ea5f7835
title: How to Change User's Domain
status: review
source:
  kind: confluence_page
  id: confluence-page:2736455738
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/2736455738
  summary: null
category:
- product-tree
- alteryx-server
- user-management
keywords:
- users
- domain
- active-directory
- migration
- how-to
created_at: 2026-04-14T15:18:28Z
updated_at: 2026-04-14T15:19:54Z
confidence: 0.87
cross_refs: []
content_hash: sha256:b1267e6a722b9b5339b0e4daafe8d551bf249bd9422c31299d9d392d6c6dc6eb
confluence_page_id: null
model_used: claude-sonnet-4-6
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

| **Key Articles** | [How To Manually Change A Domain User To New Domain](https://knowledge.alteryx.com/index/s/article/How-To-Manually-Change-A-Domain-User-To-New-Domain) (KB) |
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

| #### Find users with multiple  domains | > **📝 Note** > > These users must have their **NEW **domain records obscured before their **ORIGINAL **domain records can be updated with the NEW domain  These are **users **records for users who have signed in with BOTH their **ORIGINAL **and **NEW **domains.  The records with the **NEW **domain should be obscured.  Example results of query showing two users who have multiple users collection records with the EXISTING and NEW domains.  db.users.aggregate([   {     $project: {       sameNameWithMultipleDomains: { $toLower: { $arrayElemAt: [{ $split: ["$Email", "@"] }, 0] } },       domainPart: { $toLower: { $arrayElemAt: [{ $split: ["$Email", "@"] }, 1] } },       Email: 1     }   },   {     $group: {       _id: "$sameNameWithMultipleDomains",       emails: { $addToSet: "$Email" },       domains: { $addToSet: "$domainPart" },       count: { $sum: 1 }     }   },   {     $match: {       count: { $gt: 1 }, // Ensure there are multiple records with the same local part       "domains.1": { $exists: true } // Ensure there is more than one distinct domain     }   },   {     $project: {       _id: 0,       sameNameWithMultipleDomains: "$_id",       emails: 1,       domains: 1     }   } ]); **(Q1) Create a Mongo query for the example record below that will find duplicate Email addresses with possibly different domains.**  {     "_id" : ObjectId("678d7744d934606b31204982"),     "Role" : "Evaluated",     "Email" : "[ed.phelps@origDomain.com](mailto:ed.phelps@origDomain.com)",     "FirstName" : "Ed",     "LastName" : "Phelps" }  **(A1) Explanation:**     1. $project: This stage splits the Email field into two parts: localPart (the part before @) and domainPart (the part after @). It retains the original email for reference.    2. $group: Groups the records by localPart (the part of the email before @). It collects all the distinct emails and domains associated with the same local part and counts the number of records.    3. $match: Filters out the records that don't have multiple occurrences of the same localPart with different domains.    4. $project: Finally, it outputs the localPart, associated emails, and distinct domains.  This query helps identify cases where the same `localPart` (e.g., `ed.phelps`) is used with different domain names (e.g., `origDomain.com` and possibly others).  **(Q2)  Can you change that query to be case insensitive?**  **(A2) ** **Explanation of changes:**     - $toLower: This operator is used on both the localPart and domainPart to convert them to lowercase before performing the operations. This ensures that email comparisons are case-insensitive, so Ed.Phelps@OrigDomain.com and ed.phelps@origdomain.com will be considered the same.  Now the query will treat email addresses as case-insensitive, ensuring duplicates are identified regardless of letter casing. |
| --- | --- |
| #### Find users with the NEW domain | Search specifically for the **NEW **domain  db.users.find({"Email": /**newDomain.com**/i }) |
| #### Obscure NEW domain users that were accidentally created by logging in with the NEW domain | > **📝 Note** > > Make a copy of the **users **collection to **users_bkp_YYYYMMDD** in Studio3T before running the update query  > **📝 Note** > > [Reindex MongoDB](https://alteryx.atlassian.net/wiki/search?text=Reindex+MongoDB) after updating the **users** collection  Query will add **_old** to the **LastName** and **Email** fields for all **users **records in the **NEW **domain.  This is useful if users logged in with the **NEW** domain and created duplicate **users **collection records for themselves.  After running this query move to the query to update the **ORIGINAL users **records to the **NEW **domain.  db.users.updateMany(   {     "Email": { $regex: "@**newDomain.com**$", $options: "i" } // Match emails with domain   },   [     {       $set: {         "Email": {           $concat: [             { $arrayElemAt: [{ $split: ["$Email", "@"] }, 0] }, // Local part of the email             "_old@",             { $arrayElemAt: [{ $split: ["$Email", "@"] }, 1] } // Domain part of the email           ]         },         "LastName": { $concat: ["$LastName", "_old"] }       }     }   ] ); *** ChatGPT request to get the above query ***  **(Q3) Write a MongoDB query to add "_old" to the LastName field and "_old" to the Email field before the "@" sign for email addresses with domain "newDomain.com"**  **(A3) Explanation:**     1. $regex: "@newDomain.com$": This condition ensures that the Email field ends with "newDomain.com", matching all email addresses with this domain. The $ ensures the match happens at the end of the string, and the i option makes the match case-insensitive.    2. $set: The $set operator is used to modify the Email and LastName fields.Email field:The local part (before the @) is extracted using $split and $arrayElemAt."_old@" is appended to the local part.The domain part (after the @) is preserved as is.LastName field: "_old" is appended to the existing value in the LastName field.       - Email field:The local part (before the @) is extracted using $split and $arrayElemAt."_old@" is appended to the local part.The domain part (after the @) is preserved as is.          - The local part (before the @) is extracted using $split and $arrayElemAt.          - "_old@" is appended to the local part.          - The domain part (after the @) is preserved as is.        - LastName field: "_old" is appended to the existing value in the LastName field.     3. $split: This operator splits the Email field at the @ symbol. The local part is accessed using $arrayElemAt at index 0, and the domain part is accessed at index 1.  **Result**  This query will update all records where the `Email` domain is `"newDomain.com"`, appending `"_old"` to both the `LastName` field and the local part of the `Email` field (before the `@`). The domain part of the `Email` remains unchanged. |
| #### Update ORIGINAL domain to NEW domain | > **📝 Note** > > Make a copy of the **users **collection to **users_bkp_YYYYMMDD** in Studio3T  > **📝 Note** > > Only perform this update after running the query above to look for duplicate user email addresses with multiple domains.  Running query below when a record already exists with the **NEW** domain will cause duplicate email addresses in the **users **collection, which confuses/breaks Server.  db.users.updateMany(   {     "Email": { $regex: "@**origDomain.com**$", $options: "i" }   },   [     {       $set: {         "Email": {           $concat: [             { $arrayElemAt: [{ $split: ["$Email", "@"] }, 0] },             "@**newDomain.com**"           ]         }       }     }   ] ); |

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