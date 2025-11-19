use axum::extract::{Path, State};
use axum::response::Json;
use rovo::aide::axum::IntoApiResponse;
use rovo::schemars::JsonSchema;
use rovo::{routing::get, rovo, Router};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone)]
struct AppState {}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq)]
struct User {
    id: Uuid,
    name: String,
    email: String,
}

impl Default for User {
    fn default() -> Self {
        Self {
            id: Uuid::nil(),
            name: "John Doe".into(),
            email: "john@example.com".into(),
        }
    }
}

#[derive(Deserialize, JsonSchema)]
struct UserId {
    #[allow(dead_code)]
    id: Uuid,
}

/// Get user by ID
///
/// Retrieves a user's information using their unique identifier.
///
/// @response 200 Json<User> User found and returned successfully.
/// @example 200 User::default()
/// @response 404 () User not found in the system.
#[rovo]
async fn get_user(State(_state): State<AppState>, Path(_id): Path<UserId>) -> impl IntoApiResponse {
    Json(User::default())
}

#[test]
fn test_with_example() {
    let _state = AppState {};

    let _router: ::axum::Router = Router::<AppState>::new()
        .route("/users/{id}", get(get_user))
        .with_state(_state);
}
