output "service_url" {
  description = "Cloud Run service URL."
  value       = google_cloud_run_v2_service.curio.uri
}

output "state_bucket" {
  description = "Cloud Storage bucket used for durable state."
  value       = google_storage_bucket.state.name
}

output "pubsub_topic" {
  description = "Pub/Sub topic for Curio jobs."
  value       = google_pubsub_topic.jobs.name
}

output "dead_letter_topic" {
  description = "Pub/Sub dead-letter topic for failed Curio jobs."
  value       = google_pubsub_topic.dead_letter.name
}

output "subscription_name" {
  description = "Pub/Sub push subscription name."
  value       = google_pubsub_subscription.jobs.name
}

output "load_balancer_ip" {
  description = "Static IP of the external HTTPS load balancer. Create an A record for var.domain pointing here."
  value       = google_compute_global_address.curio.address
}

output "iap_audience" {
  description = "IAP JWT audience. After apply, run: gcloud run services update <service> --set-env-vars CURIO_IAP_AUDIENCE=$(terraform output -raw iap_audience),CURIO_SERVICE_AUTH_MODE=iap"
  value       = "/projects/${data.google_project.project.number}/global/backendServices/${google_compute_backend_service.curio.generated_id}"
}

output "entra_redirect_uri" {
  description = "Add this as a redirect URI in your Entra ID app registration (Certificates & secrets → Redirect URIs → Web)."
  value       = "https://auth.workforceidentity.com/${var.organization_id}/${google_iam_workforce_pool.main.workforce_pool_id}/providers/${google_iam_workforce_pool_provider.entra_id.provider_id}"
}

output "workforce_pool_name" {
  description = "Fully-qualified Workforce Identity Federation pool resource name."
  value       = google_iam_workforce_pool.main.name
}
