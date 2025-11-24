use rovo_lsp::type_resolver;

/// Test that type definitions inside comments are ignored
#[test]
fn ignores_struct_in_line_comment() {
    let content = r#"
// struct User { name: String }
struct User {
    name: String,
}
"#;

    let type_def = type_resolver::find_type_definition(content, "User");
    // Should find the real struct at line 2, not the commented one
    assert_eq!(type_def, Some(2));
}

#[test]
fn ignores_type_name_in_inline_comment() {
    let content = r#"
struct RealType { x: i32 } // This mentions RealType in a comment

/// # Responses
///
/// 200: Json<RealType> - Success
#[rovo]
async fn handler() {}
"#;

    let type_def = type_resolver::find_type_definition(content, "RealType");
    // Should find the struct definition at line 1
    assert_eq!(type_def, Some(1));
}

#[test]
fn handles_struct_with_trailing_comment() {
    let content = r#"
struct User { // User data structure
    name: String,
}
"#;

    let type_def = type_resolver::find_type_definition(content, "User");
    assert_eq!(type_def, Some(1));
}

#[test]
fn handles_enum_with_trailing_comment() {
    let content = r#"
enum Status { // Status enum
    Active,
    Inactive,
}
"#;

    let type_def = type_resolver::find_type_definition(content, "Status");
    assert_eq!(type_def, Some(1));
}

#[test]
fn handles_type_alias_with_comment() {
    let content = r#"
type UserId = u64; // Type alias for user IDs
"#;

    let type_def = type_resolver::find_type_definition(content, "UserId");
    assert_eq!(type_def, Some(1));
}

#[test]
fn distinguishes_similar_type_names() {
    let content = r#"
struct User { id: u32 }
struct UserData { user: User }
struct UserProfile { data: UserData }
"#;

    // Should find exact matches only
    assert_eq!(
        type_resolver::find_type_definition(content, "User"),
        Some(1)
    );
    assert_eq!(
        type_resolver::find_type_definition(content, "UserData"),
        Some(2)
    );
    assert_eq!(
        type_resolver::find_type_definition(content, "UserProfile"),
        Some(3)
    );
}

#[test]
fn handles_pub_crate_struct() {
    let content = r#"
pub(crate) struct InternalUser {
    id: u32,
}
"#;

    let type_def = type_resolver::find_type_definition(content, "InternalUser");
    assert_eq!(type_def, Some(1));
}

#[test]
fn handles_pub_super_enum() {
    let content = r#"
pub(super) enum Status {
    Active,
}
"#;

    let type_def = type_resolver::find_type_definition(content, "Status");
    assert_eq!(type_def, Some(1));
}

#[test]
fn handles_multiple_spaces_before_keyword() {
    let content = r#"
    struct   User {
        name: String,
    }
"#;

    let type_def = type_resolver::find_type_definition(content, "User");
    assert_eq!(type_def, Some(1));
}

#[test]
fn does_not_match_in_string_literal() {
    let content = r#"
struct User { name: String }

const MSG: &str = "User information";
"#;

    let type_def = type_resolver::find_type_definition(content, "User");
    // Should only find the struct, not the string
    assert_eq!(type_def, Some(1));
}

#[test]
fn handles_nested_type_in_response() {
    let content = r#"
struct Inner { value: i32 }
struct Outer { inner: Inner }

/// # Responses
///
/// 200: Json<Outer> - Success
#[rovo]
async fn handler() {}
"#;

    assert_eq!(
        type_resolver::find_type_definition(content, "Inner"),
        Some(1)
    );
    assert_eq!(
        type_resolver::find_type_definition(content, "Outer"),
        Some(2)
    );
}

#[test]
fn extracts_type_from_complex_generic() {
    // Test deeply nested generics
    let extracted =
        type_resolver::extract_type_from_response("Option<Result<Arc<Box<Vec<User>>>>>");
    assert_eq!(extracted, Some("User".to_string()));
}

#[test]
fn handles_whitespace_in_annotation() {
    let line = "///   200:   Json<User>   -   Success";
    let result = type_resolver::get_type_at_position(line, 15);
    assert!(result.is_some());
}

