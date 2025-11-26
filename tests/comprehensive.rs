use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use rovo::aide::axum::IntoApiResponse;
use rovo::schemars::JsonSchema;
use rovo::{routing::get, rovo, Router};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone)]
#[allow(dead_code)]
struct AppState {
    value: String,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug, PartialEq)]
struct Item {
    id: Uuid,
    name: String,
}

impl Default for Item {
    fn default() -> Self {
        Self {
            id: Uuid::nil(),
            name: "Test Item".to_string(),
        }
    }
}

#[derive(Deserialize, JsonSchema)]
struct ItemId {
    id: Uuid,
}

/// Get an item.
///
/// This endpoint retrieves an item by ID.
///
/// # Responses
///
/// 200: Json<Item> - Item found successfully
/// 404: () - Item not found
///
/// # Examples
///
/// 200: Item::default()
#[rovo]
async fn get_item(
    State(_state): State<AppState>,
    Path(ItemId { id }): Path<ItemId>,
) -> impl IntoApiResponse {
    if id == Uuid::nil() {
        Json(Item::default()).into_response()
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}

/// Simple handler.
///
/// # Responses
///
/// 200: Json<String> - Success
#[rovo]
async fn simple_handler(State(_state): State<AppState>) -> impl IntoApiResponse {
    Json("ok".to_string())
}

/// Multiple response codes.
///
/// # Responses
///
/// 200: Json<Item> - Success response
/// 400: () - Bad request
/// 401: () - Unauthorized
/// 500: () - Internal server error
#[rovo]
async fn multi_response(State(_state): State<AppState>) -> impl IntoApiResponse {
    Json(Item::default())
}

#[test]
fn test_macro_generates_docs_function() {
    // The macro should generate a struct that implements IntoApiMethodRouter
    let state = AppState {
        value: "test".to_string(),
    };

    // This should compile using the new routing API
    let _router: ::axum::Router = Router::<AppState>::new()
        .route("/items/{id}", get(get_item))
        .route("/simple", get(simple_handler))
        .route("/multi", get(multi_response))
        .with_state(state);
}

#[test]
fn test_docs_function_callable() {
    use rovo::aide::openapi::Operation;
    use rovo::aide::transform::TransformOperation;

    // Create a mock operation
    let mut operation = Operation::default();
    let transform = TransformOperation::new(&mut operation);

    // The __docs function should still be accessible for testing
    let _result = get_item::__docs(transform);
}

#[test]
fn test_multiple_handlers_compile() {
    // Test that multiple handlers with different signatures work
    let state = AppState {
        value: "test".to_string(),
    };

    let _router: ::axum::Router = Router::<AppState>::new()
        .route("/a", get(simple_handler))
        .route("/b", get(multi_response))
        .route("/c/{id}", get(get_item))
        .with_state(state);
}

#[test]
fn test_handler_with_path_params() {
    // Verify that handlers with path parameters compile correctly
    let state = AppState {
        value: "test".to_string(),
    };

    let _router: ::axum::Router = Router::<AppState>::new()
        .route("/items/{id}", get(get_item))
        .with_state(state);
}
