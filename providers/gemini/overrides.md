# Gemini — Provider Overrides

Read `HARNESS.md` first. Everything below is Gemini-specific and overrides or extends the shared contract.

## Gemini-Specific Runtime

- The initial Gemini integration target is a launcher compatible with the Go ADK-oriented workflow (`CURIO_GEMINI_RUNTIME=adk-go`). The Curio command model stays provider-stable if the launcher changes later.
- Use `curio agent print-env gemini` for the authoritative Gemini environment contract.
