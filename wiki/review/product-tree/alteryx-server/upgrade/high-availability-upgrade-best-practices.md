---
id: 2b7216dfee1dd0b9
title: High Availability Upgrade Best Practices
status: review
source:
  kind: confluence_page
  id: confluence-page:2383446384
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/2383446384
  summary: null
category:
- product-tree
- alteryx-server
- upgrade
keywords:
- upgrade
- high-availability
- best-practices
- ha
created_at: 2026-04-14T13:40:13Z
updated_at: 2026-04-14T13:50:12Z
confidence: 0.72
cross_refs: []
content_hash: sha256:bdd716c538edb47f1eb61a45a76b0b684ebf1edf72f3e95e31fcfbaf858beb51
confluence_page_id: null
model_used: claude-sonnet-4-6
---

note

# THIS PAGE IS BEING RE-DRAFTED IN

[DRAFT] High Availability Upgrade Best Pracices

# THIS PAGE IS BEING RE-DRAFTED IN

[[DRAFT] High Availability Upgrade Best Pracices](/wiki/spaces/SupportServer/pages/2624061518/DRAFT+High+Availability+Upgrade+Best+Pracices)

How to Upgrade a Failover Cluster Environment for Alteryx Server

By following these steps and checks, you can upgrade your Alteryx Server failover cluster environment while minimizing downtime and ensuring service continuity.

---

---

# 1. Overview

Alteryx Server supports high availability and scalability through an Active-Passive Failover configuration. In this setup, one server is active while another is on standby, ready to take over if the active server fails. This ensures minimal downtime, but it is not an active-active configuration since only one server handles the load at any given time.

---

# 2. Pre-Upgrade Checks

## 2.1 Check Customer Environment Configuration

- Active-Passive Configuration: Verify that the environment is set up as an active-passive cluster. Alteryx does not natively support active-active configurations.
- DNS Configuration: Confirm how the DNS is configured. Is it pointed to the failover cluster IP, or does it have a DNS record that runs the service even if the failover cluster service fails? This is crucial to ensure we can stop the failover cluster and upgrade a single node at a time. If the DNS is configured only to the failover cluster service, the service won't come up on Node 1 when the cluster service is turned off. You might need to add a hosts entry in C:\Windows\System32\Drivers\etc\ to bypass this.

## 2.2 Host Recovery and Sync Checks

- Ensure the customer has completed a proper host recovery.
- Verify that connections sync and credentials are created correctly when switching traffic to Node 2 or Node 3.

## 2.3 Pre-Upgrade Tools

- Pre-Upgrade Check: Run the pre-upgrade check to identify any potential issues in the database.
- Migration Prep Tool: Use the migration prep tool to check for and resolve any issues before performing the actual upgrade.

---

# 3. Upgrade Steps

## 3.1 Preparation

- Ensure the failover cluster service is stopped before starting the upgrade.
- Confirm that the service is not running on any other node.

## 3.2 Common Issues

- Active-Active Misconfiguration: Some customers may mistakenly run the service in an active-active state, which is not supported by Alteryx and can cause issues like duplicate key errors.
- Simultaneous Upgrades: Upgrading controller nodes simultaneously can result in duplicate key errors as the service starts on both nodes at the same time.

## 3.3 Step-by-Step Upgrade

1. Stop Other Services: Ensure services on other nodes, like Gallery and Worker, are stopped.
2. Upgrade Node 1: Perform the upgrade on Node 1 and ensure it completes successfully.
3. Sequential Node Upgrades:Stop Node 1.Upgrade Node 2 and Node 3 one at a time.It is possible that the controller token changes as part of upgrade if you are upgrading from a version that has 48 Characters of Controller Token. Hence verify the controller Token and Storage Keys Encrypted value to be identical.If the tokens are different after the upgrade, perform the host recovery on Node 2 and 3
   - Stop Node 1.
   - Upgrade Node 2 and Node 3 one at a time.
   - It is possible that the controller token changes as part of upgrade if you are upgrading from a version that has 48 Characters of Controller Token. Hence verify the controller Token and Storage Keys Encrypted value to be identical.
   - If the tokens are different after the upgrade, perform the host recovery on Node 2 and 3

4. Upgrade Gallery and Worker Nodes: If Gallery and Worker nodes are separate, upgrade them after the controller nodes.

## 3.4 Recommended Upgrade Order

- Ensure no other services are running during the upgrade.
- Upgrade one node at a time to avoid duplicate key errors. "Error Migrating PackageDefinitionMigration <632b0f64e5e49a0f08007586> Error: Mongo error: "E11000 duplicate key error collection: AlteryxService.AS_PackageDefinitions.22.3 index: _id_ dup key: { _id: ObjectId('632b0f64e5e49a0f08007586') }: generic server error" code: <mongodb:11000>"
- Parallelly upgrade non-controller nodes, but do not start services until the main controller node is up and running post-upgrade.

---

# 4. Post-Upgrade Sanity Checks

## 4.1 Data and Service Verification

- Collections and Indexes: Verify that all collections, users, subscriptions, workflows, etc., are loading correctly to avoid indexing issues.
- Data Connections: Ensure that data connections, DCM connections are syncing properly. Ensure the credentials are working.

## 4.2 Primary Server Switchover

- Switch Primary Server: Request the customer to switch the primary server to Node 2 and check if the data connections work as expected.
- Host Recovery Issues: If host recovery fails when switching to another node, switch back to Node 1 and perform proper host recovery on Node 2. Older versions of the server might not show credential issues unless tested, but these issues can arise post-upgrade when switching traffic.