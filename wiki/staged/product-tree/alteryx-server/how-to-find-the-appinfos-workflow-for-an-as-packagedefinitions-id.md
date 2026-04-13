---
id: aeb68099f66cc0e8
title: How to find the appinfos Workflow for an AS_PackageDefinitions._id
status: published
source:
  kind: confluence_page
  id: confluence-page:1945043613
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/1945043613
  summary: null
category:
- product-tree
- alteryx-server
keywords:
- find
- appinfos
- workflow
- record
- aspackagedefinitionsid
created_at: 2026-04-13T01:55:58Z
updated_at: 2026-04-13T01:58:55Z
confidence: 0.55
cross_refs: []
content_hash: sha256:7c9bcc11c07c33d0efece934d2f97c4d245c7d2eb9da4eefd2fcb3dc67134119
confluence_page_id: null
model_used: heuristic
---

> **ℹ️ Info**
>
> Several CrypoMigration Errors refer to a bad AS_PackageDefinitions._id and this page shows how to find the related appinfos record if the Customer is reluctant to delete the AS_PackageDefinitions record without knowing what appinfo Workflow its for

Akash Parida found that the **AlteryxGallery.appinfos.Revisions.RevisionId** is the **AlteryxService.AS_PackageDefinitions._id**.

Query **appinfos **for a specific **AS_PackageDefinitions._id**

db.getCollection('appInfos').find({"Revisions.RevisionId" : "AS_PACKAGE_DEFINITIONS_ID "})
Query to find appInfos records for multiple **AS_PackageDefinitions._id**’s at once

db.getCollection('appInfos').find({ "Revisions.RevisionId": { $in: [
       "AS_PACKAGE_DEFINITIONS_ID_1 ", 
       "AS_PACKAGE_DEFINITIONS_ID_2 ",
       "AS_PACKAGE_DEFINITIONS_ID_N "
     ] } } )
This relationship appears in the Mongo Entity-Relationship Diagram / ERD

---

# The OLD and HARD way to find the workflow is below.  Use the above instead!

| 1 | Find the workflow for this AS_PackageDefinitions._id (Hard way) | The above solution is the best.  Below steps can provide more visibility into the AS_PackageDefintions record as it will unpack the ServiceData blob.  But customers are really just interested in know which Worfklow is impacted.(1) Use the service parse macro to extract the information in AS_PackageDefinitions records so you can view the PrimaryFile name, which you can use to find the associated in appinfos record.(2) Delete the AlteryxService.AS_PackageDefinitions records identified in the CryptoMigration log file from MongoDB.(3) Find and the app in gallery UI and delete the app.(4) Deleting the app in Step 3 will create another record under AS_PackageDefinitions mentioning the package is deleted but still the crypto migration tool failed. Proceed with Steps 1 and 2 to fix the issueThe following workflow will unpack AS_PackageDefitiions so you can view the workflow file name.  The following query will find the appInfos record for that [The below may have been superseeded with the ability to directly query appinfos based on the AS_PackageDefinitions._id][Tom D] I have verified that this technique works. I did it a bit differently since the account was a bit hard to technically work with, I got the app package ID’s, got the workflows deleted (which increased the number of problem app packages) , then used Robo3T to delete all of the app packages in question. I have attached an instruction document to the customer which can be used as a starting point for others. |
| --- | --- | --- |

---

Workflow to explore AS_PackageDefinitions records that shows the workflow file name so you can find it in appinfos.

- The workflow and macro below will unpack the ServiceData blob and write the data to a JSON files that’s easy to search.
- You can search for the AS_PackageDefinitions._id from the error to see the Package PrimaryFile, which can help find the appinfos recrod.

If you’re curious

- The process to get from an appinfo record to the AS_PackageDefinitions record isn’t straightforward but you can find it here Mongo Entity-Relationship Diagram / ERD
- The process to get from AS_PackageDefinitions to appinfo is unknown based on Matt H and Mittesh