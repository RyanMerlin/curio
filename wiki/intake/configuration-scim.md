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
created_at: 2026-04-14T15:09:17Z
updated_at: 2026-04-14T15:09:17Z
confidence: null
cross_refs: []
content_hash: sha256:d5e0c4df5e69744e148981aebfcd261263bb6f69524b77b28e13348111512e2d
confluence_page_id: null
model_used: null
---

> **ℹ️ Info**
>
> Configuration options for SCIM

|  |  |
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

|  |  |
| --- | --- |
|  |  |

---

# Other SAMLs

SCIM is a standard for transferring user information and can work with OKTA and other SAMLs.  However, as of Oct-2024 we don’t have KBs for SAMLs other than Entra ID as explained above.

[SCIM Provisioning with OKTA Error authenticating Gateway Time-out](https://alteryx.lightning.force.com/kA0Uu0000000LVtKAM) (KB)