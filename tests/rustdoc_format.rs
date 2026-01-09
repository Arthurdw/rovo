use rovo::aide::axum::IntoApiResponse;
use rovo::extract::{Path, State};
use rovo::http::StatusCode;
use rovo::response::Json;
use rovo::schemars::JsonSchema;
use rovo::{routing::get, rovo, Router};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone)]
struct AppState {}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq)]
struct TodoItem {
    id: Uuid,
    title: String,
    completed: bool,
}

impl Default for TodoItem {
    fn default() -> Self {
        Self {
            id: Uuid::nil(),
            title: "Buy milk".into(),
            completed: false,
        }
    }
}

#[derive(Serialize, JsonSchema)]
struct ErrorResponse {
    error: String,
    code: String,
}

#[allow(dead_code)]
#[derive(Deserialize, JsonSchema)]
struct TodoId {
    id: Uuid,
}

// Test 1: Single-line responses
/// Get a todo item.
///
/// # Responses
///
/// 200: Json<TodoItem> - Successfully retrieved the todo item
/// 404: () - Todo item was not found
#[rovo]
async fn get_todo_single_line(
    State(_app): State<AppState>,
    Path(_id): Path<TodoId>,
) -> impl IntoApiResponse {
    Json(TodoItem::default())
}

// Test 2: Multi-line response descriptions
/// Get a todo item with detailed info.
///
/// # Responses
///
/// 200: Json<TodoItem> - Successfully retrieved the todo item from the
///      database with all associated metadata
/// 404: () - Todo item was not found in the database or has been
///      deleted by another user
/// 500: Json<ErrorResponse> - Internal server error occurred while
///      processing the request
#[rovo]
async fn get_todo_multiline_desc(
    State(_app): State<AppState>,
    Path(_id): Path<TodoId>,
) -> impl IntoApiResponse {
    Json(TodoItem::default())
}

// Test 3: Single-line examples
/// Create a todo item with examples.
///
/// # Responses
///
/// 201: Json<TodoItem> - Todo item created successfully
/// 400: Json<ErrorResponse> - Invalid input data
///
/// # Examples
///
/// 201: TodoItem { id: Uuid::nil(), title: "Buy milk".into(), completed: false }
/// 400: ErrorResponse { error: "Title cannot be empty".into(), code: "VALIDATION_ERROR".into() }
#[rovo]
async fn create_todo_single_line(
    State(_app): State<AppState>,
    Json(_input): Json<TodoItem>,
) -> impl IntoApiResponse {
    Json(TodoItem::default())
}

// Test 4: Multi-line examples
/// Create a todo item with multi-line examples.
///
/// # Responses
///
/// 201: Json<TodoItem> - Todo item created successfully
/// 400: Json<ErrorResponse> - Invalid input data
///
/// # Examples
///
/// 201: TodoItem {
///          id: Uuid::nil(),
///          title: "Buy milk".into(),
///          completed: false
///      }
/// 400: ErrorResponse {
///          error: "Title cannot be empty".into(),
///          code: "VALIDATION_ERROR".into()
///      }
#[rovo]
async fn create_todo_multiline(
    State(_app): State<AppState>,
    Json(_input): Json<TodoItem>,
) -> impl IntoApiResponse {
    Json(TodoItem::default())
}

// Test 5: Primitive type examples
/// Get status with primitive examples.
///
/// # Responses
///
/// 200: Json<String> - Operation successful
/// 201: Json<i32> - Count of items
/// 202: Json<bool> - Validation result
/// 203: Json<f64> - Progress percentage
///
/// # Examples
///
/// 200: "success"
/// 201: 42
/// 202: true
/// 203: 99.9
#[rovo]
async fn get_status_primitives(State(_app): State<AppState>) -> impl IntoApiResponse {
    Json("success")
}

// Test 6: Metadata section with single tag
/// List todos with single tag.
///
/// # Responses
///
/// 200: Json<Vec<TodoItem>> - List of todo items
///
/// # Metadata
///
/// @tag todos
#[rovo]
async fn list_todos_single_tag(State(_app): State<AppState>) -> impl IntoApiResponse {
    Json(Vec::<TodoItem>::new())
}

// Test 7: Metadata section with multiple tags
/// List todos with multiple tags.
///
/// # Responses
///
/// 200: Json<Vec<TodoItem>> - List of todo items
///
/// # Metadata
///
/// @tag todos
/// @tag lists
#[rovo]
async fn list_todos_multiple_tags(State(_app): State<AppState>) -> impl IntoApiResponse {
    Json(Vec::<TodoItem>::new())
}

// Test 8: Metadata with security
/// Protected endpoint.
///
/// # Responses
///
/// 200: Json<TodoItem> - Success
/// 401: () - Unauthorized
///
/// # Metadata
///
/// @tag todos
/// @security bearer_auth
#[rovo]
async fn get_protected_todo(
    State(_app): State<AppState>,
    Path(_id): Path<TodoId>,
) -> impl IntoApiResponse {
    Json(TodoItem::default())
}

