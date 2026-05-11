//! T2-A — page-body rewriting in process Phase 2.
//!
//! Verifies that when the agent supplies `proposed_body_markdown` and
//! `decision_section_markdown` in a route file:
//! 1. The intake page's body is REPLACED by the rewritten body.
//! 2. The decision section is prepended above the body.
//! 3. The proposal sidecar records `body_rewrite_kind` and
//!    `decision_section_present` for downstream audit.
//!
//! Backwards compatibility: when neither field is set, the body must be
//! preserved exactly (only the frontmatter gets updated, matching pre-T2-A
//! behavior). Older route files keep working.

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

fn write_intake(wiki_dir: &Path, slug: &str, title: &str, body: &str) {
    let raw = format!(
        "---\nid: {slug}\ntitle: \"{title}\"\nstatus: intake\nsource:\n  kind: web_page\n  id: src-{slug}\n  origin_url: https://x\ncategory: []\nkeywords: []\ncreated_at: \"2026-01-01T00:00:00Z\"\nupdated_at: \"2026-01-01T00:00:00Z\"\ncross_refs: []\ncontent_hash: \"h\"\n---\n\n{body}\n"
    );
    std::fs::write(wiki_dir.join("intake").join(format!("{slug}.md")), raw).unwrap();
}

fn decision_with_rewrite(
    category: &[&str],
    body: Option<&str>,
    section: Option<&str>,
    kind: Option<&str>,
) -> ReconcileDecision {
    ReconcileDecision {
        category: category.iter().map(|s| s.to_string()).collect(),
        keywords: vec!["alpha".into(), "beta".into()],
        confidence: 0.92,
        status: "staged".into(),
        summary: "test summary".into(),
        cross_refs: vec![],
        review_reason: None,
        proposed_new_subtree: None,
        proposal_rationale: None,
        merge_target: None,
        model_used: "test".into(),
        proposed_body_markdown: body.map(|s| s.to_string()),
        decision_section_markdown: section.map(|s| s.to_string()),
        body_rewrite_kind: kind.map(|s| s.to_string()),
        merge_into_slug: None,
    }
}

#[tokio::test]
async fn route_file_with_rewrite_replaces_body_and_prepends_section() {
    let tmp = tempfile::tempdir().unwrap();
    let kb_dir = tmp.path();
    scaffold_kb(kb_dir);
    let wiki_dir = kb_dir.join("wiki");
    write_intake(
        &wiki_dir,
        "synth-page",
        "Synth Page",
        "Original raw intake body. Sparse, would not survive curation as-is.\n\nMore lines.",
    );
    run_git(kb_dir, &["add", "-A"]);
    run_git(kb_dir, &["commit", "-q", "-m", "intake"]);

    let decision_section = "## Curation Decision\n\n- route: tree\n- scores: route=0.92 quality=0.80\n- recommended_action: stage";
    let proposed_body = "# Synth Page Curated Knowledge\n\n## Background\n\nThis page is a clean, fully synthesized rewrite produced by the agent during routing. Every paragraph here is intentional knowledge curation rather than raw capture from the intake source. The goal of this page is to demonstrate that the agent can produce a curated knowledge object whose body is meaningfully different from the original intake stream, with structure, headings, lists, and links.\n\n## Details\n\n- the body has three top-level sections so it scores well on structure heuristics\n- the prose is dense enough to easily clear the 120-word richness threshold\n- it includes an anchor [link](https://example.com) for the link signal\n- it carries multiple distinct sentences and concrete nouns to build unique terms\n\n## How the agent decided\n\nThe agent inspected the intake source, compared it against the existing `tree/leaf` neighborhood, found no high-overlap peers, judged the route as a confident fit, and authored this curated body so reviewers can evaluate the synthesized knowledge rather than the raw capture. Confidence in the route is high, hierarchy fit is strong, overlap risk is negligible, and the rewritten body itself is intended to be publishable on its own merits.\n\n## Conclusion\n\nWhen the agent supplies `proposed_body_markdown`, the resulting page in `staged/` should reflect THIS content, not the original intake stream. The proposal sidecar should record the rewrite kind so future audits can reconstruct what the agent did.";
    let decisions = vec![(
        "synth-page".to_string(),
        decision_with_rewrite(
            &["tree", "leaf"],
            Some(proposed_body),
            Some(decision_section),
            Some("full_synthesis"),
        ),
    )];
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

    // Page must now live under staged/tree/.
    let dest = wiki_dir
        .join("staged")
        .join("tree")
        .join("leaf")
        .join("synth-page.md");
    assert!(dest.exists(), "page must be moved to staged/tree/");

    let final_content = std::fs::read_to_string(&dest).unwrap();
    assert!(
        final_content.contains("## Curation Decision"),
        "decision section must be in final body; got:\n{final_content}"
    );
    assert!(
        final_content.contains("Synth Page Curated Knowledge"),
        "agent-rewritten body must replace intake body; got:\n{final_content}"
    );
    assert!(
        !final_content.contains("Original raw intake body"),
        "original intake body must be replaced; got:\n{final_content}"
    );

    // Proposal sidecar must record the rewrite kind.
    let proposal_path = wiki_dir
        .join("staged")
        .join("tree")
        .join("leaf")
        .join("synth-page.md.proposal.json");
    assert!(
        proposal_path.exists(),
        "proposal sidecar must exist alongside the moved page"
    );
    let proposal: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&proposal_path).unwrap()).unwrap();
    assert_eq!(
        proposal["dossier"]["body_rewrite_kind"].as_str(),
        Some("full_synthesis"),
        "dossier.body_rewrite_kind must reflect the agent's choice"
    );
    assert_eq!(
        proposal["dossier"]["decision_section_present"].as_bool(),
        Some(true),
        "dossier.decision_section_present must be true when section was supplied"
    );
}

