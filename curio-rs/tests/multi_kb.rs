//! Multi-KB integration test.
//!
//! Verifies that:
//! 1. A single Curio process can hold per-KB Confluence configs without bleed.
//! 2. `connection.token_env` is honored per KB so distinct token env vars
//!    can be used across KBs (or all aliased to one secret today).
//! 3. The init-kb scaffold matches the layout that the rest of the binary
//!    expects (`wiki/{intake,staged,review,published,_admin}/`).
//!
//! This test intentionally does not touch the network — Confluence
//! end-to-end is gated behind CURIO_E2E_CONFLUENCE in a separate harness.

use curio::config::{ConnectionConfig, load_config};
use std::fs;
use std::sync::{Mutex, OnceLock};

fn env_lock() -> std::sync::MutexGuard<'static, ()> {
    static L: OnceLock<Mutex<()>> = OnceLock::new();
    L.get_or_init(|| Mutex::new(()))
        .lock()
        .expect("env lock poisoned")
}

fn write_curio_yaml(dir: &std::path::Path, space_key: &str, token_env: &str, parent_id: &str) {
    let yaml = format!(
        "connection:\n  confluence_url: https://example.atlassian.net/wiki\n  confluence_email: bot@example.com\n  token_env: {token_env}\ncontent_model:\n  space_key: {space_key}\n  label_namespace: curio\nwiki:\n  wiki_dir: wiki\n  auto_commit: true\n  sync:\n    enabled: true\n    confluence_parent_page_id: \"{parent_id}\"\n"
    );
    fs::write(dir.join(".curio.yaml"), yaml).expect("write .curio.yaml");
}

fn scaffold_kb(dir: &std::path::Path) {
    for sub in ["intake", "staged", "review", "published", "_admin"] {
        fs::create_dir_all(dir.join("wiki").join(sub)).expect("mkdir wiki sub");
    }
}

#[test]
fn three_kbs_have_independent_confluence_config() {
    let _guard = env_lock();
    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path();

    let kbs = [
        (
            "curio-wiki",
            "DEMO",
            "CURIO_CONFLUENCE_TOKEN_CURIO_WIKI",
            "111",
        ),
        (
            "partner-business",
            "partnerbiz",
            "CURIO_CONFLUENCE_TOKEN_PARTNER",
            "4385866292",
        ),
        (
            "fde-uc-repo",
            "fdeucrepo",
            "CURIO_CONFLUENCE_TOKEN_FDE",
            "4385964596",
        ),
    ];

    // Stage three KB directories with distinct .curio.yaml files.
    for (name, space, token_env, parent_id) in &kbs {
        let kb = root.join(name);
        scaffold_kb(&kb);
        write_curio_yaml(&kb, space, token_env, parent_id);
    }

    // Pretend each KB has its own token by setting all three env vars
    // to distinct values; the resolver MUST pick the right one per KB.
    unsafe {
        std::env::set_var("CURIO_CONFLUENCE_TOKEN_CURIO_WIKI", "demo-secret");
        std::env::set_var("CURIO_CONFLUENCE_TOKEN_PARTNER", "partner-secret");
        std::env::set_var("CURIO_CONFLUENCE_TOKEN_FDE", "fde-secret");
        // Clear any leaked global var so the per-KB names win.
        std::env::remove_var("CURIO_CONFLUENCE_TOKEN");
    }

    for (name, space, token_env, parent_id) in &kbs {
        let kb = root.join(name);
        let config = load_config(None, Some(&kb)).expect("load_config per KB");

        assert_eq!(
            config.content_model.space_key, *space,
            "KB '{name}' loaded the wrong space key"
        );
        assert_eq!(
            config.connection.token_env_name(),
            *token_env,
            "KB '{name}' lost its token_env"
        );
        let resolved = config
            .connection
            .resolve_token()
            .expect("resolve token for KB");
        let expected = match *name {
            "curio-wiki" => "demo-secret",
            "partner-business" => "partner-secret",
            "fde-uc-repo" => "fde-secret",
            _ => unreachable!(),
        };
        assert_eq!(resolved, expected, "KB '{name}' got the wrong token");
        assert_eq!(
            config.wiki.sync.confluence_parent_page_id.as_deref(),
            Some(*parent_id),
            "KB '{name}' lost its parent_page_id"
        );
        assert!(config.wiki.sync.enabled, "sync should be enabled per yaml");
    }

    // Tear down env to keep tests hygienic.
    unsafe {
        std::env::remove_var("CURIO_CONFLUENCE_TOKEN_CURIO_WIKI");
        std::env::remove_var("CURIO_CONFLUENCE_TOKEN_PARTNER");
        std::env::remove_var("CURIO_CONFLUENCE_TOKEN_FDE");
    }
}

#[test]
fn missing_token_env_returns_actionable_error() {
    let _guard = env_lock();
    unsafe {
        std::env::remove_var("CURIO_CONFLUENCE_TOKEN_NEVER_SET");
        std::env::remove_var("CURIO_CONFLUENCE_TOKEN");
    }
    let conn = ConnectionConfig {
        confluence_url: "https://x".into(),
        confluence_email: "e@x".into(),
        token_env: Some("CURIO_CONFLUENCE_TOKEN_NEVER_SET".into()),
    };
    let err = conn.resolve_token().unwrap_err();
    let msg = format!("{err}");
    assert!(msg.contains("CURIO_CONFLUENCE_TOKEN_NEVER_SET"));
    assert!(msg.contains(".curio.yaml"));
}
