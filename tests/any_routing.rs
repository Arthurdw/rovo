use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{Method, Request, StatusCode};
use tower::ServiceExt;

// -- Handlers (plain axum, no #[rovo]) --

async fn echo_method(method: Method) -> String {
    method.to_string()
}

async fn catch_all(Path(path): Path<String>) -> String {
    path
}

#[derive(Clone)]
struct TestState {
    prefix: String,
}

async fn stateful_catch_all(State(state): State<TestState>, Path(path): Path<String>) -> String {
    format!("{}{}", state.prefix, path)
}

// -- Tests --

/// Test 1: any handler receives GET, POST, PUT, DELETE requests
#[tokio::test]
async fn any_handler_receives_all_methods() {
    let app = rovo::Router::new()
        .route("/test", rovo::routing::any(echo_method))
        .finish();

    for method in [Method::GET, Method::POST, Method::PUT, Method::DELETE] {
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(&method)
                    .uri("/test")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let body = axum::body::to_bytes(resp.into_body(), 1024).await.unwrap();
        assert_eq!(
            String::from_utf8(body.to_vec()).unwrap(),
            method.to_string(),
            "body mismatch for {method}"
        );
    }
}

/// Test 2: Catch-all {*path} route matches /foo/bar/baz and the path param is extractable
#[tokio::test]
async fn catchall_path_is_extractable() {
    let app = rovo::Router::new()
        .route("/api/{*path}", rovo::routing::any(catch_all))
        .finish();

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/foo/bar/baz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), 1024).await.unwrap();
    assert_eq!(String::from_utf8(body.to_vec()).unwrap(), "foo/bar/baz");
}

/// Test 3: any with catch-all works with .with_state()
#[tokio::test]
async fn any_catchall_with_state() {
    let state = TestState {
        prefix: "proxied:".to_string(),
    };

    let app = rovo::Router::<TestState>::new()
        .route(
            "/api/internal/{*path}",
            rovo::routing::any(stateful_catch_all),
        )
        .with_state(state);

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/internal/foo/bar")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), 1024).await.unwrap();
    assert_eq!(String::from_utf8(body.to_vec()).unwrap(), "proxied:foo/bar");
}
