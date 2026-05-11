# Curio Operating Contract

This contract is mandatory for Curio-assisted curation runs.

## Required loop

1. Inspect enough context to make a real editorial judgment.
2. Write down what was inferred before any mutation or publish action.
3. State what will change, what will stay staged/review, and why.
4. Apply the change through `curio-rs`.
5. Re-evaluate the corpus and repeat until the workspace is materially improved.

## Inference-first rule

- Do not let command availability replace judgment.
- Use commands to apply decisions, not to discover whether a decision exists.
- If a page is thin, duplicative, or ambiguous, consolidate, rewrite, or defer it instead of pushing it through unchanged.

## Minimum decision record

Every non-trivial curation pass must have a short decision record with:

- `inferred_shape`
- `selected_pages`
- `deferred_pages`
- `publish_rationale`
- `consolidation_rationale`
- `missing_information`

## Repeatability

- Prefer the same inspection and decision order every run.
- Keep the decision record next to the work, not only in chat.
- If the agent cannot explain why a page is being published, it is not ready to publish.
