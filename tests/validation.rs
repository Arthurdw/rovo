// This test file demonstrates compile-time validation of doc comment annotations.
// Uncomment any of the examples below to see the validation errors at compile time.

use aide::axum::IntoApiResponse;
use axum::response::Json;
use rovo::rovo;

/*
// Example 1: Invalid status code (too high)
/// Test handler
///
/// @response 999 Json<String> Invalid status code
#[rovo]
async fn invalid_status_code() -> impl IntoApiResponse {
    Json("test".to_string())
}
*/

/*
// Example 2: Missing description
/// Test handler
///
/// @response 200 Json<String>
#[rovo]
async fn missing_description() -> impl IntoApiResponse {
    Json("test".to_string())
}
*/

/*
// Example 3: Empty tag
/// Test handler
///
/// @tag
#[rovo]
async fn empty_tag() -> impl IntoApiResponse {
    Json("test".to_string())
}
*/

/*
// Example 4: Invalid operation ID (contains spaces)
/// Test handler
///
/// @id my handler
#[rovo]
async fn invalid_operation_id() -> impl IntoApiResponse {
    Json("test".to_string())
}
*/

/*
// Example 5: Unknown annotation
/// Test handler
///
/// @unknown something
#[rovo]
async fn unknown_annotation() -> impl IntoApiResponse {
    Json("test".to_string())
}
*/

/*
// Example 6: Invalid response type syntax
/// Test handler
///
/// @response 200 Json<String>> Invalid syntax
#[rovo]
async fn invalid_type_syntax() -> impl IntoApiResponse {
    Json("test".to_string())
}
*/

// This test compiles successfully - all validations pass
/// Valid handler
///
/// @tag test
/// @response 200 Json<String> Success response
#[rovo]
async fn valid_handler() -> impl IntoApiResponse {
    Json("test".to_string())
}

#[test]
fn test_valid_handler_compiles() {
    // If this test runs, it means the valid handler compiled successfully
    assert!(true);
}
