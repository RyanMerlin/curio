date: 2026-05-04
workspace: /mnt/c/code/agents/curio/curio-agent
goal: Remove public-repo leakage from tracked fixtures by stripping copied internal support content and parameterizing URL examples.
inferred_shape: Keep the manifest and review files as synthetic fixtures only; preserve top-level schema shape, remove internal Atlassian content, and use placeholder URL templates.
selected_pages: curio-rs/heal-manifest.json, curio-rs/review.json
deferred_pages: None
publish_rationale: These files are tracked and would otherwise ship internal URLs, customer references, and copied support text in a public repo.
consolidation_rationale: The fixtures only need enough structure to exercise loaders and tests; real corpus content does not belong in the public tree.
missing_information: None blocking; the files are standalone tracked fixtures.
next_action: Replace the tracked fixtures with synthetic entries and verify no internal Atlassian strings remain.
