#![allow(unused_imports)]
use rovo::aide::axum::IntoApiResponse;
use rovo::extract::Path;
use rovo::response::Json;
use rovo::rovo;
use uuid::Uuid;

/// Get item in collection.
///
/// # Path Parameters
///
/// collection_id: The collection UUID
/// wrong_name: The item index
///
/// # Responses
///
/// 200: Json<String> - Item found
#[rovo]
async fn get_item(Path((collection_id, index)): Path<(Uuid, u32)>) -> impl IntoApiResponse {
    Json(format!("Collection: {collection_id}, index: {index}"))
}

fn main() {}
