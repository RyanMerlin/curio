---
id: 0bbcd7bb8ce5d994
title: Understanding Windows Session 0 Isolation
status: review
source:
  kind: confluence_page
  id: confluence-page:2125332821
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/2125332821
  summary: null
category:
- product-tree
- alteryx-server
- administration
keywords:
- windows
- session-0
- service
- isolation
- security
created_at: 2026-04-14T15:18:28Z
updated_at: 2026-04-14T15:20:58Z
confidence: 0.87
cross_refs: []
content_hash: sha256:22861183e13aba978aa37bf04b059003408a1371c89a78e52e7063f5c58565b7
confluence_page_id: null
model_used: claude-sonnet-4-6
---

> **ℹ️ Info**
>
> This page digs into the Service running in Session 0 and the impact that has on the ability to run Batch Scripts via the Run Command

| **Author** | Michael Adler |
| --- | --- |
| CSU | Understanding Windows Session 0 Isolation (Michael Adler) 30m |
| **Key Articles** | [How-To (Excel)](https://alteryx.atlassian.net/wiki/spaces/SupportDesigner/pages?title=How-To+(Excel))  <== **search for “Server” articles** |

---

---

# Scope of Document

## In Scope

- Provide background to understand issues faced by customers using Excel automation in non-attended environment
- Provide customers with directions to Microsoft-supported solutions for implementation by customer.
- Provide customer with directions for solutions that work for some customers but are outside of Alteryx or Microsoft Support scope.

## Out of Scope

- Technical analysis, why Excel automation in non-attended environment works in one Server environment, but not in the other.
- Provide end to end solution for customer for Microsoft-supported solutions for Excel automation on their Server environment.

# Motivation

Often, a customer might run a VB Script to perform programmatically changes to an Excel file.

In a typical use case, the user will execute a[batch file](https://en.wikipedia.org/wiki/Batch_file) from Designer via **Run Command** tool. This batch file in turn will call a **VB Script** to perform changes to an Excel file. This will typically require the Excel application to be installed on the Alteryx Server machine. **VB Script** commands such as below will require Excel to be installed on the machine where the script runs:

> **📝 Note**
>
> While the workflow might run fine on a local Designer and from Designer on the Alteryx Server; running it on the Alteryx Server might fail. Why is this happening? First, we need to go a few steps back and review some Windows background.

---

# Applications, Processes, and Services

Technically, speaking from the perspective of Windows OS, an *application* comprises multiple *processes*. A process could be seen as an executing program.

The process also has the following attributes:

- Unique identifier (PID),
- Environment variables,
- Virtual memory address space,
- Executable code and many more

Processes can be seen in **Windows Task Manger** and Service in the **Windows Services app**.

*Windows Services, extended view*

Differences between **Applications** and **Services** (not exhaustive).

| **Application** | **Service** |
| --- | --- |
| Runs in Session above 0 | Started in Session 0 |
| Interactive | Non-interactive |
| Multiple instances can run at the same time | Only one instance can run at the same time |
| Exits when user logs off | Keeps running as long as OS is running. Can be stopped manually via Services. |
| Uses the user’s home folder as the working directory. | Use **%SYSTEM%/System32** as the default working directory. |

An in-depth description of this can be also found in the [Windows Internals Book](https://learn.microsoft.com/en-us/sysinternals/resources/windows-internals).

---

# Sessions Explained

What are Sessions - in this context?

Running the **tasklist** command will return a list of processes running under the local or remote computer.

For instance:

| **Image Name** | **PID** | **Session Name** | **Session#** | **Mem Usage** |
| --- | --- | --- | --- | --- |
| **Wireshark.exe** | 29316 | Console | 13 | 99,260 K |
| **pgbouncer.exe** | 6752 | Services | 0 | 8,040 K |

Each process has a session.

On Windows startup **SMSS.exe** (**Session Manager Subsystem**) amongst its multiple tasks starts Windows **Session 0**; this is the first and privileged session. Later sessions are user sessions with session ID greater or equal to **1**.

---

# Session 0 Isolation

With Windows Vista (released January 2007), Microsoft introduced a new security feature. All *services *would run in non-interactive (or console mode), isolated, and privileged **Session 0**. This was called **Session 0 Isolation**.

However, User process would run in the non-privileged **Session 1** with UI. This change dramatically improved Windows OS security (see below).

Additionally, **User Interfaces** (**UIs**) are not supported in **Session 0**. Thus, **UIs**, cannot be directly displayed in **Session 0**.

> **📝 Note**
>
> Prior to the release of Windows Vista, **Session 0** **Processes **and **Services** would both run in the same privileged **Session 0**. This is the session of the first Windows logon user on system startup.
> 
> This opened the door for malicious actors to gain privileged access and perform a [Shatter Attack](https://en.wikipedia.org/wiki/Shatter_attack).

---

# Limitations of Running Interactive Scripts on Server

Microsoft explicitly excludes support for automation of office in an unattended, i.e. Server, environment; please find below the verbiage:

> **📝 Note**
>
> Microsoft does not currently recommend, and does not support, Automation of Microsoft Office applications from any unattended, non-interactive client application or component (including ASP, [ASP.NET Core | Open-source web framework for .NET](http://asp.net/) , DCOM, and NT Services), because Office may exhibit unstable behavior and/or deadlock when Office is run in this environment. For more information, see [Considerations for server-side Automation of Office](https://learn.microsoft.com/en-us/topic/considerations-for-server-side-automation-of-office-48bcfe93-8a89-47f1-0bce-017433ad79e2).

Microsoft provides us with an incomplete list of the reasons why one shouldn’t use run automated Office in an unattended environment and provides alternatives in the article - [Considerations for unattended automation of Office in the Microsoft 365 for unattended RPA environment](https://learn.microsoft.com/en-us/office/client-developer/integration/considerations-unattended-automation-office-microsoft-365-for-unattended-rpa).

**Based on this, Support can go the extra mile to help customers who want to use office automation on an Alteryx Server, however, we should be aware of the best practices and limitations; importantly, Microsoft discourages automation for Office on a non-interactive client.**

Amongst others, one issue is running an inherently interactive process in the context of a non-interactive **Windows Session 0** on a Windows Server environment.

---

# Task Scheduler – Interactive and Non-Interactive Mode

Is it at least partially possible to simulate a non-interactive task? For instance, is it possible to simulate running a **VB Script** with interactive elements in non-interactive mode?

One powerful tool that can be used for this is the **Windows Task Scheduler**.

1. Go to Task Scheduler,
2. Select Action > Create Task
3. Select:The correct user the task will run under,The correct run mode:Run only when user is logged on (interactive mode)Run whether user is logged on or not (non-interactive mode)Optionally: Run with highest privileges (administrative permissions).In Triggers select the frequency of the task and,Under Actions select the actual task.
   1. The correct user the task will run under,
   2. The correct run mode:Run only when user is logged on (interactive mode)Run whether user is logged on or not (non-interactive mode)Optionally: Run with highest privileges (administrative permissions).
      - Run only when user is logged on (interactive mode)
      - Run whether user is logged on or not (non-interactive mode)
      - Optionally: Run with highest privileges (administrative permissions).

   3. In Triggers select the frequency of the task and,
   4. Under Actions select the actual task.

Windows Task Scheduler can be a powerful tool when comparing non-interactive and interactive mode. If an automation doesn’t run in non-interactive mode on the Server from **Task Scheduler** it would not be expected to run successfully when run from Alteryx Server.

---

# Workarounds

Not supported by Microsoft, but some customers have seen success (Alteryx articles provided “as is”):

- How to enable COM object on the server
- How to: Enable Interactive User mode for Excel Macros on Alteryx Server

Recommended by Microsoft:

- Use Microsoft Graph PAI
- Use Open XML file formats and use methods that support making changes from service

---

# Additional References

*Windows Session 0 Isolation*

- Microsoft Tech Community post on Windows Session 0 Isolation
- Microsoft Tech Community post on Windows Session from a user account control perspective

*Windows Task Scheduler*

- Task Scheduler for developers | Microsoft Learn

*Miscellaneous Windows OS Internals*

- Sysinternals Resources - Sysinternals | Microsoft Learn (advanced resources)

*Office Automation in Server Environments*

- Considerations for server-side Automation of Office
- Considerations for unattended automation of Office in the Microsoft 365 for unattended RPA environment | Microsoft Learn