// Test 9: Metadata with custom operation ID
/// Get todo with custom ID.
///
/// # Responses
///
/// 200: Json<TodoItem> - Success
///
/// # Metadata
///
/// @id get_todo_by_id
/// @tag todos
#[rovo]
async fn get_todo_custom_id(
    State(_app): State<AppState>,
    Path(_id): Path<TodoId>,
) -> impl IntoApiResponse {
    Json(TodoItem::default())
}

// Test 10: Metadata with hidden
/// Internal endpoint.
///
/// # Responses
///
/// 200: Json<String> - Success
///
/// # Metadata
///
/// @hidden
#[rovo]
async fn internal_endpoint(State(_app): State<AppState>) -> impl IntoApiResponse {
    Json("internal")
}

// Test 11: Complete example with all sections
/// Create a todo item with full documentation.
///
/// Creates a new todo item in the database. The title must be non-empty
/// and the item starts in an incomplete state by default.
///
/// # Responses
///
/// 201: Json<TodoItem> - Todo item created successfully
/// 400: Json<ErrorResponse> - Invalid input data provided
/// 401: () - Authentication required
/// 500: Json<ErrorResponse> - Internal server error
///
/// # Examples
///
/// 201: TodoItem {
///     id: Uuid::nil(),
///     title: "Buy groceries".into(),
///     completed: false,
/// }
/// 400: ErrorResponse {
///     error: "Title cannot be empty".into(),
///     code: "VALIDATION_ERROR".into(),
/// }
///
/// # Metadata
///
/// @id create_todo_item
/// @tag todos
/// @security bearer_auth
#[rovo]
async fn create_todo_complete(
    State(_app): State<AppState>,
    Json(_input): Json<TodoItem>,
) -> impl IntoApiResponse {
    Json(TodoItem::default())
}

// Test 12: Using @rovo-ignore
/// Experimental endpoint.
///
/// # Responses
///
/// 200: Json<String> - Success
///
/// # Metadata
///
/// @tag experimental
///
/// @rovo-ignore
///
/// TODO: Add more response types
/// TODO: Add authentication
/// @invalid_annotation this won't cause errors
#[rovo]
async fn experimental_endpoint(State(_app): State<AppState>) -> impl IntoApiResponse {
    Json("experimental")
}

// Test 13: 204 No Content response
/// Delete a todo item.
///
/// # Responses
///
/// 204: () - Todo item deleted successfully
/// 404: () - Todo item not found
///
/// # Metadata
///
/// @tag todos
#[rovo]
async fn delete_todo(
    State(_app): State<AppState>,
    Path(_id): Path<TodoId>,
) -> impl IntoApiResponse {
    StatusCode::NO_CONTENT
}

// Test 14: Complex nested response types
/// Get list of todo items.
///
/// # Responses
///
/// 200: Json<Vec<TodoItem>> - List of all todo items
/// 404: () - Not found
///
/// # Examples
///
/// 200: vec![
///     TodoItem {
///         id: Uuid::nil(),
///         title: "Task 1".into(),
///         completed: false,
///     },
///     TodoItem {
///         id: Uuid::nil(),
///         title: "Task 2".into(),
///         completed: true,
///     }
/// ]
///
/// # Metadata
///
/// @tag todos
#[rovo]
async fn get_nested_response(State(_app): State<AppState>) -> impl IntoApiResponse {
    Json(Vec::<TodoItem>::new())
}

#[test]
fn test_single_line_responses() {
    let _state = AppState {};
    let _router: ::axum::Router = Router::<AppState>::new()
        .route("/todo/{id}", get(get_todo_single_line))
        .with_state(_state);
}

#[test]
fn test_multiline_responses() {
    let _state = AppState {};
    let _router: ::axum::Router = Router::<AppState>::new()
        .route("/todo/{id}", get(get_todo_multiline_desc))
        .with_state(_state);
}

#[test]
fn test_single_line_examples() {
    let _state = AppState {};
    let _router: ::axum::Router = Router::<AppState>::new()
        .route("/todo", get(create_todo_single_line))
        .with_state(_state);
}

#[test]
fn test_multiline_examples() {
    let _state = AppState {};
    let _router: ::axum::Router = Router::<AppState>::new()
        .route("/todo", get(create_todo_multiline))
        .with_state(_state);
}

#[test]
fn test_primitive_examples() {
    let _state = AppState {};
    let _router: ::axum::Router = Router::<AppState>::new()
        .route("/status", get(get_status_primitives))
        .with_state(_state);
}

#[test]
fn test_metadata_single_tag() {
    let _state = AppState {};
    let _router: ::axum::Router = Router::<AppState>::new()
        .route("/todos", get(list_todos_single_tag))
        .with_state(_state);
}

#[test]
fn test_metadata_multiple_tags() {
    let _state = AppState {};
    let _router: ::axum::Router = Router::<AppState>::new()
        .route("/todos", get(list_todos_multiple_tags))
        .with_state(_state);
}

#[test]
fn test_metadata_with_security() {
    let _state = AppState {};
    let _router: ::axum::Router = Router::<AppState>::new()
        .route("/todo/{id}", get(get_protected_todo))
        .with_state(_state);
}

