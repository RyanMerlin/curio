# ── Workforce Identity Federation (WIF) — Entra ID SSO ────────────────────
#
# WIF lets Google Cloud IAP delegate authentication to your corporate Entra ID
# (Azure AD) tenant. Users log in with their existing corporate credentials;
# no Google accounts needed.
#
# ┌─────────────────────────────────────────────────────────────────────────┐
# │  Entra ID app registration — do this BEFORE running terraform apply     │
# ├─────────────────────────────────────────────────────────────────────────┤
# │  1. Entra ID → App registrations → New registration                     │
# │     Name: "Curio GCP IAP"                                               │
# │     Supported account types: single-tenant (this org only)              │
# │     Redirect URI: Web → (run terraform apply first to get this value,   │
# │       then update the registration with the entra_redirect_uri output)  │
# │                                                                         │
# │  2. In the app registration:                                            │
# │     API permissions → Add: openid, profile, email, offline_access       │
# │     Token configuration → Optional claims → id_token: email, upn        │
# │     (Optional) Token configuration → Groups → Security groups           │
# │     Certificates & secrets → New client secret → copy the value         │
# │                                                                         │
# │  3. Note from the app Overview page:                                    │
# │     Application (client) ID  → var.entra_client_id                      │
# │     Directory (tenant) ID    → var.entra_tenant_id                      │
# │     Client secret value      → var.entra_client_secret                  │
# └─────────────────────────────────────────────────────────────────────────┘

resource "google_iam_workforce_pool" "main" {
  workforce_pool_id = "${var.service_name}-workforce"
  parent            = "organizations/${var.organization_id}"
  location          = "global"
  display_name      = "Curio Workforce Pool"
  description       = "Workforce pool for Curio control plane — Entra ID SSO"
  session_duration  = "3600s"
}

resource "google_iam_workforce_pool_provider" "entra_id" {
  workforce_pool_id = google_iam_workforce_pool.main.workforce_pool_id
  provider_id       = "entra-id"
  location          = "global"
  display_name      = "Microsoft Entra ID"

  # Map Entra ID JWT claims to Google principal attributes.
  # google.subject uses sub (a stable GUID) rather than email, so renames
  # don't silently create new principals.
  attribute_mapping = {
    "google.subject"      = "assertion.sub"
    "google.display_name" = "assertion.name"
    "google.groups"       = "assertion.groups"
    "attribute.email"     = "assertion.email"
    "attribute.upn"       = "assertion.preferred_username"
  }

  # Hard-restrict to your corporate domain so that guest accounts and
  # users from other tenants are rejected even if the OIDC issuer matches.
  attribute_condition = "attribute.email.endsWith('@${var.corporate_domain}')"

  oidc {
    issuer_uri = "https://login.microsoftonline.com/${var.entra_tenant_id}/v2.0"
    client_id  = var.entra_client_id

    # Authorization code flow is required for IAP browser login
    web_sso_config {
      response_type             = "CODE"
      assertion_claims_behavior = "MERGE_USER_INFO_OVER_ID_TOKEN_CLAIMS"
    }

    client_secret {
      value {
        plain_text = var.entra_client_secret
      }
    }
  }
}

# ── IAP access binding ────────────────────────────────────────────────────
# Grants all workforce pool members roles/iap.httpsResourceAccessor on the
# Curio backend service. The attribute_condition above already restricts pool
# membership to corporate-domain users.

resource "google_iap_web_backend_service_iam_binding" "workforce_access" {
  project             = var.project_id
  web_backend_service = google_compute_backend_service.curio.name
  role                = "roles/iap.httpsResourceAccessor"

  members = [
    "principalSet://iam.googleapis.com/${google_iam_workforce_pool.main.name}/*",
  ]
}
