#![allow(unused_imports)]
use axum::response::Json;
use rovo::rovo;
use serde::Serialize;

#[derive(Serialize)]
struct User {
    id: u64,
    name: String,
}

/// Get user by ID
///
/// # Responses
///
/// 200: Json<User> - User found
/// 404: () - User not found
///
/// # Examples
///
/// 201: User { id: 1, name: "Alice".into() }
#[rovo]
async fn get_user() -> Json<User> {
    Json(User {
        id: 1,
        name: "Alice".to_string(),
    })
}

fn main() {}
