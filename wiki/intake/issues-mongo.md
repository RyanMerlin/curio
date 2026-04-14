---
id: b36853e4244a05ce
title: Issues (Mongo)
status: intake
source:
  kind: confluence_page
  id: confluence-page:1730642918
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/1730642918
  summary: null
category: []
keywords: []
created_at: 2026-04-14T15:10:53Z
updated_at: 2026-04-14T15:10:53Z
confidence: null
cross_refs: []
content_hash: sha256:12862795b88a08c95ca1f54d95e9d1f6ee184051b8642362943acd6f7dd53635
confluence_page_id: null
model_used: null
---

# Issues

---

**Dec-2025 – Vulnerability** [CVE-2025-14847](https://nvd.nist.gov/vuln/detail/CVE-2025-14847) in Mongo DB

- [Tim R Jan-23-2026]TeamsJust to close this out, we have tentative (subject to change as with any release dates) release patches/dates established for this vulnerability to be mitigated with the updated mongo dependencies in our installers, 2025.2 patch 1 was already released yesterday with the fix. The rest of the patches and dates are as follows:2025.1 Patch 4 GA – targeted for 1/292024.2 Patch 10 GA – targeted for 2/52024.1 Patch 13 GA – targeted for 2/12
   - Teams
   - Just to close this out, we have tentative (subject to change as with any release dates) release patches/dates established for this vulnerability to be mitigated with the updated mongo dependencies in our installers, 2025.2 patch 1 was already released yesterday with the fix. The rest of the patches and dates are as follows:2025.1 Patch 4 GA – targeted for 1/292024.2 Patch 10 GA – targeted for 2/52024.1 Patch 13 GA – targeted for 2/12
      - 2025.1 Patch 4 GA – targeted for 1/29
      - 2024.2 Patch 10 GA – targeted for 2/5
      - 2024.1 Patch 13 GA – targeted for 2/12

- [Tim R Jan-05-2026]For CVE-2025-14847, confirmed with engineering that we do not utilize network compression with our mongo client anyways and added this note to the doc just fyi:
   - For CVE-2025-14847, confirmed with engineering that we do not utilize network compression with our mongo client anyways and added this note to the doc just fyi:

- User-Managed Mongo can apply patch provided by Mongohttps://www.mongodb.com/company/blog/news/mongodb-server-security-update-december-2025
   - https://www.mongodb.com/company/blog/news/mongodb-server-security-update-december-2025

- Embedded Mongo, seeMongoDB zlib Compression Vulnerability (CVE-2025-14847): Mitigation and Validation Steps for Alteryx Deployments (KB)Per Stephen R in Teams, no need to remove the RuntimeSettings.xml change after applying patch that Followed by Tim's Teams comment that I can't understand [Ed P]Mongo has a priority order when deciding which compression library to use and zlib is least in priority.It's recommended to have all compression libraries enabled for clustered environments that are not embedded after patching. But embedded should not need it. [TGAL-14280] CVE-2025-14847 - JiraStephen Ruhl: MongoBleed - CVE-2025-14847 | GRP_Customer Support > URGENT | Microsoft Teams
   - MongoDB zlib Compression Vulnerability (CVE-2025-14847): Mitigation and Validation Steps for Alteryx Deployments (KB)Per Stephen R in Teams, no need to remove the RuntimeSettings.xml change after applying patch that Followed by Tim's Teams comment that I can't understand [Ed P]Mongo has a priority order when deciding which compression library to use and zlib is least in priority.It's recommended to have all compression libraries enabled for clustered environments that are not embedded after patching. But embedded should not need it.
      - Per Stephen R in Teams, no need to remove the RuntimeSettings.xml change after applying patch that
      - Followed by Tim's Teams comment that I can't understand [Ed P]Mongo has a priority order when deciding which compression library to use and zlib is least in priority.It's recommended to have all compression libraries enabled for clustered environments that are not embedded after patching. But embedded should not need it.
         - Mongo has a priority order when deciding which compression library to use and zlib is least in priority.It's recommended to have all compression libraries enabled for clustered environments that are not embedded after patching. But embedded should not need it.

   - [TGAL-14280] CVE-2025-14847 - Jira
   - Stephen Ruhl: MongoBleed - CVE-2025-14847 | GRP_Customer Support > URGENT | Microsoft Teams