#[tokio::test]
async fn route_file_without_rewrite_preserves_body_for_backwards_compat() {
    let tmp = tempfile::tempdir().unwrap();
    let kb_dir = tmp.path();
    scaffold_kb(kb_dir);
    let wiki_dir = kb_dir.join("wiki");
    let original_body = "# Untouched Page\n\nThis body is intentionally substantive so that the quality assessment passes on its own merits, exercising the backwards-compat path where the agent supplies no rewrite.\n\n## Background\n\nThe page covers a hypothetical operational topic with several distinct sentences and a clear structure. It is meant to score above the publishable threshold without any agent rewriting.\n\n## Details\n\n- has at least one bulleted list\n- spans multiple paragraphs\n- includes a [link](https://example.com) to satisfy has_links\n- exceeds the 120-word soft floor used by the quality scorer\n\n## Conclusion\n\nWith real prose, real structure, and real links, the heuristic in `quality.rs` should rate this as publishable, allowing the routing pipeline to route to staged on the multi-segment path. The agent simply chose not to rewrite the body, which is a valid choice for content that already reads well.";
    write_intake(&wiki_dir, "untouched", "Untouched Page", original_body);
    run_git(kb_dir, &["add", "-A"]);
    run_git(kb_dir, &["commit", "-q", "-m", "intake"]);

    // Decision provides NO rewrite fields — older route-file shape.
    let decisions = vec![(
        "untouched".to_string(),
        decision_with_rewrite(&["tree", "leaf"], None, None, None),
    )];
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

    let dest = wiki_dir
        .join("staged")
        .join("tree")
        .join("leaf")
        .join("untouched.md");
    assert!(dest.exists(), "page must move even without rewrite");
    let final_content = std::fs::read_to_string(&dest).unwrap();
    assert!(
        final_content.contains("This body is intentionally substantive"),
        "body must be preserved verbatim when no rewrite supplied; got:\n{final_content}"
    );

    // Proposal sidecar should record body_rewrite_kind = "none".
    let proposal_path = wiki_dir
        .join("staged")
        .join("tree")
        .join("leaf")
        .join("untouched.md.proposal.json");
    let proposal: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&proposal_path).unwrap()).unwrap();
    assert_eq!(
        proposal["dossier"]["body_rewrite_kind"].as_str(),
        Some("none"),
        "missing rewrite must default body_rewrite_kind to 'none'"
    );
    assert_eq!(
        proposal["dossier"]["decision_section_present"].as_bool(),
        Some(false),
        "decision_section_present should be false when not provided"
    );
}
