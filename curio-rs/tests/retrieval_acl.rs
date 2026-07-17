use curio::{
    acl::{AccessContext, AclSnapshot, Principal, PrincipalKind},
    retrieval::{RetrieveRequest, fetch_published_with_access, retrieve_published_with_access},
};
use std::fs;
use tempfile::tempdir;

fn page(root: &std::path::Path, name: &str, source: &str) {
    fs::create_dir_all(root.join("published")).unwrap();
    fs::write(root.join("published").join(name), format!("---\nid: {name}\ntitle: Restricted {name}\nstatus: published\nsource:\n  kind: file\n  id: {source}\n  origin_url: null\n  summary: restricted guidance\ncategory: [security]\nkeywords: [restricted]\ncreated_at: '2026-01-01T00:00:00Z'\nupdated_at: '2026-01-01T00:00:00Z'\ncontent_hash: hash\nconfluence_page_id: null\nmodel_used: null\n---\n\nRestricted guidance for operators.\n")).unwrap();
}

#[test]
fn restricted_pages_fail_closed_for_search_and_guessed_fetch() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    page(root, "secret.md", "source-secret");
    fs::create_dir_all(root.join("_admin/acl")).unwrap();
    let snapshot = AclSnapshot {
        source_id: "source-secret".into(),
        source_revision: "r1".into(),
        captured_at: "2026-07-17T00:00:00Z".into(),
        allow: vec![Principal {
            kind: PrincipalKind::User,
            id: "alice".into(),
            label: None,
        }],
        deny: vec![],
    };
    fs::write(
        root.join("_admin/acl/secret.json"),
        serde_json::to_vec(&snapshot).unwrap(),
    )
    .unwrap();
    let request = RetrieveRequest {
        query: "restricted guidance".into(),
        category: None,
        limit: 5,
    };
    let bob = AccessContext::new(["bob".into()]);
    assert!(
        retrieve_published_with_access(root, &request, Some(&bob))
            .unwrap()
            .results
            .is_empty()
    );
    let id =
        retrieve_published_with_access(root, &request, Some(&AccessContext::new(["alice".into()])))
            .unwrap()
            .results[0]
            .id
            .clone();
    assert!(fetch_published_with_access(root, &id, Some(&bob)).is_err());
}

#[test]
fn deny_overrides_group_allow_and_public_pages_remain_compatible() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    page(root, "secret.md", "source-secret");
    page(root, "public.md", "source-public");
    fs::create_dir_all(root.join("_admin/acl")).unwrap();
    let snapshot = AclSnapshot {
        source_id: "source-secret".into(),
        source_revision: "r1".into(),
        captured_at: "now".into(),
        allow: vec![Principal {
            kind: PrincipalKind::Group,
            id: "operators".into(),
            label: None,
        }],
        deny: vec![Principal {
            kind: PrincipalKind::User,
            id: "bob".into(),
            label: None,
        }],
    };
    fs::write(
        root.join("_admin/acl/secret.json"),
        serde_json::to_vec(&snapshot).unwrap(),
    )
    .unwrap();
    let mut access = AccessContext::default();
    access.group_ids.push("operators".into());
    access.principal_ids.push("bob".into());
    let response = retrieve_published_with_access(
        root,
        &RetrieveRequest {
            query: "guidance".into(),
            category: None,
            limit: 5,
        },
        Some(&access),
    )
    .unwrap();
    assert_eq!(response.results.len(), 1);
    assert_eq!(response.results[0].path, "public.md");
}
