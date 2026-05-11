# Source Corpus Tuning

SupportServer-style source spaces are not just content inputs. They are training corpora for Curio's editorial judgment.

## Workflow

1. Ingest a representative chunk of the source corpus.
2. Identify the dominant branch families and approval patterns.
3. Approve or add the missing taxonomy branches that the corpus clearly needs.
4. Rewrite the strongest leaves so they read like intentional KB articles, not raw captures.
5. Leave ambiguous or thin material in `review` so the corpus still shows active curation.
6. Publish only the pages that survive both quality and hierarchy judgment.
7. Codify the repeated decisions into harness docs, branch descriptions, and tests.

## What to learn from the corpus

- Which branch families are actually real in practice.
- Which leaf shapes are strong enough to publish with light editing.
- Which pages are structural noise, duplicate routes, or thin indexes.
- Which recurring failures are caused by missing taxonomy versus weak body content.

## What to record back into the harness

- Branch families that should become first-class `NORTHSTAR.md` nodes.
- Reusable rewrite patterns for common page types.
- Rejection reasons that recur often enough to become policy.
- Thresholds that are too strict or too loose for real corpora.

## Success criterion

Curio improves after every pass because the harness learns the corpus shape and applies that knowledge to the next run.
