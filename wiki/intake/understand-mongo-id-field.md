---
id: d28eb8deb756efad
title: Understand Mongo _id Field
status: intake
source:
  kind: confluence_page
  id: confluence-page:3018981768
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/3018981768
  summary: null
category: []
keywords: []
created_at: 2026-04-14T15:12:39Z
updated_at: 2026-04-14T15:12:39Z
confidence: null
cross_refs: []
content_hash: sha256:48bd750ad5a23020b56807fd143aa84353c2d187983eeab626360cbc03e6a48c
confluence_page_id: null
model_used: null
---

> **ℹ️ Info**
>
> The **_id** is a key field for each collection.
> 
> **_id**’s are globally unique across collections and databases for all practical purposes

---

---

# Universally Unique

Reference

- https://www.mongodb.com/docs/manual/reference/bson-types/#std-label-objectid

The 12-byte ObjectId consists of:

- A 4-byte timestamp, representing the ObjectId's creation, measured in seconds since the Unix epoch.
- A 5-byte random value generated once per client-side process. This random value is unique to the machine and process. If the process restarts or the primary node of the process changes, this value is re-generated.
- A 3-byte incrementing counter per client-side process, initialized to a random value. The counter resets when a process restarts.

---

# Cleanup the _id field

> **ℹ️ Info**
>
> If using the _id field in a workflow you may need to strip it our of the JSON it is often wrapped in

A record’s ID is necessary to Join with records in related collections. However, it will initially appear in a form that prevents using it in a Join:

To clean it up, add a Formula Tool with the expression below.

REGEX_Replace([_id], '^.*:\s\"(.*)\".*', '$1')
Then add a Select Tool to rename the **_id** field to **CollectionName_ID** to make it explicit which collection it is an id for, this ensures you can make sense of what fields you are Joining later in the workflow.