use curio::retrieval::{RetrieveRequest, retrieve_published};
use serde_json::Value;
use std::{
    path::{Path, PathBuf},
    process::Command,
};

fn run_git(dir: &Path, args: &[&str]) {
    let status = Command::new("git")
        .args(args)
        .current_dir(dir)
        .env("GIT_AUTHOR_NAME", "retrieval-test")
        .env("GIT_AUTHOR_EMAIL", "retrieval@example.com")
        .env("GIT_COMMITTER_NAME", "retrieval-test")
        .env("GIT_COMMITTER_EMAIL", "retrieval@example.com")
        .status()
        .expect("git invocation");
    assert!(status.success(), "git {:?} failed", args);
}

fn scaffold(temp: &Path, with_git: bool) -> PathBuf {
    let wiki = temp.join("wiki");
    for lane in ["intake", "staged", "review", "published"] {
        std::fs::create_dir_all(wiki.join(lane)).unwrap();
    }
    std::fs::write(
        temp.join(".curio.yaml"),
        "wiki:\n  wiki_dir: wiki\n  auto_commit: false\n  sync:\n    enabled: false\n",
    )
    .unwrap();
    if with_git {
        run_git(temp, &["init", "-q", "-b", "main"]);
    }
    wiki
}

#[allow(clippy::too_many_arguments)]
fn page(
    wiki: &Path,
    lane: &str,
    relative: &str,
    title: &str,
    category: &[&str],
    keywords: &[&str],
    summary: &str,
    origin_url: Option<&str>,
    hash: &str,
    body: &str,
) {
    let path = wiki.join(lane).join(relative);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let category = if category.is_empty() {
        "[]".to_string()
    } else {
        category
            .iter()
            .map(|item| format!("\n  - {item}"))
            .collect()
    };
    let keywords = if keywords.is_empty() {
        "[]".to_string()
    } else {
        keywords
            .iter()
            .map(|item| format!("\n  - {item}"))
            .collect()
    };
    let origin = origin_url.unwrap_or("null");
    let raw = format!(
        "---\nid: source-id-{title}\ntitle: '{title}'\nstatus: {lane}\nsource:\n  kind: file\n  id: source-{title}\n  origin_url: {origin}\n  summary: '{summary}'\ncategory: {category}\nkeywords: {keywords}\ncreated_at: '2026-01-01T00:00:00Z'\nupdated_at: '2026-05-01T00:00:00Z'\ncross_refs: []\ncontent_hash: '{hash}'\nconfluence_page_id: null\nmodel_used: null\n---\n\n{body}\n"
    );
    std::fs::write(path, raw).unwrap();
}

fn fixture() -> (tempfile::TempDir, PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    let wiki = scaffold(temp.path(), true);
    for lane in ["intake", "staged", "review"] {
        std::fs::write(
            wiki.join(lane).join("deployment.md"),
            "deployment from a non-published lane",
        )
        .unwrap();
    }
    std::fs::write(
        wiki.join("published").join("index.md"),
        "deployment generated index",
    )
    .unwrap();
    std::fs::create_dir_all(wiki.join("published/product-tree")).unwrap();
    std::fs::write(
        wiki.join("published/product-tree/index.md"),
        "deployment branch index",
    )
    .unwrap();
    std::fs::write(
        wiki.join("published").join("ignored.feedback.md"),
        "deployment sidecar",
    )
    .unwrap();

    page(
        &wiki,
        "published",
        "alpha.md",
        "Deployment Runbook",
        &["product-tree"],
        &["deployment"],
        "Published deployment summary.",
        Some("https://example.test/deployment"),
        "alpha-hash",
        "Use the deployment runbook to roll out the service.",
    );
    page(
        &wiki,
        "published",
        "keyword-a.md",
        "Operations Guide",
        &["topic-tree"],
        &["deployment"],
        "Operations reference.",
        None,
        "keyword-a-hash",
        "General operational guidance.",
    );
    page(
        &wiki,
        "published",
        "keyword-b.md",
        "Operations Guide",
        &["topic-tree"],
        &["deployment"],
        "Operations reference.",
        None,
        "keyword-b-hash",
        "General operational guidance.",
    );
    page(
        &wiki,
        "published",
        "product-tree/server.md",
        "Server Operations",
        &["product-tree", "server"],
        &[],
        "Server reference.",
        None,
        "server-hash",
        "The deployment process for the server.",
    );
    page(
        &wiki,
        "published",
        "summary.md",
        "Operations Summary",
        &["topic-tree"],
        &[],
        "Deployment summary for operators.",
        None,
        "summary-hash",
        "General operational guidance.",
    );
    page(
        &wiki,
        "published",
        "body.md",
        "Operations Notes",
        &["topic-tree"],
        &[],
        "General notes.",
        None,
        "body-hash",
        "Deployment notes are included in this body.",
    );
    run_git(temp.path(), &["add", "-A"]);
    run_git(temp.path(), &["commit", "-q", "-m", "retrieval fixture"]);
    (temp, wiki)
}

