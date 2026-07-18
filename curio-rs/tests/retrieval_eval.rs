use curio::retrieval_eval::{evaluate, load_corpus};
use serde_json::Value;
use std::path::Path;

#[test]
fn checked_in_retrieval_baseline_is_deterministic_and_cited() {
    let corpus_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/retrieval-eval/corpus.json");
    let corpus = load_corpus(&corpus_path).unwrap();
    let wiki_dir = corpus_path.parent().unwrap().join("wiki");
    let first = evaluate(&corpus, &wiki_dir).unwrap();
    let second = evaluate(&corpus, &wiki_dir).unwrap();

    assert_eq!(first.metrics.recall_at_k, 1.0);
    assert!(first.metrics.recall_at_k >= 0.85);
    assert_eq!(first.metrics.citation_coverage, 1.0);
    assert_eq!(first.metrics.acl_leak_count, 0);
    assert_eq!(first, second);

    let baseline_path = corpus_path.parent().unwrap().join("baseline.json");
    let baseline: Value =
        serde_json::from_str(&std::fs::read_to_string(baseline_path).unwrap()).unwrap();
    let expected = serde_json::json!({
        "command": "retrieval-eval",
        "ok": true,
        "data": {
            "version": first.version,
            "as_of": first.as_of,
            "stale_before": first.stale_before,
            "query_count": first.query_count,
            "metrics": first.metrics,
        }
    });
    assert_eq!(
        baseline, expected,
        "checked-in retrieval baseline is stale; regenerate it deliberately"
    );
}
