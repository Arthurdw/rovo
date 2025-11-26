#![allow(unused_imports)]
use rovo::aide::axum::IntoApiResponse;
use axum::response::Json;
use rovo::rovo;

/// Test handler
///
/// # Responses
///
/// 999: Json<String> - Invalid status code
#[rovo]
async fn test_handler() -> impl IntoApiResponse {
    Json("test".to_string())
}

fn main() {}
