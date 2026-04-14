---
id: 739baffaa1e3989a
title: Overview SSL/TLS
status: review
source:
  kind: confluence_page
  id: confluence-page:1739490453
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/1739490453
  summary: null
category:
- product-tree
- alteryx-server
- authentication
keywords:
- ssl
- tls
- https
- overview
- certificates
created_at: 2026-04-14T13:40:14Z
updated_at: 2026-04-14T13:50:42Z
confidence: 0.83
cross_refs: []
content_hash: sha256:d82864e0470a0fee39eeb4104971273e551e344eca7941a98974f47d980bc89e
confluence_page_id: null
model_used: claude-sonnet-4-6
---

note The following is from the Troubleshooting Tools Lessonly

The following is from the Troubleshooting Tools Lessonly

## SSL/TLS Handshake 

When a connection is using SSL/TLS (connections over HTTPS), an additional SSL Handshake must take place. The precursor to SSL/TLS handshake is the TCP handshake that you just learned about.

An SSL/TLS handshake takes place only after a TCP connection has been opened.

The SSL handshake determines what version of SSL/TLS will be used in the session, which cipher suite will encrypt communication, verify the server (and sometimes also the client), and establishes that a secure connection is in place before transferring data.

### How is this relevant to tracing network traffic? 

Let’s apply this knowledge to our products. The video below covers an example of making an API call using the Download tool and viewing the request in Fiddler versus Wireshark versus the DevTools. This will help you to understand the difference between them, specifically focusing on the difference between Wireshark and Fiddler.

This video will also give you a deeper understanding of the Download tool in Alteryx Designer.  [awesome video, but it’s locked inside Lessonly]