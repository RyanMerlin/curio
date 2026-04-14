---
id: 827c78a1e03b548e
title: _DRAFT for Server Upgrade Guide in Community
status: review
source:
  kind: confluence_page
  id: confluence-page:3589931151
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/3589931151
  summary: null
category:
- product-tree
- alteryx-server
- upgrade
keywords:
- upgrade
- draft
- community-guide
- how-to
created_at: 2026-04-14T15:18:28Z
updated_at: 2026-04-14T15:19:36Z
confidence: 0.6
cross_refs: []
content_hash: sha256:278ea820c85c8803529298f53ce9bf110d93a483ff7ec514d3a8a9eeb66208d3
confluence_page_id: null
model_used: claude-sonnet-4-6
---

> **ℹ️ Info**
>
> This page is a DRAFT for a Community Guide page in the same form as [SharePoint & Alteryx Guide](https://community.alteryx.com/t5/Alteryx-Community-Resources/SharePoint-amp-Alteryx-Guide/ta-p/1371125) (Comm) and the other “Guides” in Community/.

---

---

# Foundational Articles that Span the Upgrade Process

- https://help.alteryx.com/current/en/server/best-practices/server-upgrade-best-practices.html – narrative overview of upgrade best practces
- https://help.alteryx.com/current/en/server/install/install-or-upgrade-server/server-upgrade-checklist.html  – condenses the upgrade into check-list form with links to Help and KBs
- https://help.alteryx.com/current/en/server/install/install-or-upgrade-server/version-to-version-server-upgrade-guide-supported-versions.html
- Alteryx Server: Pre-Upgrade Checks (KB)  <== is this still relevant since we stopped adding to it?
- https://help.alteryx.com/current/en/server/install/install-or-upgrade-server.html
- Install or upgrade Alteryx Server <== a video!

- https://help.alteryx.com/current/en/server/install/downgrade-alteryx-server.html#downgrade-alteryx-server-7047050 and How To: Downgrade Alteryx Server (KB)  <== Should these be in the Plan or Prep sections?  And should we update it to mention IT Server snapshots?  With our in-place Mongo upgrade to 7.0 we make rollback a significant challenge.

# Plan

Narratiive

- Plan for sucess as well as the need to rollback if an in-place upgrade gets stuck.
- If you do rollback, get a free temporary Sandbox license from your AE and test the upgrade on the Sandbox to work through issues

<https://help.alteryx.com/current/en/server/install/install-or-upgrade-server/server-upgrade-checklist.html#server-upgrade-process> > **Plan**

<https://help.alteryx.com/current/en/server/install/install-or-upgrade-server/version-to-version-server-upgrade-guide-supported-versions.html>

What version can upgrade to what?

- Server Upgrade Version Paths - What version can upgrade to what versions?
- Internal document, but we need to help customers understand what’s a legal upgrade for Embedded Mongo.  Or just tell them not to jump more than one MongoDB upgrade version and refer them to https://help.alteryx.com/current/en/server/configure/database-management/mongodb-management/mongodb-schema-reference.html

22.3 = CryptoMigation

- For 22.3, plan for CryptoMigration.  Pit-stop at or before 24.2 (as this is the last version the migration tools are available in), seehttps://help.alteryx.com/20242/en/server/install/install-or-upgrade-server/migration-prep-tool.html.  Deal with all errors before attempting upgrade as the error will block upgrade.

# Prep

<https://help.alteryx.com/current/en/server/best-practices/server-upgrade-best-practices.html#1--perform-a-backup> - Backup for safe rollback procedures

[Community Utility - Environment Info Tool to Assist Upgrades +](https://alteryx.atlassian.net/wiki/search?text=Community+Utility+-+Environment+Info+Tool+to+Assist+Upgrades++)  <== I haven’t used this, but it looks very cool.

[Server Upgrade Issues-by-Version](https://alteryx.atlassian.net/wiki/search?text=Server+Upgrade+Issues-by-Version)

- <== are there some of the “Prevent” items we can pull from here to have admins.  ex: Uninstall Copilot trial before upgrading Designer to 25.1, Server Upgrade Issues-by-Version

# Upgrade

Moving to a new Machine

- https://help.alteryx.com/current/en/server/install/server-host-recovery-guide.html

Moving to Cloud Server?

- Best Practices for Deploying Alteryx Server on AWS (PDF) <== Tim R link
- Alteryx Server on Azure (KB)
- Alteryx Server on Azure (whitepaper)

Command line?

- https://help.alteryx.com/current/en/license-and-activate/license-and-activate-with-license-keys/install/use-command-line-options.html#use-command-line-options-7112666 (Help)

Silent upgrades

- Alteryx Server Silent Patch Upgrade (Comm)
- Alteryx Server Silent Upgrade (Comm)
- Alteryx Server Silent Uninstallation (Comm)

# Test

If issues are encountered, the Alteryx Server may need to be rolled back. See the <https://help.alteryx.com/current/en/server/install/downgrade-alteryx-server.html#downgrade-alteryx-server-7047050> Help page for more information.

What to provide Support if opening a case:

- Upgrade from/to versions include the full version number
- Screenshot the issue
- Explain the scope – Server not starting, issue affects all users, issue affects one/some users
- C:\ProgramData\Alteryx\RuntimeSettings.xml from Controller, Gallery, Worker adding the node type after the RuntimeSettings.xml
- Logs and where to find them that include the most recent attempt to start ServerService logService Schema MigrationLast Startup Error fileGalleryGallery Schema MigrationEmbedded Mongo version upgrade (xxx_PreUpgrade\migration.log)C:\ProgramData\Alteryx\RuntimeSettings.xmlDescribe architecture / environmentNodesAuth methodPersistence (Embedded, User-Managed, SQL DB)
   - Service log
   - Service Schema Migration
   - Last Startup Error file
   - Gallery
   - Gallery Schema Migration
   - Embedded Mongo version upgrade (xxx_PreUpgrade\migration.log)
   - C:\ProgramData\Alteryx\RuntimeSettings.xml
   - Describe architecture / environmentNodesAuth methodPersistence (Embedded, User-Managed, SQL DB)
      - Nodes
      - Auth method
      - Persistence (Embedded, User-Managed, SQL DB)