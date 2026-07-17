# Permission-preserving retrieval

Curio keeps source authorization separate from editorial Markdown. A snapshot
under `<wiki-dir>/_admin/acl/*.json` is keyed by `source_id` and records its
source revision, capture time, normalized allow principals, and deny
principals. Principal IDs are provider-qualified opaque identifiers; Curio
does not resolve identity providers in the deterministic substrate.

Pages without a snapshot retain the legacy unrestricted behavior. A page with
a snapshot is restricted unless the caller supplies an `AccessContext` that
matches an allow principal. Deny matches always win, and missing identity or
missing ACL state for an explicitly ACL-managed source fails closed. Search
filters before ranking, excerpts, and counts; fetch treats an inaccessible
known ID as not found.

The CLI accepts repeated `--principal` values for deterministic local tests.
The local stdio MCP server accepts trusted `--principal` configuration; tool
arguments are not an identity mechanism. Remote authenticated access is not
implemented in this slice.
