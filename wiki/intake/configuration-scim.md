---
id: 23af4c1264b174a6
title: Configuration (SCIM)
status: intake
source:
  kind: confluence_page
  id: confluence-page:2545582282
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/2545582282
  summary: null
category: []
keywords: []
created_at: 2026-04-14T13:40:14Z
updated_at: 2026-04-14T13:40:14Z
confidence: null
cross_refs: []
content_hash: sha256:33c364f11b8d25932e731e59e78c58d1e04d522220c0746f9ba791c31568deeb
confluence_page_id: null
model_used: null
---

> **ℹ️ Info**
>
> Configuration options for SCIM

| Key Articles | SCIM token expiry date to be extended (KB) |
| --- | --- |

---

---

Configure APOD - SCIM for Entra ID using the Microsoft Entra Connect Provisioning Agent

---

# On-Prem AD - Easy option that doesn’t use SCIM

> **ℹ️ Info**
>
> The following will work when trying to push AD Groups to any Alteryx Server using SAML

After implementing SCIM, we created an Enterprise Tool (a workflow) that performs the same functionality for AD with a less complex setup that doesn’t require an additional **Microsoft Entra Connect Provisioning Agent **Server (as is frequently required in customer’s Entra ID environments).  So, an easier configuration and doesn’t need an extra server machine

- https://help.alteryx.com/current/en/designer/workflows/enterprise-utilities/active-directory-sync.html#active-directory-sync                                                                          <== Help
- https://marketplace.alteryx.com/en-US/apps/439219/server-user-management-enterprise-utility <== Marketplace download
- Enterprise Utility - Active Directory Sync +                                       <== Confluence

The Help doesn’t state the need to schedule this Enterprise Tool to ensure AD updates automatically appear in Server User Groups.

Technically, this is not SCIM as it works directly with the database rather than making the SCIM standard API calls. But the results are the same: Active Directory updates are synced to the Server.

---

# Entra ID

> **ℹ️ Info**
>
> Most customers sync their on-prem AD with their cloud Entra ID, so SCIM for Entra ID will, essentially, sync their AD user groups with the Server

| Basic setup for Entra ID | Create Entra ID Apphttps://help.alteryx.com/current/en/server/configure/configure-alteryx-server-authentication/configure-alteryx-server-for-scim-with-azure-active-directory.html  https://help.alteryx.com/current/en/server/configure/configure-alteryx-server-authentication/configure-saml-2-0-on-alteryx-server-for-azure-active-directory.html |
| --- | --- |
| Additional steps needed when Entra ID doesn’t have line-of-sight access to Alteryx Server | The above basic setup mentions that when Entra ID doesn’t have line-of-sight access to Server (which is common) an intermediary Server machine must be added called a Microsoft Entra Connect Provisioning Agenthttps://learn.microsoft.com/en-us/entra/identity/app-provisioning/on-premises-scim-provisioning The following is a walk-through of setting up the Provisioning Agent on APoDshttps://alteryx.atlassian.net/wiki/spaces/SupportServer/pages/1999176109/How+to+Configure+SCIM+for+Entra+ID+using+the+Microsoft+Entra+Connect+Provisioning+Agent?atl_f=content-tree Example of not having line-of-sight [00790386]We were facing issue while establishing a connection between the SCIM which is enabled in the Microsoft Entra App to the Azure VM which is hosted in azure UBS network. The issue is the ENTRA app is in Microsoft domain and the VM is in UBS domain, there is a firewall block happening and the UBS network is not allowing to connect to the Entra App. |

---

# Other SAMLs

SCIM is a standard for transferring user information and can work with OKTA and other SAMLs.  However, as of Oct-2024 we don’t have KBs for SAMLs other than Entra ID as explained above.

[SCIM Provisioning with OKTA Error authenticating Gateway Time-out](https://alteryx.lightning.force.com/kA0Uu0000000LVtKAM) (KB)