use rovo_lsp::type_resolver;
use rovo_lsp::utils;

#[test]
fn finds_struct_definition() {
    let content = r#"
struct User {
    name: String,
}

/// # Responses
///
/// 200: Json<User> - Success
#[rovo]
async fn handler() {}
"#;
    let type_def = type_resolver::find_type_definition(content, "User");
    assert_eq!(type_def, Some(1));
}

#[test]
fn finds_enum_definition() {
    let content = r#"
enum Status {
    Active,
    Inactive,
}

/// # Responses
///
/// 200: Json<Status> - Success
#[rovo]
async fn handler() {}
"#;
    let type_def = type_resolver::find_type_definition(content, "Status");
    assert_eq!(type_def, Some(1));
}

#[test]
fn finds_type_alias_definition() {
    let content = r#"
type UserId = u64;

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
fn ignores_type_in_comment() {
    let content = r#"
// This is about User but not the definition
/// Some comment mentioning User
struct User {
    name: String,
}

/// # Responses
///
/// 200: Json<User> - Success
#[rovo]
async fn handler() {}
"#;
    // Should find the actual struct definition, not the comment
    let type_def = type_resolver::find_type_definition(content, "User");
    assert_eq!(type_def, Some(3));
}

#[test]
fn extracts_type_from_json_wrapper() {
    let extracted = type_resolver::extract_type_from_response("Json<User>");
    assert_eq!(extracted, Some("User".to_string()));
}

#[test]
fn extracts_type_from_nested_wrappers() {
    let extracted = type_resolver::extract_type_from_response("Json<Vec<Option<User>>>");
    assert_eq!(extracted, Some("User".to_string()));
}

#[test]
fn extracts_type_from_result() {
    let extracted = type_resolver::extract_type_from_response("Result<User>");
    // Result with single type parameter
    assert_eq!(extracted, Some("User".to_string()));
}

#[test]
fn gets_type_at_position_in_response() {
    let line = "/// 200: Json<TodoItem> - Success";
    // Position on "TodoItem"
    let result = type_resolver::get_type_at_position(line, 15);
    assert!(result.is_some());
    let (response_type, start, end) = result.unwrap();
    assert_eq!(response_type, "Json<TodoItem>");
    assert!(start <= 9 && end >= 23); // Should cover the type
}

#[test]
fn gets_type_at_position_in_example() {
    let line = "/// 200: Vec<User>";
    // Position on "Vec<User>"
    let result = type_resolver::get_type_at_position(line, 12);
    assert!(result.is_some());
}

#[test]
fn no_type_at_wrong_position() {
    let line = "/// 200: Json<TodoItem> - Success";
    // Position on "Success" (not the type)
    let result = type_resolver::get_type_at_position(line, 28);
    assert!(result.is_none());
}

#[test]
fn utf16_conversion_ascii() {
    let line = "Hello, world!";
    assert_eq!(utils::utf16_pos_to_byte_index(line, 0), Some(0));
    assert_eq!(utils::utf16_pos_to_byte_index(line, 5), Some(5));
    assert_eq!(utils::utf16_pos_to_byte_index(line, 13), Some(13));
}

#[test]
fn utf16_conversion_chinese_characters() {
    // "User世界" - Chinese characters are 3 bytes in UTF-8, 1 UTF-16 code unit each
    let line = "/// 200: User世界 - Success";
    // Position 13 in UTF-16 should be at byte index 13 (after "User")
    let byte_idx = utils::utf16_pos_to_byte_index(line, 13);
    assert_eq!(byte_idx, Some(13));
}

#[test]
fn utf16_conversion_emoji() {
    // Emoji is 4 bytes in UTF-8, 2 UTF-16 code units (surrogate pair)
    let line = "/// 200: Json<User👋> - Success";
    // Position 18 in UTF-16 (after "User👋") should account for surrogate pair
    let byte_idx = utils::utf16_pos_to_byte_index(line, 18);
    assert!(byte_idx.is_some());
}

#[test]
fn utf16_conversion_out_of_bounds() {
    let line = "Hello";
    assert_eq!(utils::utf16_pos_to_byte_index(line, 100), None);
}

#[test]
fn utf16_conversion_at_end() {
    let line = "Hello";
    assert_eq!(utils::utf16_pos_to_byte_index(line, 5), Some(5));
}

#[test]
fn byte_to_utf16_ascii() {
    let line = "Hello, world!";
    assert_eq!(utils::byte_index_to_utf16_col(line, 0), 0);
    assert_eq!(utils::byte_index_to_utf16_col(line, 5), 5);
}

#[test]
fn byte_to_utf16_unicode() {
    let line = "Hello 世界";
    assert_eq!(utils::byte_index_to_utf16_col(line, 6), 6); // Start of '世'
    assert_eq!(utils::byte_index_to_utf16_col(line, 9), 7); // Start of '界'
}

#[test]
fn finds_pub_struct() {
    let content = r#"
pub struct User {
    name: String,
}
"#;
    let type_def = type_resolver::find_type_definition(content, "User");
    assert_eq!(type_def, Some(1));
}

#[test]
fn finds_pub_enum() {
    let content = r#"
pub enum Status {
    Active,
}
"#;
    let type_def = type_resolver::find_type_definition(content, "Status");
    assert_eq!(type_def, Some(1));
}

#[test]
fn does_not_match_substring() {
    let content = r#"
struct User {
    name: String,
}

struct UserData {
    user: User,
}
"#;
    // Should find "User", not "UserData"
    let type_def = type_resolver::find_type_definition(content, "User");
    assert_eq!(type_def, Some(1));
}

#[test]
fn handles_type_with_generics_in_definition() {
    let content = r#"
struct Response<T> {
    data: T,
}
"#;
    let type_def = type_resolver::find_type_definition(content, "Response");
    assert_eq!(type_def, Some(1));
}

#[test]
fn extracts_type_from_arc_box_rc() {
    assert_eq!(
        type_resolver::extract_type_from_response("Arc<User>"),
        Some("User".to_string())
    );
    assert_eq!(
        type_resolver::extract_type_from_response("Box<User>"),
        Some("User".to_string())
    );
    assert_eq!(
        type_resolver::extract_type_from_response("Rc<User>"),
        Some("User".to_string())
    );
}

#[test]
fn ignores_whitespace_in_type_extraction() {
    let extracted = type_resolver::extract_type_from_response("  Json<User>  ");
    assert_eq!(extracted, Some("User".to_string()));
}
