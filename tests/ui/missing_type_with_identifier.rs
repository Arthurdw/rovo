#![allow(unused)]

use aide::axum::IntoApiResponse;
use axum::response::Json;
use rovo::rovo;

/// Handler that appears to use Todo as type but it's actually missing the type
///
/// @response 200 Todo user information
#[rovo]
async fn test_handler() -> impl IntoApiResponse {
    Json("test".to_string())
}

fn main() {}
