#![allow(unused_imports)]
use rovo::aide::axum::IntoApiResponse;
use rovo::extract::State;
use rovo::rovo;

#[derive(Clone)]
struct Foo;

#[derive(Clone)]
struct Bar;

/// Test handler
///
/// # Responses
///
/// 200: String - OK
#[rovo(state = Foo)]
async fn test_handler(State(_bar): State<Bar>) -> impl IntoApiResponse {
    "test"
}

fn main() {}
