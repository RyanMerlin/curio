---
id: e78679a3afdecd8b
title: How to Setup Hyper-V on your Laptop
status: intake
source:
  kind: confluence_page
  id: confluence-page:2003075393
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/2003075393
  summary: null
category: []
keywords: []
created_at: 2026-04-14T15:09:17Z
updated_at: 2026-04-14T15:09:17Z
confidence: null
cross_refs: []
content_hash: sha256:f6c0d7426434cb54b8880ae640006585b7af5f32f961299eec2f56d5c71b9fe3
confluence_page_id: null
model_used: null
---

Hyper-V is a Microsoft hardware virtualization product which lets you create and run virtual machines. Each virtual machine acts like a independent instance which can run operating system and other supporting programs much like a computer/laptop. Your Window 11 Enterprise laptops have the capability to run Hyper-V and launch/run multiple VMs provided you have the right system resources.  This can be especially useful when testing product features to understand how a the functionality works and whether its a defect.

none

# Scope

- Install and configure Hyper-V hypervisor on your laptop
- Install and Configure 2 Virtual Machines
- Install and Configure Alteryx Server 2024.1 Instance.
- Install and Configure Alteryx Designer 2024.1 Instance.

# Limitations 

- Launching multiple VM is dependent on resources available on your laptop (i.e. Hard drive, Ram , CPU Core, Virtual Processors). For the purposes of this article we will use 1TB SSD Hard Drive, 32Gb Ram, 8 Cores with 16 Virtual Processors. A Maximum of 3 instances can be running at the same time.
- Hyper-V VMs and Alteryx products should only be used for light and less resource intensive tasks since we are configuring the VMs and Alteryx below the minimum specs and for testing purposes only.
- This is for INTERNAL USE ONLY.
- The Window Server Operating system used to configure VM are evaluation licenses only which expire every 6 months. After which your VMs will auto shutdown every hour. To overcome this its recommended you recreate the instance every 6 months. However for small use cases an 1 hour may be enough for troubleshooting

# Use-cases 

- A customer is unable to upload workflow to the gallery following an upgrade. You suspect its API related and want to compare the API calls between 2022.1 and 2024.1. In this instance you can quickly start up the instance,  install Fiddler on VMs hosting both version and compare the Fiddler trace.
- SAML Authentication not working in 2024.1 with Okta. you would like to compare the SAML Tracer against the customers SAML trace to check for discrepancies
- Check feature differences and how they work between versions and check if the defect has been resolved.
- Use cases can be extended to use 3rd party application and/or windows server features (such as active directory, DNS, DHCP, ADFS, etc)

# Requirements 

