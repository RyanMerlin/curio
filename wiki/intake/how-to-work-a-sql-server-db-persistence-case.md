---
id: e5b2bcb284ca4c91
title: How to Work a SQL Server DB Persistence Case
status: intake
source:
  kind: confluence_page
  id: confluence-page:2469298674
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/2469298674
  summary: null
category: []
keywords: []
created_at: 2026-04-14T15:02:19Z
updated_at: 2026-04-14T15:02:19Z
confidence: null
cross_refs: []
content_hash: sha256:c4f333f7c3abe75ab65ffb8519492b907273c1a6ca693d32ff82f67d716dcb55
confluence_page_id: null
model_used: null
---

| Initial Questions |  |
| --- | --- |
| Alteryx Server version | Go to the Server Designer and get a screenshot of the Designer version (Help > About) |
| SQL Server version | Get from the SQL Server Database Admin |
| SQL Server Driver version | A screenshot of ODBC Data Sources 64-bit app > DriversNote: The SQL persistence connection will only work with 'ODBC Driver 17' and will fail with driver version 18 |
| What is the error in Alteryx System Settings, if applicable? | A screenshot of error in Alteryx System Settings |
| Get Alteryx Server logs | latest Service log file should show the latest Alteryx Service start up errorAlteryxService and AlteryxGallery schema migration logs (alteryx-migration.csv) from Service and Gallery log directories: https://help.alteryx.com/current/en/server/configure/configure-and-use-server-logs.html |
| What type of authentication are they using for SQL Server? | The connection strings are different for SQL Server authentication versus Kerberos or Windows authentication |
| Is TLS/SSL enabled on SQL Server? | The connection strings are different depending on whether TLS/SSL is enabled |

| Key Links |  |
| --- | --- |
| Key Articles | https://help.alteryx.com/current/en/server/configure/database-management/sql-db-management/configure-sql-server.html#configure-sql-server https://help.alteryx.com/current/en/server/configure/database-management/sql-db-management/sql-db-advanced-connection-strings.html |

| Check | Steps |
| --- | --- |
| Do we support the Driver and Version? | Confirm they are using a supported version SQL Server and the corresponding driver.As of 9/9/2024, Alteryx Server supports MSSQL Server 2019 and 2022: https://help.alteryx.com/current/en/server/configure/database-management/sql-db-management/configure-sql-server.html#configure-sql-server Supported SQL Server database drivers: https://help.alteryx.com/current/en/designer/data-sources/microsoft-sql-server-2012,-2014,-2016,-and-2019.html If they are not using the support version, have them install the correct version and perform a small test. |
| Do we support the SQL Server authentication type? | As of 9/9/2024, Alteryx Server supports the following authentication methods:SQL Server authenticationWindows or Kerberos authentication |
| Controller and Server UI Persistence Strings | The connection strings should match the expected format in the Help documentation: https://help.alteryx.com/current/en/server/configure/database-management/sql-db-management/sql-db-advanced-connection-strings.html#sql-db-advanced-connection-strings |
| SQL Server host’s Fully Qualified Domain Name (FQDN) in connection strings | The host FQDN should resolve to the correct IP address. Run a DNS resolution check: https://knowledge.alteryx.com/index/s/article/How-to-troubleshoot-DNS-issues-1583461654658 |
| Connection to SQL Server from Alteryx Server host machine, outside of Alteryx Server | From the Alteryx Server host machine:Can they connect to SQL Server using SQL Server Management Studio (SSMS)?Create an ODBC DSN for SQL Server using the same ODBC Driver and test the connection in the DSN setup window |
| If TLS/SSL is enabled on SQL Server, is the certificate valid and trusted by Alteryx Server? | The certificate should be valid and may have to be added to the Windows certificate stores on the Alteryx Server host machine if it is not issued by a recognized Certificate Authority |
| Did the AlteryxService and AlteryxGallery schemas create successfully? | Check the AlteryxService and AlteryxGallery schema migration logs (alteryx-migration.csv) for schema errors |
| Does the database access user have permission to read from, write to, and create the AlteryxGallery and AlteryxService databases and tables? | The database access user applied in the connection strings will need the CREATE TABLE permission to create the Alteryx Server schema tables in the SQL Server databases when starting Alteryx Server for the first time. Specifically this requires SA user level full control, so that ALTER DATABASE can be run after creating the tables. After the databases have been successfully created, SA access is no longer needed. |
| Are the AlteryxGallery and AlteryxService databases dedicated to Alteryx Server? | The databases should not contain any other tables not used by Alteryx Server |
| Performance issues? | How far is the Gallery from the SQL Server?  Run a pathping command from the Gallery node to see how far away the SQL Server is:pathping /n SQL_SERVER_HOST |