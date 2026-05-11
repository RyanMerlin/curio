# ── IAP + Load Balancer stack ──────────────────────────────────────────────
#
# Architecture:
#   Internet → Cloud Armor (IP allowlist) → External HTTPS LB
#           → IAP (Entra ID auth via WIF) → Serverless NEG → Cloud Run
#
# Pub/Sub push bypasses the LB and hits Cloud Run directly (it is
# "internal" traffic and uses its own OIDC verification in the service).
#
# After running `terraform apply`:
#   1. Point your DNS A record for var.domain at the load_balancer_ip output.
#   2. Wait for the managed SSL cert to provision (~10-30 min after DNS propagates).
#   3. Run the post-apply command from the iap_audience output to wire auth in Cloud Run.

# ── Global static IP ──────────────────────────────────────────────────────

resource "google_compute_global_address" "curio" {
  name    = "${var.service_name}-ip"
  project = var.project_id

  depends_on = [google_project_service.required]
}

# ── Managed SSL certificate ────────────────────────────────────────────────

resource "google_compute_managed_ssl_certificate" "curio" {
  name    = "${var.service_name}-cert"
  project = var.project_id

  managed {
    domains = [var.domain]
  }

  depends_on = [google_project_service.required]
}

# ── Serverless NEG → Cloud Run ─────────────────────────────────────────────

resource "google_compute_region_network_endpoint_group" "curio" {
  name                  = "${var.service_name}-neg"
  network_endpoint_type = "SERVERLESS"
  region                = var.region
  project               = var.project_id

  cloud_run {
    service = google_cloud_run_v2_service.curio.name
  }

  depends_on = [google_project_service.required]
}

# ── IAP OAuth2 client ──────────────────────────────────────────────────────
# The IAP OAuth Admin API (google_iap_brand / google_iap_client) is retired
# as of early 2026. Create the OAuth client manually before running apply:
#
#   GCP Console → APIs & Services → Credentials → Create Credentials
#   → OAuth client ID → Application type: Web application
#   → Name: "Curio IAP"
#   → Add redirect URI: the entra_redirect_uri output value
#   → Copy Client ID and Secret → set as iap_oauth2_client_id /
#     iap_oauth2_client_secret in terraform.tfvars
#
# Also configure the OAuth consent screen (Internal, your org domain) if you
# have not already done so for this project.

# ── Cloud Armor security policy (VPN / IP restriction) ─────────────────────
# Created only when corporate_ip_ranges is non-empty.
# This is the network-layer enforcement:
#   requests not coming from these CIDRs are blocked at the LB before
#   reaching IAP or the backend.
# Use your corporate VPN's egress IPs and office gateway IPs here.

resource "google_compute_security_policy" "curio" {
  count   = length(var.corporate_ip_ranges) > 0 ? 1 : 0
  name    = "${var.service_name}-armor"
  project = var.project_id

  rule {
    action      = "allow"
    priority    = 1000
    description = "Allow corporate network and VPN egress CIDRs"
    match {
      versioned_expr = "SRC_IPS_V1"
      config {
        src_ip_ranges = var.corporate_ip_ranges
      }
    }
  }

  rule {
    action      = "deny(403)"
    priority    = 2147483647
    description = "Default deny"
    match {
      versioned_expr = "SRC_IPS_V1"
      config {
        src_ip_ranges = ["*"]
      }
    }
  }

  depends_on = [google_project_service.required]
}

# ── Backend service with IAP enabled ──────────────────────────────────────

resource "google_compute_backend_service" "curio" {
  name                  = "${var.service_name}-backend"
  project               = var.project_id
  load_balancing_scheme = "EXTERNAL_MANAGED"
  protocol              = "HTTPS"
  enable_cdn            = false

  backend {
    group = google_compute_region_network_endpoint_group.curio.id
  }

  iap {
    enabled              = true
    oauth2_client_id     = var.iap_oauth2_client_id
    oauth2_client_secret = var.iap_oauth2_client_secret
  }

  security_policy = length(var.corporate_ip_ranges) > 0 ? google_compute_security_policy.curio[0].id : null
}

# ── HTTPS load balancer ────────────────────────────────────────────────────

resource "google_compute_url_map" "curio" {
  name            = "${var.service_name}-url-map"
  project         = var.project_id
  default_service = google_compute_backend_service.curio.id
}

resource "google_compute_target_https_proxy" "curio" {
  name             = "${var.service_name}-https-proxy"
  project          = var.project_id
  url_map          = google_compute_url_map.curio.id
  ssl_certificates = [google_compute_managed_ssl_certificate.curio.id]
}

resource "google_compute_global_forwarding_rule" "curio_https" {
  name                  = "${var.service_name}-https"
  project               = var.project_id
  ip_protocol           = "TCP"
  load_balancing_scheme = "EXTERNAL_MANAGED"
  port_range            = "443"
  target                = google_compute_target_https_proxy.curio.id
  ip_address            = google_compute_global_address.curio.id
}

# ── HTTP → HTTPS redirect ──────────────────────────────────────────────────

resource "google_compute_url_map" "http_redirect" {
  name    = "${var.service_name}-http-redirect"
  project = var.project_id

  default_url_redirect {
    https_redirect         = true
    redirect_response_code = "MOVED_PERMANENTLY_DEFAULT"
    strip_query            = false
  }
}

resource "google_compute_target_http_proxy" "http_redirect" {
  name    = "${var.service_name}-http-redirect"
  project = var.project_id
  url_map = google_compute_url_map.http_redirect.id
}

resource "google_compute_global_forwarding_rule" "curio_http" {
  name                  = "${var.service_name}-http"
  project               = var.project_id
  ip_protocol           = "TCP"
  load_balancing_scheme = "EXTERNAL_MANAGED"
  port_range            = "80"
  target                = google_compute_target_http_proxy.http_redirect.id
  ip_address            = google_compute_global_address.curio.id
}
