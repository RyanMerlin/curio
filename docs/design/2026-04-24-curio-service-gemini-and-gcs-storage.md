# Curio Service Gemini Backend and Durable State

date: 2026-04-24
workspace: /mnt/c/code/agents/curio/curio-agent
goal: make Gemini the primary service backend while keeping provider selection pluggable, and make service state durable on Cloud Run
inferred_shape: Cloud Run should use Cloud Storage volume mounts for registry/jobs/audit state; repo mirrors and worktrees should remain on ephemeral local storage because git needs stronger filesystem semantics than the mounted bucket provides
selected_pages: curio-rs/src/service/providers.rs, curio-rs/src/service/runtime.rs, deploy/cloud-run/Dockerfile, deploy/cloud-run/terraform/main.tf, deploy/cloud-run/terraform/variables.tf, deploy/cloud-run/README.md
deferred_pages: cloud provider adapters beyond Gemini/OpenAI/passthrough, database-backed state store, Pub/Sub dead-letter and replay tuning
publish_rationale: Gemini is the enterprise primary backend and the service needs a durable filesystem-backed state root that survives Cloud Run restarts
consolidation_rationale: reuse the existing file-backed service runtime and mount a Cloud Storage bucket for the persistent files instead of introducing a second database layer
missing_information: exact bucket name and project-level IAM policy for the deployment target
next_action: wire Gemini adapter defaults, switch deployment to Gemini and Cloud Storage volume mounts, and verify the service still builds and tests cleanly
