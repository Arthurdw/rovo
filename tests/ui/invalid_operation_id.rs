#![allow(unused_imports)]
use rovo::aide::axum::IntoApiResponse;
use axum::response::Json;
use rovo::rovo;

/// Test handler with invalid operation ID
///
/// # Metadata
///
/// @id get-user-by-id
#[rovo]
async fn test_handler() -> impl IntoApiResponse {
    Json("test".to_string())
}

fn main() {}
