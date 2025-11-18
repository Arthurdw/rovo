use aide::axum::IntoApiResponse;
use axum::extract::{Path, State};
use axum::response::Json;
use rovo::{routing::get, rovo, Router};
use schemars::JsonSchema;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    // Mock state for testing
}

#[derive(Clone, Debug, serde::Serialize, JsonSchema, PartialEq)]
struct TodoItem {
    id: Uuid,
    description: String,
    complete: bool,
}

impl Default for TodoItem {
    fn default() -> Self {
        Self {
            id: Uuid::nil(),
            description: "fix bugs".into(),
            complete: false,
        }
    }
}

#[derive(Deserialize, JsonSchema)]
struct SelectTodo {
    /// The ID of the Todo.
    #[allow(dead_code)]
    id: Uuid,
}

/// Get a single Todo item.
///
/// Retrieve a Todo item by its ID.
///
/// @response 200 Json<TodoItem> A single Todo item.
/// @response 404 () Todo was not found.
#[rovo]
async fn get_todo(
    State(_app): State<AppState>,
    Path(_todo): Path<SelectTodo>,
) -> impl IntoApiResponse {
    Json(TodoItem::default())
}

#[test]
fn test_macro_compiles() {
    // This test ensures the macro compiles correctly with the new routing API
    let _state = AppState {};

    // Use the new drop-in replacement routing function with our Router
    let _router: Router<()> = Router::<AppState>::new()
        .route("/todo/{id}", get(get_todo))
        .with_state(_state);
}
