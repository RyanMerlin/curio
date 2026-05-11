data "google_project" "project" {
  project_id = var.project_id
}

locals {
  state_bucket_location  = coalesce(var.state_bucket_location, var.region)
  gitlab_secret_project  = coalesce(var.gitlab_token_secret_project_id, var.project_id)
  gitlab_secret_resource = "projects/${local.gitlab_secret_project}/secrets/${var.gitlab_token_secret_name}"
  pubsub_service_agent   = "service-${data.google_project.project.number}@gcp-sa-pubsub.iam.gserviceaccount.com"
  required_apis = toset([
    "run.googleapis.com",
    "pubsub.googleapis.com",
    "secretmanager.googleapis.com",
    "storage.googleapis.com",
    "aiplatform.googleapis.com",
    "compute.googleapis.com",
    "iap.googleapis.com",
    "iam.googleapis.com",
    "cloudresourcemanager.googleapis.com",
  ])
}

provider "google" {
  project = var.project_id
  region  = var.region
}

resource "google_project_service" "required" {
  for_each = local.required_apis

  project            = var.project_id
  service            = each.value
  disable_on_destroy = false
}

resource "google_storage_bucket" "state" {
  name                        = var.state_bucket_name
  location                    = local.state_bucket_location
  uniform_bucket_level_access = true
  public_access_prevention    = "enforced"
  force_destroy               = false

  depends_on = [google_project_service.required]
}

resource "google_storage_bucket_iam_member" "state_writer" {
  bucket = google_storage_bucket.state.name
  role   = "roles/storage.objectUser"
  member = "serviceAccount:${var.service_account_email}"
}

resource "google_project_iam_member" "vertex_ai_user" {
  project = var.project_id
  role    = "roles/aiplatform.user"
  member  = "serviceAccount:${var.service_account_email}"
}

resource "google_secret_manager_secret_iam_member" "gitlab_token_accessor" {
  project   = local.gitlab_secret_project
  secret_id = var.gitlab_token_secret_name
  role      = "roles/secretmanager.secretAccessor"
  member    = "serviceAccount:${var.service_account_email}"
}

resource "google_cloud_run_v2_service" "curio" {
  name         = var.service_name
  location     = var.region
  ingress      = "INGRESS_TRAFFIC_INTERNAL_LOAD_BALANCER"
  launch_stage = "BETA"

  template {
    execution_environment            = "EXECUTION_ENVIRONMENT_GEN2"
    max_instance_request_concurrency = 1
    service_account                  = var.service_account_email
    timeout                          = "${var.timeout_seconds}s"

    scaling {
      min_instance_count = var.min_instances
      max_instance_count = var.max_instances
    }

    volumes {
      name = "curio-state"
      gcs {
        bucket    = google_storage_bucket.state.name
        read_only = false
      }
    }

    containers {
      image = var.image

      ports {
        container_port = 8080
      }

      resources {
        limits = {
          cpu    = var.cpu
          memory = var.memory
        }

        cpu_idle = false
      }

      volume_mounts {
        name       = "curio-state"
        mount_path = var.state_mount_path
      }

      env {
        name  = "CURIO_REPO_ROOT"
        value = "/workspace"
      }

      env {
        name  = "CURIO_BINARY"
        value = "/usr/local/bin/curio"
      }

      env {
        name  = "CURIO_SERVICE_BIND_ADDR"
        value = "0.0.0.0:8080"
      }

      env {
        name  = "CURIO_SERVICE_PROVIDER_BACKEND"
        value = "gemini"
      }

      env {
        name  = "CURIO_SERVICE_PROVIDER_MODEL"
        value = var.curio_service_provider_model
      }

      env {
        name  = "CURIO_VERTEX_PROJECT_ID"
        value = var.project_id
      }

      env {
        name  = "CURIO_VERTEX_LOCATION"
        value = var.vertex_location
      }

      env {
        name  = "CURIO_SERVICE_REGISTRY"
        value = "${var.state_mount_path}/workspaces.json"
      }

      env {
        name  = "CURIO_SERVICE_JOBS"
        value = "${var.state_mount_path}/jobs.jsonl"
      }

      env {
        name  = "CURIO_SERVICE_AUDIT"
        value = "${var.state_mount_path}/audit.jsonl"
      }

      env {
        name  = "CURIO_SERVICE_CACHE"
        value = "/tmp/curio/cache"
      }

      env {
        name  = "CURIO_GITLAB_USERNAME"
        value = "oauth2"
      }

      env {
        name = "CURIO_GITLAB_TOKEN"
        value_source {
          secret_key_ref {
            secret  = local.gitlab_secret_resource
            version = "latest"
          }
        }
      }
    }
  }

  depends_on = [
    google_project_service.required,
    google_storage_bucket_iam_member.state_writer,
    google_project_iam_member.vertex_ai_user,
    google_secret_manager_secret_iam_member.gitlab_token_accessor,
  ]
}

resource "google_cloud_run_v2_service_iam_member" "invoker" {
  project  = google_cloud_run_v2_service.curio.project
  location = google_cloud_run_v2_service.curio.location
  name     = google_cloud_run_v2_service.curio.name
  role     = "roles/run.invoker"
  member   = "serviceAccount:${var.service_account_email}"
}

resource "google_service_account_iam_member" "pubsub_token_creator" {
  service_account_id = "projects/${var.project_id}/serviceAccounts/${var.service_account_email}"
  role               = "roles/iam.serviceAccountTokenCreator"
  member             = "serviceAccount:${local.pubsub_service_agent}"
}

resource "google_pubsub_topic" "jobs" {
  name       = var.pubsub_topic_name
  depends_on = [google_project_service.required]
}

resource "google_pubsub_topic" "dead_letter" {
  name       = var.dead_letter_topic_name
  depends_on = [google_project_service.required]
}

resource "google_pubsub_topic_iam_member" "dead_letter_publisher" {
  topic  = google_pubsub_topic.dead_letter.name
  role   = "roles/pubsub.publisher"
  member = "serviceAccount:${local.pubsub_service_agent}"
}

resource "google_pubsub_subscription" "jobs" {
  name  = var.subscription_name
  topic = google_pubsub_topic.jobs.name

  ack_deadline_seconds = 600

  push_config {
    push_endpoint = "${google_cloud_run_v2_service.curio.uri}/v1/pubsub/jobs"

    oidc_token {
      service_account_email = var.service_account_email
      audience              = google_cloud_run_v2_service.curio.uri
    }
  }

  dead_letter_policy {
    dead_letter_topic     = google_pubsub_topic.dead_letter.id
    max_delivery_attempts = var.max_delivery_attempts
  }

  retry_policy {
    minimum_backoff = "10s"
    maximum_backoff = "600s"
  }

  depends_on = [
    google_cloud_run_v2_service_iam_member.invoker,
    google_service_account_iam_member.pubsub_token_creator,
    google_pubsub_topic_iam_member.dead_letter_publisher,
  ]
}

resource "google_pubsub_subscription_iam_member" "dead_letter_subscriber" {
  subscription = google_pubsub_subscription.jobs.name
  role         = "roles/pubsub.subscriber"
  member       = "serviceAccount:${local.pubsub_service_agent}"
}
