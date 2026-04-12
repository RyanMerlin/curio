---
id: 3eea4a24505d9806
title: Server Health Check
status: staged
source:
  kind: confluence_page
  id: confluence-page:2002419792
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/2002419792
  summary: null
category:
- product-tree
- alteryx-server
keywords:
- server
- health
- check
- mongodb
- sharepoint
created_at: 2026-04-12T20:59:51Z
updated_at: 2026-04-12T21:05:15Z
confidence: 0.55
cross_refs: []
content_hash: sha256:c9d8168074be07759847493c62b8c9fa71d87fa4f5734f750d63cd49c6195bca
confluence_page_id: null
model_used: heuristic
---

> **ℹ️ Info**
>
> The Server Health check reviews the last 3 months of Server use to make recomendations regarding sizing.
> 
> **It does NOT perform a Health Check** in regards to the health of the Server, ie **it does NOT**:
> 
> - look for referenntial integrity issues
> - data anomalies like those found in the Pre-Upgrade Checks workflow
> - issues with the database or DCM/Shared Database Connections encryption, or
> - anything else related to the actual health of the Server.
> 
> While previously only available to TAMs, as of Dec-2023 anyone can distribute this tool, however interpreting the results may require a TAM’s assistance.

> **📝 Note**
>
> 24.2 adjusted sizing recommendations for ALL server versions and the Server Health Check is being adjusted accordingly.  This will change the recommendations it produces for 24.1 and prior Servers.
> 
> [Server Sizing Methodology Changes & Messaging](https://alteryx0.sharepoint.com/sites/SolutionsArchitecture/SitePages/Server-Sizing-Methodology-Changes-%26-Messaging.aspx?xsdata=MDV8MDJ8fDdkMzM1YWU4MzkyOTRhMzgzOGY4MDhkY2ZkZmNmZjRjfDUyMmYzOWQ5MzAzZDQ4OGY5ZGViYTZkNzdmMWVhZmQ4fDB8MHw2Mzg2NjQ1MTAyMzY1MzkxMjh8VW5rbm93bnxWR1ZoYlhOVFpXTjFjbWwwZVZObGNuWnBZMlY4ZXlKV0lqb2lNQzR3TGpBd01EQWlMQ0pRSWpvaVYybHVNeklpTENKQlRpSTZJazkwYUdWeUlpd2lWMVFpT2pFeGZRPT18MXxMMk5vWVhSekx6RTVPakJtTXpCa1pqa3hMV0prTlRNdE5ETXdOaTFoTVRnNExUWXpNV0kyTnpRek9HUmxZbDlqTm1Zd1l6RTRaaTFqTm1FeExUUmhZamN0T1dJNE15MDJOelkwT1RVMk5XUXlPVEpBZFc1eExtZGliQzV6Y0dGalpYTXZiV1Z6YzJGblpYTXZNVGN6TURnMU5ESXlNekExT1E9PXwxMDg5OTVhZmI2MDE0MzdkZDY5YzA4ZGNmZGZjZmY0OXw0MDIzOWFkNjJkNWI0MDYyOWY1NzQ1OTRiYzFmYzRlMg%3D%3D&sdata=WURZMFptclJYTlVnUnoxd3RicmI0TTJaT0hQSyt1bExRQlpTb1M1T1lvVT0%3D&ovuser=522f39d9-303d-488f-9deb-a6d77f1eafd8%2Cmhochstein%40alteryx.com&OR=Teams-HL&CT=1730909068579&clickparams=eyJBcHBOYW1lIjoiVGVhbXMtRGVza3RvcCIsIkFwcFZlcnNpb24iOiI0OS8yNDEwMDMyNDkxNiIsIkhhc0ZlZGVyYXRlZFVzZXIiOmZhbHNlfQ%3D%3D) (Sharepoint)

| Access | Alteryx Workbench (SharePoint) > Server Health Check |
| --- | --- |
| Key Articles | Server Health Check: Overview (SharePoint) |
| Notes | [Michael Spoula Jun-26-2023] Do note that the newer versions for TLS-enabled MongoDB connections will require the Simba MongoDB driver with an appropriate alias configured [Zach Hamilton - Jun-26-2023] We support both MongoDB Atlas and TLS-enabled MongoDB deployments using the Simba MongoDB Driver |

---

| Nov-02-2023 Meeting with Dan Hilton |  |
| --- | --- |
| Memory | They use Available Physical memory, which is less than Installed memory |
| Server Health Check | Can view it to see how it’s picking up values/ |
| Best practices | Allow server to manageYesWhen to increase #simNon-cpu intensive.  Ex: largely In-DB workflowsWhat fails - different When to decrease #simNeverWhen to Increase memory?Boosts performae for most intensibve workflowsHow to handle HUGE amounts of mmeory, can they up the memory per WF.  YesWhen to Decrease MemoryIn-DB doesn’t need muchHow to understand We can’t track what RAM and CPU usage Mongo uses |
| Health Check | Unlocked version of Server Health Checkhttps://nam02.safelinks.protection.outlook.com/?url=https%3A%2F%2Fdrop.alteryx.com%2Fpublic%2Ffolder%2F1tnf4mm3de271a1aad-gba%2FServerHealthCheck&data=04%7C01%7Czhamilton%40alteryx.com%7C03a658bd9fb841b9292d08d9e4e4960e%7C522f39d9303d488f9deba6d77f1eafd8%7C0%7C0%7C637792492721469844%7CUnknown%7CTWFpbGZsb3d8eyJWIjoiMC4wLjAwMDAiLCJQIjoiV2luMzIiLCJBTiI6Ik1haWwiLCJXVCI6Mn0%3D%7C3000&sdata=iaAMjHlZpw2PNnWKhOHiE0O6kDsJXW1vktvii2Go2%2Fo%3D&reserved=0 pswd: oZ9BgyLL5!2Server Health Check: Overview (SharePoint) |