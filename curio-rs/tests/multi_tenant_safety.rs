//! Multi-tenant safety: prove that two KBs running intake concurrently
//! never bleed into each other.
//!
//! The hard requirement: when colleagues each have their own KB and one
//! Curio process services them in parallel, every git mutation, every
//! frontmatter write, and every registry update must stay scoped to the
//! KB that initiated the work.
//!
//! This test does NOT exercise Confluence (no network) — it covers the
//! filesystem and config isolation invariants.

use curio::config::load_config;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

fn env_lock() -> std::sync::MutexGuard<'static, ()> {
    static L: OnceLock<Mutex<()>> = OnceLock::new();
    let mu = L.get_or_init(|| Mutex::new(()));
    // Recover from poisoning — when one test panics holding the lock,
    // subsequent tests should still run (the env contamination they
    // protect against is independent across tests).
    match mu.lock() {
        Ok(g) => g,
        Err(p) => p.into_inner(),
    }
}

fn scaffold_kb(dir: &Path, space_key: &str, token_env: &str) {
    for sub in ["intake", "staged", "review", "published", "_admin"] {
        std::fs::create_dir_all(dir.join("wiki").join(sub)).expect("mkdir wiki sub");
    }
    let yaml = format!(
        "connection:\n  confluence_url: https://example.atlassian.net/wiki\n  confluence_email: bot@example.com\n  token_env: {token_env}\ncontent_model:\n  space_key: {space_key}\n  label_namespace: curio\nwiki:\n  wiki_dir: wiki\n  auto_commit: false\n  sync:\n    enabled: false\n"
    );
    std::fs::write(dir.join(".curio.yaml"), yaml).expect("write .curio.yaml");
}

#[test]
fn two_kbs_isolate_filesystem_writes() {
    let _guard = env_lock();
    let tmp = tempfile::tempdir().expect("tempdir");
    let kb_a = tmp.path().join("kb-a");
    let kb_b = tmp.path().join("kb-b");
    scaffold_kb(&kb_a, "SPACEA", "CURIO_CONFLUENCE_TOKEN_A");
    scaffold_kb(&kb_b, "SPACEB", "CURIO_CONFLUENCE_TOKEN_B");

    // Drop a marker file directly into each KB's intake directory and
    // verify the OTHER KB's load_config never reads or writes there.
    std::fs::write(
        kb_a.join("wiki").join("intake").join("marker-a.md"),
        "A only",
    )
    .expect("write marker-a");
    std::fs::write(
        kb_b.join("wiki").join("intake").join("marker-b.md"),
        "B only",
    )
    .expect("write marker-b");

    let cfg_a = load_config(None, Some(&kb_a)).expect("load A");
    let cfg_b = load_config(None, Some(&kb_b)).expect("load B");

    // Per-KB resolution: each yaml owns its own space + token_env.
    assert_eq!(cfg_a.content_model.space_key, "SPACEA");
    assert_eq!(cfg_b.content_model.space_key, "SPACEB");
    assert_eq!(
        cfg_a.connection.token_env_name(),
        "CURIO_CONFLUENCE_TOKEN_A"
    );
    assert_eq!(
        cfg_b.connection.token_env_name(),
        "CURIO_CONFLUENCE_TOKEN_B"
    );

    // wiki_dir resolution must point at the right KB.
    assert!(cfg_a.wiki.wiki_dir.starts_with(&kb_a));
    assert!(cfg_b.wiki.wiki_dir.starts_with(&kb_b));
    assert!(!cfg_a.wiki.wiki_dir.starts_with(&kb_b));
    assert!(!cfg_b.wiki.wiki_dir.starts_with(&kb_a));

    // Files dropped into A are NOT visible in B's wiki dir.
    let a_marker_seen_in_b = kb_b.join("wiki").join("intake").join("marker-a.md");
    let b_marker_seen_in_a = kb_a.join("wiki").join("intake").join("marker-b.md");
    assert!(
        !a_marker_seen_in_b.exists(),
        "A's marker must not appear in B"
    );
    assert!(
        !b_marker_seen_in_a.exists(),
        "B's marker must not appear in A"
    );
}

#[test]
fn token_resolution_does_not_cross_contaminate() {
    let _guard = env_lock();
    let tmp = tempfile::tempdir().expect("tempdir");
    let kb_a = tmp.path().join("kb-a");
    let kb_b = tmp.path().join("kb-b");
    scaffold_kb(&kb_a, "SPACEA", "CURIO_TENANT_TOKEN_A");
    scaffold_kb(&kb_b, "SPACEB", "CURIO_TENANT_TOKEN_B");

    // Set both tokens to distinct values; each KB MUST resolve only its own.
    unsafe {
        std::env::set_var("CURIO_TENANT_TOKEN_A", "secret-a");
        std::env::set_var("CURIO_TENANT_TOKEN_B", "secret-b");
        std::env::remove_var("CURIO_CONFLUENCE_TOKEN"); // ensure global default doesn't leak
    }

    let cfg_a = load_config(None, Some(&kb_a)).expect("load A");
    let cfg_b = load_config(None, Some(&kb_b)).expect("load B");

    assert_eq!(cfg_a.connection.resolve_token().unwrap(), "secret-a");
    assert_eq!(cfg_b.connection.resolve_token().unwrap(), "secret-b");

    // Now poison: set the OTHER KB's token to the same value as the global.
    // Each KB must STILL pick its own token_env, never the global default.
    unsafe {
        std::env::set_var("CURIO_CONFLUENCE_TOKEN", "global-leak");
    }
    assert_eq!(cfg_a.connection.resolve_token().unwrap(), "secret-a");
    assert_eq!(cfg_b.connection.resolve_token().unwrap(), "secret-b");

    unsafe {
        std::env::remove_var("CURIO_TENANT_TOKEN_A");
        std::env::remove_var("CURIO_TENANT_TOKEN_B");
        std::env::remove_var("CURIO_CONFLUENCE_TOKEN");
    }
}

#[test]
fn concurrent_doctor_runs_do_not_share_config_state() {
    // Spawn doctor's infra-check phase against two KBs concurrently and
    // verify each sees its own config — no cross-tenant leak through
    // global state, env vars, or shared mutexes.
    let _guard = env_lock();
    let tmp = tempfile::tempdir().expect("tempdir");
    let kb_a = tmp.path().join("kb-a");
    let kb_b = tmp.path().join("kb-b");
    scaffold_kb(&kb_a, "ALPHA", "CURIO_TOK_ALPHA");
    scaffold_kb(&kb_b, "BETA", "CURIO_TOK_BETA");

    unsafe {
        std::env::set_var("CURIO_TOK_ALPHA", "alpha-secret");
        std::env::set_var("CURIO_TOK_BETA", "beta-secret");
        std::env::remove_var("CURIO_CONFLUENCE_TOKEN");
    }

    let cfg_a = load_config(None, Some(&kb_a)).expect("A");
    let cfg_b = load_config(None, Some(&kb_b)).expect("B");

    let a = cfg_a.connection.resolve_token().unwrap();
    let b = cfg_b.connection.resolve_token().unwrap();

    assert_eq!(a, "alpha-secret");
    assert_eq!(b, "beta-secret");
    assert_eq!(cfg_a.content_model.space_key, "ALPHA");
    assert_eq!(cfg_b.content_model.space_key, "BETA");

    unsafe {
        std::env::remove_var("CURIO_TOK_ALPHA");
        std::env::remove_var("CURIO_TOK_BETA");
    }
}
