# Curio CLI Documentation (for Agents)

This document provides a workflow for agents using the `curio` CLI to manage content in a knowledge base.

## Agent Workflow

The typical workflow for an agent is as follows:

1.  **Search for content to process:** Use the `search` command to find content in the "Intake" or "Staged" status.
2.  **Analyze the content:** Use the `agent-analyze` command to process the content. This may involve summarizing the content, adding metadata, and suggesting a title.
3.  **Resolve the content to a "gold" page:** Use the `gold-resolve` command to find or create a canonical "gold" page for the content.
4.  **Publish the content:** Use the `gold-publish` command to publish the content to the "gold" page.

## Harness Workflow

When the task is about starting a provider in the Curio workspace rather than operating on Confluence content:

1.  Use `curio agent doctor` to verify harness readiness.
2.  Use `curio agent prepare <provider>` to inspect the launch plan.
3.  Use `curio agent launch <provider>` to start the provider in the Curio workspace.

## Commands

### `search`

Use this command to find content to process. You can search by labels, text, and content type.

**Usage Example:**

```bash
# Search for pages with the "intake" status
curio search --labels "curio-status-intake" --limit 1
```

This command will return a JSON object with a list of pages. You can then parse this JSON to get the page ID of the page you want to process.

### `agent-analyze`

Once you have a page ID, use this command to analyze the content.

**Usage Example:**

```bash
curio agent-analyze --page-id "12345"
```

This command will analyze the page and update it with the analysis results.

### `gold-resolve`

After analyzing the page, use this command to resolve it to a "gold" page.

**Usage Example:**

```bash
curio gold-resolve --page-id "12345"
```

This command will find or create a "gold" page for the content and update the original page with a link to the "gold" page.

### `gold-publish`

Finally, use this command to publish the content to the "gold" page.

**Usage Example:**

```bash
curio gold-publish --page-id "12345"
```

This command will copy the content from the original page to the "gold" page and update the status of the original page to "published".

## Review Commands

If a page requires human review, an agent may need to interact with the `review` commands.

### `review approve`

If a change is approved, the agent can use this command to approve it.

**Usage Example:**

```bash
curio review approve --page-id "12345"
```

### `review reject`

If a change is rejected, the agent can use this command to reject it.

**Usage Example:**

```bash
curio review reject --page-id "12345" --reason "The analysis was inaccurate."
```
