date: 2026-04-24
workspace: /mnt/c/code/agents/curio/wiki
goal: Make CURIO a polished Confluence homepage and cut over to NORTHSTAR prose plus _admin/config.yaml as the deterministic config source.
inferred_shape: Keep CURIO as the actual space homepage, publish NORTHSTAR prose with config YAML below it, and remove the split between settings.yaml and mirrored _config/northstar.md.
selected_pages: CURIO, Northstar, Getting Started, Downloads, Admin
deferred_pages: Review subtree polish, published branch summaries, non-CURIO spaces
publish_rationale: The space homepage must work as a user-facing landing page without hiding the managed tree, and the config model should be single-source rather than duplicated.
consolidation_rationale: NORTHSTAR should hold human charter prose; _admin/config.yaml should hold all machine-readable taxonomy and runtime settings.
missing_information: None blocking; repository links and homepage model were confirmed.
next_action: Update init/load/sync/rendering paths, migrate live wiki files, then run focused tests.
