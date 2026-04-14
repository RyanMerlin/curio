---
id: 136765445eecf31c
title: How To (SQL DB Persistence)
status: intake
source:
  kind: confluence_page
  id: confluence-page:2345107478
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/2345107478
  summary: null
category: []
keywords: []
created_at: 2026-04-14T15:02:19Z
updated_at: 2026-04-14T15:02:19Z
confidence: null
cross_refs: []
content_hash: sha256:b7433dd6a8c8dc89df81dc3319f99da1c74a8824da274aa6ebb41104d35434f0
confluence_page_id: null
model_used: null
---

---

---

|  |  |
| --- | --- |
| View SQL DB database | SQL Server Management Studio (SSMS) |
| Get copy of customer's database | Using SQL Server Management Studio, they backup the database and you restore it.  Then set you APOD to the restored databases.  Collation should transfer.ChatGPT prompts:What are the steps for someone to export their SQL Server database and the steps for me to import it into my SQL Server using SQL Server Management Studio?…Does the collation transfer in option A?Backup (from ChatGPT)Open SQL Server Management Studio (SSMS) and connect to your SQL Server.In Object Explorer: right-click the database > Tasks > Back Up…Backup type: FullDestination: click Remove any defaults you don’t want, then Add… and choose a path + filename like alteryx_server_2025-12-16.bakBack up to:  Disk will back up on the SQL Server machine.  Ensure you can access the file and you don’t fill a drive on that machine(Optional but good) Click Options:Check Verify backup when finishedClick OK to run the backup.Send you the .bak file (plus any password needed to access the transfer location—no SSMS password is embedded in the .bak).Restore (from ChatGPT)Problem:  Not sure how to get the BAK onto the EC2 SQL Server VM nor reference the file on my APOD in SSMS.  The notation I can use to access a regular APOD from another machine doesn’t work for the SQL Server EC2 VM:  \\pod-8476841\C$\sqlBkpTried sharing the folder and granting Everyone Read/Write access, but SSMS said it could not find the folder.  The EC2 SQL Server user is localhost, which seems not to be included in “Everyone”.Michael Phipps suggests trying to use the Windows SQL APOD rather than the EC2.  But I get a bit stuck b/c I don’t know the sa user password and the remoteuser I login to the Windows SQL Server instance isnt an admin, so can’t reset the sa password.Copy the .bak file onto your SQL Server machine (or somewhere the SQL Server service account can read).In SSMS: connect to your SQL Server.Right-click Databases → Restore Database…Choose Device → click … → Add → select the .bak file → OKPick the database name (SSMS usually fills it in).Click Files (or Options, depending on SSMS version) and ensure the data/log file paths are valid on your server.If paths don’t exist (common when restoring from another server), use Relocate all files to folder (if shown) or manually set new paths.Click Options:If you’re overwriting an existing DB with the same name, check Overwrite the existing database (WITH REPLACE) (only if you mean it).Click OK to restore.After restore (often required):Fix orphaned users (if logins don’t match on your server). Common quick fix:Create/login-map as needed, or use ALTER USER ... WITH LOGIN = ...Confirm DB owner (optional): ALTER AUTHORIZATION ON DATABASE::YourDb TO sa;SQL Server APOD doesn’t seem to be able to restore databasesPer TimR, need to install SQL Server yourself on an APOD to have the level of control needed restore databasesSlack APOD channel query w/o resolution https://alteryxbroomfield.slack.com/archives/C016T32JZE0/p1765916127127059 Trying a Windows SQL Server APODNeed to reset the SA password - FAILED, no error but the password is not getting updated.  net stop MSSQLSERVERnet start MSSQLSERVER /m"SQLCMD"sqlcmd -E -S localhost1> ALTER LOGIN sa ENABLE;2> GO2> ALTER LOGIN sa WITH PASSWORD = 'Nottingham1!';3> GO1> Ctrl-C to leave SQLCMDnet stop MSSQLSERVERnet start MSSQLSERVER |
| Create a table under the alteryx_server or alteryx_service schema | The alteryx_server and alteryx_service are “schemas”.  One way to create a table under the schema is toClick the parent databaseClick New QueryEnter a query to create a table.  Enter alteryx_server or alteryx_service, depending on the database schema#E3FCEFCREATE TABLE alteryx_server.MY_TABLE (    OrderID INT IDENTITY(1,1) PRIMARY KEY,    OrderDate DATETIME NOT NULL,    CustomerID INT NOT NULL); |
| Drop all tables | Option - Ask DBA to create new DatabasesOption - Script to delete tableshttps://stackoverflow.com/questions/8439650/how-to-drop-all-tables-in-a-sql-server-database [Michael P] In SSMS:Right click the databaseGo to "Tasks"Click "Generate Scripts"In the "Choose Objects" section, select "Script entire database and all database objects"In the "Set Scripting Options" section, click the "Advanced" buttonOn "Script DROP and CREATE" switch "Script CREATE" to "Script DROP" and press OKThen, either save to file, clipboard, or new query window.Run script.We may need to delete also Views, SPs.https://www.scholarhat.com/tutorial/sqlserver/drop-all-tables-stored-procedure-views-and-triggers#:~:text=How%20to%20delete%20all%20tables,%27 |
| Change the SA password | https://www.cocosenor.com/articles/office/how-to-change-sql-sa-password-from-a-command-prompt.html |
| Change the dbadmin password on an APOD | ALTER LOGIN dbadmin WITH PASSWORD = 'NEW_PASSWORD' OLD_PASSWORD = 'OLD_PASSWORD';https://learn.microsoft.com/en-us/sql/t-sql/statements/alter-login-transact-sql?view=sql-server-ver16 |