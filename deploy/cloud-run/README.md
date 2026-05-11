# Cloud Run Control Plane

This directory wires the Curio control plane for Cloud Run.

## What it deploys

- `curio-service` as the HTTP control plane
- `curio` alongside it so the service can execute deterministic workspace operations
- Gemini on Vertex AI as the primary service backend
- GitLab push access through `CURIO_GITLAB_TOKEN`
- Pub/Sub push ingress for async jobs
- Cloud Storage-backed durable state for the workspace registry, job store, and audit log

## Required IAM and secrets

- Cloud Run runtime service account with `roles/secretmanager.secretAccessor`
- Cloud Run runtime service account with `roles/storage.objectUser` on the state bucket
- Cloud Run runtime service account with `roles/aiplatform.user` for Gemini / Vertex AI
- Pub/Sub service agent with `roles/iam.serviceAccountTokenCreator` on the push service account
- Pub/Sub service agent with `roles/pubsub.publisher` on the dead-letter topic
- Pub/Sub service agent with `roles/pubsub.subscriber` on the jobs subscription
- Pub/Sub push service account with `roles/run.invoker`
- Secret Manager secrets for:
  - `CURIO_GITLAB_TOKEN`

## Runtime state

The Terraform module creates and mounts a Cloud Storage bucket at `STATE_MOUNT_PATH` for:

- `workspaces.json`
- `jobs.jsonl`
- `audit.jsonl`

The repo cache and git worktrees stay on local ephemeral storage at `CURIO_SERVICE_CACHE=/tmp/curio/cache`.

## Build

Use the Dockerfile in this directory to build an image that contains both Curio binaries.

## Deploy

Initialize and apply the Terraform module:

```bash
cd deploy/cloud-run/terraform
terraform init
terraform apply
```

If `terraform` is not available in your environment, install OpenTofu and use the same commands with `tofu`.

`terraform.tfvars.example` shows the required inputs.
