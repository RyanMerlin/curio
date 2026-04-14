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
created_at: 2026-04-14T15:18:28Z
updated_at: 2026-04-14T15:20:38Z
confidence: 0.88
cross_refs: []
content_hash: sha256:c0af747eecd76c3cbd2fd35f7bc491eb56cd4c199ad31144f677adf3e511568c
confluence_page_id: null
model_used: claude-sonnet-4-6
---

| **Issue** | Changing server authentication type to Windows Authentication can cause Gallery menu to disappear or profile page lacks permission etc. after login. |
| --- | --- |
| **Screenshot** |  |
| **Possible Cause** | Existing Gallery user records may contain incomplete identity information after the authentication change. e.g. If the `WindowsIdentity` field in MongoDB is `null`, Gallery access may fail. |
| **Resolution Overview** | Remove the affected user record from MongoDB so it can be automatically re-created on the next login.  Two remediation methods are available:     - Method 1: Robo 3T (GUI)    - Method 2: MongoDB via Command Line (CMD)Requires ability to run Command Prompt as Administrator       - Requires ability to run Command Prompt as Administrator |

# Method 1: Using Robo 3T (Recommended)

|  | **Steps ** | **Detail** |
| --- | --- | --- |
| 1 | Step 1: Configure the Robo 3T Connection | Additionprocedure is described in [How To: Connect to a user-managed MongoDB with Robo 3T](https://knowledge.alteryx.com/index/s/article/How-To-Connect-to-a-user-managed-MongoDB-with-Robo-3T)  **Connection Tab** Name: *[Any, used Alteryx Gallery in this example]* Host: localhost Port: 27018  **Authentication Tab** Check Perform authentication Database: AlteryxGallery Username: user Password: *[Copy the value from Alteryx System Settings → Persistence → Database → Password]* |
| 2 | Step 2: Access the Users Collection | After connecting:     1. Expand AlteryxGallery    2. Expand AlteryxGallery (database)    3. Expand Collections    4. Double-click users  The objects displayed on the right represent user information stored in MongoDB.  Identify the Target User by expanding the user objects and identify the affected account using fields such as: Email Name Within the same object, locate the **WindowsIdentity **field |
| 3 | Step 3: Validate `WindowsIdentity` (Decision Point) | If `WindowsIdentity` is **Null**, this is highly likely to be the cause of the login issue, continue the procedure follows. If not, this article doesn’t match  > **📝 Note** > > **Sid **mismatch could also cause this issue.  > **⚠️ Warning** > > If the description here does not match your situation, this article does **not** apply to your issue. |
| 4 | Step 4: Delete the Invalid User Document | To correct the issue:     1. Right-click the affected user object    2. Select Delete Document |
| 5 | Step 5: After Check | 1. Click the Play (▶) button in the upper-left corner to reload the data    2. Confirm that the deleted user object no longer appears    3. Log in to Alteryx Server (the Gallery menu should now display correctly.)    4. (Optional) If click the Play (▶) button and reload the data again, a new user object will be created again with correct info as shown in Step 3. |

# Method 2: Using Command Line (CMD)

|  | **Steps ** | **Detail** |
| --- | --- | --- |
| 1 | Step 1: Open Command Prompt and Connect to MongoDB | 1. On the Alteryx Server machine, open Command Prompt, right-click → Run as administrator.    2. Change directory to the Alteryx bin directoryFor the default installation path:  (Adjust the path if Alteryx is installed elsewhere.) |
| 2 | Step 2: Retrieve the MongoDB (non-admin) password used by Alteryx Server | After connecting, run:  Example:  Copy the returned password string. You will use this value in the next step. |
| 3 | Step 3: Connect to MongoDB shell (mongosh) | Run:  Replace `passwordstring` with the password from step 2.  Example:  > **ℹ️ Info** > > Please use non-admin password |
| 4 | Step 4: Query the user record (identify the affected account) | Example: search by email (recommended):  If you don’t know the exact email, you can list a subset of fields (useful for searching): |
| 5 | Step 5: Validate `WindowsIdentity` (Decision Point) | In the returned user document, locate the field below, if the value is null, proceed to next step.     - WindowsIdentity  > **📝 Note** > > **Sid **mismatch could also cause this issue.  > **⚠️ Warning** > > If the description here does not match your situation, this article does **not** apply to your issue. |
| 6 | Step 6: Delete the Invalid User Document | Delete the specific user record by email:  Example output: |
| 7 | Step 7: After Check | 1. To confirm deletion, run the command in step 4 again, make sure no user info returns.     1. Log in to Alteryx Server (the Gallery menu should now display correctly.)    2. (Optional) If run the command in step 4 again, a new user object will be created again with correct info as shown in Step 4. |