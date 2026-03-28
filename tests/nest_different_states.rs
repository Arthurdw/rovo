use aide::openapi::OpenApi;
use axum::body::Body;
use axum::http::Request;
use rovo::aide;
use rovo::extract::State;
use rovo::http::StatusCode;
use rovo::response::Json;
use rovo::schemars::JsonSchema;
use rovo::{routing::get, rovo, Router};
use serde::{Deserialize, Serialize};
use tower::util::ServiceExt;

// Two completely separate state types
#[derive(Clone)]
struct StateA {
    value_a: &'static str,
}

#[derive(Clone)]
struct StateB {
    value_b: u32,
}

/// Get A
///
/// # Responses
///
/// 200: Json<ResponseA> - Success
///
/// # Metadata
///
/// @tag group_a
#[rovo]
async fn handler_a(State(state): State<StateA>) -> Json<ResponseA> {
    Json(ResponseA {
        msg: state.value_a.to_string(),
    })
}

/// Get B
///
/// # Responses
///
/// 200: Json<ResponseB> - Success
///
/// # Metadata
///
/// @tag group_b
#[rovo]
async fn handler_b(State(state): State<StateB>) -> Json<ResponseB> {
    Json(ResponseB { num: state.value_b })
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct ResponseA {
    msg: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct ResponseB {
    num: u32,
}

#[tokio::test]
async fn test_nest_different_state_types() {
    let state_a = StateA { value_a: "hello" };
    let state_b = StateB { value_b: 42 };

    let mut api = OpenApi::default();
    api.info.title = "Multi-state API".to_string();

    // This is the pattern from the issue: nest routers with different state types
    let app = Router::new()
        .nest(
            "/api/a",
            Router::new()
                .route("/resource", get(handler_a))
                .with_state(state_a),
        )
        .nest(
            "/api/b",
            Router::new()
                .route("/resource", get(handler_b))
                .with_state(state_b),
        )
        .with_oas(api)
        .finish();

    // Verify route A works and returns correct data from StateA
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/a/resource")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let result: ResponseA = serde_json::from_slice(&body).unwrap();
    assert_eq!(result.msg, "hello");

    // Verify route B works and returns correct data from StateB
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/b/resource")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let result: ResponseB = serde_json::from_slice(&body).unwrap();
    assert_eq!(result.num, 42);
}

#[tokio::test]
async fn test_nest_axum_router_into_stateless_parent() {
    // Nest a single with_state router into a stateless parent
    let state_a = StateA { value_a: "works" };

    let app = Router::<()>::new()
        .nest(
            "/nested",
            Router::new()
                .route("/endpoint", get(handler_a))
                .with_state(state_a),
        )
        .finish();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/nested/endpoint")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let result: ResponseA = serde_json::from_slice(&body).unwrap();
    assert_eq!(result.msg, "works");
}

#[tokio::test]
async fn test_nest_same_state_still_works() {
    // Ensure the existing behavior (nesting Router<S> into Router<S>) is not broken
    let state_a = StateA { value_a: "same" };

    let mut api = OpenApi::default();
    api.info.title = "Same-state API".to_string();

    let app = Router::new()
        .nest("/nested", Router::new().route("/endpoint", get(handler_a)))
        .with_oas(api)
        .with_state(state_a);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/nested/endpoint")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let result: ResponseA = serde_json::from_slice(&body).unwrap();
    assert_eq!(result.msg, "same");
}

#[tokio::test]
async fn test_nest_different_states_with_parent_state() {
    // Parent router also has its own state, plus nested routers with different states

    /// Get from parent
    ///
    /// # Responses
    ///
    /// 200: Json<ResponseA> - Success
    #[rovo]
    async fn parent_handler(State(state): State<StateA>) -> Json<ResponseA> {
        Json(ResponseA {
            msg: state.value_a.to_string(),
        })
    }

    let state_a = StateA {
        value_a: "from_parent",
    };
    let state_b = StateB { value_b: 99 };

    let app = Router::new()
        .route("/parent", get(parent_handler))
        .nest(
            "/child",
            Router::new()
                .route("/endpoint", get(handler_b))
                .with_state(state_b),
        )
        .with_state(state_a);

    // Verify parent route works
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/parent")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let result: ResponseA = serde_json::from_slice(&body).unwrap();
    assert_eq!(result.msg, "from_parent");

    // Verify nested child route with different state works
    let response = app
        .oneshot(
            Request::builder()
                .uri("/child/endpoint")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let result: ResponseB = serde_json::from_slice(&body).unwrap();
    assert_eq!(result.num, 99);
}

#[tokio::test]
async fn test_nest_three_different_states() {
    // Three separate state types nested into a single parent
    #[derive(Clone)]
    struct StateC {
        flag: bool,
    }

    #[derive(Serialize, Deserialize, JsonSchema)]
    struct ResponseC {
        flag: bool,
    }

    /// Get C
    ///
    /// # Responses
    ///
    /// 200: Json<ResponseC> - Success
    #[rovo]
    async fn handler_c(State(state): State<StateC>) -> Json<ResponseC> {
        Json(ResponseC { flag: state.flag })
    }

    let app = Router::<()>::new()
        .nest(
            "/a",
            Router::new()
                .route("/data", get(handler_a))
                .with_state(StateA { value_a: "aaa" }),
        )
        .nest(
            "/b",
            Router::new()
                .route("/data", get(handler_b))
                .with_state(StateB { value_b: 7 }),
        )
        .nest(
            "/c",
            Router::new()
                .route("/data", get(handler_c))
                .with_state(StateC { flag: true }),
        )
        .finish();

    // Verify all three routes
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/a/data")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let result: ResponseA = serde_json::from_slice(&body).unwrap();
    assert_eq!(result.msg, "aaa");

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/b/data")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let result: ResponseB = serde_json::from_slice(&body).unwrap();
    assert_eq!(result.num, 7);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/c/data")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let result: ResponseC = serde_json::from_slice(&body).unwrap();
    assert!(result.flag);
}

#[tokio::test]
async fn test_nest_different_states_nonexistent_route_returns_404() {
    let app = Router::<()>::new()
        .nest(
            "/api/a",
            Router::new()
                .route("/resource", get(handler_a))
                .with_state(StateA { value_a: "x" }),
        )
        .finish();

    // Route that doesn't exist should return 404
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/b/resource")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_nest_plain_axum_router() {
    // Nest a plain axum::Router (not from rovo) directly
    let axum_router = axum::Router::new().route("/health", axum::routing::get(|| async { "ok" }));

    let app = Router::<()>::new().nest("/api", axum_router).finish();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(&body[..], b"ok");
}
