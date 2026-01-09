#![allow(unused_imports)]
use rovo::aide::axum::IntoApiResponse;
use rovo::extract::Path;
use rovo::response::Json;
use rovo::rovo;
use uuid::Uuid;

/// Get a single item.
///
/// # Path Parameters
///
/// id: The unique identifier
///
/// # Responses
///
/// 200: Json<String> - Item found
#[rovo]
async fn get_item(Path(id_test): Path<Uuid>) -> impl IntoApiResponse {
    Json(format!("Item: {id_test}"))
}

fn main() {}
