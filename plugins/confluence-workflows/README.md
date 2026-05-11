# Curio Confluence Workflows Plugin

This plugin bundles guidance for the current `curio-rs` command surface:

- `onboard`
- `init`
- `intake`
- `process`
- `resolve`
- `publish`
- `search`
- `review`
- `reindex`
- `sync`
- `sharpen`

It is for the post-bootstrap Curio workflow:

- intake requests become proposals
- strong proposals move to `staged`
- ambiguous, structural, or low-signal proposals move to `review`
- only approved staged proposals publish into `published`
- Confluence mirrors the curated `CURIO` tree and the human review surface
