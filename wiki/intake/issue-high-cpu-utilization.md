---
id: 61b26d6a5d2dda30
title: Issue - High CPU Utilization
status: intake
source:
  kind: confluence_page
  id: confluence-page:2983231849
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/2983231849
  summary: null
category: []
keywords: []
created_at: 2026-04-14T15:16:58Z
updated_at: 2026-04-14T15:16:58Z
confidence: null
cross_refs: []
content_hash: sha256:7498314168380c7a8428d2baf3de33a88936c70208266a3a2ae7803cef790c59
confluence_page_id: null
model_used: null
---

| **Issue** | > **📝 Note** > > Customer seeing high CPU utilization (spiking to 90%+) |
| --- | --- |
| **Screenshot** |  |
| **Related Issues** |  |

# Troubleshooting

|  | **Check** | **Steps** |
| --- | --- | --- |
| 1 | **sessions records**  too many sessions records | **Precondition**     - Embedded Mongo is largely contributing to the high CPU utilization, example:  **Cause**     - A large number of sessions records can lead to this issue as Mongo has to search through them for each user interaction with the Server UI.  **Troubleshooting/Resolution**     - Delete old sessions records    - Mongo shows High CPU utilization |
| 2 | **Specific Workflow running**  Some tools can pin CPU | **Cause**     - A particular workflow may be using a tool that can cause high CPU utilizationpython toolpython-based tools (ie, Connectors)large datasets with certain ODBCs (Redshift for example).       - python tool       - python-based tools (ie, Connectors)       - large datasets with certain ODBCs (Redshift for example).   **Troubleshooting**     - A good starting point would be narrowing down the times, and gathering Engine logs from those times to identify the resource-heavy job    - If the customer already noted "during one specific workflow run" getting the workflow name and a copy of it to review what it is doing will be helpful |
| 3 | **RuntimeSettings.xml**  #Simultaneous and Memory Limit set too high | **Cause**     - Customer is exceeding the #Simultaneous or Memory Limit recomednations  **Troubleshooting**     - Validate their RuntimeSettings.xmlRuntimeSettings.xml Validation       - RuntimeSettings.xml Validation   **Resolution**     - Set Server back to recommended #sim and Memory Liit |
| 4 | **Upgrade to 24.2.1.14** | **Cause **     - Defect that leads to slowly increasing CPU utilizationTGAL-1213877dcf2c9-72f3-3ff6-8e99-fe88e9f473f1System Jira       - TGAL-1213877dcf2c9-72f3-3ff6-8e99-fe88e9f473f1System Jira   **Resolution**     - Patch |
| 5 | **Service log** **Gallery Log** | **Cause**     - An overloaded Server can exhibit high Memory utilization and show issues in logs.  In a previous step you already reviewed the RuntimeSettings.xml for a customer setting #sim and Memory Limit too high.    - The errors in the Gallery and Service log shown below support the Server being overloaded.Server logsocket timeoutsinvalid AS_Queue IDGallery loginvalid AS_Queue ID       - Server logsocket timeoutsinvalid AS_Queue ID          - socket timeouts          - invalid AS_Queue ID        - Gallery loginvalid AS_Queue ID          - invalid AS_Queue ID    **Troubleshooting**     - Review Service logs forService Log Error - Socket timed out https://alteryx.atlassian.net/wiki/spaces/SupportServer/pages/1653211967       - Service Log Error - Socket timed out       - https://alteryx.atlassian.net/wiki/spaces/SupportServer/pages/1653211967     - Review Gallery logs forhttps://alteryx.atlassian.net/wiki/spaces/SupportServer/pages/1653539055       - https://alteryx.atlassian.net/wiki/spaces/SupportServer/pages/1653539055 |