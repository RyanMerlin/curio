# Retrieval Evaluation Baseline

Curio's deterministic retrieval baseline is checked in at
`curio-rs/tests/fixtures/retrieval-eval/`. It is deliberately synthetic and
credential-free so that retrieval changes can be compared in CI without a live
knowledge base or network access.

Run the human-readable report from `curio-rs/` with:

```bash
cargo run --quiet --bin curio-retrieval-eval -- \
  --corpus tests/fixtures/retrieval-eval/corpus.json
```

Use `--json` for a structured report suitable for CI artifacts or comparisons.
The corpus includes current deployment guidance, a rollback procedure, security
permissions, a stale legacy page, and a deployment telemetry near-match. Query
expectations use the stable path-derived retrieval IDs emitted by `curio
retrieve`.

The current baseline on 2026-07-17 is:

| Metric | Baseline |
| --- | ---: |
| recall@k | 1.000 |
| mean reciprocal rank | 0.833 |
| citation coverage | 1.000 |
| stale-result rate | 0.222 |
| ACL-leak count | 0 |

The stale-result rate is intentionally non-zero: the fixture proves that an
outdated page is detectable rather than silently treating every published page
as fresh. ACL leak cases are represented in the corpus now and remain zero until
the retrieval contract gains permission principals and filtering.

The complete machine-readable report is checked in at
`curio-rs/tests/fixtures/retrieval-eval/baseline.json`; CI also uploads the
run as a `retrieval-baseline` artifact. The evaluator fails when citation
coverage is below `1.0`, result IDs are duplicated, or repeated runs change
ordering. The `curio_uri` field is additive provenance; existing `local:` IDs
and JSON consumers remain unchanged.

Any retrieval backend change should run this evaluator and include before/after
metrics in its pull request description. A backend must not become the default
based on demo impressions alone.
