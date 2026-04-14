---
id: 2243e20d252927b7
title: How to Setup Microsoft failover cluster service in AWS
status: intake
source:
  kind: confluence_page
  id: confluence-page:1999044872
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/1999044872
  summary: null
category: []
keywords: []
created_at: 2026-04-14T15:06:10Z
updated_at: 2026-04-14T15:06:10Z
confidence: null
cross_refs: []
content_hash: sha256:7cc14788f2448a151542a4e89a6c7ae5763bcf0ffc08d41c5f472e97a2568d01
confluence_page_id: null
model_used: null
---

> **ℹ️ Info**
>
> There are multiple ways to configure Microsoft failover environments which largely depends on the customers requirements and current infrastructure used to support high availability. The following articles/help documents refer different architectures and resiliency options, together with setup instructions on how to build the environment. Please review the article and ensure you have a good understanding of the requirements before continuing.

|  |  |
| --- | --- |
|  |  |

---

---

# Scope

- Setup two Domain Controllers with DNS in two different availability zones
- Setup two complete Alteryx Server nodes and configure Microsoft failover clustering hosted in an AWS environment.
- Setup MongoDB Atlas
- Configure Route 53
- Configure Security Groups

# Out of Scope

This article assumes some aspects such as, AWS VPC, Availability Zones and Other configurations are already setup by IT.

# Requirements

- Access to AWS Management Console
- Pre-Built Amazon VPC
- Active Directory Domain Controller with integrated DNS
- 2 Amazon EC2 Instances with Alteryx Server 2023.2
- MongoDB Atlas
- Window Server Failover Clustering
- Route 53 Access

# Section 1 - Create 4 EC2 Instances in two different availability Zones. 