fn request(category: Option<&str>, limit: usize) -> RetrieveRequest {
    RetrieveRequest {
        query: "deployment".into(),
        category: category.map(str::to_string),
        limit,
    }
}

#[test]
fn published_only_excludes_lanes_indexes_and_sidecars() {
    let (_temp, wiki) = fixture();
    let result = retrieve_published(&wiki, &request(None, 20)).unwrap();
    let paths: Vec<_> = result
        .results
        .iter()
        .map(|item| item.path.as_str())
        .collect();
    assert_eq!(
        paths,
        vec![
            "alpha.md",
            "keyword-a.md",
            "keyword-b.md",
            "summary.md",
            "body.md",
            "product-tree/server.md",
        ]
    );
}

#[test]
fn ordering_and_tie_breaks_are_repeatable() {
    let (_temp, wiki) = fixture();
    let first = retrieve_published(&wiki, &request(None, 20)).unwrap();
    let second = retrieve_published(&wiki, &request(None, 20)).unwrap();
    assert_eq!(first, second);
    assert_eq!(first.results[0].path, "alpha.md");
    assert_eq!(first.results[1].path, "keyword-a.md");
    assert_eq!(first.results[2].path, "keyword-b.md");
    assert!(
        first
            .results
            .iter()
            .position(|item| item.path == "summary.md")
            .unwrap()
            < first
                .results
                .iter()
                .position(|item| item.path == "body.md")
                .unwrap()
    );
}

#[test]
fn category_filter_and_limit_are_deterministic() {
    let (_temp, wiki) = fixture();
    let limited = retrieve_published(&wiki, &request(Some("product-tree"), 1)).unwrap();
    assert_eq!(limited.count, 1);
    assert_eq!(limited.results[0].path, "alpha.md");
    let all = retrieve_published(&wiki, &request(Some("product-tree"), 20)).unwrap();
    assert_eq!(
        all.results
            .iter()
            .map(|item| item.path.as_str())
            .collect::<Vec<_>>(),
        vec!["alpha.md", "product-tree/server.md"]
    );
}

#[test]
fn provenance_includes_source_hash_time_authority_and_git_commit() {
    let (_temp, wiki) = fixture();
    let result = retrieve_published(&wiki, &request(None, 5)).unwrap();
    let alpha = &result.results[0];
    assert_eq!(
        alpha.source_uri.as_deref(),
        Some("https://example.test/deployment")
    );
    assert_eq!(alpha.content_hash, "alpha-hash");
    assert_eq!(alpha.updated_at, "2026-05-01T00:00:00Z");
    assert_eq!(alpha.authority, "published");
    let commit = alpha.last_commit.as_ref().unwrap();
    assert_eq!(commit.author, "retrieval-test");
    assert_eq!(commit.subject, "retrieval fixture");
    assert_eq!(commit.hash.len(), 40);
}

#[test]
fn non_git_workspace_returns_null_commit() {
    let temp = tempfile::tempdir().unwrap();
    let wiki = scaffold(temp.path(), false);
    page(
        &wiki,
        "published",
        "page.md",
        "Deployment Page",
        &["product-tree"],
        &[],
        "Summary.",
        None,
        "hash",
        "Deployment body.",
    );
    let result = retrieve_published(&wiki, &request(None, 5)).unwrap();
    assert!(result.results[0].last_commit.is_none());
}

#[test]
fn empty_and_stopword_only_queries_are_rejected() {
    let (_temp, wiki) = fixture();
    for query in ["", "the and of"] {
        let error = retrieve_published(
            &wiki,
            &RetrieveRequest {
                query: query.into(),
                category: None,
                limit: 5,
            },
        )
        .expect_err("invalid query must fail");
        let validation = error
            .downcast_ref::<curio::error::CliValidationError>()
            .unwrap();
        assert_eq!(validation.code, "invalid_query");
        assert!(validation.message.contains("meaningful"));
    }
}

#[test]
fn cli_emits_actionable_json_validation_error() {
    let (_temp, wiki) = fixture();
    let output = Command::new(env!("CARGO_BIN_EXE_curio"))
        .args([
            "--kb-dir",
            wiki.parent().unwrap().to_str().unwrap(),
            "retrieve",
            "--query",
            "the and of",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["command"], "retrieve");
    assert_eq!(json["ok"], false);
    assert_eq!(json["error"]["code"], "invalid_query");
    assert!(json["error"]["hint"].as_str().unwrap().contains("--query"));
}
