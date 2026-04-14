---
id: 2653a41c13721587
title: UIPath Partnership
status: intake
source:
  kind: confluence_page
  id: confluence-page:1739851422
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/1739851422
  summary: null
category: []
keywords: []
created_at: 2026-04-14T15:16:58Z
updated_at: 2026-04-14T15:16:58Z
confidence: null
cross_refs: []
content_hash: sha256:6810d90b30646820d755534c3133e9be3e9a4a67b5786ed2ee764a068cf86a87
confluence_page_id: null
model_used: null
---

> **ℹ️ Info**
>
> UIPath has an “Activity Pack” to call Server endpoints from the UIPath product.  This is in addition to the UIPath Tool we provide to call UIPath from a workflow.

# Issue / Resolution

| > **⚠️ Warning** > > Error > Could not deserialize the response body.  **Issue**: After upgrade to 2022.1 the UIPath integrations with Server broke.  Clicking **Test Connection** errors with .  Case 00620132  This does not affect our UIPath Tool, just UIPath’s application that calls our API.  **Resolution**:  The customer needs to update their “Activity Pack” to the post-OAuth2 API Activity Pack.  We changed our **GET /v1/workflows/subscription **endpoint contract in 2021.4 in the move from OAuth1 to OAuth2, specifically     - OAuth1 => "packageType": 1,    - OAuth2 => "packageType": "Module", |
| --- |