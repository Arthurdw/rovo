use aide::{axum::IntoApiResponse, openapi::OpenApi};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    Extension,
};
use rovo::{rovo, Router};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing::info;
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    pub todos: Arc<Mutex<HashMap<Uuid, TodoItem>>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct TodoItem {
    pub id: Uuid,
    pub description: String,
    pub complete: bool,
}

impl Default for TodoItem {
    fn default() -> Self {
        Self {
            id: Uuid::nil(),
            description: "Sample todo item".into(),
            complete: false,
        }
    }
}

#[derive(Deserialize, JsonSchema)]
struct TodoId {
    /// The unique identifier of the todo item.
    id: Uuid,
}

#[derive(Deserialize, JsonSchema)]
pub struct CreateTodoRequest {
    /// Description of the todo item.
    pub description: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct UpdateTodoRequest {
    /// Optional new description.
    pub description: Option<String>,
    /// Optional completion status.
    pub complete: Option<bool>,
}

/// Get a single Todo item.
///
/// Retrieve a Todo item by its ID from the database.
///
/// @tag todos
/// @response 200 Json<TodoItem> Successfully retrieved the todo item.
/// @example 200 TodoItem::default()
/// @response 404 () Todo item was not found.
#[rovo]
async fn get_todo(
    State(app): State<AppState>,
    Path(TodoId { id }): Path<TodoId>,
) -> impl IntoApiResponse {
    if let Some(todo) = app.todos.lock().unwrap().get(&id) {
        (StatusCode::OK, Json(todo.clone())).into_response()
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}

/// List all Todo items.
///
/// Returns a list of all todo items in the system.
///
/// @tag todos
/// @response 200 Json<Vec<TodoItem>> List of all todo items.
#[rovo]
async fn list_todos(State(app): State<AppState>) -> Json<Vec<TodoItem>> {
    let todos: Vec<TodoItem> = app.todos.lock().unwrap().values().cloned().collect();
    Json(todos)
}

/// Create a new Todo item.
///
/// Creates a new todo item with the provided description.
///
/// @tag todos
/// @response 201 Json<TodoItem> Todo item created successfully.
/// @example 201 TodoItem::default()
#[rovo]
async fn create_todo(
    State(app): State<AppState>,
    Json(req): Json<CreateTodoRequest>,
) -> (StatusCode, Json<TodoItem>) {
    let todo = TodoItem {
        id: Uuid::new_v4(),
        description: req.description,
        complete: false,
    };
    app.todos.lock().unwrap().insert(todo.id, todo.clone());
    (StatusCode::CREATED, Json(todo))
}

/// Update an existing Todo item.
///
/// Updates the description and/or completion status of a todo item.
///
/// @tag todos
/// @response 200 Json<TodoItem> Todo item updated successfully.
/// @example 200 TodoItem::default()
/// @response 404 () Todo item was not found.
#[rovo]
async fn update_todo(
    State(app): State<AppState>,
    Path(TodoId { id }): Path<TodoId>,
    Json(req): Json<UpdateTodoRequest>,
) -> impl IntoApiResponse {
    let mut todos = app.todos.lock().unwrap();

    if let Some(todo) = todos.get_mut(&id) {
        if let Some(description) = req.description {
            todo.description = description;
        }
        if let Some(complete) = req.complete {
            todo.complete = complete;
        }
        (StatusCode::OK, Json(todo.clone())).into_response()
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}

/// Delete a Todo item.
///
/// Permanently deletes a todo item by its ID.
///
/// @tag todos
/// @response 204 () Todo item deleted successfully.
/// @response 404 () Todo item was not found.
#[rovo]
async fn delete_todo(
    State(app): State<AppState>,
    Path(TodoId { id }): Path<TodoId>,
) -> impl IntoApiResponse {
    if app.todos.lock().unwrap().remove(&id).is_some() {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}

async fn serve_api(Extension(api): Extension<OpenApi>) -> axum::Json<OpenApi> {
    axum::Json(api)
}

#[tokio::main]
async fn main() {
    use rovo::routing::get;

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,todo_api=debug".into()),
        )
        .init();

    let state = AppState {
        todos: Arc::new(Mutex::new(HashMap::new())),
    };

    let mut api = OpenApi::default();
    api.info.title = "Todo API Example".to_string();
    api.info.description = Some("OpenAPI documentation example using rovo".to_string());

    // Build the router with Swagger UI and API documentation - all in one place!
    let app = Router::new()
        .nest(
            "/api",
            Router::new()
                .route("/todos", get(list_todos).post(create_todo))
                .route(
                    "/todos/{id}",
                    get(get_todo).patch(update_todo).delete(delete_todo),
                ),
        )
        .with_swagger("/", "/api.json")
        .with_api_json("/api.json", serve_api)
        .with_state(state)
        .finish_api_with_extension(api);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    info!("Server started successfully");
    info!("Address: http://127.0.0.1:3000");

    axum::serve(listener, app).await.unwrap();
}
