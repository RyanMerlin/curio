# Curio Cloud Run Terraform Infra

date: 2026-04-24
workspace: /mnt/c/code/agents/curio/curio-agent
goal: replace the Cloud Run shell deployment path with Terraform-managed infrastructure for the Curio service
inferred_shape: manage the Cloud Run service, durable GCS state bucket, Pub/Sub topics/subscriptions, Secret Manager access, and IAM as Terraform resources
selected_pages: deploy/cloud-run/terraform/main.tf, deploy/cloud-run/terraform/variables.tf, deploy/cloud-run/terraform/outputs.tf, deploy/cloud-run/README.md
deferred_pages: multi-environment workspace registry provisioning, Cloud SQL backend, private service networking
publish_rationale: infrastructure should be declarative and reviewable instead of imperative shell orchestration
consolidation_rationale: keep the service code and the infra definition separate while collapsing the deploy path to a single Terraform standard
missing_information: final state bucket naming convention and whether production wants a dedicated push service account
next_action: add Terraform configuration, remove the bootstrap scripts, and validate the module with `terraform` or `tofu`
