// This test file demonstrates compile-time validation of doc comment annotations.
// Uncomment any of the examples below to see the validation errors at compile time.

/*
// Example 1: Invalid status code (too high)
/// Test handler
///
/// # Responses
///
/// 999: Json<String> - Invalid status code
#[rovo]
async fn invalid_status_code() -> impl IntoApiResponse {
    Json("test".to_string())
}
*/

/*
// Example 2: Empty tag
/// Test handler
///
/// # Metadata
///
/// @tag
#[rovo]
async fn empty_tag() -> impl IntoApiResponse {
    Json("test".to_string())
}
*/

/*
// Example 3: Invalid operation ID (contains spaces)
/// Test handler
///
/// # Metadata
///
/// @id my handler
#[rovo]
async fn invalid_operation_id() -> impl IntoApiResponse {
    Json("test".to_string())
}
*/

/*
// Example 4: Unknown annotation
/// Test handler
///
/// # Metadata
///
/// @unknown something
#[rovo]
async fn unknown_annotation() -> impl IntoApiResponse {
    Json("test".to_string())
}
*/
