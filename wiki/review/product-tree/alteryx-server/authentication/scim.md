---
id: b0213797ee4f82da
title: SCIM
status: review
source:
  kind: confluence_page
  id: confluence-page:2545254568
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/2545254568
  summary: null
category:
- product-tree
- alteryx-server
- authentication
keywords:
- scim
- provisioning
- sso
- hub
created_at: 2026-04-14T13:40:14Z
updated_at: 2026-04-14T13:50:47Z
confidence: 0.8
cross_refs: []
content_hash: sha256:ec5853303096858a0b633a20d21b4645bc74b615f6254f2fea8343a2ecf90df4
confluence_page_id: null
model_used: claude-sonnet-4-6
---

---

*[Organized section — child pages listed separately]*

---

> **ℹ️ Info**
>
> Supported in 23.1+

> **ℹ️ Info**
>
> **DEFINITION**
> 
> SCIM (System Cross-domain Identity Management) is a protocol that standardizes how identity information is exchanged between one entity and another.
> 
> We typically use it to sync User Group and User information from **Entra ID** to a Server using **Entra ID SAML** auth.  As most customers sync their **on-prem AD** with their **cloud Entra ID**, a SCIM configuration would effectively sync their **AD Groups** to their Alteryx Server.
> 
> But SCIM is an endpoint standard independent of a specific product.  Therefore it could also be used to sync **Okta** User Groups to Alteryx Server using **Okta SAML**.

> **ℹ️ Info**
>
> **HOW GROUPS APPEAR**
> 
> Note:  The User Groups** **being synced will appear as **Custom Groups** in Server.  It is NOT recommended to edit these **Custom Groups** directly, but no guard rails in the product prevent this.
> 
> SCIM **pushes** AD updates, it doesn’t interrogate Server to find the current state.  Therefore, it will not re-add a user that was manually removed from a Custom Group.

> **📝 Note**
>
> **An easier (but less comprehensive) alternative can be recommended for AD**
> 
> The **AD Sync Utility **is a workflow that performs some of the key SCIM functions with a less complex setup that doesn’t require an additional **Microsoft Entra Connect Provisioning Agent **Server (as is frequently required in customer’s Entra ID environments).
> 
> - https://help.alteryx.com/current/en/designer/workflows/enterprise-utilities/active-directory-sync.html#active-directory-sync                                                                       <== Help
> - https://marketplace.alteryx.com/en-US/apps/439219/server-user-management-enterprise-utility  <== Marketplace
> - Enterprise Utility - Active Directory Sync                                          <== Confluence
> 
> Technically, this is not SCIM as it works directly with the database rather than making the SCIM standard API calls.  But it will sync user lists from AD groups to Server Custom Groups.