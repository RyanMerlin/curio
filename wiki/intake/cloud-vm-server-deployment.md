---
id: 36b557ee678b5cbc
title: Cloud / VM Server Deployment
status: intake
source:
  kind: confluence_page
  id: confluence-page:1698109505
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/1698109505
  summary: null
category: []
keywords: []
created_at: 2026-04-14T15:06:10Z
updated_at: 2026-04-14T15:06:10Z
confidence: null
cross_refs: []
content_hash: sha256:82104ff53eede5b1285458dabcfe2827c933b6dcec91dc6e8d27961b8e385bbc
confluence_page_id: null
model_used: null
---

|  |  |
| --- | --- |
|  |  |

# Moving On-Prem to VM

> **ℹ️ Info**
>
> Exchange from Teams on answering customer request on how best to move on-prem Server VMs to Azure

[Susan K] Hey Everyone, need your input on the following questions. This has been ongoing with Advanced customer (Medline), AE, Joanna G and myself. My case has been closed for some time as the customer was asking if Alteryx support using Azure Migrate to lift and shift on-prem Alteryx Server VMs to Azure. I had said we had no experience with this and provided Server Host Recovery Guide as best practice. He's been pushing for a call to discuss still using Azure migrate. AE did have a call with him and the following questions resulted:

- Alteryx support is recommending 8 hours of professional service consultation for this need (~$3500).
- Per our discussion, I don’t think we need professional service for this. All we need is 30 minutes of connect with the right audience on Alteryx's end including you who can answer a few questions. As mentioned, we have done this exercise with all other vendors in the BI space successfully without using professional services (some with very basic app support setup). Here is a list of questions,What is Alteryx's recommended way of migrating from an on-prem VM to an Azure VM?Azure migration is a tool that Medline will leverage but does Alteryx support VM snapshot restore method on new VM (i.e. Lift and shift) for migration?If Alteryx does not support this method and Medline would still proceed with it (of course testing in a lower environment first),If an issue occurs during the migration using the Azure migrate approach, will Alteryx provide support?If Medline successfully migrates using this method and after some time (1-2 weeks or beyond) an issue occurs with the application hosted in Azure cloud, will Alteryx provide the same level of support they are providing right now? We want to ensure that once the application is live in Azure VM for some time, working as expected, and if any issue occurs there should not be any impact to our support agreement.
   - What is Alteryx's recommended way of migrating from an on-prem VM to an Azure VM?
   - Azure migration is a tool that Medline will leverage but does Alteryx support VM snapshot restore method on new VM (i.e. Lift and shift) for migration?
   - If Alteryx does not support this method and Medline would still proceed with it (of course testing in a lower environment first),If an issue occurs during the migration using the Azure migrate approach, will Alteryx provide support?If Medline successfully migrates using this method and after some time (1-2 weeks or beyond) an issue occurs with the application hosted in Azure cloud, will Alteryx provide the same level of support they are providing right now? We want to ensure that once the application is live in Azure VM for some time, working as expected, and if any issue occurs there should not be any impact to our support agreement.
      - If an issue occurs during the migration using the Azure migrate approach, will Alteryx provide support?
      - If Medline successfully migrates using this method and after some time (1-2 weeks or beyond) an issue occurs with the application hosted in Azure cloud, will Alteryx provide the same level of support they are providing right now? We want to ensure that once the application is live in Azure VM for some time, working as expected, and if any issue occurs there should not be any impact to our support agreement.

I don't want Alteryx Support sucked into a black hole in an area where we have already stated that we have no expertise, or experience. I want to answer Server Host Recovery Guide again, but I'm sure they want to go ahead with lifting and shifting.

## [Cameron] These would be my answers

- What is Alteryx's recommended way of migrating from an on-prem VM to an Azure VM?Our recommended way is to use the Server Host Recovery Guide.
- Azure migration is a tool that Medline will leverage but does Alteryx support VM snapshot restore method on new VM (i.e. Lift and shift) for migration?We still recommend using the Server Host Recovery Guide. Any time you move the database from one machine, to another, that guide must be followed.
- If Alteryx does not support this method and Medline would still proceed with it (of course testing in a lower environment first),If an issue occurs during the migration using the Azure migrate approach, will Alteryx provide support?We won't provide support for the Azure Migrate process itself, this would need to be delegated to Azure support. If you experience issues specifically with Alteryx after the Azure migrate is completed, we can assist in troubleshooting the Alteryx end. However, if you don't follow the Server Host Recovery we will still revert to recommending this process, regardless of how you migrate to this new VM. If Medline successfully migrates using this method and after some time (1-2 weeks or beyond) an issue occurs with the application hosted in Azure cloud, will Alteryx provide the same level of support they are providing right now? We want to ensure that once the application is live in Azure VM for some time, working as expected, and if any issue occurs there should not be any impact to our support agreement. The only issue here is that if you choose to perform the migration process and do not perform the server host recovery, this could lead to a working database initially and lead to problems down the road. We can't stress enough how important it is to follow our recommended process. However, in terms of what support is provided, we will still provide the same support. If you choose to perform the Azure Migrate path, please follow the server host recovery guide regardless to avoid issues. For example, you may be able to migrate to a new VM, but follow the server host recovery on the new VM to properly decrypt the database.
   - If an issue occurs during the migration using the Azure migrate approach, will Alteryx provide support?We won't provide support for the Azure Migrate process itself, this would need to be delegated to Azure support. If you experience issues specifically with Alteryx after the Azure migrate is completed, we can assist in troubleshooting the Alteryx end. However, if you don't follow the Server Host Recovery we will still revert to recommending this process, regardless of how you migrate to this new VM.
   - If Medline successfully migrates using this method and after some time (1-2 weeks or beyond) an issue occurs with the application hosted in Azure cloud, will Alteryx provide the same level of support they are providing right now? We want to ensure that once the application is live in Azure VM for some time, working as expected, and if any issue occurs there should not be any impact to our support agreement. The only issue here is that if you choose to perform the migration process and do not perform the server host recovery, this could lead to a working database initially and lead to problems down the road. We can't stress enough how important it is to follow our recommended process. However, in terms of what support is provided, we will still provide the same support.If you choose to perform the Azure Migrate path, please follow the server host recovery guide regardless to avoid issues. For example, you may be able to migrate to a new VM, but follow the server host recovery on the new VM to properly decrypt the database.