---
id: fa93e3a8f0bcfdd9
title: NTLM Use on Server
status: intake
source:
  kind: confluence_page
  id: confluence-page:3754000425
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/3754000425
  summary: null
category: []
keywords: []
created_at: 2026-04-14T15:18:28Z
updated_at: 2026-04-14T15:18:28Z
confidence: null
cross_refs: []
content_hash: sha256:499bfe2eff91e15f16dab1bb37a6e412548634d04e9d05a40a03ace97b28a824
confluence_page_id: null
model_used: null
---

> **ℹ️ Info**
>
> NTLM (New Technology LAN Manager) is a suite of Microsoft security protocols for authenticating users and computers on Windows networks
> 
> It’s being deprecated, so it’s a great example of why you should never name something “New”.

“Is NTLMv1 authentication used in Alteryx for connections?  What impact will there be as my organization decommissions NTLMv1?“From Michael Phipps in 00805430 Jan-2026Does Alteryx use NTLMv1 for connections?Alteryx does not require NTLMv1. In Windows environments, authentication for “Integrated Windows Authentication” (IWA) is negotiated by Windows/SSPI based on your domain policy and the target service configuration. If your environment is already using Kerberos successfully for IWA, then disabling/decommissioning NTLMv1 should not impact normal Alteryx operation.Will decommissioning NTLMv1 have any impact?In most cases, no — especially if Kerberos is being used end-to-end. The only time you may see impact is if a specific connection cannot use Kerberos and Windows falls back to an NTLM variant during negotiation. Typical causes of fallback include:SPN / delegation / DNS issues preventing Kerberos for a given endpoint (Kerberos fails and Windows negotiates another method).Accessing services by IP address or using aliases/CNAMEs that aren’t configured for Kerberos/SPNs.Older/legacy endpoints or drivers (some database/proxy scenarios) that may not support Kerberos cleanly and rely on NTLM negotiation.Cross-domain/trust constraints where Kerberos isn’t available for that resource.If your organization is decommissioning NTLMv1 only (and not NTLM entirely), the risk is likely limited, since NTLMv2 may still be allowed for any rare fallback cases. If your organization plans to disable all NTLM (v1 and v2), the validation becomes more important.Recommended validation (quick and practical):Identify a small set of critical workflows that use IWA (e.g., database connections, file shares, web endpoints).After the policy change in a test group (or pilot OU), re-run those workflows in Designer/Server.Confirm Kerberos is being used to the target services (your Windows/security team can validate via logs or authentication event data).If you’d like, we can help you narrow the scope:If you can share (at a high level) which connection types you’re using with IWA (e.g., SQL Server, SharePoint/web endpoints, UNC file shares, proxies, etc.) and whether this is Designer, Server, or both, we can call out the most likely areas to validate in your setup.We also see that this case was opened with a Severity Level 2. Based on our Support Policy Guidelines, a case is considered a SEV 2 when a product defect is causing major but intermittent loss of production service with the operation of the Alteryx Product able to continue in a restricted manner. For more information, review our Support Policy Guidelines here:  https://www.alteryx.com/support-policy-and-guidelines  We are modifying the severity of the case accordingly.