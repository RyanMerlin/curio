//! T2-C — multi-source synthesis: 1 intake request → N proposals.
//!
//! Verifies the merge_into_slug path: when the agent decides that two
//! intake pages from the same `curio intake` invocation belong in a
//! single proposal, the secondary's body is folded into the primary
//! under a "## Merged source" heading, the secondary disappears from
//! intake, and the primary's proposal dossier records ALL contributing
//! sources (not just the primary's own).

use curio::commands::process_intake::run_process;
use curio::reconcile::ReconcileDecision;
use std::path::Path;
use std::process::Command;

fn run_git(dir: &Path, args: &[&str]) {
    let status = Command::new("git")
        .args(args)
        .current_dir(dir)
        .env("GIT_AUTHOR_NAME", "test")
        .env("GIT_AUTHOR_EMAIL", "test@example.com")
        .env("GIT_COMMITTER_NAME", "test")
        .env("GIT_COMMITTER_EMAIL", "test@example.com")
        .status()
        .expect("git invocation");
    assert!(status.success(), "git {:?} failed", args);
}

fn scaffold_kb(kb_dir: &Path) {
    let wiki_dir = kb_dir.join("wiki");
    for sub in ["intake", "staged", "review", "published", "_admin"] {
        std::fs::create_dir_all(wiki_dir.join(sub)).unwrap();
        std::fs::write(wiki_dir.join(sub).join(".gitkeep"), "").unwrap();
    }
    std::fs::write(wiki_dir.join("NORTHSTAR.md"), "# NORTHSTAR\n\ntest\n").unwrap();
    std::fs::write(
        wiki_dir.join("_admin").join("config.yaml"),
        "schema_version: 2\nnodes:\n  - title: Tree\n    slug: tree\n    description_markdown: \"trunk\"\n    children:\n      - title: Leaf\n        slug: leaf\n        description_markdown: \"leaf\"\n        children: []\n",
    )
    .unwrap();
    std::fs::write(
        kb_dir.join(".curio.yaml"),
        "connection:\n  confluence_url: https://x.atlassian.net/wiki\n  confluence_email: bot@x\n  token_env: CURIO_TOK_TEST\ncontent_model:\n  space_key: TEST\n  label_namespace: curio\nwiki:\n  wiki_dir: wiki\n  auto_commit: true\n  sync:\n    enabled: false\n",
    )
    .unwrap();
    run_git(kb_dir, &["init", "-q", "-b", "main"]);
    run_git(kb_dir, &["add", "-A"]);
    run_git(kb_dir, &["commit", "-q", "-m", "scaffold"]);
}

fn write_intake_pair(wiki_dir: &Path) {
    // Two sources from the SAME intake request — share request_id.
    let request_id = "intake-test-request-001";
    let primary = format!(
        "---\nid: primary\ntitle: \"Primary Source\"\nstatus: intake\nsource:\n  kind: web_page\n  id: src-primary\n  origin_url: https://x/primary\ncategory: []\nkeywords: []\ncreated_at: \"2026-01-01T00:00:00Z\"\nupdated_at: \"2026-01-01T00:00:00Z\"\ncross_refs: []\ncontent_hash: h1\nintake_request_id: \"{request_id}\"\n---\n\nPrimary body content explaining the topic.\n"
    );
    let secondary = format!(
        "---\nid: secondary\ntitle: \"Secondary Source\"\nstatus: intake\nsource:\n  kind: web_page\n  id: src-secondary\n  origin_url: https://x/secondary\ncategory: []\nkeywords: []\ncreated_at: \"2026-01-01T00:00:00Z\"\nupdated_at: \"2026-01-01T00:00:00Z\"\ncross_refs: []\ncontent_hash: h2\nintake_request_id: \"{request_id}\"\n---\n\nSecondary body adding supplementary detail to the same topic.\n"
    );
    std::fs::write(wiki_dir.join("intake").join("primary.md"), primary).unwrap();
    std::fs::write(wiki_dir.join("intake").join("secondary.md"), secondary).unwrap();
}

fn decision(
    slug: &str,
    category: &[&str],
    body: &str,
    merge_into: Option<&str>,
) -> (String, ReconcileDecision) {
    (
        slug.to_string(),
        ReconcileDecision {
            category: category.iter().map(|s| s.to_string()).collect(),
            keywords: vec!["alpha".into(), "beta".into()],
            confidence: 0.9,
            status: "staged".into(),
            summary: "consolidated proposal".into(),
            cross_refs: vec![],
            review_reason: None,
            proposed_new_subtree: None,
            proposal_rationale: None,
            merge_target: None,
            model_used: "test".into(),
            proposed_body_markdown: Some(body.into()),
            decision_section_markdown: None,
            body_rewrite_kind: Some("full_synthesis".into()),
            merge_into_slug: merge_into.map(String::from),
        },
    )
}

