---
id: 5015ad615db965e9
title: Get Customer's Mongo Collections as YXDBs
status: intake
source:
  kind: confluence_page
  id: confluence-page:1840678524
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/1840678524
  summary: null
category: []
keywords: []
created_at: 2026-04-14T15:18:28Z
updated_at: 2026-04-14T15:18:28Z
confidence: null
cross_refs: []
content_hash: sha256:83c02c33ed42b3e6b68f38f686b9b2f605616c71fcd191efac313d98643e28ca
confluence_page_id: null
model_used: null
---

> **ℹ️ Info**
>
> This article explains the first of the options below for reviewing a customer’s database
> 
> - have the user dump Mongo collections to YXDB files and upload them to a Drop link
> - run diagnostic workfows in the user’s environment
> - have customer provide a Mongo backup, RuntimeSettings.xml. and Controller token so you can perform a Host Recovery

# How to get Embedded Mongo Collections From a Customer

> **ℹ️ Info**
>
> Configure and send the workflow below to the customer. It will save the contents of the collections you need to YXDB files. Collections with a **ServiceData** blob will unpack the fields from the blob.

(1) Download and configure ayxGetCollections.yxmc for the Collections you want.

(2) Configure the **macro** by enabling the Containers for the Collections you want from the user.

(3) Edit the files in Notepad++ and set the version number at the top of the XML to match customer’s version so they can run the macro in the workflow.

(4) **ZIP** the files

(5) Create a **Drop link**

(6) Send **ZIP** and **Drop link** with the directions below

To gather the Mongo Collections for further analysis can you please :

(1) Open the attached **ayxGetMongoCollections_v##.zip** file in Designer on your Server machine.

(2) Enter your **Mongo Non-Admin password **in the workflow as instructed in the workflow instructions.

(3) **Run **the workflow

(4) **ZIP** the files created in collections subfolder

(5) Upload the **ZIP file** to the following secure **Drop location**:

– DROP_LINK
– password:  PASSWORD
## Old versions