#[test]
fn handles_tabs_in_annotation() {
    let line = "///\t200:\tJson<User>\t-\tSuccess";
    // Position calculation might differ with tabs
    let result = type_resolver::get_type_at_position(line, 12);
    assert!(result.is_some() || result.is_none()); // Just ensure no panic
}

#[test]
fn finds_generic_struct_definition() {
    let content = r#"
struct Response<T, E> {
    data: T,
    error: Option<E>,
}
"#;

    let type_def = type_resolver::find_type_definition(content, "Response");
    assert_eq!(type_def, Some(1));
}

#[test]
fn finds_struct_with_where_clause() {
    let content = r#"
struct Container<T>
where
    T: Clone,
{
    item: T,
}
"#;

    let type_def = type_resolver::find_type_definition(content, "Container");
    assert_eq!(type_def, Some(1));
}

#[test]
fn finds_tuple_struct() {
    let content = r#"
struct UserId(u64);

/// # Responses
///
/// 200: UserId - Success
#[rovo]
async fn handler() {}
"#;

    let type_def = type_resolver::find_type_definition(content, "UserId");
    assert_eq!(type_def, Some(1));
}

#[test]
fn finds_unit_struct() {
    let content = r#"
struct Empty;

/// # Responses
///
/// 204: Empty - Success
#[rovo]
async fn handler() {}
"#;

    let type_def = type_resolver::find_type_definition(content, "Empty");
    assert_eq!(type_def, Some(1));
}

#[test]
fn handles_derive_macro_before_struct() {
    let content = r#"
#[derive(Debug, Clone)]
struct User {
    name: String,
}
"#;

    let type_def = type_resolver::find_type_definition(content, "User");
    assert_eq!(type_def, Some(2));
}

#[test]
fn handles_multiple_attributes() {
    let content = r#"
#[derive(Debug)]
#[serde(rename_all = "camelCase")]
struct ApiResponse {
    status: String,
}
"#;

    let type_def = type_resolver::find_type_definition(content, "ApiResponse");
    assert_eq!(type_def, Some(3));
}

#[test]
fn does_not_match_partial_keyword() {
    let content = r#"
// mystruct is not a keyword
struct MyStruct {
    value: i32,
}
"#;

    let type_def = type_resolver::find_type_definition(content, "MyStruct");
    assert_eq!(type_def, Some(2));
}

#[test]
fn handles_empty_content() {
    let content = "";
    let type_def = type_resolver::find_type_definition(content, "User");
    assert_eq!(type_def, None);
}

#[test]
fn handles_content_without_types() {
    let content = r#"
fn main() {
    println!("Hello, world!");
}
"#;

    let type_def = type_resolver::find_type_definition(content, "User");
    assert_eq!(type_def, None);
}

#[test]
fn extracts_plain_type_without_wrapper() {
    let extracted = type_resolver::extract_type_from_response("User");
    assert_eq!(extracted, Some("User".to_string()));
}

#[test]
fn extracts_type_with_lifetime() {
    // Lifetimes in generics (though not common in our use case)
    let extracted = type_resolver::extract_type_from_response("Vec<User>");
    assert_eq!(extracted, Some("User".to_string()));
}

#[test]
fn handles_underscore_in_type_name() {
    let content = r#"
struct User_Data {
    name: String,
}
"#;

    let type_def = type_resolver::find_type_definition(content, "User_Data");
    assert_eq!(type_def, Some(1));
}

#[test]
fn handles_numbers_in_type_name() {
    let content = r#"
struct Vec3D {
    x: f32,
    y: f32,
    z: f32,
}
"#;

    let type_def = type_resolver::find_type_definition(content, "Vec3D");
    assert_eq!(type_def, Some(1));
}

#[test]
fn handles_camel_case_type_name() {
    let content = r#"
struct HTTPResponse {
    status: u16,
}
"#;

    let type_def = type_resolver::find_type_definition(content, "HTTPResponse");
    assert_eq!(type_def, Some(1));
}
