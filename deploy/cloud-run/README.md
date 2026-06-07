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

## Public repo boundary

- `terraform.tfvars.example` is the public template only.
- Create an ignored `deploy/cloud-run/terraform/terraform.tfvars` locally for real deployment values.
- Keep project IDs, domains, OAuth client secrets, service account emails, and secret names out of tracked files.
- `deploy/cloud-run/state/workspaces.json` is a placeholder fixture for the demo harness, not a live deployment registry.
- For WSL2, use `deploy/cloud-run/wsl2-gcloud-bootstrap.sh` to build a writable Linux gcloud config and verify Google API connectivity. Put one-off defaults in the ignored `deploy/cloud-run/wsl2-gcloud.local.env`.

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
