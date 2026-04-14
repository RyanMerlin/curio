---
id: 617fc6591c8d4f03
title: Logs and Traces
status: review
source:
  kind: confluence_page
  id: confluence-page:1709640770
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/1709640770
  summary: null
category:
- product-tree
- alteryx-server
- administration
keywords:
- logs
- traces
- diagnostic
- service-logs
- debugging
created_at: 2026-04-14T13:40:14Z
updated_at: 2026-04-14T13:50:36Z
confidence: 0.88
cross_refs: []
content_hash: sha256:97e986000b3185287853abe4cdb9d67d66c6dcde5bff0c429dc83b7b77bfec00
confluence_page_id: null
model_used: claude-sonnet-4-6
---

> **ℹ️ Info**
>
> This page covers Server logs
> 
> See also: **Support-Desginer > **Logs and Traces

ToolsAlteryx Workbench (SharePoint) <== Tech Support File ViewerHelphttps://help.alteryx.com/current/en/server/configure/configure-and-use-server-logs.html

Log or TraceDescriptionAlteryx System Settings Change LogRuntimeSettingsAuditLog.csvLog changes made by Alteryx System Settings appCryptoMigration Log File AlteryxServiceMigrator_#.logCryptoMigration logs record the MongoDB re-encryption process when upgrading to (or through) 2022.3Engine LogsAlteryx_Log_#######_1.logEngine logs save the data from the workflow Results pane, or what would appear in the Results pane when running the workflow on Server.Gallery Logsalteryx-YYYY-MM-DD.csvGallery logs record Gallery activity.  Timestamps are in UTC.HAR Tracexxx.harA HAR Trace captures the browser’s requests to and responses from the Server UILastStartupError.txtLastStartupError.txt The LastStartupError.txt file records the critical error that occurred when the Service failed to start.  This error will also appear in the Service log along with other messages.Mongo logsmongoDump.logmopngoRestore.logmigration.logMongoDB Backup MongoDB Restore migration.log (embedded Mongo version upgrade) MongoDB Transaction Log RuntimeSettings.xml fileRuntimeSettings.xmlRuntimeSettings.xml contains the Server or Admin Designer’s System Settings.SSO / AAS Logs  (SAML)alteryx-sso-YYYYMMMDD.logaas-log-YYYYMMMDD.logSAML login traceSchema Migration logsalteryx-gallery-migration.csvalteryx-service-migration.csvalteryx-migration.csvThe Schema Migration log records the schema migration portion of a Server upgrade and can confirm that step completed successfullySCIMWe can review SCIM calls to Server in the Gallery logs.  Customers must review the Provisioning Agent logs themselves.Service logs AlteryxServiceLog.logAlteryxServiceLog_YMD_xx.logService Logs capture activity of the Service, communication between Server components, and the startup and shutdown of processes on the Controller and Worker machines.SQL DB Migration Workflowalteryx-migration.csvResults pane of workflowother locations….Mongo to SQL DB migration workflowThe workflow also seems to create a log SQL DB Migration - Migration Workflow - Log Errors SQL DB AlteryxServerMigrator.exeTBDUtility to create SQL DB databases and tables for migrationSystem Information NFO file myLog.nfoThe System Information NFO file provides the hardware and software environment of the machine including cores, RAM, environment variables, etc.Tech Support File Designer 2021.4+ provides an option to collect several log files in a single ZIP file.  In a Server environment, you can request this for the Controller and other nodes.Powershell search logs RECURSIVELYpowershellFilter for specific file extension:powershell

# Public articles for collecting logs

General KB requesting several of the main log files

- Part 1: What does Support need to troubleshoot Server/Gallery/Scheduler? (KB)
- Part 2: What does Support need to troubleshoot Server/Gallery/Scheduler? (KB)
- How To: Attach Server Log Files (KB)  <== this vanished Jun-2025
- Engaging Customer Support: Best Practices  (KB)

KBs that can be sent to request standard information based on area of issue:

- Designer (418700)
- Server/Gallery/Scheduling (418804)
- Promote (418755)
- Connectors (418690)
- Database (418855)
- Allocate/Behavioral Data (419291)
- Connect (418678)
- Designer Cloud (834061)
- Fuzzy Matching (418727)
- Python (418775)
- Engine (418709)
- Macros/Apps (418740)
- Spatial (418813)
- Predictive (418751)
- Reporting/Visualytics (418837)

Older process

- How To: Obtain Logs the Easy Way! (KB)