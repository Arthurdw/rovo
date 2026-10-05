//! Extractors beyond `Json`/`Query`/`Path` must work in `#[rovo]` handlers and
//! show up in the generated spec.

use rovo::prelude::*;
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
#[allow(dead_code)]
struct LoginForm {
    username: String,
    password: String,
}

/// Log in with a form.
///
/// # Responses
///
/// 204: () - Logged in
#[rovo]
async fn login(rovo::extract::Form(_form): rovo::extract::Form<LoginForm>) -> impl IntoApiResponse {
    StatusCode::NO_CONTENT
}

/// Upload a file.
///
/// # Responses
///
/// 204: () - Uploaded
#[cfg(feature = "multipart")]
#[rovo]
async fn upload(mut multipart: rovo::extract::Multipart) -> impl IntoApiResponse {
    while let Ok(Some(_field)) = multipart.next_field().await {}
    StatusCode::NO_CONTENT
}

fn request_body_content_types(app: ::axum::Router, path: &str) -> Vec<String> {
    use axum::body::Body;
    use axum::http::Request;
    use tower::util::ServiceExt;

    let rt = tokio::runtime::Runtime::new().unwrap();
    let spec: serde_json::Value = rt.block_on(async {
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api.json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        serde_json::from_slice(&body).unwrap()
    });

    spec["paths"][path]["post"]["requestBody"]["content"]
        .as_object()
        .expect("Should have a request body")
        .keys()
        .cloned()
        .collect()
}

#[test]
fn test_form_request_body() {
    let app = Router::<()>::new()
        .route("/login", post(login))
        .with_oas(OpenApi::default())
        .finish();

    assert_eq!(
        request_body_content_types(app, "/login"),
        ["application/x-www-form-urlencoded"]
    );
}

#[cfg(feature = "multipart")]
#[test]
fn test_multipart_request_body() {
    let app = Router::<()>::new()
        .route("/upload", post(upload))
        .with_oas(OpenApi::default())
        .finish();

    assert_eq!(
        request_body_content_types(app, "/upload"),
        ["multipart/form-data"]
    );
}
