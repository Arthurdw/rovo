#![allow(unused_imports)]
use rovo::aide::axum::IntoApiResponse;
use axum::response::Json;
use rovo::rovo;

/// Test handler with empty tag
///
/// @tag
/// @response 200 Json<String> Success
#[rovo]
async fn test_handler() -> impl IntoApiResponse {
    Json("test".to_string())
}

fn main() {}
