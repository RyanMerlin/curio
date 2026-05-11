use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate should live under the repo root")
        .to_path_buf()
}

#[test]
fn synthetic_demo_workspace_has_the_expected_shape() {
    let wiki = repo_root().join("docs").join("wiki-demo");

    assert!(
        wiki.join("README.md").is_file(),
        "demo landing page missing"
    );
    assert!(
        wiki.join("_config").join("northstar.md").is_file(),
        "demo NORTHSTAR missing"
    );
    assert!(
        wiki.join("_config").join("settings.yaml").is_file(),
        "demo settings missing"
    );

    let expected = [
        wiki.join("intake").join("example-intake.md"),
        wiki.join("staged")
            .join("product-tree")
            .join("demo-ingest.md"),
        wiki.join("review")
            .join("product-tree")
            .join("demo-review.md"),
        wiki.join("published").join("index.md"),
        wiki.join("published").join("product-tree").join("index.md"),
        wiki.join("published")
            .join("product-tree")
            .join("demo-publish.md"),
    ];

    for path in expected {
        assert!(
            path.is_file(),
            "expected demo wiki file missing: {}",
            path.display()
        );
    }
}

#[test]
fn example_workspace_alias_is_present_in_the_example_workspace_file() {
    let workspace_file = repo_root().join("curio.workspaces.example.toml");
    let raw = std::fs::read_to_string(&workspace_file)
        .unwrap_or_else(|err| panic!("failed to read {}: {}", workspace_file.display(), err));

    assert!(
        raw.contains("name = \"wiki-demo\""),
        "expected wiki-demo workspace alias to be registered"
    );
    assert!(
        raw.contains("${REPO_ROOT}/docs/wiki-demo"),
        "expected the example workspace to point at the synthetic demo repo root"
    );
}
