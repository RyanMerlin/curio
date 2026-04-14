---
id: 5daa2ec356c501fa
title: OAuth1 vs OAuth2
status: intake
source:
  kind: confluence_page
  id: confluence-page:1640792985
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/1640792985
  summary: null
category: []
keywords: []
created_at: 2026-04-14T15:02:18Z
updated_at: 2026-04-14T15:02:18Z
confidence: null
cross_refs: []
content_hash: sha256:4591f21c36986303ed5c5dba3e27a01ad95f980dead77c0d71868be33b1f9b6a
confluence_page_id: null
model_used: null
---

> **ℹ️ Info**
>
> Details on the Server API’s transition from OAuth1 to OAuth2

| Server | OAuth1 | OAuth2 | n/a |
| --- | --- | --- | --- |
| 22.1+ | noRed | yesGreen |  |
| 21.4crossover edition | yesGreen | yesGreen |  |
| 21.3and prior | yesGreen | noRed |  |

# Links

| Call Server API from Workflow | Server API Tool + How to use the V3 API Pack |
| --- | --- |
| Difference | In OAuth1 customers canMake an API call by passing their key and secret in the API callIn OAuth2 they need to FIRST, Make a call to /webapi/oauth2/token with thier Key and Secret.  This returns a Bearer Token that expirs in ~1hr. SECOND, Make the API call passing the Bearer Token.The above is standard for OAuth2, it’s not specific to our API.  Therefore, they can find many examples online specific to the language they're developing in.  Helping people construct their OAuth2 code is out of scope. |
| Tutorials | https://www.oauth.com/oauth2-servers/differences-between-oauth-1-2/ https://www.thedataschool.com.au/shiva-ravi/connecting-to-api-using-oauth-2-from-alteryx/         <==  [Tim R] “the data school link does talk about OAuth 2.0 but doesn't talk                    about interfacing with our Gallery specifically, which I believe is a bit                    different implementation of OAuth 2.0” |
| Help | https://help.alteryx.com/current/en/developer-help/apis/server-api-overview.html https://help.alteryx.com/current/server/oauth1-oauth2-instructions |