# Source adapter contract

`curio::source_adapter::SourceAdapter` is the provider-neutral read boundary
between source collection and intake. It exposes adapter identity/version,
capabilities, stable `SourceItem` identities, item fetch, and opaque cursor
sync events (`create`, `update`, `delete`, `move`, and `acl_change`). The
reference `LocalMarkdownAdapter` walks a Markdown/Git tree read-only and uses
normalized relative paths as stable source IDs.

Adapters do not route, publish, or silently delete canonical pages. Deletes,
moves, and ACL changes remain explicit events for the existing intake/proposal
workflow. Future provider adapters can share conformance tests for stable
identity, unchanged re-sync, cursor handling, and malformed source data.
