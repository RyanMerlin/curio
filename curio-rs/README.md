# Curio

A lightweight knowledge curator agent.

## Overview

Curio is a command-line interface (CLI) for managing content in a knowledge base. It provides a set of commands for ingesting, processing, and publishing content, as well as for managing the knowledge base itself.

## Commands

The following is a summary of the available commands. For more detailed information, please refer to the documentation.

- `agent prepare <codex|claude|gemini>`: Builds a launch plan for a provider.
- `agent launch <codex|claude|gemini>`: Launches a provider in the Curio workspace.
- `agent doctor [provider]`: Verifies harness readiness for one or all providers.
- `agent list-providers`: Lists supported providers and availability.
- `agent list-skills`: Lists Curio-authored harness skills.
- `agent list-plugins`: Lists Curio plugins from the marketplace catalog.
- `agent print-env <codex|claude|gemini>`: Prints the environment Curio injects for a provider.
- `bootstrap`: Creates and verifies the core Confluence structure.
- `intake-create`: Ingests content from a URL, file, or folder.
- `process-intake`: Processes content from the "Intake" stage.
- `search`: Searches Confluence for content.
- `agent-analyze`: For an external agent to analyze content.
- `gold-resolve`: Finds or creates a canonical "gold" page.
- `gold-publish`: Publishes resolved content to its canonical "gold" page.
- `review approve`: Approves a staged item for publishing.
- `review reject`: Rejects a staged item.

## Documentation

For more detailed information on how to use the Curio CLI, please refer to the following documents:

- [CLI for Humans](./docs/cli_for_humans.md)
- [CLI for Agents](./docs/cli_for_agents.md)
