---
id: 3af7e3bf114bd9cd
title: How to Work an API Case
status: intake
source:
  kind: confluence_page
  id: confluence-page:1766166726
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/1766166726
  summary: null
category: []
keywords: []
created_at: 2026-04-14T15:16:58Z
updated_at: 2026-04-14T15:16:58Z
confidence: null
cross_refs: []
content_hash: sha256:a683a62d086fc0ee5f0a9c3d3ae8db24b5583da1abb1fdd13efb32bd307d6e81
confluence_page_id: null
model_used: null
---

| **Initial Questions** |  |
| --- | --- |
| **What’s your Server version?** | **Server UI > Avatar > Profile > Versions > Server** |
| **What Role is the user making the call?** | Many calls behave differently for Curators vs Artisans. |
| **Have you tested the call in Swagger?** | We provide limited support after proving the call works in Swagger from a user’s machine.  We don’t provide assistance for getting OAuth2 working in their script. |
| **Does the call work for a Colleague or when made on the Server?** | This can indicate proxy or firewall issues limiting the particular user or user’s machine. |
| **Have you tested the call in Postman or cURL?** | They should be able to make the call from Postman or cURL. |
| **Where are you making the call from?** | This is more informational.  If they are using the V3 API Pack, you can test that on your APOD.  Get API pack in [Introducing the Alteryx Server v3 API](https://community.alteryx.com/t5/Engine-Works/Introducing-the-Alteryx-Server-v3-API/ba-p/899228) (899228). |

| **Testing Tools** |  |
| --- | --- |
| **Swagger** | Access the API with Swagger  Swagger can determine if the call is working as expected and/or differently for Curator vs Artisan.  Since the API documentation is light, testing in Swagger will determine how an API works. |
| httpbin.org | Quickly shows what the API call is passing.  This can be used to compare a working Postman or cURL call to a failing call from their script or the Download Tool.  **To use **– replace the endpoint being called with <http://httpbin.org/anything>     - This will return JSON showing everything the call sent.  Postman and cURL add Headers to the call that may be required but not being passed by the script or Download Tool.  Many settings are case-sensitive. |
| Fiddler | Since the calls to [httpbin.org](http://httpbin.org) don’t include the our API’s OAuth2 Bearer Token and the subsequent authentication performed by our API, you can use Fiddler to compare working calls from Postman or cURL to the failing call from their script of the Download Tool.  If Fiddler fixes the issue, that is a strong indication that their proxy is the problem since Fiddler replaces their proxy. |
| Postman  cURL | 3rd-party API testing tools.  You can ask customer to ensure they can make the call work in Postman or cURL.  If they can’t, they need to work on their call before it will work in their script or the Download Tool. |

| **Check** | **Steps** |
| --- | --- |
| **API’s Confluence page** | Known issues or nuances with specific API endpoints are listed here.  Endpoints  > [V3](https://alteryx.atlassian.net/wiki/spaces/SupportServer/pages/1681363286)  > [V2](https://alteryx.atlassian.net/wiki/spaces/SupportServer/pages/1681396400)  > [V1](https://alteryx.atlassian.net/wiki/spaces/SupportServer/pages/1681461806) |
| **Jira** | Jira cards typically use the API call as listed in Swagger in their title and are easy to find.  Check Jira before going deep into troubleshooting. |
| **Base Address URL** | The **Alteryx System Settings > Galley > General > Base Address** URL should end with **/gallery**.  If it does not, the Web API will need to use a different port and, if SSL is enable, the certificate will need to be bound to that port. |
| **Web API URL** | The **Alteryx System Settings > Galley > General > Web API** URL should be the same FQDN as the Base Address but ending with **/api** instead of** /gallery**.  Confirm it is HTTPS if SSL is enabled. |
| **Do other calls work?** | Test calling a simple endpoint that doesn’t require parameter |
| **Test the call as a Curator vs Artisan** | Setup a same-version APOD and test the call as Curator vs Artisan to determine the expected behavior. |
| **If the call works in Postman or cURL** |  |
| **Test with **[httpbin.org](http://httpbin.org)** and Fiddler** | Use these tools to see what’s different in the calls between Postman or cURL and their script.  [httpbin.org](http://httpbin.org) is easier to start with since it doesn’t require installation.  Fiddler will show the OAuth2 process and will replace their local proxy (so if Fiddler fixes the issue, it’s likely due to their proxy). |
|  |  |