#[test]
fn test_metadata_custom_id() {
    let _state = AppState {};
    let _router: ::axum::Router = Router::<AppState>::new()
        .route("/todo/{id}", get(get_todo_custom_id))
        .with_state(_state);
}

#[test]
fn test_metadata_hidden() {
    let _state = AppState {};
    let _router: ::axum::Router = Router::<AppState>::new()
        .route("/internal", get(internal_endpoint))
        .with_state(_state);
}

#[test]
fn test_complete_documentation() {
    let _state = AppState {};
    let _router: ::axum::Router = Router::<AppState>::new()
        .route("/todo", get(create_todo_complete))
        .with_state(_state);
}

#[test]
fn test_rovo_ignore() {
    let _state = AppState {};
    let _router: ::axum::Router = Router::<AppState>::new()
        .route("/experimental", get(experimental_endpoint))
        .with_state(_state);
}

#[test]
fn test_delete_endpoint() {
    let _state = AppState {};
    let _router: ::axum::Router = Router::<AppState>::new()
        .route("/todo/{id}", get(delete_todo))
        .with_state(_state);
}

#[test]
fn test_nested_response_types() {
    let _state = AppState {};
    let _router: ::axum::Router = Router::<AppState>::new()
        .route("/nested", get(get_nested_response))
        .with_state(_state);
}

// Test: Examples starting on next line
/// Get todo with example on next line.
///
/// # Path Parameters
///
/// id: The todo ID
///
/// # Responses
///
/// 200: Json<TodoItem> - Success
///
/// # Examples
///
/// 200:
/// TodoItem {
///     id: Uuid::nil(),
///     title: "Buy milk".into(),
///     completed: false
/// }
#[rovo]
#[allow(unused_variables)]
async fn get_todo_next_line_example(
    State(_app): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoApiResponse {
    Json(TodoItem::default())
}

#[test]
fn test_example_starts_next_line() {
    let _state = AppState {};
    let _router: ::axum::Router = Router::<AppState>::new()
        .route("/todos/{id}", get(get_todo_next_line_example))
        .with_state(_state);
}

// Test: Unmarked code blocks
/// Get todo with unmarked code block.
///
/// # Responses
///
/// 200: Json<TodoItem> - Success
///
/// # Examples
///
/// 200:
/// ```
/// TodoItem {
///     id: Uuid::nil(),
///     title: "Buy milk".into(),
///     completed: false
/// }
/// ```
#[rovo]
async fn get_todo_code_block_unmarked(
    State(_app): State<AppState>,
    Path(_id): Path<Uuid>,
) -> impl IntoApiResponse {
    Json(TodoItem::default())
}

#[test]
fn test_code_block_unmarked() {
    let _state = AppState {};
    let _router: ::axum::Router = Router::<AppState>::new()
        .route("/todos/{id}", get(get_todo_code_block_unmarked))
        .with_state(_state);
}

// Test: Code block marked with rust
/// Get todo with rust-marked code block.
///
/// # Responses
///
/// 200: Json<TodoItem> - Success
///
/// # Examples
///
/// 200:
/// ```rust
/// TodoItem {
///     id: Uuid::nil(),
///     title: "Buy milk".into(),
///     completed: false
/// }
/// ```
#[rovo]
async fn get_todo_code_block_rust(
    State(_app): State<AppState>,
    Path(_id): Path<Uuid>,
) -> impl IntoApiResponse {
    Json(TodoItem::default())
}

#[test]
fn test_code_block_rust() {
    let _state = AppState {};
    let _router: ::axum::Router = Router::<AppState>::new()
        .route("/todos/{id}", get(get_todo_code_block_rust))
        .with_state(_state);
}

// Test: Code block marked with rs
/// Get todo with rs-marked code block.
///
/// # Responses
///
/// 200: Json<TodoItem> - Success
///
/// # Examples
///
/// 200:
/// ```rs
/// TodoItem {
///     id: Uuid::nil(),
///     title: "Buy milk".into(),
///     completed: false
/// }
/// ```
#[rovo]
async fn get_todo_code_block_rs(
    State(_app): State<AppState>,
    Path(_id): Path<Uuid>,
) -> impl IntoApiResponse {
    Json(TodoItem::default())
}

#[test]
fn test_code_block_rs() {
    let _state = AppState {};
    let _router: ::axum::Router = Router::<AppState>::new()
        .route("/todos/{id}", get(get_todo_code_block_rs))
        .with_state(_state);
}

// Test: Code block on same line as status code
/// Get todo with code block on same line.
///
/// # Responses
///
/// 200: Json<TodoItem> - Success
///
/// # Examples
///
/// 200: ```
/// TodoItem {
///     id: Uuid::nil(),
///     title: "Buy milk".into(),
///     completed: false
/// }
/// ```
#[rovo]
async fn get_todo_code_block_same_line(
    State(_app): State<AppState>,
    Path(_id): Path<Uuid>,
) -> impl IntoApiResponse {
    Json(TodoItem::default())
}

#[test]
fn test_code_block_same_line() {
    let _state = AppState {};
    let _router: ::axum::Router = Router::<AppState>::new()
        .route("/todos/{id}", get(get_todo_code_block_same_line))
        .with_state(_state);
}
