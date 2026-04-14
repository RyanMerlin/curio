---
id: 7b91168d1dc895c4
title: Server Issue - Windows Authentication Login Issues Following an Authentication Method Change
status: review
source:
  kind: confluence_page
  id: confluence-page:3701637234
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/3701637234
  summary: null
category:
- product-tree
- alteryx-server
- authentication
keywords:
- authentication
- windows-auth
- login-issues
- method-change
- issue
created_at: 2026-04-14T13:40:14Z
updated_at: 2026-04-14T13:50:49Z
confidence: 0.88
cross_refs: []
content_hash: sha256:eb99503a129d355870fb69a420bd93eb7d33ec0f38dde644b4de6eea2586c5a3
confluence_page_id: null
model_used: claude-sonnet-4-6
---

| Issue | Changing server authentication type to Windows Authentication can cause Gallery menu to disappear or profile page lacks permission etc. after login. |
| --- | --- |
| Screenshot |  |
| Possible Cause | Existing Gallery user records may contain incomplete identity information after the authentication change. e.g. If the WindowsIdentity field in MongoDB is null, Gallery access may fail. |
| Resolution Overview | Remove the affected user record from MongoDB so it can be automatically re-created on the next login.Two remediation methods are available:Method 1: Robo 3T (GUI)Method 2: MongoDB via Command Line (CMD)Requires ability to run Command Prompt as Administrator |

# Method 1: Using Robo 3T (Recommended)

|  | Steps | Detail |
| --- | --- | --- |
| 1 | Step 1: Configure the Robo 3T Connection | Additionprocedure is described in How To: Connect to a user-managed MongoDB with Robo 3TConnection TabName: [Any, used Alteryx Gallery in this example]Host: localhostPort: 27018Authentication TabCheck Perform authenticationDatabase: AlteryxGalleryUsername: userPassword: [Copy the value from Alteryx System Settings → Persistence → Database → Password] |
| 2 | Step 2: Access the Users Collection | After connecting:Expand AlteryxGalleryExpand AlteryxGallery (database)Expand CollectionsDouble-click usersThe objects displayed on the right represent user information stored in MongoDB.Identify the Target User by expanding the user objects and identify the affected account using fields such as:EmailNameWithin the same object, locate the WindowsIdentity field |
| 3 | Step 3: Validate WindowsIdentity (Decision Point) | If WindowsIdentity is Null, this is highly likely to be the cause of the login issue, continue the procedure follows. If not, this article doesn’t match Sid mismatch could also cause this issue.Screenshot shows correct statusIf the description here does not match your situation, this article does not apply to your issue. |
| 4 | Step 4: Delete the Invalid User Document | To correct the issue:Right-click the affected user objectSelect Delete Document |
| 5 | Step 5: After Check | Click the Play (▶) button in the upper-left corner to reload the dataConfirm that the deleted user object no longer appearsLog in to Alteryx Server (the Gallery menu should now display correctly.)(Optional) If click the Play (▶) button and reload the data again, a new user object will be created again with correct info as shown in Step 3. |

# Method 2: Using Command Line (CMD)

|  | Steps | Detail |
| --- | --- | --- |
| 1 | Step 1: Open Command Prompt and Connect to MongoDB | On the Alteryx Server machine, open Command Prompt, right-click → Run as administrator.Change directory to the Alteryx bin directoryFor the default installation path:(Adjust the path if Alteryx is installed elsewhere.) |
| 2 | Step 2: Retrieve the MongoDB (non-admin) password used by Alteryx Server | After connecting, run:Example:Copy the returned password string. You will use this value in the next step. |
| 3 | Step 3: Connect to MongoDB shell (mongosh) | Run:Replace passwordstring with the password from step 2.Example:Please use non-admin password |
| 4 | Step 4: Query the user record (identify the affected account) | Example: search by email (recommended):Screenshot shows correct statusIf you don’t know the exact email, you can list a subset of fields (useful for searching): |
| 5 | Step 5: Validate WindowsIdentity (Decision Point) | In the returned user document, locate the field below, if the value is null, proceed to next step.WindowsIdentitySid mismatch could also cause this issue.If the description here does not match your situation, this article does not apply to your issue. |
| 6 | Step 6: Delete the Invalid User Document | Delete the specific user record by email:Example output: |
| 7 | Step 7: After Check | To confirm deletion, run the command in step 4 again, make sure no user info returns. Log in to Alteryx Server (the Gallery menu should now display correctly.)(Optional) If run the command in step 4 again, a new user object will be created again with correct info as shown in Step 4. |