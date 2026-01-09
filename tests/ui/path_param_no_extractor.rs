#![allow(unused_imports)]
use rovo::aide::axum::IntoApiResponse;
use rovo::response::Json;
use rovo::rovo;

/// Get all items.
///
/// # Path Parameters
///
/// id: The item identifier
///
/// # Responses
///
/// 200: Json<String> - Items found
#[rovo]
async fn get_items() -> impl IntoApiResponse {
    Json("items".to_string())
}

fn main() {}
