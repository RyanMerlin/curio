---
id: 6b531530a731bd65
title: Blue-Green Deployment Checklist
status: intake
source:
  kind: confluence_page
  id: confluence-page:3221160357
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/3221160357
  summary: null
category: []
keywords: []
created_at: 2026-04-14T15:06:10Z
updated_at: 2026-04-14T15:06:10Z
confidence: null
cross_refs: []
content_hash: sha256:ab56f56bf9080fe5ded2df368f5b0dbfea3e43882c1ffd9433bc7df3579c64d9
confluence_page_id: null
model_used: null
---

Can build off of the current [Server Upgrade Checklist](https://help.alteryx.com/current/en/server/install/install-or-upgrade-server/server-upgrade-checklist.html#upgrade-7047039), with additional considerations (edits shown in yellow highlight,  removed portions in magenta highlight with strikethrough ):

# [EDITED] Blue-Green Deployment Server Upgrade Checklist

Your Server configuration is unique and upgrading it is a project that requires planning and preparatory work to be successful. This checklist ensures you consider all tasks that might be needed for your Blue-Green  upgrade and directs you to Help and Knowledge Base articles for detailed step-by-step procedures.

If you would like help preparing or executing your upgrade, please speak with your Account Executive for options.

## What is a Blue-Green deployment?

A Sandbox server upgrade becomes the new Production environment after validation. This eliminates the risk of your Production Server being down for an indeterminate amount of time as it is not upgraded in place. Blue-Green deployment validates that the Server environment and required database drivers, DSNs, Connectors and other settings are fully understood as they must be set up on the Sandbox for validation.

|  |  |  |  |  |  |  |
| --- | --- | --- | --- | --- | --- | --- |
|  |  |  |  |  |  |  |

## Server Upgrade Overview

Testing your upgrade process prior to upgrading your Production server is the **best way to ensure your Server upgrade process will run smoothly in your production environment**.

Ideally, start with a same-version Sandbox/Dev/Test Server and upgrade it, see [Alteryx Server Sandbox Environment](https://knowledge.alteryx.com/index/s/article/Alteryx-Server-Sandbox-Environment). If you have a multinode environment, testing is still effective on a single machine that runs Controller + Server UI + Worker. Similarly, if you have User-Managed MongoDB, restoring a database backup to the test machine's embedded Mongo can help validate the upgrade. Contact your Account Executive for information on a Sandbox license.

**At a bare minimum**, you should install the target version of Designer on a user's machine to test critical workflows in the new version. For more information, go to [Install Two Versions of Designer on the Same Machine](https://help.alteryx.com/current/en/license-and-activate/install/install-two-versions-of-designer-on-the-same-machine.html).

**Ideal process:**

## Server Upgrade Process

### Plan

|  |  |
| --- | --- |
|  |  |
|  |  |
|  |  |
|  |  |
|  |  |
|  |  |
|  |  |
|  |  |
|  |  |
|  |  |
|  |  |
|  |  |
|  |  |
|  |  |

### Prep Work

|  |  |
| --- | --- |
|  |  |
|  |  |
|  |  |
|  |  |
|  |  |
|  |  |
|  |  |
|  |  |
|  |  |

### Upgrade

|  |  |
| --- | --- |
|  |  |
|  |  |
|  |  |
|  |  |
|  |  |
|  |  |
|  |  |
|  |  |
|  |  |
|  |  |
|  |  |
|  |  |
|  |  |
|  |  |
|  |  |
|  |  |

### Test

|  |  |
| --- | --- |
|  |  |
|  |  |
|  |  |

**Switch Traffic/Go Live**

|  |  |
| --- | --- |
|  |  |
|  |  |

### Troubleshoot

|  |  |
| --- | --- |
|  |  |
|  |  |
|  |  |
|  |  |
|  |  |
|  |  |
|  |  |
|  |  |