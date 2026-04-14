---
id: 54252f8b774443bb
title: '[DRAFT] High Availability Upgrade Best Practices'
status: intake
source:
  kind: confluence_page
  id: confluence-page:2624061518
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/2624061518
  summary: null
category: []
keywords: []
created_at: 2026-04-14T15:18:28Z
updated_at: 2026-04-14T15:18:28Z
confidence: null
cross_refs: []
content_hash: sha256:0aefcf6800ffb0a2a2b1bbf9ced1e2bb407801c06b4f14825dc31fb4344196d1
confluence_page_id: null
model_used: null
---

This page is a draft update for

[High Availability Upgrade Best Practices](https://alteryx.atlassian.net/wiki/search?text=High+Availability+Upgrade+Best+Practices)

The page presents Best Practices for upgrading an HA Server Environment

---

---

# Before Attempting an HA Upgrade

For a Prod environment, you should have performed the following before attempting the Prod upgrade:

**Option 1**
Successfully upgraded and tested a UAT or Dev server that used a recent backup of the Prod database (from the [Host Recovery](https://help.alteryx.com/current/en/server/install/server-host-recovery-guide.html) process)

**Option 2** 
Successfully upgraded and tested a Sandbox Server by taking the following steps:

- Install same-version Server to Sandbox
- Copy the Mongo database to a Sandbox User-Managed MongoDB
- Perform a Server Host Recovery
- Connect Sandbox to the Sandbox User-Managed MongoDB
- Confirm encryption key stores are functioning
- Upgrade the Sandbox
- Test critical workflows are performing as expected

# Overview

> **ℹ️ Info**
>
> Downtime will be required for your initial validation and during the upgrades of each HA Node.  The HA environment cannot remain functional during this process.

The following sections will detail these steps

- Confirm Only One HA Controller is Active
- Validate Primary and Failover Server Key Stores
- Stop MS Failover ClusterDetermine a Strategy to Access Each HA Gallery DirectlyStop the MS Failover Cluster to ensure it doesn’t failover and start an HA Failover Controller during the HA Primary upgrade
   - Determine a Strategy to Access Each HA Gallery Directly
   - Stop the MS Failover Cluster to ensure it doesn’t failover and start an HA Failover Controller during the HA Primary upgrade

- Upgrade HA Primary NodeUpgrade all nodes (Controller, Gallery(s), Worker(s))Start ControllerIf not included in the Controller node, start one Gallery to perform MongoDB Schema migrationsStart WorkersTest critical workflowsStop Controller
   - Upgrade all nodes (Controller, Gallery(s), Worker(s))
   - Start Controller
   - If not included in the Controller node, start one Gallery to perform MongoDB Schema migrations
   - Start Workers
   - Test critical workflows
   - Stop Controller

- Upgrade each HA Failover NodeUpgrade all nodes (Controller, Gallery(s), Worker(s))Start ControllerIf not included in the Controller node, start one Gallery (database has already been schema migrated)Start WorkersTest critical workflowsStop Controller
   - Upgrade all nodes (Controller, Gallery(s), Worker(s))
   - Start Controller
   - If not included in the Controller node, start one Gallery (database has already been schema migrated)
   - Start Workers
   - Test critical workflows
   - Stop Controller

## HA Architecture Diagram

Note that only the HA Primary Controller is running and all HA Nodes share the same MognoDB (likely a multinode replica set)

---

# Confirm Only One HA Controller is Active

> **ℹ️ Info**
>
> Alteryx HA configuration is **Active-Passive** in which only ONE Controller should be running

To ensure only one HA Controller is active, please confirm:

- Each Controller Alteryx Service is set to Manual start [tbd-should Gallery and Worker Nodes be running?]

- Only the HA Primary Controller is Running out of all the HA Controllers

---

# Validate Primary and Failover Server Key Stores

> **ℹ️ Info**
>
> Key Stores decrypt
> 
> - Credentials
> - DCM Connections
> - Shared Database Connections
> 
> A Server can start and APPEAR to run even when its Key Stores are not functioning. However, it will fail when running a workflow using any of the above elements.

**Option 1**
[**TBD - determine a minimal Key Store test**]

**Option 2 **
Run workflows that use

- a Credential
- DCM
- a Shared Callery Connection

---

# Stop MS Failover Cluster

## Determine a Strategy to Access Each HA Gallery Directly

> **ℹ️ Info**
>
> Determine how you can directly access each HA Node for testing when the MS Failover Cluster is stopped

The DNS for the Gallery URL may point to

- Option 1The MS Failover Cluster (cluster.company.com).  This relies on the MS Failover Cluster to forward the URL to the HA Node that is currently running. When you stop the MS Failover Cluster the URL will no longer resolve and you will be unable to start any of the HA Controllers for testing unless you edit the Controller’s hosts file [tbd hos to do this]
- Option 2Directly to the HA Nodes (direct.company.com).  In this scenario, the DNS will point the URL to each of the HA Gallery IPs and function when any Gallery is running. [tbd-correct?].  No additional settings are required for testing individual HA Nodes with this configuration.

## Stop the MS Failover Cluster 

> **ℹ️ Info**
>
> During HA Primary upgrade, you need to ensure the MS Failover Cluster does not attempt to start an HA Failover Node

Stop the MS Failover Cluster

[**TBD - Link to MS documentation for this**]

---

# Upgrade HA Primary Node 

> **ℹ️ Info**
>
> Upgrading and starting the HA Primary Node will lead to upgrade Schema Migrations in the database

Please see upgrade documentation, including the links to the **Server Upgrade Checklist** and **Version-to-Version Guide**:  <https://help.alteryx.com/current/en/server/install/install-or-upgrade-server.html##>

Perform the following steps in the HA Primary environment

- Upgrade all Alteryx nodes (Controller, Gallery(s), Worker(s))
- Start Controller
- If not included in the Controller node, start one GalleryThe Gallery will perform MongoDB Schema migrations
   - The Gallery will perform MongoDB Schema migrations

- Start Workers
- Test critical workflows
- Stop Controller

AHHHHHHHHH

- It is possible that the controller token changes as part of upgrade if you are upgrading from a version that has 48 Characters of Controller Token. Hence verify the controller Token and Storage Keys Encrypted value to be identical.
- If the tokens are different after the upgrade, perform the host recovery on Node 2 and 3

---

# Upgrade each HA Failover Node

> **ℹ️ Info**
>
> Upgrading and starting the HA Failover Nodes will confirm they are ready to function in a failover situation

Perform the following steps for each HA Failover Node

- Upgrade all Alteryx nodes (Controller, Gallery(s), Worker(s))
- Start Controller
- If not included in the Controller node, start one GalleryNote: database has already been schema-migrated
   - Note: database has already been schema-migrated

- Start Workers
- Test critical workflows
- Stop Controller

---

Draft provided to

- 00734720