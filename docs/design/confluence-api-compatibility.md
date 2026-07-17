# Confluence API compatibility matrix

Curio deliberately keeps a compatibility-led mix of Confluence REST v1 and v2
endpoints. Confluence is a one-way mirror; Git remains the source of truth.

| Curio operation | Current endpoint | v2 status | Decision |
| --- | --- | --- | --- |
| Create page with storage macros | `POST /rest/api/content` | v2 body representation is not equivalent for Curio storage macros | Keep v1 until macro compatibility is proven |
| Update page body | `PUT /rest/api/content/{id}` | Available, but storage/macro behavior differs | Keep v1 and retain version-conflict retry |
| Read page/version | `GET /api/v2/pages/{id}` | Equivalent for current metadata | Use v2 |
| Read page body | `GET /api/v2/pages/{id}?body-format=storage` | Equivalent for current reads | Use v2 |
| Descendants/children | `GET /api/v2/pages/...` | Cursor/continuation links | Use v2 with same-origin pagination validation |
| Folder descendants | `GET /api/v2/folders/{id}/descendants` | Cursor/continuation links | Use v2 with bounded pagination |
| CQL search | `GET /rest/api/content/search` | No drop-in equivalent used by Curio | Keep v1 with continuation handling |
| Content properties | `/rest/api/content/{id}/property/...` | Migration requires a separate storage/scopes review | Keep v1 |
| Labels and attachments | `/rest/api/content/...` | Compatibility varies by operation | Reassess separately before migration |

Any migration must prove storage-format macro rendering, required scopes, and
same-root update safety in mock tests and the dedicated CURIO sandbox first.
