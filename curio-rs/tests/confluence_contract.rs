use axum::{Json, Router, extract::Query, http::StatusCode, response::IntoResponse};
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

#[tokio::test]
async fn sibling_base_path_continuation_is_rejected() {
    async fn descendants(request: axum::extract::Request) -> Json<serde_json::Value> {
        let host = request
            .headers()
            .get(axum::http::header::HOST)
            .unwrap()
            .to_str()
            .unwrap();
        Json(json!({
            "results": [],
            "_links": {"next": format!("http://{host}/wiki-evil/api/v2/pages/root/descendants?cursor=x")}
        }))
    }
    let base = start_router(Router::new().fallback(descendants)).await;
    let client = ConfluenceClient::new(base, "test@example.com".into(), "secret".into(), None)
        .expect("client");
    let error = client
        .get_page_descendants_v2("root")
        .await
        .expect_err("sibling base path must fail");
    assert!(error.to_string().contains("cross-origin"));
}

#[tokio::test]
async fn outside_root_title_collision_is_rejected_before_create() {
    async fn collision(request: axum::extract::Request) -> impl IntoResponse {
        let path = request.uri().path();
        match (request.method(), path) {
            (&axum::http::Method::GET, "/wiki/rest/api/content") => Json(json!({
                "results": [{"id": "foreign", "ancestors": [{"id": "root"}]}]
            }))
            .into_response(),
            (&axum::http::Method::GET, "/wiki/api/v2/pages/foreign") => {
                Json(json!({"id": "foreign", "parentId": "elsewhere"})).into_response()
            }
            (&axum::http::Method::GET, "/wiki/api/v2/pages/elsewhere") => {
                Json(json!({"id": "elsewhere"})).into_response()
            }
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "unexpected mutation").into_response(),
        }
    }

    let base = start_router(Router::new().fallback(collision)).await;
    let client = ConfluenceClient::new(
        base,
        "test@example.com".into(),
        "secret".into(),
        Some("root".into()),
    )
    .expect("client");
    let error = client
        .create_or_update_page("CURIO", Some("root"), "Foreign title", "storage", "body")
        .await
        .expect_err("outside-root title collision must be rejected");
    assert!(
        error
            .to_string()
            .contains("outside the configured CURIO output root")
    );
}

#[tokio::test]
async fn delete_retries_transient_server_errors() {
    async fn delete_page() -> impl IntoResponse {
        static ATTEMPTS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let attempt = ATTEMPTS.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        if attempt == 0 {
            (StatusCode::SERVICE_UNAVAILABLE, "retry").into_response()
        } else {
            StatusCode::NO_CONTENT.into_response()
        }
    }

    let base = start_router(Router::new().route(
        "/wiki/rest/api/content/owned",
        axum::routing::delete(delete_page),
    ))
    .await;
    let client = ConfluenceClient::new(base, "test@example.com".into(), "secret".into(), None)
        .expect("client");
    client.delete_page("owned").await.expect("delete retries");
}

#[tokio::test]
async fn v2_update_refetches_after_version_conflict() {
    async fn update(request: axum::extract::Request) -> impl IntoResponse {
        static GETS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        static PUTS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let path = request.uri().path();
        match (request.method(), path) {
            (&axum::http::Method::GET, "/wiki/rest/api/content") => Json(json!({
                "results": [{"id": "page", "ancestors": [{"id": "root"}]}]
            }))
            .into_response(),
            (&axum::http::Method::GET, "/wiki/api/v2/pages/page") => {
                let get = GETS.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                let version = if get < 2 { 1 } else { 2 };
                Json(json!({"id": "page", "version": {"number": version}, "parentId": "root"}))
                    .into_response()
            }
            (&axum::http::Method::PUT, "/wiki/api/v2/pages/page") => {
                let put = PUTS.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                if put == 0 {
                    (StatusCode::CONFLICT, "concurrent edit").into_response()
                } else {
                    Json(json!({"id": "page"})).into_response()
                }
            }
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "unexpected request").into_response(),
        }
    }

    let base = start_router(Router::new().fallback(update)).await;
    let client = ConfluenceClient::new(
        base,
        "test@example.com".into(),
        "secret".into(),
        Some("root".into()),
    )
    .expect("client");
    let page_id = client
        .create_or_update_page("CURIO", Some("root"), "Concurrent page", "storage", "body")
        .await
        .expect("update refetches after conflict");
    assert_eq!(page_id, "page");
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