- Hyper-Visor enabled at the BIOS Level
- Window 11 Enterprise
- Windows Server 2022 Evaluation License (https://www.microsoft.com/en-us/evalcenter/evaluate-windows-server-2022 ).
- Minimum 500GB free hard drive space, with 16GB Ram, 8 Core 16 Virtual Processors

# Install Hyper-V on Windows

1. To install Hyper-V on your laptop, navigate to control panel>>select Programs and Features then select Turn Windows Features on or off.
2. Here you should see Hyper-V listed. Select the checkbox and click Ok
3. This should install all the required features and prompt you to restart the laptop. Select restart now to complete the setup
4. Once restarted, Hyper-V should be installed. You can open the console by searching for the Hyper-V Manager Console from the Windows Menu

# Section-1Section 1: Install and Configure 2 Virtual Machines

Before creating a virtual machine, download the evaluation windows OS from <https://www.microsoft.com/en-us/evalcenter/evaluate-windows-server-2022>   and save it to a location on your C drive (**Important: remember to download this as an .ISO file**) . This should be accessible to the Hyper-V Manager. (Note: Typically the Hyper-V virtual machine window is empty however my instances already has multiple environments setup so please ignore them)

1. Right Click on your Machine name (example: AYX-LT-GNVN4M3), New>> Virtual Machine
2. A Virtual machine wizard will appear. Click Next to continue
3. Name the Virtual machine something relevant (example 2024.1 New Release). If you wish to store the virtual machine in a different location, select the checkbox and specify the location. (Note: This is not mandatory), Then click Next
4. Select Generation 2 and click Next. (Note: As per the description Gen 1 is only required if your looking into Install 32-bit operating systems.
5. For the startup memory enter 4096 (this may be default). Depending on the amount of RAM available you may choose to increase this. For now this is ok. Uncheck Use Dynamic Memory for the this virtual machine.(To learn more about dynamic memory click the following link). Click Next to continue
6. Select the default switch from the drop down menu and click Next
7. You can choose to rename the virtual hard disk however in this instance its relevant to the usecase. Change the size of the virtual hard disk to something appropriate. Remember, check your laptops disk space to ensure you have enough space. Windows Server 2022 by default needs a minimum of 32 GB. I recommend between 70-100GB depending on the usecase. You can choose to store the virtual hard disk in a different location but the default are fine aswell. Then click Next
8. Select “Install an operating system from a bootable image file” and browse to the location of the Windows Server OS .ISO. Select and Click Open. Then Next to continue
9. Click Finish on the Summary page to finish the installation
10. The instance will automatically appear under Virtual Machines. See below:
11. Right Click the instance and select the settings from the menu. Select Processor under Add Hardware and change the Number of virtual processors to 4. (The configuration here is typically dependent on the number of logical processors you have available on your laptop. Always leave a minimum of 4 logical processors for the laptop. To find the number of logical processor on your machine, Open System information>> The Processor row will show you both Cores and Logical processors available). Select Ok to save the changes
12. Follow Steps 1 - 11 from Section 1 and create a second instance.
13. ConfigureVMOnce the VMs are launched we need to configure them to run the Server Operating system. Start by Double-clicking the VM which launches virtual sessions for the machine, and Click on Start
14. This will start the instance, quickly press any key to ensure the VM Boots directly from the .ISO image we configured earlier.
15. Select the Language, Time and Keyboard Layout of your choice. example below and Click Next
16. Click Install Now to Continue, Select Window Server 2022 Data Center Evaluation (Desktop Experience), Click Next
17. Select the License Agreement Checkbox and Click Next
18. Select Custom: Install Microsoft Server Operating System only (Advanced)
19. Click Next to Continue
20. The Server operating system will start installing. Once complete the server will automatically restart.
21. Enter a password of your choice. This is will be the default administrator password for your account. Click Finish to Continue
22. Log into the instance using the credentials you just created.
23. At this is stage, the environment can be setup to connect to an active directory or any environment depending on the request. For the purposes of this article we will setup as a standard server with Alteryx server installed. Download, Install and Configure 2024.1 Alteryx Server from downloads.alteryx.com. The recording shows Alteryx Server configured using built in authentication. You can also reference the help docs for installation instructions https://help.alteryx.com/current/en/server/install/install-or-upgrade-server.html#download-server-installation-file Note: Remember to note down your built-in credentials for the next steps of the this article
24. Follow Steps 13-23 in section 1 to configure the second VM but this time install Alteryx Designer 2024.1. The next steps will demonstrate the connection between two instances and any considerations. The following recordings shows an attempted connection between Alteryx designer and the server environment. Its shows a failed connection and how to check for the following:Check the Alteryx Server machine can be Pinged from the Designer instanceCheck the windows firewall has been configured with the correct allow rules or switch off to ensure connectivity is successful (You can also modify the firewall to allow communication through specific port. Please see help documentation on required ports https://help.alteryx.com/current/en/server/system-requirements.html#idm44990413668080 )Check for any proxy configuration preventing the connection from succeedingOnce the default connectivity restrictions are overcome, the connection will be successful as per recording above.
   1. Check the Alteryx Server machine can be Pinged from the Designer instance
   2. Check the windows firewall has been configured with the correct allow rules or switch off to ensure connectivity is successful (You can also modify the firewall to allow communication through specific port. Please see help documentation on required ports https://help.alteryx.com/current/en/server/system-requirements.html#idm44990413668080 )
   3. Check for any proxy configuration preventing the connection from succeedingOnce the default connectivity restrictions are overcome, the connection will be successful as per recording above.

# Section 2: How to use checkpoints

Its often required to test connectivity using different configurations for the same version. Such examples include:

1. Testing with different authentication types
2. Different patch versions
3. Reverting changes due to a corruption during testing.

Hyper V Checkpoints (otherwise known as snapshots) give you the option to revert to older snapshots within a few minutes to test different scenarios at will. The next steps will show how revert snapshots to test two different authentication methods as an example

1. Launch the Hyper V Manager, Right Click on the Alteryx Server New Release VM (2024.1 New Release) and select Checkpoint
2. Notice a new checkpoint has been created. Right-Click the checkpoint and select Rename. Rename it 2024.1 New Release Built-In Auth
3. Right-Click on the VM, and select Connect
4. Once connected sign in using your Windows Credentials, Open Alteryx System Settings and click next until you get to Controller Persistence. Under the data folder modify the MongoDB folder name to add V2. This will create a new MongoDB embedded database. Click Next to continue
5. Click Next until you get to Server UI Authentication. This example covers SAML Authentication with Okta. Click Here for instructions on how to configure SAML authentication with Okta. Once configure click Next to continue
6. Click Next and Finish the Configuration
7. Once complete, confirm the Alteryx Service has started. Open the browser in In-cognito and connect to Server UI, Click on Sign-in Notice the authentication method is configured for Okta
8. Now open the Hyper V Manager again, Select the 2024.1 New release VM, Right-Click and Select checkpoint. Under checkpoint Right-click the new checkpoint and select Rename. Name it 2024.1 New release SAML Okta.
9. Here comes the Magic, Select the previous checkpoint Name 2024.1 New Release Built-in Auth, Right Click and Apply. With in a matter of minutes the instance is reverted.
10. Connect to the VM again, Sign-in using your Windows Credentials, Open Alteryx system settings, Navigate to Server UI Authentication, Notice the Authentication is Built-in again. Open the Browser, Search for the Server UI Base address, Click Sign-in and the Built-in Sign options are available again
11. To switch back, Open the Hyper V Manager again. Select the 2024.1 New Release VM, Under checkpoints select 2024.1 New Release SAML Okta, Right Click and Apply. This will revert the changes back to SAML Authentication. You get the idea…it can be useful for many scenarios and available to you until the licenses expire. Even after this the instances are still available, You can restart the instance straight away and continue for another hour. The recommendation is to delete and create the VM again.

# Complete