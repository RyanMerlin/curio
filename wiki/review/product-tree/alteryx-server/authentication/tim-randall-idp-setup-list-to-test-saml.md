---
id: 1294bfedc8f7b04c
title: (Tim Randall) IDP setup list to test (SAML)
status: review
source:
  kind: confluence_page
  id: confluence-page:2003271981
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/2003271981
  summary: null
category:
- product-tree
- alteryx-server
- authentication
keywords:
- saml
- idp
- testing
- draft
- personal-working-page
created_at: 2026-04-14T15:18:28Z
updated_at: 2026-04-14T15:20:52Z
confidence: 0.5
cross_refs: []
content_hash: sha256:e2b948a989eac494ee597616cafc9792c9b83220a83928216b9080e1371a78df
confluence_page_id: null
model_used: claude-sonnet-4-6
---

Working page for Tim Randall

---

---

# Easily configurable

| **IDP** | **Developer Account/Env setup** | **Configuration setup** |
| --- | --- | --- |
| **Azure AD** | <https://developer.microsoft.com/en-us/microsoft-365/dev-program>  (90 day exp) | [Configuring SAML 2.0 on Alteryx Server for Azure AD](https://knowledge.alteryx.com/index/s/article/Configuring-SAML-2-0-on-Alteryx-Server-for-Azure-AD) (KB) |
| **JumpCloud** | <https://jumpcloud.com/lp/cloud-directory-fava-bean> | [Configuring SAML on Alteryx Server with JumpCloud](https://knowledge.alteryx.com/index/s/article/Configuring-SAML-on-Alteryx-Server-with-JumpCloud) (KB) |
| **Okta** | <https://dev-418598-admin.oktapreview.com/admin/dashboard> | [Configuring SAML on Alteryx Server for Okta](https://knowledge.alteryx.com/index/s/article/Configuring-SAML-on-Alteryx-Server-for-Okta-1583461082739) (KB) |
| **OneLogin** | <https://www.onelogin.com/register/142498> | [Configuring SAML on Alteryx Server for OneLogin](https://knowledge.alteryx.com/index/s/article/Configuring-SAML-on-Alteryx-Server-for-OneLogin-1583461566692) (KB) |
| **PingOne** | <https://www.pingidentity.com/en/try-ping.html> (30 days) | [Configuring SAML on Alteryx Server for PingOne](https://knowledge.alteryx.com/index/s/article/Configuring-SAML-on-Alteryx-Server-for-PingOne-1583461082735) (KB) |

---

# Require environment set-up/license (perhaps we can inquire with ones requiring a license on possible sandbox licenses in the future?):

| **IDP** | **Developer Account/Env setup** | **Configuration setup** |
| --- | --- | --- |
| **ADFS** | <https://learn.microsoft.com/en-us/microsoft-365/troubleshoot/active-directory/set-up-adfs-for-single-sign-on>  <https://learn.microsoft.com/en-us/windows-server/identity/ad-fs/deployment/install-the-ad-fs-role-service> (more fine-tuned set-up doc TBD soon) | [Configuring SAML on Alteryx Server for ADFS](https://knowledge.alteryx.com/index/s/article/Configuring-SAML-on-Alteryx-Server-for-ADFS-1583461562791) (KB) |
| **PingFederate** | <https://docs.pingidentity.com/r/en-us/pingfederate-110/help_initialsetup_settinguppingfederate> | [PingFederate (SAML)](https://alteryx.atlassian.net/wiki/search?text=PingFederate+(SAML)) |
| **WebSeal (IBM)** | N/A | N/A |