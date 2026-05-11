variable "project_id" {
  description = "Google Cloud project ID."
  type        = string
}

variable "region" {
  description = "Google Cloud region for Cloud Run and Pub/Sub."
  type        = string
}

variable "image" {
  description = "Container image URI for the Curio Cloud Run service."
  type        = string
}

variable "service_name" {
  description = "Cloud Run service name."
  type        = string
  default     = "curio-control-plane"
}

variable "service_account_email" {
  description = "Service account email used by the Cloud Run service and Pub/Sub push auth."
  type        = string
}

variable "gitlab_token_secret_name" {
  description = "Secret Manager secret name that stores the GitLab token."
  type        = string
  default     = "CURIO_GITLAB_TOKEN"
}

variable "gitlab_token_secret_project_id" {
  description = "Project ID containing the GitLab token secret. Defaults to project_id."
  type        = string
  default     = null
}

variable "state_bucket_name" {
  description = "Cloud Storage bucket name used for durable service state."
  type        = string
}

variable "state_bucket_location" {
  description = "Cloud Storage bucket location. Defaults to the Cloud Run region."
  type        = string
  default     = null
}

variable "state_mount_path" {
  description = "Mount path for the Cloud Storage bucket in Cloud Run."
  type        = string
  default     = "/state"
}

variable "curio_service_provider_model" {
  description = "Gemini model used by the Curio service backend."
  type        = string
  default     = "gemini-2.5-pro"
}

variable "vertex_location" {
  description = "Vertex AI location for Gemini calls."
  type        = string
  default     = "us-central1"
}

variable "pubsub_topic_name" {
  description = "Pub/Sub topic that receives Curio jobs."
  type        = string
  default     = "curio-jobs"
}

variable "dead_letter_topic_name" {
  description = "Pub/Sub dead-letter topic for failed Curio jobs."
  type        = string
  default     = "curio-jobs-dead-letter"
}

variable "subscription_name" {
  description = "Pub/Sub push subscription name."
  type        = string
  default     = "curio-jobs-push"
}

variable "min_instances" {
  description = "Minimum Cloud Run instances."
  type        = number
  default     = 0
}

variable "max_instances" {
  # TODO: lift to >1 after Phase 2 Firestore migration (JSONL state is unsafe for concurrent instances)
  description = "Maximum Cloud Run instances. Pinned to 1 until Phase 2."
  type        = number
  default     = 1
}

variable "cpu" {
  description = "Cloud Run CPU limit."
  type        = string
  default     = "2"
}

variable "memory" {
  description = "Cloud Run memory limit."
  type        = string
  default     = "2Gi"
}

variable "timeout_seconds" {
  description = "Cloud Run request timeout in seconds."
  type        = number
  default     = 3600
}

variable "max_delivery_attempts" {
  description = "Maximum delivery attempts before Pub/Sub forwards to the dead-letter topic."
  type        = number
  default     = 5
}

# ── IAP + LB ───────────────────────────────────────────────────────────────

variable "domain" {
  description = "Fully-qualified domain name for the service (e.g. curio.example.com). A managed SSL cert is provisioned for this domain."
  type        = string
}

variable "iap_oauth2_client_id" {
  description = "OAuth 2.0 client ID for IAP. Create via GCP Console → APIs & Services → Credentials → OAuth client ID (Web application). See iap.tf for full instructions."
  type        = string
}

variable "iap_oauth2_client_secret" {
  description = "OAuth 2.0 client secret for IAP."
  type        = string
  sensitive   = true
}

variable "corporate_ip_ranges" {
  description = "CIDR blocks to allow through Cloud Armor (corporate office IPs, VPN egress IPs). Leave empty to skip IP restriction and rely on IAP auth alone."
  type        = list(string)
  default     = []
}

# ── Workforce Identity Federation ──────────────────────────────────────────

variable "organization_id" {
  description = "GCP organization numeric ID (find with: gcloud organizations list)."
  type        = string
}

variable "entra_tenant_id" {
  description = "Microsoft Entra ID (Azure AD) tenant ID (Directory ID from the Entra ID overview)."
  type        = string
}

variable "entra_client_id" {
  description = "Entra ID app registration Application (client) ID."
  type        = string
}

variable "entra_client_secret" {
  description = "Entra ID app registration client secret value."
  type        = string
  sensitive   = true
}

variable "corporate_domain" {
  description = "Corporate email domain used to restrict WIF access (e.g. example.com). Only users with @this-domain addresses can authenticate."
  type        = string
}
