use axum::response::Json;
use rovo::aide::axum::IntoApiResponse;
use rovo::rovo;

/// Test handler for rovo-ignore
///
/// # Responses
///
/// 200: Json<String> - Success response
///
/// # Metadata
///
/// @tag test
/// @rovo-ignore
/// Everything after this line should be ignored
/// @invalid_annotation this would normally cause an error
/// @respons 404 typo that would normally be caught
/// Additional documentation that won't be parsed
#[rovo]
async fn test_handler() -> impl IntoApiResponse {
    Json("test".to_string())
}

#[test]
fn test_rovo_ignore_compiles() {
    // This test verifies that:
    // 1. The function compiles successfully even with invalid annotations after @rovo-ignore
    // 2. The @rovo-ignore annotation stops parsing at the correct point
    //
    // If @rovo-ignore wasn't working, the @invalid_annotation and @respons would
    // cause compile-time errors. The fact that this test compiles proves it works.
}

fn main() {}