1. Open my Apps (https://myapplications.microsoft.com/ ) and Select AWS SSO app>>AWS Account>>alteryx-sub-et(cloudteam+e3t@alteryx.com)>>Management console. (This account may only be available to E3T members. If you don't have access please reach out E3T.  Ensure the Region (Top right) is Oregon.

1. Search EC2 in the search bar and click on EC2- Virtual Servers in the Cloud
2. Click on Launch instance

### First Instance will be Domain Controller 1

1. Enter a Name (This can be anything), Example: DC01
2. For Application and OS Images - Select Windows Microsoft. For the AMI Select Microsoft Windows Server 2022 Base free tier eligible
3. Under instance type - select t3.medium (2 vCPU 4 GiB Memory)
4. For key pair (login)For-key-pair - Select Create New key pair>>Enter any Key pair name (example: remote sessions)>>Click Create key pair. This will download a .pem file which will be used to decrypt the password when remoting into the instance. Note : Store the key pair in a safe place as it will be referenced for all instances.
5. Network settings - Select edit and configure as follows:VPC - leave this as default et-networkingSubnet - subnet-02999cc07d5d539fd (This is in Availability Zone: us-west-2c) Security-GroupFirewall (Security Groups) - Enter a new security group name (Example: DC Security Groups)Inbound security group rules - Change the Source Type to Custom and add the VPC Subnet range. This is  172.27.205.0/24
   1. VPC - leave this as default et-networking
   2. Subnet - subnet-02999cc07d5d539fd (This is in Availability Zone: us-west-2c)
   3. Security-GroupFirewall (Security Groups) - Enter a new security group name (Example: DC Security Groups)
   4. Inbound security group rules - Change the Source Type to Custom and add the VPC Subnet range. This is  172.27.205.0/24

6. Configure Storage - Change the GiB Volume to 70 GB
7. Click on Launch Instance to complete setup.
8. Click on View all instances to go back to EC2 page. Here you can view the status of each instance you created.
9. Next we need to configure the newly created  security group (Step 8c) to restrict Inbound traffic to machines  just within the VPC. This will allow internal traffic but block external access. Click on Security Groups in the left navigation pane, Search for DC Security Group and click on the Security Group ID as shown below link underline in red.
10. Select Edit Inbound Rules

1. Change the second Inbound rule type from  Custom to All traffic and Replace the IP address with 172.0.0.0/8. Add another rule, For type select All Traffic and Enter the IP address 10.48.0.0/16. Then Save rule. See below:

### Second Instance for Domain Controller 2

1. Follow Step 4 to 11 to create the second instance but with this following changes to:Name - DC02Key pair (Login) - Select the existing key pair created in Step 7 (remote sessions)Network Setting - Select edit:Subnet - Change the subnet to a different availability zone i.e (subnet-0bef5efb85e22714a, Availability Zone: us-west-2a)Under Firewall (Security Groups) - Check Select existing security group and choose DC Security Group. This was created in the previous section
   1. Name - DC02
   2. Key pair (Login) - Select the existing key pair created in Step 7 (remote sessions)
   3. Network Setting - Select edit:Subnet - Change the subnet to a different availability zone i.e (subnet-0bef5efb85e22714a, Availability Zone: us-west-2a)Under Firewall (Security Groups) - Check Select existing security group and choose DC Security Group. This was created in the previous section
      1. Subnet - Change the subnet to a different availability zone i.e (subnet-0bef5efb85e22714a, Availability Zone: us-west-2a)
      2. Under Firewall (Security Groups) - Check Select existing security group and choose DC Security Group. This was created in the previous section

### Alteryx Server First Instance 

1. Follow Step 4 to 11 to create the second instance but with following changes to:Name - Alteryx Server 1Key pair (Login) - Select the existing key pair created in Step 7 (remote sessions)Network Setting - Select Edit:Subnet - Change the subnet to a different availability zone (subnet-02999cc07d5d539fd, Availability Zone: us-west-2c)Under Firewall (Security Groups) - Check Select existing security group and choose DC Security Group which was created in in the previous section
   1. Name - Alteryx Server 1
   2. Key pair (Login) - Select the existing key pair created in Step 7 (remote sessions)
   3. Network Setting - Select Edit:Subnet - Change the subnet to a different availability zone (subnet-02999cc07d5d539fd, Availability Zone: us-west-2c)Under Firewall (Security Groups) - Check Select existing security group and choose DC Security Group which was created in in the previous section
      1. Subnet - Change the subnet to a different availability zone (subnet-02999cc07d5d539fd, Availability Zone: us-west-2c)
      2. Under Firewall (Security Groups) - Check Select existing security group and choose DC Security Group which was created in in the previous section

### Alteryx Server Second Instance 

1. Follow step 4 to 11 to create the second instance but with this following changes to:Name - Alteryx Server 2Key pair (Login) - Select the existing key pair created in Step 7 (remote sessions)Network Setting - Select Edit:Subnet - Change the subnet to a different availability zone (subnet-0bef5efb85e22714a, Availability Zone: us-west-2a)Under Firewall (Security Groups) - Check Select existing security group and choose DC Security Group which was created in in the previous section
   1. Name - Alteryx Server 2
   2. Key pair (Login) - Select the existing key pair created in Step 7 (remote sessions)
   3. Network Setting - Select Edit:Subnet - Change the subnet to a different availability zone (subnet-0bef5efb85e22714a, Availability Zone: us-west-2a)Under Firewall (Security Groups) - Check Select existing security group and choose DC Security Group which was created in in the previous section
      1. Subnet - Change the subnet to a different availability zone (subnet-0bef5efb85e22714a, Availability Zone: us-west-2a)
      2. Under Firewall (Security Groups) - Check Select existing security group and choose DC Security Group which was created in in the previous section

---

# Section 2 - Setting up Domain Controllers between two availability zones

> **ℹ️ Info**
>
> Once the instances have been deployed, We need to configure two machines as Domain Controllers. The following steps will show how to configure and replicate a Domain Controllers **DC01** and **DC02**.

## DC01 Primary Domain Controller setup

1. Click on the Instances tab in AWS and select DC01. Click on Connect

1. Next, Click on RDP Client tab. Download the remote desktop file which will allow remote into the machine. Open the remote desktop file and you should be prompted with connection window to enter a password. (Note: You must be connected to VPN)

1. For the password select Get password (underlined above) and upload the private key file created in Section 1 step 7. Then select Decrypt Password

1. Copy and paste the decrypted password into the remote desktop session and Click OK

1. Once you have logged into the server we need to configure the server to have a static IP Address. To do this, open the control panel>>Click Network and Internet>> then Network and Sharing Center>>Click the Private network Ethernet Adaptor(in My instance it Ethernet 3 shown below)>>Properties>>Select Internet Protocol Version 4 (TCP/IPv4)>>then Properties. Change the properties from:Obtain an IP address automatically to Use the following IP addressObtain DNS Server address automatically to Use the following DNS server address

1. Next Open Cmd prompt>>Type in Ipconfig /all and hit Enter. Copy the details from the cmd prompt to the IPv4 window. The following is my example. Click Ok to save the changesCmd IPv4 Address 172.27.205.74 >> IP address 172.27.205.74Cmd Subnet Mask 255.255.255.224 >> Subnet Mask 255.255.255.224Cmd Default Gateway 172.27.205.65 >> Default Gateway 172.27.205.65For Preferred DNS Server use the machines IP Address 172.27.205.74DNS Servers 172.27.205.2 >>Alternate DNS Server 172.27.205.2Note: you may temporary lose connection to the remote desktop session. this is normal

1. Once you are re-connected, Close all tabs then open the server manager console. Click on Add roles and features

1. Click Next through the following promptsBefore you begin - NextInstallation Type - NextServer Destination Server - Next
   1. Before you begin - Next
   2. Installation Type - Next
   3. Server Destination Server - Next

2. Under Select Server Roles, check Active Directory Domain Services, click on Add Features

1. Then select the DNS Server, click on Add Features . Click Next to continue

1. Click Next for the following prompts:FeaturesFeature - AD DSDNS Server
   1. Features
   2. Feature - AD DS
   3. DNS Server

2. Confirmation, please ensure you select “Restart the the destination server automatically if required”. Then click install to continue. Note: you may be prompted again for the restart. If so click yes
3. The final results should look like this. Click Close and restart the machine if required

1. The next step is to Promote the machine to act as a Domain Controller. To do this, click on the Notifications Icon in Server Manager and select “Promote this server to a domain controller“

1. Change the selection to Add a new forest and enter any domain name. For example: ayxtest.ayx. Click Next to continue. Note: Important - The root domain name must have a certificate installed with subject alternatives included. i recommend adding both the machine name and the specified domain root domain name (example: machine name: WIN-WJEMN4JKE and Root domain name: ayxtest.ayx)  For testing purposes you can use a self signed cert. This article walks you through configuring a cert for the domain https://knowledge.alteryx.com/index/s/article/Configuring-Alteryx-Server-for-SSL-Obtaining-and-Installing-Certificates-1583459841225

1. Enter a password of your choice. Leave all other settings to the default configuration.

1. Click Next for:DNS OptionsAdditional OptionsPaths - Leave as defaultReview OptionsThe final prerequisutes check should pass, Then click Install to continue
   1. DNS Options
   2. Additional Options
   3. Paths - Leave as default
   4. Review Options
   5. The final prerequisutes check should pass, Then click Install to continue

2. The machine will automatically restart to at which the machine will be joined to ayxtest.ayx. Reconnect to the instance and check the following to confirm its domain joinedClick on the windows Icon (bottom left), Search for and click on Settings, Click on Advanced System Settings on the right, Click on the Computer Name tab, Here you should see the machine is Domain Joined
   1. Click on the windows Icon (bottom left), Search for and click on Settings, Click on Advanced System Settings on the right, Click on the Computer Name tab, Here you should see the machine is Domain Joined

## Create a Domain Account Create-a-Domain-Account

19. In order to use authentication services within the domain you must have an account crated in active directory which is has domain admin permissions. Open **Active Directory Users and Computers. **

1. Right Click Users container, Select New, then User. Enter the firstname, Lastname and User Logon Name. Then click Next to Continue . Example:FirstName: MiteshLastName: NarottamUser Logon Name: Mitesh.narottam
   1. FirstName: Mitesh
   2. LastName: Narottam
   3. User Logon Name: Mitesh.narottam

2. Click Next

1. Enter a password, Uncheck “User must change password at next logon”, then Click Next

1. Click Finish,

1. The account should be created. Right Click the newly created user and Select Properties. Select the Member Of Tab. Add the Domain Admin group. This will assign Admin privileges to the account. Select Apply and OK

## DC02 Secondary Domain Controller setup

Most of the setup for the DC02 will be the same. The following will reference steps from Section 2 - DC01 Primary Domain Controller setup

1. Follow steps 1 to 5 from Section 2, Then continue here
2. Open Cmd prompt>>Type in Ipconfig /all and hit Enter. Enter details from the Cmd prompt to the IPv4 window. the following is my exampleCmd IPv4 Address 172.27.205.6 >> IP address 172.27.205.6Cmd Subnet Mask 255.255.255.224 >> Subnet Mask 255.255.255.224Cmd Default Gateway 172.27.205.1 >> Default Gateway 172.27.205.1For Preferred DNS Server use the IP Address of DC01. In my example 172.27.205.74DNS Servers 172.27.205.2 >>Alternate DNS Server 172.27.205.2Note: you may temporary lose connection to the remote desktop server. this is normal

1. Once complete follow steps 7 to 13 from Section 2
2. Then Select “Add a domain controller to existing domain”. Under Domain Click on Select. You should be prompted to enter credentials. Enter the credentials you created in earlier section (This is typically the domain\loginname). see example below. Then click on OK. ayxtest.ayx\mitesh.narottamPassword12

1. if it authenticates successfully, a new window with the domain will appear. Example: ayxtest.ayx. Select the domain and click Ok. Click Next to continue

1. Enter a passwords of your choice. Leave all other settings to the default configuration.

1. Click Next for the following options:DNS OptionsAdditional OptionPathReview OptionsClick Install on the Prerequisites Check
   1. DNS Options
   2. Additional Option
   3. Path
   4. Review Options
   5. Click Install on the Prerequisites Check

2. Once complete the machine will automatically restart. At this point the secondary domain controller is setup and both DC01 and DC02 are replicating. Going forward remote into the machine using the domain credentials created in Section 2 Create a Domain Admin Account. Example: ayxtest.ayx\mitesh.narottamPassword12

---

# Section 3 - Configures EC2 Instances (Servers 1 and 2) and Install Alteryx Server

> **ℹ️ Info**
>
> This section covers the following
> 
> - Remote into Alteryx Servers 1 and 2
> - Configure Network Setting
> - Join the machines to the newly created domain
> - Install Alteryx Server (This example will use 2023.2). Remember to License the Server as well.

1. Click on the Instance tab in AWS and select Alteryx Server 1. Do the same for Alteryx Server 2 you’ve completed server 1. Click on Connect

1. Under Connect to instance window, Click on RDP Client tab. Download the remote desktop file. Click on the downloaded remote session file and you should be prompted with a connection window to enter a password. (Note: You must be connected to VPN)

1. For the password select Get password (underlined above) and upload the private key file created in Section 1 step 7. Then select Decrypt Password

1. Copy and paste the decrypted password into the remote desktop session and Click OK

1. Configure the server to have a static IP Address and point to the DNS Server. To do this open the Control panel>>Select Network and Internet>>Network and Sharing Center>>Select the Private network Ethernet Adaptor(in My instances it Ethernet 2 shown below)>>Properties>>highlight Internet Protocol Version 4 (TCP/IPv4) by selecting it>>Properties. Here you can change the properties from:Obtain an IP address automatically to Use the following IP addressObtain DNS Server address automatically to Use the following DNS server address

1. Open cmd prompt>>Type in Ipconfig /all and hit Enter. Enter the details from the Cmd prompt to the IPv4 window. The following an example. your details will be different.Cmd IPv4 Address 172.27.205.74 >> IP address 172.27.205.74Cmd Subnet Mask 255.255.255.224 >> Subnet Mask 255.255.255.224Cmd Default Gateway 172.27.205.65 >> Default Gateway 172.27.205.65Important: For Preferred DNS Server us must use the newly created Domain Controllers which has the DNS Server role installed. For Server 1 its the IP Address for DC01, For Server 2 its the IP address for DC02. Note: This step is required in order to join the machines to the domainDNS Servers 172.27.205.2 >>Alternate DNS Server 172.27.205.2Note: you may temporary lose connection to the remote desktop server. this is normal

1. Next, Open Control Panel>>Select System and Security>>System>>Select Advanced System Setting >>Select Computer Name Tab>>Select Change>>Click on Domain>>Enter the domain name (example: ayxtest.ayx)>>You should be prompted to enter domain credentials. Enter the credentials you created in Section 2 Step 20 >>Click Okayxtest.ayx\mitesh.narottamPassword

1. If successful, You should be prompted with Welcome to the domain, Click Ok and machine will restart to apply the changes

1. Going forward you should remote into the machine using the domain credentials. ayxtest.ayx\mitesh.narottamPassword
2. Remote into both machines (Server 1 and 2) and Install Alteryx Server (2023.2). Remember to license the machine

---

# Section 4 - Install and Configure Servers 1 and 2 with Microsoft failover Clustering

## Install Failover Clustering Feature

1. You must complete the installation for both Servers 1 and 2 using the following instructions. Remote into the servers using the domain credentials>>Open Server Manager>>Click on Add Roles and Features

1. Click Next through the following prompts:Before You BeginInstallation TypeServer SelectionServer Roles
2. Under Features, Click Failover Clustering, Add Roles, Click Next to continue

1. Click Install under the confirmation window

1. Click Close and Reboot both instances

## Configure Failover Clustering Feature - Only required on 1 Node

This configuration in only required on one node and  can be done on either Servers 1 or 2.

1. Once you have added Failover Clustering Feature to each node, The next step is to create a cluster.
2. Open Server Manager.
3. From the Tools menu, select Failover Cluster Manager.
4. Select Create Cluster under Actions
5. The Create Cluster Wizard will open, Select Next on Before You Begin.
6. Click Browse and Enter the Hostname for both Server 1 and Server 2, Then Click Ok. Click Next to continue. See screen prints below:

1. Under Validation Warning , select Yes. Then Next, .
2. This will open a validation wizard. Click on Next to continue
3. On the Testing Options screen, select Run all tests (recommended), then select Next.
4. On the Confirmation screen, verify the cluster names are correct. Then select Next to proceed.

1. Upon selecting Next, the new cluster will be configured
2. Once the cluster has been configured, you should receive a Summary screen stating you have successfully completed the Create Cluster Wizard. Select Finish to close the Create Cluster Wizard.

1. The Access Point for Administering the Cluster screen will automatically open, enter a Cluster Name. The Cluster Name will be added to both  DNS and Active Directory domain. Once you have entered a Cluster Name (Example: ca-cluster-a), Enter an unused IP Address for each Subnet. ( I used the next available IP which was 172.27.205.67 and 172.27.205.3) Select Next to proceed to the confirmation screen.

1. Click Next to proceed.

1. Upon selecting Next, the new Cluster will be configured and added to DNS.
2. Once the cluster has been configured, you should receive a Summary screen stating you have successfully completed the Create Cluster Wizard. Select Finish to close the Create Cluster Wizard.

## Add a Cluster Role

1. Now that you have created a cluster, you need to add a Cluster Role. These steps can be completed from any of the servers that the Failover Clustering Feature has been enabled on. Within the Failover Cluster Manager console, expand the newly created Cluster, highlight Roles on the left and from within the Actions menu on the right, select Configure Role.
2. In the High Availability Wizard, select Next on the Before you Begin screen.
3. On the Select Role screen, highlight the Generic Service role and select Next.

1. On the Select Service screen, select Alteryx Service and select Next to proceed.

1. Client-Access-PointOn the Client Access Point screen, enter a DNS name that will be used for accessing the cluster role. This is the DNS name that will be used when configuring Server UI and Worker nodes to access the High Availability Controller cluster. Enter any available IP address within in each subnet\Availability Zone. You can ping the IP to confirm it if availableping 172.27.205.68ping 172.27.205.4Note: This step is important as the IPs will be used to assign secondary IPs for reach instance in AWS.

1. Select Next on the Select Storage and Replicate Registry Settings screens.
2. On the Confirmation screen, verify the settings and select Next.

1. Upon clicking Next, the Cluster Role will be created and added to DNS. Once the High Availability role has been created, you should receive a Summary screen stating the high availability was successfully configured for the role. Select Finish to close the High Availability Wizard.

1. Microsoft Failover Clustering will now manage the state of the AlteryxService.exe on each of the nodes in the cluster. The AlteryxService.exe will be started on the “Owner” (active) node and the failover nodes will be in a stopped state. In the event of a failure on the “Owner” node, Microsoft Failover Clustering will start the AlteryxService.exe on one of the failover nodes and automatically direct traffic to the active Alteryx Controller.
2. Stop the cluster role service

## Setup a Quorum Witness for your cluster

1. The cluster quorum consists of a majority voting node within an active cluster plus one additional node called a Witness. This will make sure the system still runs even if one availability zone unexpectedly shutdown. There are many options available when creating a witness. For this article we will create a file share witness. A file share witness should typically be independent node that is not part of the high availability environment, this includes the DCs (A good example would be Nas, or independent file share). Since this article is just an example, we will use DC01 to create a SMB  file share.
2. Login into DC01 and  Open File explorer. Open C:\ProgramData. Right click in the empty space and Select New, then Folder
3. Name it File Share, Then right click on the folder and select properties. Click on Sharing Tab>>Select Everyone (Note: you can add other group to prevent opening to everyone), Then click  Add and Change the permission level to Read/Write. Click on Share to continue
4. Note down the File share path in the next window(\\EC2AMAZ-1HH27K2\File Share), then click Done
5. Remote into Server 1, Open File Explorer>>Click on This PC, then the Computer tab and Add network location.Click Next on Welcome WizardSelect Choose a custom network location and Click Next,  Add the Network share path from the earlier step. (Example: \\EC2AMAZ-1HH27K2\File Share), Then Click NextFile-shareEnter a name of your choosing, then Click NextClick Finish to add the Network ShareIf configured correctly the File path should open
   1. Click Next on Welcome Wizard
   2. Select Choose a custom network location and Click Next,
   3. Add the Network share path from the earlier step. (Example: \\EC2AMAZ-1HH27K2\File Share), Then Click NextFile-share
   4. Enter a name of your choosing, then Click Next
   5. Click Finish to add the Network Share
   6. If configured correctly the File path should open

6. Open the Failover Cluster Manager, Right Click on the Cluster, Select More Actions, then Configure Cluster Quorum Settings
7. For Configure Cluster Quorum WizardClick NextSelect the quorum witness, then Click NextSelect Configure a file share witness, Click NextEnter the File Share Path from Step 35 c. Click Next to continueClick Next on Confirmation and Finish for the Summary
   1. Click Next
   2. Select the quorum witness, then Click Next
   3. Select Configure a file share witness, Click Next
   4. Enter the File Share Path from Step 35 c. Click Next to continue
   5. Click Next on Confirmation and Finish for the Summary

---

# Section 6 - Configure MongoDB Atlas 

1. Login or Create a Mongo DB Account by navigating to https://account.mongodb.com/account/login. Only free tier is required for this example
2. Under Data Services select Create
3. Make the following selections for the free tier MongoDB Atlas, Then Click on CreateCredsTemplate: MO FreeProvider: AzureRegion: California (westus)Name: Any Name (Example Cluster0)Under Security Quick Start select the following, then Click Finish and CloseHow would you like to Authenticate your connection - Username and PasswordUsername - UserPassword - Password1Click - Create User
   1. Under Security Quick Start select the following, then Click Finish and CloseHow would you like to Authenticate your connection - Username and PasswordUsername - UserPassword - Password1Click - Create User

b. Where would you like to connect from - Select **My Local Environment** then in the Add entries to your IP Access List, Enter your **Public IP**. To find your Public IP, **Open Google Browser **from the Alteryx Server Machine>>Search **What is my IP** (Example: **66.159.216.231**)

c. Go back to **Overview**

d. Next go to **Database Access** in the right **Navigation Pane**>> Edit the **Newly Created User**>>Change the **Built-in Role** to Atlas Admin>>Then click on **Update User. **
**Note:** This can be changed at later date to accommodate for policy restrictions

---

# Section 7 - Configure Alteryx Server on Server Nodes 1 and 2

> **ℹ️ Info**
>
> Once the Pre-work of creating the Cluster in Atlas is done, its time to configure Alteryx System Setting on both Servers 1 and 2. First we must configure both environments to work with Atlas outside of MS Failover.

## Configure Alteryx on Server 1

1. Disable MS Failover Clustering if its enabled. Open Server Manager>> Click on Tools >>Select Failover Cluster Manager. Select Roles and the status should be Running
2. Right Click on cs-cltr-1 and Stop Role. This will disable failover clustering

1. Close all windows and open Alteryx System settings. These steps will only cover changes required. All other settings can be default or based on your preference.      Under Controller Persistence:Under-Controller-PersistenceDatabase Type - User-managed MongoDBAdvanced User- Managed MongoDB -  Check this boxMongoDB Connection - The format of the connection string is as follows. The Underline sections below should be replaced with your values from Atlasmongodb+srv://username:P%40ssw0rd@host-0-1mngx.mongodb.net/AlteryxService?retryWrites=true&w=majorityExample: mongodb+srv://User:Password1@cluster0.clsk7jd.mongodb.net/AlteryxService?retryWrites=true&w=majorityNote: To find this information in Atlas, Navigate to the Overview Page>>Click on Connect(Green Button)>>Select Compass>>Here the connection string contains the cluster url: cluster0.c1sk7jd.mongodb.net/. The username and password is from Section 6 Step 3 a Under Server UIGeneral General - Change the Base and Web API Address to a name of your choice followed by the Hosted Zone Name specified in Route 53. For this environment its e3t.ayxcloud.com . Example: http://myalteryxgallery.e3t.ayxcloud.com/gallery. Important: Remember to add this FQDN/Base Address to the host file and specify the loopback address (Click here for instructions). This required to ensure the service will start. Authentication - Change to Windows Authentication. Since its joined to the newly created domain, the Default Gallery Administrator should find your username. (example: ayxtest\mitesh.narottam)Gallery PersistenceGallery-Persistence - Under Gallery persistence options, The format of the connection string is as follows. The underlined section below should be replaced with your values from Atlasmongodb+srv://username:P%40ssw0rd@host-0-1mngx.mongodb.net/AlteryxGallery?retryWrites=true&w=majorityExample: mongodb+srv://User:Password1@cluster0.clsk7jd.mongodb.net/AlteryxGallery?retryWrites=true&w=majority
   1. Database Type - User-managed MongoDB
   2. Advanced User- Managed MongoDB -  Check this box
   3. MongoDB Connection - The format of the connection string is as follows. The Underline sections below should be replaced with your values from Atlasmongodb+srv://username:P%40ssw0rd@host-0-1mngx.mongodb.net/AlteryxService?retryWrites=true&w=majorityExample: mongodb+srv://User:Password1@cluster0.clsk7jd.mongodb.net/AlteryxService?retryWrites=true&w=majorityNote: To find this information in Atlas, Navigate to the Overview Page>>Click on Connect(Green Button)>>Select Compass>>Here the connection string contains the cluster url: cluster0.c1sk7jd.mongodb.net/. The username and password is from Section 6 Step 3 a Under Server UI
   4. General General - Change the Base and Web API Address to a name of your choice followed by the Hosted Zone Name specified in Route 53. For this environment its e3t.ayxcloud.com . Example: http://myalteryxgallery.e3t.ayxcloud.com/gallery. Important: Remember to add this FQDN/Base Address to the host file and specify the loopback address (Click here for instructions). This required to ensure the service will start.
   5. Authentication - Change to Windows Authentication. Since its joined to the newly created domain, the Default Gallery Administrator should find your username. (example: ayxtest\mitesh.narottam)
   6. Gallery PersistenceGallery-Persistence - Under Gallery persistence options, The format of the connection string is as follows. The underlined section below should be replaced with your values from Atlasmongodb+srv://username:P%40ssw0rd@host-0-1mngx.mongodb.net/AlteryxGallery?retryWrites=true&w=majorityExample: mongodb+srv://User:Password1@cluster0.clsk7jd.mongodb.net/AlteryxGallery?retryWrites=true&w=majority

2. Once your happy with all other configurations, Click Next through the prompts and Finish.
3. If all parameters were entered correctly, The Alteryx Service will start and you should see database being created in Atlas. To check if the DB has been created in Atlas>>Select Database>>Browse Collections button>>Collections Tab. See example below:
4. Note: If your connection fails then check the following:Confirm your Atlas Database user account is has Atlas Admin permissions. Check the MongoDB connection string is correct, It might help to install studio 3T on the server and validate the connection works. Copy the connection string to Notepad on the machine to validate before copying into Alteryx System  Settings.Check your public IP address has been added to the Network Access list in Atlas
   1. Confirm your Atlas Database user account is has Atlas Admin permissions.
   2. Check the MongoDB connection string is correct, It might help to install studio 3T on the server and validate the connection works.
   3. Copy the connection string to Notepad on the machine to validate before copying into Alteryx System  Settings.
   4. Check your public IP address has been added to the Network Access list in Atlas

## Configure Alteryx on Server 2

1. Turn off the Alteryx Service on Server 1 and Copy the following to Server 2Runtimesettings.xmlController Token - 7630d0212aa63146563ea99d7b78d3f5d73af92ae3d169ce3f2dabf03d2050ec
2. Follow steps 2.4 to 2.12 of the Host Recovery Guide
3. Turn on the Alteryx Service on Server node 1 and follow the Encryption Key Transfer process. Once complete, turn off the Alteryx Service on Node 1 again.
4. Next we need to clear the locks in MongoDB to ensure we can test against Server 2 independently. Navigate to Atlas Console. Under Overview select Database followed by Browse Collections
5. Select the Collections Tab and Under AlteryxGallery DB find the locks collection. Hover over any locks and clear them from the collection
6. Open Alteryx System Settings. Most of the configuration will already be present in Alteryx systems settings since the runtime settings were copied from Server 1. The following steps will only cover the changes required. Under Controller>>PersistenceReinsert the MongoDB Connection String from Section 7 Step 3 - Example: mongodb+srv://User:Password1@cluster0.clsk7jd.mongodb.net/AlteryxGallery?retryWrites=true&w=majority
   1. Reinsert the MongoDB Connection String from Section 7 Step 3 - Example: mongodb+srv://User:Password1@cluster0.clsk7jd.mongodb.net/AlteryxGallery?retryWrites=true&w=majority

**Server UI>>General**

b. **General** - Change the Base and Web API Address to a name of your choice. I named mine **myalteryxgallery.e3t.ayxcloud.com** and added a record to loopback address in the host file configuration for an easier setup. Click [Here](https://knowledge.alteryx.com/index/s/article/How-and-Why-to-do-a-Hosts-file-modification)for steps on how to do this.

c. **Gallery Persistence** - Under Gallery persistence options, Reinsert the Web Persistence Connection string from [Section 7 Step 3f](#Gallery-Persistence). Example: mongodb+srv://**User**:**Password1**@[cluster0.clsk7jd.mongodb.net](http://cluster0.clsk7jd.mongodb.net)/AlteryxGallery?retryWrites=true&w=majority

d. If all parameters where entered correctly, The Alteryx Service will start

1. Note: If your connection fails check the following:Confirm your Atlas Database User account is has Atlas Admin permissions.Check the MongoDB connection string is correct, It might help to install studio 3T on the server and validate the connection works. Copy the connection string to Notepad on the machine to validate before copying into Alteryx System  Settings.Check your public IP address has been added to the Network Access list in Atlas
   1. Confirm your Atlas Database User account is has Atlas Admin permissions.
   2. Check the MongoDB connection string is correct, It might help to install studio 3T on the server and validate the connection works.
   3. Copy the connection string to Notepad on the machine to validate before copying into Alteryx System  Settings.
   4. Check your public IP address has been added to the Network Access list in Atlas

---

# Section 8 - Configure Windows Firewall, EC2 with Secondary IPs, Route 53 and DNS

> **ℹ️ Info**
>
> Route 53 needs to be configured to allow the Base Address to resolve both within the domain and outside the domain (but still restricted to the VPC). To ensure name resolution and connectivity is not blocked between nodes please disable windows firewall on all nodes. Customers may not be able to do this due to internal security policies and IT restrictions, in which case please advise them to work with there IT to set the appropriate windows firewall rules.

## Windows Firewall

1. To disable windows firewall, Remote into DC01, Click on the windows icon, search and select Windows Defender Firewall, Click on Turn of Windows Defender Firewall on or off
2. Select Turn off Windows Defender Firewall for Domain, Private and Public. Click OK
3. Do the Same for DC02, Server 1 and Server 2

## Configure EC2 with Secondary IP Address in AWS

1. Next we need to assign the Cluster Role IP addresses to the appropriate Server machines in AWS as secondary IP addresses. The refers to Section 4 Step 26. Server 1 - Secondary Cluster Role IP address is 172.27.205.68Server 2 - Secondary Cluster Role IP address is 172.27.205.4Log into the AWS Management Console and select EC2 instances tab. Check Server 1 and Navigate to Actions>>Networking>>Manage IP AddressesExpand the eth0 drop down, Click on Assign new IP Address and Enter the IP Address (172.27.205.68). Then click Save
   1. Log into the AWS Management Console and select EC2 instances tab. Check Server 1 and Navigate to Actions>>Networking>>Manage IP Addresses
   2. Expand the eth0 drop down, Click on Assign new IP Address and Enter the IP Address (172.27.205.68). Then click Save

2. Do the same for Server 2 except the IP Address to assign is 172.27.205.4

## Configure Route 53Configure-Route-53

1. Go to Home Page of AWS Management Console, Search and Select Route 53. Click on Hosted Zones, then et3.ayxcloud.com
2. Click on Create Record
3. Enter the following to create a Failover DNS Record, then Click Create RecordsUnder Record Name: myalteryxgalleryRecord Type: A - Routes traffic to an IPv4 address and some AWS ResourcesValue: The Cluster Role IP Addresses (example: 172.27.205.4 and 172.27.205.68)Routing Policy: FailoverFailover record type: SecondaryRecord ID: 1
   1. Under Record Name: myalteryxgallery
   2. Record Type: A - Routes traffic to an IPv4 address and some AWS Resources
   3. Value: The Cluster Role IP Addresses (example: 172.27.205.4 and 172.27.205.68)
   4. Routing Policy: Failover
   5. Failover record type: Secondary
   6. Record ID: 1

4. Once this completed You Environment is setup.

## Configure DNS for Internal Name Resolution

The DNS Server must be configured with an alias record to ensure internal name resolution is possible.

1. Log into DC01 or 2 and open the DNS Manager Console. Expand Forward Lookup Zones and the domain you created (example: ayxtest.ayx)
2. Right-click the empty space and select New Alias (CNAME).
3. Under the Alias name enter the Base Gallery URL without https and gallery. Example: myalteryxgallery.e3t.ayxcloud.com. Then select browse, Double click on the node until you get to the forward lookup zone, then select the Host (A) Record which is the failover cluster role (example: cs-cltr-1.ayxtest.ayx) and Click Ok

---

# Complete

[no content here, yet]