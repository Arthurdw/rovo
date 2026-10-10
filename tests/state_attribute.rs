//! `#[rovo(state = T)]` sets the router state type for handlers whose
//! extractors need state but that have no `State<T>` parameter.

use axum::body::Body;
use axum::http::Request;
use rovo::aide::OperationInput;
use rovo::extract::FromRequestParts;
use rovo::http::request::Parts;
use rovo::prelude::*;
use std::convert::Infallible;
use tower::util::ServiceExt;

#[derive(Clone)]
struct AppState {
    greeting: &'static str,
}

/// Reads the greeting from the router state.
struct Greeting(&'static str);

impl FromRequestParts<AppState> for Greeting {
    type Rejection = Infallible;

    async fn from_request_parts(_parts: &mut Parts, state: &AppState) -> Result<Self, Infallible> {
        Ok(Self(state.greeting))
    }
}

impl OperationInput for Greeting {}

/// Say hello.
///
/// # Responses
///
/// 200: String - The greeting
#[rovo(state = AppState)]
async fn hello(Greeting(greeting): Greeting) -> impl IntoApiResponse {
    greeting
}

/// Say hello with a fully qualified state type.
///
/// # Responses
///
/// 200: String - The greeting
#[rovo(state = crate::AppState)]
async fn hello_qualified(Greeting(greeting): Greeting) -> impl IntoApiResponse {
    greeting
}

async fn fetch(app: ::axum::Router, uri: &str) -> (StatusCode, String) {
    let response = app
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    (status, String::from_utf8(body.to_vec()).unwrap())
}

#[tokio::test]
async fn test_explicit_state_type() {
    let app = Router::new()
        .route("/hello", get(hello))
        .route("/hello-qualified", get(hello_qualified))
        .with_oas(OpenApi::default())
        .with_state(AppState { greeting: "hi" })
        .finish();

    assert_eq!(
        fetch(app.clone(), "/hello").await,
        (StatusCode::OK, "hi".to_string())
    );
    assert_eq!(
        fetch(app.clone(), "/hello-qualified").await,
        (StatusCode::OK, "hi".to_string())
    );

    let (status, spec) = fetch(app, "/api.json").await;
    assert_eq!(status, StatusCode::OK);
    let spec: serde_json::Value = serde_json::from_str(&spec).unwrap();
    assert_eq!(spec["paths"]["/hello"]["get"]["operationId"], "hello");
    assert_eq!(
        spec["paths"]["/hello-qualified"]["get"]["operationId"],
        "hello_qualified"
    );
}
