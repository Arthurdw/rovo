//! A minimal rovo API with a documented endpoint and auto-generated OpenAPI spec.

use rovo::aide::axum::IntoApiResponse;
use rovo::aide::openapi::OpenApi;
use rovo::response::Json;
use rovo::schemars::JsonSchema;
use rovo::{routing::get, rovo, Router};
use serde::Serialize;

#[derive(Debug, Serialize, JsonSchema)]
struct Greeting {
    message: String,
}

/// Say hello
///
/// Returns a friendly greeting.
///
/// # Responses
///
/// 200: Json<Greeting> - A greeting
///
/// # Metadata
///
/// @tag greetings
#[rovo]
async fn hello() -> impl IntoApiResponse {
    Json(Greeting {
        message: "Hello, world!".into(),
    })
}

#[tokio::main]
async fn main() {
    let mut api = OpenApi::default();
    api.info.title = "Hello World API".into();
    api.info.version = "0.1.0".into();

    let app = Router::new()
        .route("/hello", get(hello))
        .with_oas(api)
        .finish();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("Listening on http://127.0.0.1:3000");
    println!("OpenAPI spec at http://127.0.0.1:3000/api.json");
    axum::serve(listener, app).await.unwrap();
}
