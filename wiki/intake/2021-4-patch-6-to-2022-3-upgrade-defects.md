---
id: 0d51911641ce3fc2
title: 2021.4 Patch 6 to 2022.3 Upgrade Defects
status: intake
source:
  kind: confluence_page
  id: confluence-page:1868234983
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/1868234983
  summary: null
category: []
keywords: []
created_at: 2026-04-14T15:09:17Z
updated_at: 2026-04-14T15:09:17Z
confidence: null
cross_refs: []
content_hash: sha256:1cf1e487df05472765c0f5a716474a3ac1f74fd0417df7649523b34cbcd3d8d8
confluence_page_id: null
model_used: null
---

TCPE-70277dcf2c9-72f3-3ff6-8e99-fe88e9f473f1System JIRA (TCPE-702)

**Affected Versions**: 2021.4 Patch 6 to 2022.1 Patch 2

**Symptoms**: Post server host recovery, accessing any data connections will result in the page with a spinning icon.

**Pitfall**: Post server host recovery, if an upgrade is done to 2022.1 or 2022.3 (2023.1 not tested), the entire Gallery will be inaccessible.  You do not want to be asking users to upgrade.  They need to revert back to the pre server host recovery Alteryx Server.

TCPE-74177dcf2c9-72f3-3ff6-8e99-fe88e9f473f1System JIRA (TCPE-741)

# DCM

- TGAL-725677dcf2c9-72f3-3ff6-8e99-fe88e9f473f1System JIRA (TGAL-7256)TGAL-744677dcf2c9-72f3-3ff6-8e99-fe88e9f473f1System JIRA (TGAL-7446)TGAL-744777dcf2c9-72f3-3ff6-8e99-fe88e9f473f1System JIRA (TGAL-7447)
   - TGAL-744677dcf2c9-72f3-3ff6-8e99-fe88e9f473f1System JIRA (TGAL-7446)
   - TGAL-744777dcf2c9-72f3-3ff6-8e99-fe88e9f473f1System JIRA (TGAL-7447)