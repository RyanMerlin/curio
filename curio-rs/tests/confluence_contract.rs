use axum::{Json, Router, extract::Query};
use curio::confluence::ConfluenceClient;
use serde_json::json;
use std::collections::HashMap;
use tokio::net::TcpListener;

async fn start_router(router: Router) -> String {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind mock server");
    let addr = listener.local_addr().expect("mock address");
    tokio::spawn(async move {
        axum::serve(listener, router).await.expect("mock server");
    });
    format!("http://{}/wiki", addr)
}

#[tokio::test]
async fn descendants_follow_same_origin_continuation() {
    async fn descendants(Query(query): Query<HashMap<String, String>>) -> Json<serde_json::Value> {
        if query.contains_key("cursor") {
            Json(json!({"results": [{"id": "second"}]}))
        } else {
            Json(json!({
                "results": [{"id": "first"}],
                "_links": {"next": "/wiki/api/v2/pages/root/descendants?cursor=next"}
            }))
        }
    }
    let base = start_router(Router::new().fallback(descendants)).await;
    let client = ConfluenceClient::new(base, "test@example.com".into(), "secret".into(), None)
        .expect("client");
    let pages = client
        .get_page_descendants_v2("root")
        .await
        .expect("pagination succeeds");
    assert_eq!(pages.len(), 2);
}

#[tokio::test]
async fn cross_origin_continuation_is_rejected() {
    async fn descendants() -> Json<serde_json::Value> {
        Json(json!({
            "results": [],
            "_links": {"next": "https://evil.example/wiki/api/v2/pages/root/descendants?cursor=x"}
        }))
    }
    let base = start_router(Router::new().fallback(descendants)).await;
    let client = ConfluenceClient::new(base, "test@example.com".into(), "secret".into(), None)
        .expect("client");
    let error = client
        .get_page_descendants_v2("root")
        .await
        .expect_err("cross-origin continuation must fail");
    assert!(error.to_string().contains("cross-origin"));
}

#[test]
fn production_urls_require_wiki_and_https() {
    assert!(
        ConfluenceClient::new(
            "https://example.atlassian.net".into(),
            "e".into(),
            "t".into(),
            None,
        )
        .is_err()
    );
    assert!(
        ConfluenceClient::new(
            "http://example.atlassian.net/wiki".into(),
            "e".into(),
            "t".into(),
            None,
        )
        .is_err()
    );
}
