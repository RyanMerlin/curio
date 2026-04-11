# Curio CLI Documentation (for Humans)

This document provides instructions on how to use the `curio` command-line interface.

## Global Options

These options can be used with any command:

- `--config <path>`: Path to the configuration file.
- `--dry-run`: If true, print what would be done without making changes.
- `--space <key>`: Confluence space key to operate within.
- `--root-page-id <id>`: Confluence root page ID to operate within.
- `--workdir <path>`: Working directory for ephemeral files.
- `--log-level <level>`: Set the logging level (e.g., `INFO`, `DEBUG`, `TRACE`).

## Commands

### `agent`

Harness commands for provider startup in the top-level Curio workspace.

**Usage Examples:**

```bash
curio agent doctor
curio agent prepare codex
curio agent launch claude
curio agent print-env gemini
```

### `bootstrap`

Creates and verifies the core Confluence structure (folders, templates). This should be the first command you run when setting up a new space.

**Usage:**

```bash
curio bootstrap
```

### `intake-create`

Ingest content from various sources into Confluence. You must specify one of the following input sources:

- `--url <url>`: URL of a web page or Confluence link to ingest.
- `--file <path>`: Path to a local file to ingest.
- `--folder <path>`: Path to a local folder to ingest recursively.

**Options:**

- `--subject-hint <hint>`: (Optional) A hint for the subject of the content.
- `--metadata <json>`: (Optional) JSON string of initial metadata to merge.

**Usage Examples:**

```bash
# Ingest from a URL
curio intake-create --url "https://example.com/article"

# Ingest from a local file with a subject hint
curio intake-create --file ./my-document.md --subject-hint "My Document"
```

### `process-intake`

Processes content from the Intake stage, moving it to Staged or Review.

**Options:**

- `--limit <number>`: The maximum number of intake items to process in one run. Defaults to 10.

**Usage:**

```bash
curio process-intake --limit 5
```

### `search`

Searches Confluence for content based on various criteria.

**Options:**

- `--labels <label>`: Labels to filter by (e.g., "curio-status-staged"). Can be specified multiple times.
- `--text <query>`: Free-text search query.
- `--content-type <type>`: Content type to filter by (e.g., "page", "blogpost").
- `--limit <number>`: Maximum number of results to return. Defaults to 20.

**Usage Examples:**

```bash
# Search for pages with the "staged" status
curio search --labels "curio-status-staged"

# Search for content with the text "hello world"
curio search --text "hello world"
```

### `agent-analyze`

Command for an external agent to analyze content in Confluence.

**Options:**

- `--page-id <id>`: Optional: Process a specific page by its ID.
- `--status <status>`: Optional: Process pages with a specific status (e.g., "intake", "analyzing").
- `--limit <number>`: The maximum number of items to analyze in one run. Defaults to 10.

**Usage:**

```bash
curio agent-analyze --status "intake" --limit 5
```

### `gold-resolve`

Finds or creates a canonical "gold" page for a given subject.

**Options:**

- `--page-id <id>`: The ID of the page to resolve (e.g., a page from the "Staged" or "Review" areas).

**Usage:**

```bash
curio gold-resolve --page-id "12345"
```

### `gold-publish`

Publishes resolved content to its canonical "gold" page.

**Options:**

- `--page-id <id>`: The ID of the resolved page (which contains the link to the "gold" page).

**Usage:**

```bash
curio gold-publish --page-id "12345"
```

### `review`

A group of commands for reviewing content.

#### `review approve`

Approves a staged item or a change proposal for publishing.

**Options:**

- `--page-id <id>`: The ID of the page to approve.

**Usage:**

```bash
curio review approve --page-id "12345"
```

#### `review reject`

Rejects a staged item or a change proposal.

**Options:**

- `--page-id <id>`: The ID of the page to reject.
- `--reason <reason>`: The reason for the rejection.

**Usage:**

```bash
curio review reject --page-id "12345" --reason "This is not a good fit for our knowledge base."
```