#[tokio::test]
async fn merge_into_consolidates_two_sources_into_one_proposal() {
    let tmp = tempfile::tempdir().unwrap();
    let kb_dir = tmp.path();
    scaffold_kb(kb_dir);
    let wiki_dir = kb_dir.join("wiki");
    write_intake_pair(&wiki_dir);
    run_git(kb_dir, &["add", "-A"]);
    run_git(kb_dir, &["commit", "-q", "-m", "intake"]);

    // Agent decides: secondary merges into primary; primary gets a
    // synthesized body that consolidates both sources.
    let consolidated_body = "# Consolidated Topic Reference\n\n## Overview\n\nThis page consolidates the primary intake source together with the merged-in secondary intake source into a single curated, hierarchically-placed knowledge object covering the unified subject family. The curation explicitly preserves provenance for both contributing sources so a reviewer can trace any specific claim back to its original capture in [primary](https://example.com/primary) or [secondary](https://example.com/secondary).\n\n## Conceptual scope\n\nReaders consulting this entry should expect coverage of installation, configuration, troubleshooting, migration considerations, version-specific guidance, performance tuning recommendations, audit posture, governance boundaries, and operational gotchas. Each subsection below ties back to concrete decisions surfaced during curation.\n\n## Synthesis details\n\nMaterial originally captured separately across multiple ingestion artifacts has been folded together where overlap was substantive and complementary, deliberately rejecting the alternative of producing parallel near-duplicate proposals. The agent inferred that splitting these knowledge fragments across distinct pages would force readers to bounce between near-duplicates without measurable retrieval benefit. Consolidation here improves discoverability, reduces maintenance burden, and stabilizes the hierarchical neighborhood.\n\n## Operational guidance\n\nAdministrators should reference [internal runbooks](https://example.com/runbooks) when applying configuration changes derived from this consolidated reference. Sequence the operational rollout deliberately: validate prerequisites, snapshot existing settings, apply incremental adjustments, monitor stability indicators, and roll forward only after observing healthy operational signals.\n\n## Maintenance posture\n\nKeep this entry sharpened: when new evidence arrives that materially changes the recommendations, route a fresh intake through Curio and explicitly evaluate whether the new material extends the consolidation, supersedes parts of it, or warrants a separate sibling page.";
    let decisions = vec![
        decision("primary", &["tree", "leaf"], consolidated_body, None),
        decision("secondary", &["tree", "leaf"], "", Some("primary")),
    ];
    let routes_path = kb_dir.join("routes.json");
    std::fs::write(&routes_path, serde_json::to_string(&decisions).unwrap()).unwrap();

    let config = curio::config::load_config(None, Some(kb_dir)).expect("load_config");
    run_process(
        &config,
        false,
        false,
        100,
        false,
        false,
        Some(routes_path),
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .await
    .expect("process apply");

    // Primary moved to staged/tree/leaf/.
    let primary_dest = wiki_dir
        .join("staged")
        .join("tree")
        .join("leaf")
        .join("primary.md");
    assert!(
        primary_dest.exists(),
        "primary must land in staged/tree/leaf/"
    );

    // Secondary is gone — both from intake/ and from staged/.
    assert!(
        !wiki_dir.join("intake").join("secondary.md").exists(),
        "secondary must be removed from intake after merge"
    );
    assert!(
        !wiki_dir
            .join("staged")
            .join("tree")
            .join("leaf")
            .join("secondary.md")
            .exists(),
        "secondary must NOT have a separate staged page"
    );

    // Primary's body must contain the agent's consolidated content.
    let primary_content = std::fs::read_to_string(&primary_dest).unwrap();
    assert!(
        primary_content.contains("Consolidated Topic"),
        "primary must carry the consolidated body; got:\n{primary_content}"
    );

    // Proposal sidecar lists BOTH contributing sources.
    let proposal_path = wiki_dir
        .join("staged")
        .join("tree")
        .join("leaf")
        .join("primary.md.proposal.json");
    let proposal: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&proposal_path).unwrap()).unwrap();
    let source_ids: Vec<&str> = proposal["dossier"]["source_ids"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    assert!(
        source_ids.contains(&"src-primary"),
        "dossier.source_ids must include primary source; got {source_ids:?}"
    );
    assert!(
        source_ids.contains(&"src-secondary"),
        "dossier.source_ids must include merged-in secondary; got {source_ids:?}"
    );
    let source_urls: Vec<&str> = proposal["dossier"]["source_locations"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    assert!(
        source_urls.iter().any(|u| u.contains("primary")),
        "dossier.source_locations must include primary URL; got {source_urls:?}"
    );
    assert!(
        source_urls.iter().any(|u| u.contains("secondary")),
        "dossier.source_locations must include secondary URL; got {source_urls:?}"
    );
    // Proposal kind must reflect the consolidation.
    assert_eq!(
        proposal["kind"].as_str(),
        Some("consolidation"),
        "merged-source proposal kind should be 'consolidation'"
    );
}
