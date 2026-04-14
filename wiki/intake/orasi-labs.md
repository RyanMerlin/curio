---
id: f9596bb1f4df5326
title: Orasi Labs
status: intake
source:
  kind: confluence_page
  id: confluence-page:2506621562
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/2506621562
  summary: null
category: []
keywords: []
created_at: 2026-04-14T15:06:10Z
updated_at: 2026-04-14T15:06:10Z
confidence: null
cross_refs: []
content_hash: sha256:6445ee79f5f26fd6caae48dd9ae3f9b992a476b3344c4aaadd8346ba4d0fe8a9
confluence_page_id: null
model_used: null
---

> **📝 Note**
>
> **OrasiLabs has been shutdown and the platform is no longer available. As of 10/06/2025 there will not be a replacement. **

> **ℹ️ Info**
>
> Orasi Labs allow you to start with a Server that needs some action or is in a failed state and needs troubleshooting.  Each lab has a **ReadMe.txt** on the desktop to help you get started.

|  |  |
| --- | --- |
|  |  |

---

# Labs

---

# How to

## Copy and Paste

Ctrl-C and Ctrl-V don’t work. Right-click and choose Copy and Paste instead.

To paste from your machine into the lab, select **Options** on the right of the lab and paste into the **Clipboard**. .The test will then be available to Ctrl-V in Orasi Labs.

Alternatively, log into your email account and email yourself.

---

# Lab design ideas

- ReadMe.txt on the desktop with the instructions/goal of the lab and link to Confluence support page for the lab.  For more complex labs it can be what the customer would say when they opened the case.  This makes all labs self-explanatory.  For the latter, it can include the questions we'd ask followed by their answers.  Ex:Q: Was this an in-place upgrade or did you restore the DB from another machine?   A: This is a Sandbox and we restored the DB from our Prod   Q: Did you follow SHRG?   A: Customer shrugs, "I don't know"For a failed SHRG upgrade we can have a folder with the necessary info from the source Server (except for the pita Envryttion Key transfer, but that doesn't block the upgrade, so they can skip it for the exercise).
- Public workflow so we see something that we can run successfully after the rollback to get confirmation all is well.  It can be a short MadLibs App with 2-3 questions.
- Default File Explorer to show file extensions (it's already set to show hidden files).  This is a pet peeve of mine and I make customers and GDC check it.
- Pre Install Notepad++ to make it easier for ppl to look at the ASMongoDBVersion.bin files.
- Accompanying Confluence pageHints section with each step hidden in an /expand section.  This ensures ppl don't get stuck.Explore section encouraging them to look at, ex:ASMongoDBVersion.bin to see how to confirm what's in a folder with a link to the Mongo Schema page so they can identify which Mongo folder goes with which version.The two RTS files to see differences inEncrypted keysThe new <MiigrationVersioinNumber> in RuntimeSettings_22_2_migration.xmlErrors section that shows errors that can be encountered when a step is missed.  Ex:  If they don't flip the RuntimeSettings.xml files or the MongoDB folder.  Even if they don't encounter the error it will prepare them for the customer missing a step (which can happen if the customer started the rollback and opened the case when it failed).
   - Hints section with each step hidden in an /expand section.  This ensures ppl don't get stuck.
   - Explore section encouraging them to look at, ex:ASMongoDBVersion.bin to see how to confirm what's in a folder with a link to the Mongo Schema page so they can identify which Mongo folder goes with which version.The two RTS files to see differences inEncrypted keysThe new <MiigrationVersioinNumber> in RuntimeSettings_22_2_migration.xml
      - ASMongoDBVersion.bin to see how to confirm what's in a folder with a link to the Mongo Schema page so they can identify which Mongo folder goes with which version.
      - The two RTS files to see differences inEncrypted keysThe new <MiigrationVersioinNumber> in RuntimeSettings_22_2_migration.xml
         - Encrypted keys
         - The new <MiigrationVersioinNumber> in RuntimeSettings_22_2_migration.xml

   - Errors section that shows errors that can be encountered when a step is missed.  Ex:  If they don't flip the RuntimeSettings.xml files or the MongoDB folder.  Even if they don't encounter the error it will prepare them for the customer missing a step (which can happen if the customer started the rollback and opened the case when it failed).