---
id: 24c0f8f7d99c30ef
title: Embedded MongoDB upgrade / migration
status: staged
source:
  kind: confluence_page
  id: confluence-page:2314994152
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/2314994152
  summary: null
category:
- by-account
keywords:
- upgrade
- mongodb
- embedded
- version
- mongo
created_at: 2026-04-12T20:59:51Z
updated_at: 2026-04-12T20:59:58Z
confidence: 0.55
cross_refs: []
content_hash: sha256:906f6483e73e2a272449d2c03cca4a7eea072b46e46657528ed91fc2bb8bb023
confluence_page_id: null
model_used: heuristic
---

---

---

> **ℹ️ Info**
>
> **Embedded MongoDB** is upgraded during some Server upgrades, see:
> 
> - https://help.alteryx.com/current/en/server/configure/database-management/mongodb-management/mongodb-schema-reference.html
> 
> **User-Managed** and **Atlas Cloud MongoDB** version upgrades are not performed by an Alteryx Server Upgrade, customers must manage those themselves

> **📝 Note**
>
> **Review the Mongo version upgrade log, xxx_PreUpgrade\migration.log, for errors**
> 
> - migration.log (embedded Mongo version upgrade)

> **📝 Note**
>
> **Server upgrade can only upgrade ONE Embedded MongoDB version at a time**
> 
> Customers who are moving to an Alteryx Server version that uses an Embedded MongoDB version that is TWO versions higher must make a "pitstop" upgrade
> 
> **Example**:  Customer is upgrading 23.1 to 24.2.  They must upgrade 23.1 to 23.2 or 24.1 first, then upgrade to 24.2 (so Mongo will upgrade 4.2->6.0 and then 6.0->7.0):
> 
> **     Server Ver    Embedded Mongo Ver**
> **     **24.2               7.0
> **     **23.2/24.1       6.0
> **     **23.1               4.2

> **📝 Note**
>
> **What is customer unchecks option to upgrade Embedded MongoDB?**
> 
> In Mongo upgrades prior to 7.0, the customer was provided the OPTION to upgrade the Embedded MongoDB version.  Since this is a required step, unchecking the MongoDB upgrade leads to the Service being set to Manually run and refusing to start.
> 
> See How to upgrade Embedded MongoDB version manually