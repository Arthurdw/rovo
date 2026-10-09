#![allow(deprecated)]

use rovo::aide::axum::IntoApiResponse;
use rovo::extract::State;
use rovo::response::Json;
use rovo::schemars::JsonSchema;
use rovo::{routing::get, rovo, Router};
use serde::Serialize;

#[derive(Clone)]
struct AppState {}

#[derive(Clone, Debug, Serialize, JsonSchema, PartialEq)]
struct User {
    id: u64,
    name: String,
}

impl Default for User {
    fn default() -> Self {
        Self {
            id: 1,
            name: "Test User".into(),
        }
    }
}

/// Multi-line response description test
///
/// # Responses
///
/// 200: Json<User> - User found successfully. This is a
///     multi-line description that continues on the next line
///     and even continues further.
/// 404: () - User not found
#[rovo]
async fn multiline_response_description(State(_state): State<AppState>) -> impl IntoApiResponse {
    Json(User::default())
}

/// Code block example test
///
/// # Responses
///
/// 200: Json<User> - User found
///
/// # Examples
///
/// 200:
/// ```rust
/// User {
///     id: 1,
///     name: "Alice".into()
/// }
/// ```
#[rovo]
async fn codeblock_example(State(_state): State<AppState>) -> impl IntoApiResponse {
    Json(User::default())
}

/// Multi-line example with nested braces
///
/// # Responses
///
/// 200: Json<User> - Success
///
/// # Examples
///
/// 200: User {
///     id: 1,
///     name: "Bob".into()
/// }
#[rovo]
async fn multiline_example_braces(State(_state): State<AppState>) -> impl IntoApiResponse {
    Json(User::default())
}

/// Handler with deprecated attribute
///
/// # Responses
///
/// 200: Json<User> - Success
#[deprecated(note = "Use v2 API instead")]
#[rovo]
async fn deprecated_handler(State(_state): State<AppState>) -> impl IntoApiResponse {
    Json(User::default())
}

/// Handler with empty code block example
///
/// # Responses
///
/// 200: Json<User> - Success
///
/// # Examples
///
/// 200: ```
/// User::default()
/// ```
#[rovo]
async fn codeblock_inline_example(State(_state): State<AppState>) -> impl IntoApiResponse {
    Json(User::default())
}

#[test]
fn test_multiline_response_description_compiles() {
    let state = AppState {};
    let _router: ::axum::Router = Router::<AppState>::new()
        .route("/multiline", get(multiline_response_description))
        .with_state(state)
        .finish();
}

#[test]
fn test_codeblock_example_compiles() {
    let state = AppState {};
    let _router: ::axum::Router = Router::<AppState>::new()
        .route("/codeblock", get(codeblock_example))
        .with_state(state)
        .finish();
}

#[test]
fn test_multiline_example_braces_compiles() {
    let state = AppState {};
    let _router: ::axum::Router = Router::<AppState>::new()
        .route("/multiline-braces", get(multiline_example_braces))
        .with_state(state)
        .finish();
}

#[test]
fn test_deprecated_handler_compiles() {
    let state = AppState {};
    let _router: ::axum::Router = Router::<AppState>::new()
        .route("/deprecated", get(deprecated_handler))
        .with_state(state)
        .finish();
}

#[test]
fn test_codeblock_inline_example_compiles() {
    let state = AppState {};
    let _router: ::axum::Router = Router::<AppState>::new()
        .route("/codeblock-inline", get(codeblock_inline_example))
        .with_state(state)
        .finish();
}

#[derive(serde::Deserialize, JsonSchema)]
#[allow(dead_code)]
struct UserState {
    online: bool,
}

#[derive(serde::Deserialize, JsonSchema)]
#[allow(dead_code)]
struct Page {
    page: u32,
}

/// A type whose name ends in `State` must not be taken for the `State` extractor.
///
/// # Responses
///
/// 204: () - Updated
#[rovo]
async fn set_user_state(
    rovo::extract::Query(_filter): rovo::extract::Query<UserState>,
    Json(_body): Json<Page>,
) -> impl IntoApiResponse {
    rovo::http::StatusCode::NO_CONTENT
}

#[test]
fn test_type_ending_in_state_is_not_state_extractor() {
    let _router: ::axum::Router = Router::new()
        .route("/user-state", rovo::routing::post(set_user_state))
        .finish();
}
