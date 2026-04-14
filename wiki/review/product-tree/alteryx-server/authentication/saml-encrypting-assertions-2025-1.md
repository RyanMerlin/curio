---
id: 7c38a39859629956
title: SAML Encrypting Assertions 2025.1
status: review
source:
  kind: confluence_page
  id: confluence-page:3061187123
  origin_url: https://alteryx.atlassian.net/wiki/spaces/CURIO/pages/3061187123
  summary: null
category:
- product-tree
- alteryx-server
- authentication
keywords:
- saml
- encryption
- assertions
- '2025.1'
- sso
created_at: 2026-04-14T13:40:14Z
updated_at: 2026-04-14T13:50:45Z
confidence: 0.88
cross_refs: []
content_hash: sha256:cf767912592201c53b3eecc56054a8e4af5ec274a2f1db918933ef96963fb81b
confluence_page_id: null
model_used: claude-sonnet-4-6
---

> **ℹ️ Info**
>
> Starting from 2025.1 We now support SAML assertion encryption,  This article covers aspects of this feature and what to look out for when troubleshooting.

---

---

# What are SAML Assertions and Encryption?

SAML assertions are the messages that are exchanged between an identity provider (IdP) and service provider (SP) that confidentially identify who a user is, what pertinent information exists about them, and what they're authorized or entitled to access.

The (SP) Assertion consumer Service validates the SAML Response(from the IDP) to ensure the information provided is valid before providing access.

---

# Examples of Assertions

- Attributes such as firstname, lastname email
- X509 Certificate
- Conditions such as the validity period of the SAML response

Using CA signed certificate we can encrypt and sign the above values to ensure its not displayed in a trace(such as Fiddler or SAML Tracer) and provide another layer of security. The public/private keypair is used decrypt the values before parsing to the SPs Assertion Consumer Service for authentication/authorization

### Typical Example

The following is a typical example of Azure SAML Response showing all values. Notice some of the highlighted values displaying the Assertions and attributes.

