---
id: ba71bd942dd96673
title: SSL Utility - Powershell Script to Monitor SSL Certificate for Expiration
status: review
source:
  kind: confluence_page
  id: confluence-page:3255763378
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/3255763378
  summary: null
category:
- product-tree
- alteryx-server
- authentication
keywords:
- ssl
- certificate
- powershell
- expiration
- monitoring
- utility
created_at: 2026-04-14T15:18:28Z
updated_at: 2026-04-14T15:20:49Z
confidence: 0.87
cross_refs: []
content_hash: sha256:e8a8bb52095e0cd299c082c2aa631481b69535d6a17e04ea46b663696e005d50
confluence_page_id: null
model_used: claude-sonnet-4-6
---

> **ℹ️ Info**
>
> Script customers can schedule to be alerted when their SSL Certificate is close to expiring.
> 
> Created by Mathew Sebastion

| **Access** | **📘 Certificate Expiry Monitor: Full Implementation Guide**  **✅ 1. Script Overview**  This PowerShell script:     - Searches for a certificate by thumbprint in the Windows LocalMachine\My store,    - Calculates days left until expiry,    - Sends an email alert if expiry is within a defined threshold (e.g., 30 days),    - Uses a secure password file (smtp-cred.xml) to avoid storing plain-text credentials.  **📁 2. Folder Structure (Suggested)**  C:\ └── Temp\     ├── Check-CertExpiry.ps1     └── smtp-cred.xml         <-- Encrypted credentials  **🔐 3. Step-by-Step: Secure Credential Storage**  **➤ Run this manually once (in PowerShell):**  powershell  # This prompts for SMTP username/password and encrypts it Get-Credential \| Export-Clixml -Path "C:\Temp\smtp-cred.xml"  **🔐 What This Does:**     - Saves the credential in encrypted form.    - Only readable by the same user on the same machine.    - No passwords stored in plain text.  **📜 4. Final Script: Check-CertExpiry.ps1**  Save the following script as C:\Temp\Check-CertExpiry.ps1:  1. Replace the thumprint with the Alteryx SSL Certificate     1. Update your SMTP Settings to trigger email alerts  powershell script  # === CONFIGURATION === $targetThumbprint = "**f6c74f278df7bdd4e54d9c42fd4dd1138c2d320c**" $storeLocation = "LocalMachine" $storeName = "My" $expiryThresholdDays = 30  # === EMAIL SETTINGS === $smtpServer = "[mail.xxx.com](http://mail.alteryx.com)"  $smtpPort = 25 $from = "[xxx.email.com](mailto:mathew.sebastian@alteryx.com)" $to = "[xxx.email.com](mailto:mathew.sebastian@alteryx.com)" $secureCred = Import-Clixml -Path "C:\Temp\smtp-cred.xml"  # === SANITIZE THUMBPRINT === $normalizedThumbprint = ($targetThumbprint -replace '[^\da-fA-F]', '').ToLower()  # === OPEN CERT STORE === try {     $store = New-Object System.Security.Cryptography.X509Certificates.X509Store($storeName, $storeLocation)     $store.Open("ReadOnly") } catch {     Write-Error "Could not open certificate store. Error: $_"     return }  # === FIND CERT === $cert = $store.Certificates \| Where-Object {     ($_.Thumbprint -replace '[^\da-fA-F]', '').ToLower() -eq $normalizedThumbprint }  # === HANDLE MATCH === if ($cert) {     $daysLeft = ($cert.NotAfter - (Get-Date)).Days     Write-Host "Certificate found:"     Write-Host "  Subject: $($cert.Subject)"     Write-Host "  Thumbprint: $($cert.Thumbprint)"     Write-Host "  Expiry Date: $($cert.NotAfter)"     Write-Host "  Days Left: $daysLeft"  if ($daysLeft -le $expiryThresholdDays) {         Write-Warning "Certificate is expiring in $daysLeft day(s)!"  $subject = "SSL Certificate Expiry Warning: $($cert.Subject)"         $body = @" The following certificate is expiring soon:  Subject: $($cert.Subject) Thumbprint: $($cert.Thumbprint) Expiry Date: $($cert.NotAfter) Days Remaining: $daysLeft  Please take appropriate action.  -Certificate Monitor Script "@  Send-MailMessage -From $from -To $to -Subject $subject -Body $body -SmtpServer $smtpServer -Port $smtpPort -UseSsl -Credential $secureCred     } } else {     Write-Error "❌ No certificate found matching the thumbprint: $normalizedThumbprint" }  $store.Close()  **⏱️ 5. Schedule via Task Scheduler**  **🧭 Open Task Scheduler**     1. Press Win + R, type taskschd.msc, and hit Enter.    2. Click "Create Task…" (not Basic Task) for more control.  **✅ Task Settings**  **General Tab**     - Name: Certificate Expiry Monitor    - Run as: Your user (the one that created smtp-cred.xml)    - ✅ Check: “Run with highest privileges”  **Triggers Tab**     - New…Begin the task: On a scheduleE.g., Daily at 9:00 AM       - Begin the task: On a schedule       - E.g., Daily at 9:00 AM   **Actions Tab**     - New…Program/script:powershell.exeAdd arguments:-ExecutionPolicy Bypass -File "C:\Temp\Check-CertExpiry.ps1"       - Program/script:powershell.exe       - Add arguments:-ExecutionPolicy Bypass -File "C:\Temp\Check-CertExpiry.ps1" |
| --- | --- |
| **How to Schedule it** | tbd |