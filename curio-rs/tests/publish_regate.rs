//! Phase C — publish-time re-gate.
//!
//! Verifies that:
//! 1. `curio publish <slug>` REFUSES when the staged page fails the
//!    publish-time quality gate.
//! 2. `curio publish <slug> --force` succeeds, AND records the bypassed
//!    dimensions to `wiki/_admin/log.md` so an auditor can reconstruct
//!    what was waived.

use curio::commands::gold_publish::run_publish;
use curio::config::Config;
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
    // Minimal NORTHSTAR.md (intake walker skips this so frontmatter not needed).
    std::fs::write(wiki_dir.join("NORTHSTAR.md"), "# NORTHSTAR\n\ntest\n").unwrap();
    // Workspace config with a taxonomy that has a "test-tree" branch.
    let config_yaml = r#"schema_version: 2
nodes:
  - title: Test Tree
    slug: test-tree
    description_markdown: ""
    children: []
"#;
    std::fs::write(wiki_dir.join("_admin").join("config.yaml"), config_yaml).unwrap();
    // .curio.yaml so load_config wires the wiki_dir correctly.
    let curio_yaml = r#"connection:
  confluence_url: https://example.atlassian.net/wiki
  confluence_email: bot@example.com
  token_env: CURIO_CONFLUENCE_TOKEN_TEST
content_model:
  space_key: TEST
  label_namespace: curio
wiki:
  wiki_dir: wiki
  auto_commit: true
  sync:
    enabled: false
"#;
    std::fs::write(kb_dir.join(".curio.yaml"), curio_yaml).unwrap();

    run_git(kb_dir, &["init", "-q", "-b", "main"]);
    run_git(kb_dir, &["add", "-A"]);
    run_git(kb_dir, &["commit", "-q", "-m", "scaffold"]);
}

fn write_staged_page(wiki_dir: &Path, slug: &str, title: &str, body: &str) {
    let staged_dir = wiki_dir.join("staged").join("test-tree");
    std::fs::create_dir_all(&staged_dir).unwrap();
    let raw = format!(
        "---\nid: {slug}\ntitle: \"{title}\"\nstatus: staged\nsource:\n  kind: web_page\n  id: src\n  origin_url: https://x\ncategory:\n  - test-tree\nkeywords: []\ncreated_at: \"2026-01-01T00:00:00Z\"\nupdated_at: \"2026-01-01T00:00:00Z\"\ncross_refs: []\ncontent_hash: \"h\"\n---\n\n{body}\n"
    );
    std::fs::write(staged_dir.join(format!("{slug}.md")), raw).unwrap();
}

fn load_kb_config(kb_dir: &Path) -> Config {
    curio::config::load_config(None, Some(kb_dir)).expect("load_config")
}

#[tokio::test]
async fn publish_refuses_low_quality_page() {
    let tmp = tempfile::tempdir().unwrap();
    let kb_dir = tmp.path();
    scaffold_kb(kb_dir);

    // Body designed to fail assess_quality: short, no structure, contains
    // the "todo" placeholder pattern.
    write_staged_page(&kb_dir.join("wiki"), "weak", "Weak Page", "todo\n");
    run_git(kb_dir, &["add", "-A"]);
    run_git(kb_dir, &["commit", "-q", "-m", "stage"]);

    let config = load_kb_config(kb_dir);
    let res = run_publish(
        &config,
        false,
        false,
        "weak".into(),
        Some("test-tree".into()),
        false, // force = false
    )
    .await;
    let err = res.expect_err("publish must refuse low-quality page");
    let msg = format!("{:#}", err);
    assert!(
        msg.contains("too weak") || msg.contains("information quality"),
        "expected quality-gate refusal, got: {msg}"
    );

    // Page must still be in staged, not moved.
    assert!(
        kb_dir.join("wiki/staged/test-tree/weak.md").exists(),
        "page must stay in staged after refusal"
    );
    assert!(
        !kb_dir.join("wiki/published/test-tree/weak.md").exists(),
        "page must NOT have been published"
    );
}

#[tokio::test]
async fn publish_force_bypasses_gate_and_logs_audit() {
    let tmp = tempfile::tempdir().unwrap();
    let kb_dir = tmp.path();
    scaffold_kb(kb_dir);

    write_staged_page(&kb_dir.join("wiki"), "weak", "Weak Page", "todo\n");
    run_git(kb_dir, &["add", "-A"]);
    run_git(kb_dir, &["commit", "-q", "-m", "stage"]);

    let config = load_kb_config(kb_dir);
    let res = run_publish(
        &config,
        false,
        false,
        "weak".into(),
        Some("test-tree".into()),
        true, // force = true
    )
    .await;
    res.expect("publish --force must succeed");

    // Page must be moved.
    assert!(
        kb_dir.join("wiki/published/test-tree/weak.md").exists(),
        "force-publish must have moved the page"
    );
    assert!(
        !kb_dir.join("wiki/staged/test-tree/weak.md").exists(),
        "page must no longer be in staged"
    );

    // Audit log must record the bypass.
    let log =
        std::fs::read_to_string(kb_dir.join("wiki/_admin/log.md")).expect("log.md must exist");
    assert!(
        log.contains("FORCE BYPASSED"),
        "audit log must record bypass; got:\n{log}"
    );
    assert!(
        log.contains("quality"),
        "audit log must name the bypassed dimension; got:\n{log}"
    );
}