> **ℹ️ Info**
>
> <samlp:Response xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol"
>                 ID="_b3056067-2c4e-4986-a339-f3e72f7561f7"
>                 Version="2.0"
>                 IssueInstant="2025-05-20T11:19:16.579Z"
>                 Destination="<https://win-8a4sosipqnt.ayx.ayx/webapi/Saml2/Acs>"
>                 InResponseTo="ide3c324bff3394b07b01bb3b05a72ef4f">
>     <Issuer xmlns="urn:oasis:names:tc:SAML:2.0:assertion"><<https://sts.windows.net/30f6e3b6-e2ba-458d-bc44-60528bee0bd0/%3C/Issuer%3E%3E> 
>     [samlp:Status](#)
>         <samlp:StatusCode Value="urn:oasis:names:tc:SAML:2.0:status:Success" />
>     </samlp:Status>
>    ** <Assertion xmlns="urn:oasis:names:tc:SAML:2.0:assertion"**
> **               ID="_d0455c45-8e67-4251-bcdb-a28a3d980100"**
> **               IssueInstant="2025-05-20T11:19:16.575Z"**
> **               Version="2.0">**
>         <Issuer><<https://sts.windows.net/30f6e3b6-e2ba-458d-bc44-60528bee0bd0/%3C/Issuer%3E%3E> 
>         <Signature xmlns="<http://www.w3.org/2000/09/xmldsig#> ">
>             <SignedInfo>
>                 <CanonicalizationMethod Algorithm="<http://www.w3.org/2001/10/xml-exc-c14n#> " />
>                 <SignatureMethod Algorithm="<http://www.w3.org/2001/04/xmldsig-more#rsa-sha256> " />
>                 <Reference URI="#_d0455c45-8e67-4251-bcdb-a28a3d980100">
>                     <Transforms>
>                         <Transform Algorithm="<http://www.w3.org/2000/09/xmldsig#enveloped-signature> " />
>                         <Transform Algorithm="<http://www.w3.org/2001/10/xml-exc-c14n#> " />
>                     </Transforms>
>                     <DigestMethod Algorithm="<http://www.w3.org/2001/04/xmlenc#sha256> " />
>                     <DigestValue>nifYJfcwJS9/12BHIQIRTkxYfiacs0VuFCKse3BB8/o=</DigestValue>
>                 </Reference>
>             </SignedInfo>
>             **<SignatureValue>G91wI5pAmYfDmPgiJXsgHIyZBnZu11g6MM4cBUs/FsIL9jRuehsWSIgZw1f9qeD+XNZDUS7kVW70QsbP8PB73MdwItzig5fBhm+jsl3XeiU9XwaTQD5PnC24d17HklqOX4uRLHalWFdQS5c2FPPu1p8DAEosPj/cv/P5ZdijxXHKcnccNLluEYghnJCHmVyJlfRDI07hJOim2xqg2e9Y99Yx/IB/TN9SBX0jsbcPnlyZU5QMoQczlf75x0yrFqYpaLola1DPKzQlEyvQvq4H4id2T9ndiHigeKeDnwYcgVEZveniyHAvzLNNPBFFkcK3uoiJ989HQ3WDXuRoiLBzDA==</SignatureValue>									**	   
>             <KeyInfo>
>                 <X509Data>
>                     **<X509Certificate>MIIC8DCCAdigAwIBAgIQd2yxlloAdYxGgD7846TzgTANBgkqhkiG9w0BAQsFADA0MTIwMAYDVQQDEylNaWNyb3NvZnQgQXp1cmUgRmVkZXJhdGVkIFNTTyBDZXJ0aWZpY2F0ZTAeFw0yMzEwMDIxNDExNDhaFw0yNjEwMDIxNDExNDdaMDQxMjAwBgNVBAMTKU1pY3Jvc29mdCBBenVyZSBGZWRlcmF0ZWQgU1NPIENlcnRpZmljYXRlMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAnLBqgjjYckhfdhDaK5OTAphcwUNDjzSJbRkbxUl+oky4k0R/zG/mMmcuU+qQLfi2WZ8EUVry9CRfnxbgq/B0rlMiCjFnp0VKNn3pB5DZaz1fEgOG6pXFfM9sbkzrdRnHkzGNLyiYDP/TNO2wvx9u2y4JdAz6Qfhx2CzAystWBZtjnJKwYzbwygcClhycTxNHqq1sRO7ZmkPbF6nNxQxz3ev+dKHYz4VTPOw8QIHuE0P+T/0dszC0oVyw943Q+da7Fo450bzjsP2IsBOr1e4b3Tjvn2S+l3X1sXK5seyBidm6JIvQzx0dZglSdkviRvsQwsRULfAfPgr/nV2PreQ1GQIDAQABMA0GCSqGSIb3DQEBCwUAA4IBAQBew0QESMB2m4zgN432rC2tJhrjxJJXI4JZFbjwq9LxwVf5KUn/8SwlkZqCroMYyJc7LMI6weag9x9v8noqHC8iP+YjZQ9NytxQC+UoaNtcTAFo348jh9YK8cSCpqIlnQi5qO7/xBShSQ0mbHAiVGfeOmtchN6GzErh/MizmGJO3En2pePffclFOMAcO8yhydSGGOz4xV72YBx1DX0xPN6XF4454LGmRjDEERDnXya9+LLEQ+xnk35QHYW6gQknOZlXErDdQspBtej+yZv08GQ1LJSUeb3NlM1vcZc/AR4fbrB8JRTBktzRN3Z9OQMfHFWMra3GIjS+uQoy4uErzx+y</X509Certificate>**
>                 </X509Data>					
>             </KeyInfo>
>         </Signature>
>         <Subject>
>            ** <NameID Format="urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress">**[mitesh.narottam@alteryx.com](mailto:mitesh.narottam@alteryx.com)**</NameID>**
> **            <SubjectConfirmation Method="urn:oasis:names:tc:SAML:2.0:cm:bearer">**
> **                <SubjectConfirmationData InResponseTo="ide3c324bff3394b07b01bb3b05a72ef4f"**
> **                                         NotOnOrAfter="2025-05-20T12:19:16.288Z"**
> **                                         Recipient="**<https://win-8a4sosipqnt.ayx.ayx/webapi/Saml2/Acs>**"/>**
>             </SubjectConfirmation>
>         </Subject>
>         **<Conditions NotBefore="2025-05-20T11:14:16.288Z"**
> **                    NotOnOrAfter="2025-05-20T12:19:16.288Z"**
>                     >
>             <AudienceRestriction>
>                 <Audience><[https://win-8a4sosipqnt.ayx.ayx/webapi/Saml2</Audience>>](https://win-8a4sosipqnt.ayx.ayx/webapi/Saml2%3C/Audience%3E%3E)
>             </AudienceRestriction>
>         </Conditions>
>         <AttributeStatement>
>             <Attribute Name="<http://schemas.microsoft.com/identity/claims/tenantid>">
>                 <AttributeValue>30f6e3b6-e2ba-458d-bc44-60528bee0bd0</AttributeValue>
>             </Attribute>
>             <Attribute Name="<http://schemas.microsoft.com/identity/claims/objectidentifier>">
>                 <AttributeValue>b64ff4ee-5aef-43be-b767-34a5916d8c03</AttributeValue>
>             </Attribute>
>             <Attribute Name="<http://schemas.microsoft.com/identity/claims/identityprovider>">
>                 <AttributeValue><<https://sts.windows.net/522f39d9-303d-488f-9deb-a6d77f1eafd8/%3C/AttributeValue%3E%3E> 
>             </Attribute>
>             <Attribute Name="<http://schemas.microsoft.com/claims/authnmethodsreferences>">
>                 <AttributeValue><[http://schemas.microsoft.com/ws/2008/06/identity/authenticationmethod/password</AttributeValue>>](http://schemas.microsoft.com/ws/2008/06/identity/authenticationmethod/password%3C/AttributeValue%3E%3E)
>             </Attribute>
>   **          <Attribute Name="firstName">**
> **                <AttributeValue>Mitesh</AttributeValue>**
>             </Attribute>
>      **       <Attribute Name="lastName">**
> **                <AttributeValue>Narottam</AttributeValue>**
>             </Attribute>
>           **  <Attribute Name="email">**
> **                <AttributeValue>**[mitesh.narottam@alteryx.com](mailto:mitesh.narottam@alteryx.com)**</AttributeValue>**
>             </Attribute>
>         </AttributeStatement>
>         <AuthnStatement AuthnInstant="2025-05-20T11:15:44.607Z"
>                         SessionIndex="_d0455c45-8e67-4251-bcdb-a28a3d980100"
>                         >
>             <AuthnContext>
>                 <AuthnContextClassRef>urn:oasis:names:tc:SAML:2.0:ac:classes:Password</AuthnContextClassRef>
>             </AuthnContext>
>         </AuthnStatement>
>     </Assertion>
> </samlp:Response>

### Encrypted SAML Example

When encrypted, the SAML Response Changes to

> **ℹ️ Info**
>
> <samlp:Response xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol"
>                 ID="_1d0164cc-d888-4bbe-a592-9691d593ef51"
>                 Version="2.0"
>                 IssueInstant="2025-05-20T11:46:54.689Z"
>                 Destination="<https://win-8a4sosipqnt.ayx.ayx/webapi/Saml2/Acs>"
>                 InResponseTo="id0f8b097a777c4000b671df3a24735c2b"
>                 >
>     <Issuer xmlns="urn:oasis:names:tc:SAML:2.0:assertion"><<https://sts.windows.net/30f6e3b6-e2ba-458d-bc44-60528bee0bd0/%3C/Issuer%3E%3E> 
>     [samlp:Status](#)
>         <samlp:StatusCode Value="urn:oasis:names:tc:SAML:2.0:status:Success" />
>     </samlp:Status>
>    ** <EncryptedAssertion xmlns="urn:oasis:names:tc:SAML:2.0:assertion">**
> **        <xenc:EncryptedData xmlns:xenc="**<http://www.w3.org/2001/04/xmlenc#>** "**
> **                            Type="**<http://www.w3.org/2001/04/xmlenc#Element>** "**
> 
> ** <xenc:EncryptionMethod Algorithm="<**<http://www.w3.org/2001/04/xmlenc#aes256-cbc>** >" />**
> **        <KeyInfo xmlns="<**<http://www.w3.org/2000/09/xmldsig#>** >">**
> 
> **  <e:EncryptedKey xmlns:e="<<**<http://www.w3.org/2001/04/xmlenc#>** >>">**
> **            <e:EncryptionMethod Algorithm="<<**<http://www.w3.org/2001/04/xmlenc#rsa-oaep-mgf1p>** >>">**
> 
> **<DigestMethod Algorithm="<<<**<http://www.w3.org/2000/09/xmldsig#sha1>** >>>" />**
> **        </e:EncryptionMethod>**
> **        <KeyInfo>**
> **            <o:SecurityTokenReference xmlns:o="<<<**<http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-wssecurity-secext-1.0.xsd>**>>>">**
> **                <X509Data>**
> **                    <X509IssuerSerial>**
> **                        <X509IssuerName>**[E=mitesh.narottam@alteryx.com](mailto:E=mitesh.narottam@alteryx.com)**, CN=WIN-8A4SOSIPQNT.ayx.ayx, OU=Alteryx, O=Alteryx, L=London, S=uk, C=UK</X509IssuerName>**
> **                        <X509SerialNumber>705589322655184988706920421358426712185360904869</X509SerialNumber>**
> **                    </X509IssuerSerial>**
> **                </X509Data>**
> **            </o:SecurityTokenReference>**
> **        </KeyInfo>**
> **        <e:CipherData>**
> **            <e:CipherValue>LjVyAs89Cw+beqHwM3FxpReIw2cJNIhtBDw0td80Bms8LzzJkW3fc9jg7EQXzfnY0zDOxqpQ6kqxWDV12ypE6v8QnuE0XBaoteQ5XUJfhVAZHlOVRUYdgBVQE10V9uWS+l08lekKVsE8VxGjcSHv8tHiYBiL/EPY2zGSsN9Z8TjHdgLX3uwc0yS1ALAPNK0anvw2yTAvqGEiGrzEy+hvJnxcLHG2paDVJKS5vg7sq1559Tys3oOJOTFSbQNPsHLgXuR/17jcYjT/o8JUcMi8sbjTI/+qs78s6Qar+vVpamHaK15X7Tzq+oegYllJnnxEMeLsEP/7yvcLg0si/D3Z9A==</e:CipherValue>**
> **        </e:CipherData>**
> **    </e:EncryptedKey>**
> **</KeyInfo>**
> [xenc:CipherData](#)
> **    **[xenc:CipherValue](#)**7ZEiVBzl8oJG65n5v+13NgxYEGcA6h9mXN3trIfYaabWhdEMA3/syn3tHaSIpTwBMK5fJNvdx2A2wDLpimYyL62E5ZUv6TM9M940ZZhVNxoJMw4jq44Gi2FFP31m6eGx7VdvnNUEkS4KqUXUfEpJ0xU4FvacNL0xjqHHyzymZKDEE2HOx7HfumXyBrTDCvhNS4LsuyvGfWzI347VFfYY5AeQoecjl7iz86yYQEJy1v4hkKQM8cW7B73XSXtQ==</xenc:CipherValue>**
> **</xenc:CipherData>**

Example of Okta

---

# How to setup SAML token encryption to use Encrypted SAML Assertions in Azure.

This section assumes the Basic SAML Configuration is setup in Azure, Alteryx system settings and a preconfigured cert with the private key issued from a valid CA

1. Login into the Azure Portal https://portal.azure.com/
2. Select Microsoft Entra ID and Enterprise application
3. Search select you SAML Application
4. Select Token Encryption
5. Select Import Certificate and Upload your public cert (.crt)to azure
6. Once uploaded, this will need to be activated. select the 3 dots on the right, then Activate token encryption certificate
7. Ensure the associated private key is installed in the window store on the Alteryx server and confirm all cert dependencies/chain is correctly located on your machine
8. Open Alteryx system settings on the server and Navigate to Server UI>>Authentication
9. Check Encrypt Assertions
10. Under Decryption Certificate Hash, Enter the Thumbprint of the from the private key. Example below:

1. Click Next through all the prompts followed by Finish. If successful the Alteryx service will start successfully and any SAML traces will present the SAML Assertions/Attributes as encrypted.

# How to setup SAML token encryption to use Encrypted SAML Assertions in Okta 

This section assumes the Basic SAML Configuration is setup in Okta, Alteryx system settings and a preconfigured cert with the private key issued from a valid CA

1. Login into the Okta Portal
2. Select Applications from the left navigation pane
3. Search select you SAML Application
4. Select General>>Click Edit on SAML Settings>>Next
5. Under SAML Settings, select show advanced settings
6. Under Assertion Encryption change the dropdown to Encrypted (you can leave the algorithms to there defaults)
7. Under Encryption Certificate>>Click on browse and upload the public key crt.
8. Click Next and Finish to complete the setup
9. Ensure the associated private key is installed in the window store on the Alteryx server and confirm all cert dependencies/chain is correctly located on your machine
10. Open Alteryx system settings on the server and Navigate to Server UI>>Authentication
11. Check Encrypt Assertions
12. Under Decryption Certificate Hash, Enter the Thumbprint of the from the private key. Example below:

---

# Troubleshooting

## What if a decryption hash is inserted incorrectly? 

The Alteryx service will fail to start and sso logs will show the following:

- ERROR,1,AlteryxServerWebApiHost,ssoLogger,ConfigureSamlIdentityProvider,,,,WIN-8A4SOSIPQNT,,,,,,Exception thrown when configuring SSO IdP.,"System.Exception: Certificate with thumbprint '76634fd5bc020d515fd8218cc7206c9bcb47c91' not found in LocalMachine/Personal store->   at Alteryx.Server.WebApiHost.Services.Impl.Saml2Service.GetX509Certificate2(String certHash)->   at Alteryx.Server.WebApiHost.Services.Impl.Saml2Service.ConfigureSamlIdentityProvider(ILogger ssoLogger, Saml2AuthenticationOptions saml2options)"

## What if the Pki is not available in the Windows certificate store?

When attempting to Verify the IDP you may see

Or

After signing in you may see

Or

The service will fail to start.

Check the sso logs (typically located in C:\ProgramData\Alteryx\logs) for the confirmation.

- ERROR,1,AlteryxServerWebApiHost,ssoLogger,ConfigureSamlIdentityProvider,,,,WIN-8A4SOSIPQNT,,,,,,Exception thrown when configuring SSO IdP.,"System.Exception: Certificate with thumbprint '76634fd5bc020d515fd8218cc7206c9bcb47c91' not found in LocalMachine/Personal store->   at Alteryx.Server.WebApiHost.Services.Impl.Saml2Service.GetX509Certificate2(String certHash)->   at Alteryx.Server.WebApiHost.Services.Impl.Saml2Service.ConfigureSamlIdentityProvider(ILogger ssoLogger, Saml2AuthenticationOptions saml2options)"

## What if Encrypt Assertions is enabled on the IDP but not in Alteryx?

Alteryx will not be able to validate the assertions as the assertions are encrypted with a public key. This will present the following:

or

---

# Defect

Cosmetic defect when resizing Alteryx system settings.

GCSE-330277dcf2c9-72f3-3ff6-8e99-fe88e9f473f1System Jira