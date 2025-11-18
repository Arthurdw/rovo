use aide::axum::{routing::get_with, ApiRouter, IntoApiResponse};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use rovo::rovo;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone)]
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
/// @response 200 Json<Item> Item found successfully.
/// @example 200 Item::default()
/// @response 404 () Item not found.
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
/// @response 200 Json<String> Success.
#[rovo]
async fn simple_handler(State(_state): State<AppState>) -> impl IntoApiResponse {
    Json("ok".to_string())
}

/// Multiple response codes.
///
/// @response 200 Json<Item> Success response.
/// @response 400 () Bad request.
/// @response 401 () Unauthorized.
/// @response 500 () Internal server error.
#[rovo]
async fn multi_response(State(_state): State<AppState>) -> impl IntoApiResponse {
    Json(Item::default())
}

#[test]
fn test_macro_generates_docs_function() {
    // The macro should generate a {function_name}_docs function
    let state = AppState {
        value: "test".to_string(),
    };

    // This should compile - the _docs functions exist
    let _router: ApiRouter<AppState> = ApiRouter::new()
        .api_route("/items/{id}", get_with(get_item, get_item_docs))
        .api_route("/simple", get_with(simple_handler, simple_handler_docs))
        .api_route("/multi", get_with(multi_response, multi_response_docs))
        .with_state(state);
}

#[test]
fn test_docs_function_callable() {
    use aide::transform::TransformOperation;
    use aide::openapi::Operation;

    // Create a mock operation
    let mut operation = Operation::default();
    let transform = TransformOperation::new(&mut operation);

    // The docs function should be callable and return a TransformOperation
    let _result = get_item_docs(transform);
}

#[test]
fn test_multiple_handlers_compile() {
    // Test that multiple handlers with different signatures work
    let state = AppState {
        value: "test".to_string(),
    };

    let _router: ApiRouter<AppState> = ApiRouter::new()
        .api_route("/a", get_with(simple_handler, simple_handler_docs))
        .api_route("/b", get_with(multi_response, multi_response_docs))
        .api_route("/c/{id}", get_with(get_item, get_item_docs))
        .with_state(state);
}

#[test]
fn test_handler_with_path_params() {
    // Verify that handlers with path parameters compile correctly
    let state = AppState {
        value: "test".to_string(),
    };

    let _router: ApiRouter<AppState> =
        ApiRouter::new()
            .api_route("/items/{id}", get_with(get_item, get_item_docs))
            .with_state(state);
}
