#![allow(unused_imports)]
use rovo::aide::axum::IntoApiResponse;
use rovo::response::Json;
use rovo::rovo;

/// Test handler
///
/// # Responses
///
/// 200: Json<String> - OK
#[rovo(stat = ())]
async fn test_handler() -> impl IntoApiResponse {
    Json("test".to_string())
}

fn main() {}
