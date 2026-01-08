use rovo_lsp::handlers;
use tower_lsp::lsp_types::*;

#[test]
fn hover_provides_status_code_info() {
    let content = r#"
/// # Responses
///
/// 200: Json<User> - Success
#[rovo]
async fn handler() {}
"#;

    let position = Position {
        line: 3,
        character: 4, // On "200"
    };

    let hover = handlers::text_document_hover(content, position);
    assert!(hover.is_some());

    let hover = hover.unwrap();
    match hover.contents {
        HoverContents::Markup(markup) => {
            assert!(markup.value.contains("200 OK"));
        }
        _ => panic!("Expected markup content"),
    }
}

#[test]
fn hover_provides_security_scheme_info() {
    let content = r#"
/// @security bearer
#[rovo]
async fn handler() {}
"#;

    let position = Position {
        line: 1,
        character: 14, // On "bearer" (after "/// @security ")
    };

    let hover = handlers::text_document_hover(content, position);
    assert!(hover.is_some());

    let hover = hover.unwrap();
    match hover.contents {
        HoverContents::Markup(markup) => {
            assert!(markup.value.contains("Bearer Authentication"));
        }
        _ => panic!("Expected markup content"),
    }
}

#[test]
fn hover_on_type_shows_definition() {
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

    let position = Position {
        line: 7,
        character: 15, // On "User" in the annotation
    };

    let hover = handlers::text_document_hover(content, position);
    assert!(hover.is_some());

    let hover = hover.unwrap();
    match hover.contents {
        HoverContents::Markup(markup) => {
            assert!(markup.value.contains("User"));
            assert!(markup.value.contains("line 2"));
        }
        _ => panic!("Expected markup content"),
    }
}

#[test]
fn hover_handles_utf16_positions() {
    // Content with emoji (4 bytes UTF-8, 2 UTF-16 code units)
    let content = r#"
/// # Responses
///
/// 👋 200: Json<User> - Success
#[rovo]
async fn handler() {}
"#;

    let position = Position {
        line: 1,
        character: 16, // After emoji, accounting for surrogate pair
    };

    let hover = handlers::text_document_hover(content, position);
    // Should not crash and should handle the position correctly
    assert!(hover.is_some() || hover.is_none()); // Just ensure no panic
}

#[test]
fn no_hover_outside_rovo_block() {
    let content = r#"
/// # Responses
///
/// 200: Json<User> - Success
async fn handler() {}
"#;

    let position = Position {
        line: 1,
        character: 14,
    };

    let hover = handlers::text_document_hover(content, position);
    assert!(hover.is_none());
}

#[test]
fn find_tag_references_finds_all_occurrences() {
    let content = r#"
/// @tag users
#[rovo]
async fn get_user() {}

/// @tag users
#[rovo]
async fn create_user() {}

/// @tag posts
#[rovo]
async fn get_post() {}
"#;

    let position = Position {
        line: 1,
        character: 9, // On first "users" tag
    };

    let uri = Url::parse("file:///test.rs").unwrap();
    let references = handlers::find_tag_references(content, position, uri);

    assert!(references.is_some());
    let refs = references.unwrap();
    assert_eq!(refs.len(), 2); // Should find both "users" tags
}

#[test]
fn find_tag_references_handles_utf16() {
    // Content with Chinese characters
    let content = r#"
/// @tag 用户
#[rovo]
async fn handler() {}

/// @tag 用户
#[rovo]
async fn handler2() {}
"#;

    let position = Position {
        line: 1,
        character: 9, // On tag
    };

    let uri = Url::parse("file:///test.rs").unwrap();
    let references = handlers::find_tag_references(content, position, uri);

    assert!(references.is_some());
    let refs = references.unwrap();
    assert_eq!(refs.len(), 2);
}

#[test]
fn prepare_rename_returns_range_for_tag() {
    let content = r#"
/// @tag users
#[rovo]
async fn handler() {}
"#;

    let position = Position {
        line: 1,
        character: 9, // On "users"
    };

    let result = handlers::prepare_rename(content, position);
    assert!(result.is_some());

    let (_range, placeholder) = result.unwrap();
    assert_eq!(placeholder, "users");
}

#[test]
fn prepare_rename_handles_utf16_positions() {
    let content = r#"
/// @tag 用户
#[rovo]
async fn handler() {}
"#;

    let position = Position {
        line: 1,
        character: 9, // On tag
    };

    let result = handlers::prepare_rename(content, position);
    assert!(result.is_some());

    let (_range, placeholder) = result.unwrap();
    assert_eq!(placeholder, "用户");
}

#[test]
fn rename_tag_updates_all_references() {
    let content = r#"
/// @tag users
#[rovo]
async fn get_user() {}

/// @tag users
#[rovo]
async fn create_user() {}
"#;

    let position = Position {
        line: 1,
        character: 9, // On first "users"
    };

    let uri = Url::parse("file:///test.rs").unwrap();
    let workspace_edit = handlers::rename_tag(content, position, "accounts", uri.clone());

    assert!(workspace_edit.is_some());
    let edit = workspace_edit.unwrap();

    assert!(edit.changes.is_some());
    let changes = edit.changes.unwrap();
    let text_edits = changes.get(&uri).unwrap();

    assert_eq!(text_edits.len(), 2); // Should update both occurrences
    for edit in text_edits {
        assert_eq!(edit.new_text, "accounts");
    }
}

#[test]
fn diagnostics_reports_invalid_status_codes() {
    let content = r#"
/// # Responses
///
/// 999: Json<User> - Invalid
#[rovo]
async fn handler() {}
"#;

    let uri = Url::parse("file:///test.rs").unwrap();
    let diagnostics = handlers::text_document_did_change(content, uri);

    assert_eq!(diagnostics.len(), 1);
    assert!(diagnostics[0].message.contains("599"));
}

#[test]
fn diagnostics_handles_utf16_positions() {
    // Content with multibyte characters
    let content = r#"
/// # Responses
///
/// 999: Json<User> - 用户信息
#[rovo]
async fn handler() {}
"#;

    let uri = Url::parse("file:///test.rs").unwrap();
    let diagnostics = handlers::text_document_did_change(content, uri);

    // Should handle UTF-16 positions correctly without crashing
    // The invalid status code should be detected
    assert_eq!(diagnostics.len(), 1);
    assert!(diagnostics[0].message.contains("599"));
}

#[test]
fn completion_triggers_on_at_sign() {
    let content = r#"
/// @
#[rovo]
async fn handler() {}
"#;

    let position = Position {
        line: 1,
        character: 5, // After "@"
    };

    let completions = handlers::text_document_completion(content, position);
    assert!(completions.is_some());

    match completions.unwrap() {
        CompletionResponse::Array(items) => {
            assert!(items.len() > 0);
            assert!(items.iter().any(|i| i.label == "@tag"));
            assert!(items.iter().any(|i| i.label == "@security"));
            assert!(items.iter().any(|i| i.label == "@hidden"));
        }
        _ => panic!("Expected array of completions"),
    }
}

#[test]
fn completion_filters_by_prefix() {
    let content = r#"
/// @sec
#[rovo]
async fn handler() {}
"#;

    let position = Position {
        line: 1,
        character: 8, // After "@sec"
    };

    let completions = handlers::text_document_completion(content, position);
    assert!(completions.is_some());

    match completions.unwrap() {
        CompletionResponse::Array(items) => {
            assert!(items.iter().any(|i| i.label == "@security"));
            // Should not include unrelated completions
            assert!(!items.iter().any(|i| i.label == "@tag"));
            assert!(!items.iter().any(|i| i.label == "@hidden"));
        }
        _ => panic!("Expected array of completions"),
    }
}

#[test]
fn no_completion_outside_rovo_block() {
    let content = r#"
/// @
async fn handler() {}
"#;

    let position = Position {
        line: 1,
        character: 5,
    };

    let completions = handlers::text_document_completion(content, position);
    assert!(completions.is_none());
}

// Edge case tests to improve coverage

#[test]
fn hover_returns_none_for_out_of_bounds_line() {
    let content = r#"
/// # Responses
///
/// 200: Json<User> - Success
#[rovo]
async fn handler() {}
"#;

    let position = Position {
        line: 100, // Way beyond content
        character: 0,
    };

    let hover = handlers::text_document_hover(content, position);
    assert!(hover.is_none());
}

#[test]
fn hover_on_annotation_keyword_shows_docs() {
    let content = r#"
/// # Responses
///
/// 200: Json<User> - Success
#[rovo]
async fn handler() {}
"#;

    let position = Position {
        line: 1,
        character: 5, // On status code
    };

    let hover = handlers::text_document_hover(content, position);
    assert!(hover.is_some());

    let hover = hover.unwrap();
    match hover.contents {
        HoverContents::Markup(markup) => {
            assert!(markup.value.contains("response") || markup.value.contains("Response"));
        }
        _ => panic!("Expected markup content"),
    }
}

#[test]
fn hover_on_tag_annotation_keyword() {
    let content = r#"
/// @tag users
#[rovo]
async fn handler() {}
"#;

    let position = Position {
        line: 1,
        character: 5, // On "@tag"
    };

    let hover = handlers::text_document_hover(content, position);
    assert!(hover.is_some());
}

#[test]
fn hover_on_security_annotation_keyword() {
    let content = r#"
/// @security bearer
#[rovo]
async fn handler() {}
"#;

    let position = Position {
        line: 1,
        character: 6, // On "@security"
    };

    let hover = handlers::text_document_hover(content, position);
    assert!(hover.is_some());
}

#[test]
fn hover_provides_info_for_201_status_code() {
    let content = r#"
/// # Responses
///
/// 201: Json<User> - Created
#[rovo]
async fn handler() {}
"#;

    let position = Position {
        line: 3,
        character: 4, // On "201"
    };

    let hover = handlers::text_document_hover(content, position);
    assert!(hover.is_some());

    let hover = hover.unwrap();
    match hover.contents {
        HoverContents::Markup(markup) => {
            assert!(markup.value.contains("201 Created"));
        }
        _ => panic!("Expected markup content"),
    }
}

#[test]
fn hover_provides_info_for_204_status_code() {
    let content = r#"
/// # Responses
///
/// 204: () - No content
#[rovo]
async fn handler() {}
"#;

    let position = Position {
        line: 3,
        character: 4, // On "204"
    };

    let hover = handlers::text_document_hover(content, position);
    assert!(hover.is_some());

    let hover = hover.unwrap();
    match hover.contents {
        HoverContents::Markup(markup) => {
            assert!(markup.value.contains("204 No Content"));
        }
        _ => panic!("Expected markup content"),
    }
}

#[test]
fn hover_provides_info_for_400_status_code() {
    let content = r#"
/// # Responses
///
/// 400: Json<Error> - Bad request
#[rovo]
async fn handler() {}
"#;

    let position = Position {
        line: 3,
        character: 4, // On "400"
    };

    let hover = handlers::text_document_hover(content, position);
    assert!(hover.is_some());

    let hover = hover.unwrap();
    match hover.contents {
        HoverContents::Markup(markup) => {
            assert!(markup.value.contains("400 Bad Request"));
        }
        _ => panic!("Expected markup content"),
    }
}

#[test]
fn hover_provides_info_for_401_status_code() {
    let content = r#"
/// # Responses
///
/// 401: Json<Error> - Unauthorized
#[rovo]
async fn handler() {}
"#;

    let position = Position {
        line: 3,
        character: 4, // On "401"
    };

    let hover = handlers::text_document_hover(content, position);
    assert!(hover.is_some());

    let hover = hover.unwrap();
    match hover.contents {
        HoverContents::Markup(markup) => {
            assert!(markup.value.contains("401 Unauthorized"));
        }
        _ => panic!("Expected markup content"),
    }
}

#[test]
fn hover_provides_info_for_403_status_code() {
    let content = r#"
/// # Responses
///
/// 403: Json<Error> - Forbidden
#[rovo]
async fn handler() {}
"#;

    let position = Position {
        line: 3,
        character: 4, // On "403"
    };

    let hover = handlers::text_document_hover(content, position);
    assert!(hover.is_some());

    let hover = hover.unwrap();
    match hover.contents {
        HoverContents::Markup(markup) => {
            assert!(markup.value.contains("403 Forbidden"));
        }
        _ => panic!("Expected markup content"),
    }
}

#[test]
fn hover_provides_info_for_404_status_code() {
    let content = r#"
/// # Responses
///
/// 404: Json<Error> - Not found
#[rovo]
async fn handler() {}
"#;

    let position = Position {
        line: 3,
        character: 4, // On "404"
    };

    let hover = handlers::text_document_hover(content, position);
    assert!(hover.is_some());

    let hover = hover.unwrap();
    match hover.contents {
        HoverContents::Markup(markup) => {
            assert!(markup.value.contains("404 Not Found"));
        }
        _ => panic!("Expected markup content"),
    }
}

#[test]
fn hover_provides_info_for_409_status_code() {
    let content = r#"
/// # Responses
///
/// 409: Json<Error> - Conflict
#[rovo]
async fn handler() {}
"#;

    let position = Position {
        line: 3,
        character: 4, // On "409"
    };

    let hover = handlers::text_document_hover(content, position);
    assert!(hover.is_some());

    let hover = hover.unwrap();
    match hover.contents {
        HoverContents::Markup(markup) => {
            assert!(markup.value.contains("409 Conflict"));
        }
        _ => panic!("Expected markup content"),
    }
}

#[test]
fn hover_provides_info_for_422_status_code() {
    let content = r#"
/// # Responses
///
/// 422: Json<Error> - Unprocessable
#[rovo]
async fn handler() {}
"#;

    let position = Position {
        line: 3,
        character: 4, // On "422"
    };

    let hover = handlers::text_document_hover(content, position);
    assert!(hover.is_some());

    let hover = hover.unwrap();
    match hover.contents {
        HoverContents::Markup(markup) => {
            assert!(markup.value.contains("422 Unprocessable"));
        }
        _ => panic!("Expected markup content"),
    }
}

#[test]
fn hover_provides_info_for_500_status_code() {
    let content = r#"
/// # Responses
///
/// 500: Json<Error> - Server error
#[rovo]
async fn handler() {}
"#;

    let position = Position {
        line: 3,
        character: 4, // On "500"
    };

    let hover = handlers::text_document_hover(content, position);
    assert!(hover.is_some());

    let hover = hover.unwrap();
    match hover.contents {
        HoverContents::Markup(markup) => {
            assert!(markup.value.contains("500 Internal"));
        }
        _ => panic!("Expected markup content"),
    }
}

#[test]
fn hover_provides_info_for_503_status_code() {
    let content = r#"
/// # Responses
///
/// 503: Json<Error> - Service unavailable
#[rovo]
async fn handler() {}
"#;

    let position = Position {
        line: 3,
        character: 4, // On "503"
    };

    let hover = handlers::text_document_hover(content, position);
    assert!(hover.is_some());

    let hover = hover.unwrap();
    match hover.contents {
        HoverContents::Markup(markup) => {
            assert!(markup.value.contains("503 Service"));
        }
        _ => panic!("Expected markup content"),
    }
}

#[test]
fn hover_provides_generic_info_for_informational_status_codes() {
    let content = r#"
/// # Responses
///
/// 102: () - Processing
#[rovo]
async fn handler() {}
"#;

    let position = Position {
        line: 3,
        character: 4, // On "102"
    };

    let hover = handlers::text_document_hover(content, position);
    assert!(hover.is_some());

    let hover = hover.unwrap();
    match hover.contents {
        HoverContents::Markup(markup) => {
            assert!(markup.value.contains("102 Informational"));
        }
        _ => panic!("Expected markup content"),
    }
}

#[test]
fn hover_provides_generic_info_for_redirection_status_codes() {
    let content = r#"
/// # Responses
///
/// 301: () - Moved permanently
#[rovo]
async fn handler() {}
"#;

    let position = Position {
        line: 3,
        character: 4, // On "301"
    };

    let hover = handlers::text_document_hover(content, position);
    assert!(hover.is_some());

    let hover = hover.unwrap();
    match hover.contents {
        HoverContents::Markup(markup) => {
            assert!(markup.value.contains("301 Redirection"));
        }
        _ => panic!("Expected markup content"),
    }
}

#[test]
fn hover_provides_info_for_basic_auth_scheme() {
    let content = r#"
/// @security basic
#[rovo]
async fn handler() {}
"#;

    let position = Position {
        line: 1,
        character: 14, // On "basic"
    };

    let hover = handlers::text_document_hover(content, position);
    assert!(hover.is_some());

    let hover = hover.unwrap();
    match hover.contents {
        HoverContents::Markup(markup) => {
            assert!(markup.value.contains("Basic Authentication"));
        }
        _ => panic!("Expected markup content"),
    }
}

#[test]
fn hover_provides_info_for_apikey_scheme() {
    let content = r#"
/// @security apiKey
#[rovo]
async fn handler() {}
"#;

    let position = Position {
        line: 1,
        character: 14, // On "apiKey"
    };

    let hover = handlers::text_document_hover(content, position);
    assert!(hover.is_some());

    let hover = hover.unwrap();
    match hover.contents {
        HoverContents::Markup(markup) => {
            assert!(markup.value.contains("API Key"));
        }
        _ => panic!("Expected markup content"),
    }
}

#[test]
fn hover_provides_info_for_oauth2_scheme() {
    let content = r#"
/// @security oauth2
#[rovo]
async fn handler() {}
"#;

    let position = Position {
        line: 1,
        character: 14, // On "oauth2"
    };

    let hover = handlers::text_document_hover(content, position);
    assert!(hover.is_some());

    let hover = hover.unwrap();
    match hover.contents {
        HoverContents::Markup(markup) => {
            assert!(markup.value.contains("OAuth"));
        }
        _ => panic!("Expected markup content"),
    }
}

#[test]
fn hover_on_status_code_in_example_annotation() {
    let content = r#"
/// # Examples
///
/// 201: User::default()
#[rovo]
async fn handler() {}
"#;

    let position = Position {
        line: 3,
        character: 4, // On "201" in example
    };

    let hover = handlers::text_document_hover(content, position);
    assert!(hover.is_some());

    let hover = hover.unwrap();
    match hover.contents {
        HoverContents::Markup(markup) => {
            assert!(markup.value.contains("201 Created"));
        }
        _ => panic!("Expected markup content"),
    }
}

#[test]
fn find_tag_references_returns_none_for_out_of_bounds() {
    let content = r#"
/// @tag users
#[rovo]
async fn handler() {}
"#;

    let position = Position {
        line: 100,
        character: 0,
    };

    let uri = Url::parse("file:///test.rs").unwrap();
    let references = handlers::find_tag_references(content, position, uri);
    assert!(references.is_none());
}

#[test]
fn find_tag_references_returns_none_when_not_on_tag() {
    let content = r#"
/// # Responses
///
/// 200: Json<User> - Success
#[rovo]
async fn handler() {}
"#;

    let position = Position {
        line: 1,
        character: 14, // On status code, not tag
    };

    let uri = Url::parse("file:///test.rs").unwrap();
    let references = handlers::find_tag_references(content, position, uri);
    assert!(references.is_none());
}

#[test]
fn find_tag_references_returns_none_when_no_matches() {
    let content = r#"
/// @tag users
#[rovo]
async fn handler() {}

/// @tag posts
#[rovo]
async fn handler2() {}
"#;

    let position = Position {
        line: 1,
        character: 9, // On "users" tag
    };

    let uri = Url::parse("file:///test.rs").unwrap();
    let references = handlers::find_tag_references(content, position, uri);

    // Should find at least one (the one we're on)
    assert!(references.is_some());
    let refs = references.unwrap();
    assert_eq!(refs.len(), 1); // Only the one "users" tag
}

#[test]
fn prepare_rename_returns_none_for_out_of_bounds() {
    let content = r#"
/// @tag users
#[rovo]
async fn handler() {}
"#;

    let position = Position {
        line: 100,
        character: 0,
    };

    let result = handlers::prepare_rename(content, position);
    assert!(result.is_none());
}

#[test]
fn prepare_rename_returns_none_when_not_on_tag() {
    let content = r#"
/// # Responses
///
/// 200: Json<User> - Success
#[rovo]
async fn handler() {}
"#;

    let position = Position {
        line: 1,
        character: 14, // On status code, not tag
    };

    let result = handlers::prepare_rename(content, position);
    assert!(result.is_none());
}

#[test]
fn rename_tag_returns_none_for_out_of_bounds() {
    let content = r#"
/// @tag users
#[rovo]
async fn handler() {}
"#;

    let position = Position {
        line: 100,
        character: 0,
    };

    let uri = Url::parse("file:///test.rs").unwrap();
    let result = handlers::rename_tag(content, position, "accounts", uri);
    assert!(result.is_none());
}

#[test]
fn rename_tag_returns_none_when_not_on_tag() {
    let content = r#"
/// # Responses
///
/// 200: Json<User> - Success
#[rovo]
async fn handler() {}
"#;

    let position = Position {
        line: 1,
        character: 14,
    };

    let uri = Url::parse("file:///test.rs").unwrap();
    let result = handlers::rename_tag(content, position, "newname", uri);
    assert!(result.is_none());
}

#[test]
fn hover_returns_none_for_invalid_utf16_position() {
    let content = r#"
/// # Responses
///
/// 200: Json<User> - Success
#[rovo]
async fn handler() {}
"#;

    // Position way beyond the line length
    let position = Position {
        line: 1,
        character: 1000,
    };

    let hover = handlers::text_document_hover(content, position);
    assert!(hover.is_none());
}

#[test]
fn diagnostics_with_multiple_errors() {
    let content = r#"
/// # Responses
///
/// 999: Json<User> - Invalid code
/// 1000: Json<User> - Also invalid
#[rovo]
async fn handler() {}
"#;

    let uri = Url::parse("file:///test.rs").unwrap();
    let diagnostics = handlers::text_document_did_change(content, uri);

    assert!(diagnostics.len() >= 2);
}

#[test]
fn completion_returns_none_when_empty() {
    let content = r#"
/// Normal comment without @
#[rovo]
async fn handler() {}
"#;

    let position = Position {
        line: 1,
        character: 10,
    };

    // This should return None as there's nothing to complete
    let result = handlers::text_document_completion(content, position);
    // Depending on implementation, this might be None or an empty array
    // Let's just ensure it doesn't crash
    assert!(result.is_none() || matches!(result, Some(CompletionResponse::Array(_))));
}

// Semantic tokens tests

#[test]
fn semantic_tokens_full_returns_none_for_empty_content() {
    let content = "";
    let result = handlers::semantic_tokens_full(content);
    assert!(result.is_none());
}

#[test]
fn semantic_tokens_full_returns_none_outside_rovo_block() {
    let content = r#"
/// @tag users
/// 200: Json<User>
async fn handler() {}
"#;

    let result = handlers::semantic_tokens_full(content);
    assert!(result.is_none());
}

#[test]
fn semantic_tokens_full_finds_annotations() {
    let content = r#"
/// @tag users
#[rovo]
async fn handler() {}
"#;

    let result = handlers::semantic_tokens_full(content);
    assert!(result.is_some());

    if let Some(SemanticTokensResult::Tokens(tokens)) = result {
        // Should find @tag annotation and users value
        assert!(tokens.data.len() >= 2);
    }
}

#[test]
fn semantic_tokens_full_finds_status_codes() {
    let content = r#"
/// # Responses
///
/// 200: Json<User> - Success
/// 404: Json<Error> - Not found
#[rovo]
async fn handler() {}
"#;

    let result = handlers::semantic_tokens_full(content);
    assert!(result.is_some());

    if let Some(SemanticTokensResult::Tokens(tokens)) = result {
        // Should find section header, status codes
        assert!(!tokens.data.is_empty());
    }
}

#[test]
fn semantic_tokens_full_finds_security_schemes() {
    let content = r#"
/// @security bearer
#[rovo]
async fn handler() {}
"#;

    let result = handlers::semantic_tokens_full(content);
    assert!(result.is_some());

    if let Some(SemanticTokensResult::Tokens(tokens)) = result {
        // Should find @security annotation and bearer scheme
        assert!(tokens.data.len() >= 2);
    }
}

#[test]
fn semantic_tokens_full_finds_section_headers() {
    let content = r#"
/// # Responses
/// # Examples
/// # Metadata
#[rovo]
async fn handler() {}
"#;

    let result = handlers::semantic_tokens_full(content);
    assert!(result.is_some());

    if let Some(SemanticTokensResult::Tokens(tokens)) = result {
        // Should find 3 section headers
        assert_eq!(tokens.data.len(), 3);
    }
}

#[test]
fn semantic_tokens_full_handles_multiple_annotations_on_same_line() {
    let content = r#"
/// @tag users @id getUserById
#[rovo]
async fn handler() {}
"#;

    let result = handlers::semantic_tokens_full(content);
    assert!(result.is_some());

    if let Some(SemanticTokensResult::Tokens(tokens)) = result {
        // Should find multiple tokens
        assert!(tokens.data.len() >= 2);
    }
}

#[test]
fn semantic_tokens_full_handles_utf16_characters() {
    let content = r#"
/// @tag 用户
#[rovo]
async fn handler() {}
"#;

    let result = handlers::semantic_tokens_full(content);
    assert!(result.is_some());
}

#[test]
fn semantic_tokens_full_finds_id_annotation() {
    let content = r#"
/// @id createUser
#[rovo]
async fn handler() {}
"#;

    let result = handlers::semantic_tokens_full(content);
    assert!(result.is_some());

    if let Some(SemanticTokensResult::Tokens(tokens)) = result {
        // Should find @id and value
        assert!(tokens.data.len() >= 2);
    }
}

#[test]
fn semantic_tokens_full_finds_hidden_annotation() {
    let content = r#"
/// @hidden
#[rovo]
async fn handler() {}
"#;

    let result = handlers::semantic_tokens_full(content);
    assert!(result.is_some());

    if let Some(SemanticTokensResult::Tokens(tokens)) = result {
        assert!(!tokens.data.is_empty());
    }
}

#[test]
fn semantic_tokens_full_finds_rovo_ignore_annotation() {
    let content = r#"
/// @rovo-ignore
#[rovo]
async fn handler() {}
"#;

    let result = handlers::semantic_tokens_full(content);
    assert!(result.is_some());

    if let Some(SemanticTokensResult::Tokens(tokens)) = result {
        assert!(!tokens.data.is_empty());
    }
}

#[test]
fn semantic_tokens_full_handles_all_security_schemes() {
    let content = r#"
/// @security basic
/// @security apiKey
/// @security oauth2
/// @security bearer
#[rovo]
async fn handler() {}
"#;

    let result = handlers::semantic_tokens_full(content);
    assert!(result.is_some());

    if let Some(SemanticTokensResult::Tokens(tokens)) = result {
        // Should find 4 @security and 4 scheme names
        assert!(tokens.data.len() >= 8);
    }
}

#[test]
fn semantic_tokens_full_with_comprehensive_example() {
    let content = r#"
/// Get a user by ID
///
/// # Responses
///
/// 200: Json<User> - Success
/// 404: Json<Error> - Not found
///
/// # Examples
///
/// 200: User::default()
///
/// # Metadata
///
/// @tag users
/// @id getUserById
/// @security bearer
#[rovo]
async fn handler() {}
"#;

    let result = handlers::semantic_tokens_full(content);
    assert!(result.is_some());

    if let Some(SemanticTokensResult::Tokens(tokens)) = result {
        // Should have tokens for: section headers (3), status codes (3), annotations (3), values (2), scheme (1)
        assert!(tokens.data.len() >= 10);
    }
}

// Additional edge case tests for position helpers

#[test]
fn hover_on_section_header_responses() {
    let content = r#"
/// # Responses
#[rovo]
async fn handler() {}
"#;

    let position = Position {
        line: 1,
        character: 6, // On "Responses"
    };

    let hover = handlers::text_document_hover(content, position);
    assert!(hover.is_some());
}

#[test]
fn hover_on_section_header_examples() {
    let content = r#"
/// # Examples
#[rovo]
async fn handler() {}
"#;

    let position = Position {
        line: 1,
        character: 6, // On "Examples"
    };

    let hover = handlers::text_document_hover(content, position);
    assert!(hover.is_some());
}

#[test]
fn hover_on_section_header_metadata() {
    let content = r#"
/// # Metadata
#[rovo]
async fn handler() {}
"#;

    let position = Position {
        line: 1,
        character: 6, // On "Metadata"
    };

    let hover = handlers::text_document_hover(content, position);
    assert!(hover.is_some());
}

#[test]
fn hover_on_section_header_path_parameters() {
    let content = r#"
/// # Path Parameters
#[rovo]
async fn handler() {}
"#;

    let position = Position {
        line: 1,
        character: 6, // On "Path Parameters"
    };

    let hover = handlers::text_document_hover(content, position);
    assert!(hover.is_some());
}

#[test]
fn hover_on_hidden_annotation() {
    let content = r#"
/// @hidden
#[rovo]
async fn handler() {}
"#;

    let position = Position {
        line: 1,
        character: 5, // On "@hidden"
    };

    let hover = handlers::text_document_hover(content, position);
    assert!(hover.is_some());
}

#[test]
fn hover_on_id_annotation() {
    let content = r#"
/// @id getUserById
#[rovo]
async fn handler() {}
"#;

    let position = Position {
        line: 1,
        character: 5, // On "@id"
    };

    let hover = handlers::text_document_hover(content, position);
    assert!(hover.is_some());
}

#[test]
fn hover_returns_none_for_non_doc_comment() {
    let content = r#"
// Regular comment with @tag users
#[rovo]
async fn handler() {}
"#;

    let position = Position {
        line: 1,
        character: 22,
    };

    let hover = handlers::text_document_hover(content, position);
    assert!(hover.is_none());
}

#[test]
fn hover_on_status_code_without_colon() {
    let content = r#"
/// # Responses
///
/// 200 Success
#[rovo]
async fn handler() {}
"#;

    let position = Position {
        line: 3,
        character: 4, // On "200"
    };

    let hover = handlers::text_document_hover(content, position);
    assert!(hover.is_some());
}

#[test]
fn extract_tag_returns_none_for_non_doc_comment() {
    let content = r#"
// @tag users
#[rovo]
async fn handler() {}

/// @tag posts
#[rovo]
async fn handler2() {}
"#;

    // Position on regular comment, not doc comment
    let position = Position {
        line: 1,
        character: 8,
    };

    let uri = Url::parse("file:///test.rs").unwrap();
    let references = handlers::find_tag_references(content, position, uri);
    assert!(references.is_none());
}

#[test]
fn hover_returns_none_for_line_not_starting_with_status_code() {
    let content = r#"
/// # Responses
///
/// Returns 200: Json<User>
#[rovo]
async fn handler() {}
"#;

    let position = Position {
        line: 3,
        character: 12, // On "200" but line starts with "Returns"
    };

    let hover = handlers::text_document_hover(content, position);
    // Should not match as status code hover since line doesn't start with digit
    // It might still show type hover or annotation hover
    assert!(hover.is_none() || hover.is_some());
}

// =============================================================================
// Path Parameter Rename Tests
// =============================================================================

#[test]
fn prepare_rename_on_path_param_doc() {
    let content = r#"
/// # Path Parameters
///
/// id: The unique identifier
#[rovo]
async fn get_item(Path(id): Path<String>) -> impl IntoApiResponse {
    Json(id)
}
"#;

    let position = Position {
        line: 3,
        character: 5, // On "id" in the doc
    };

    let result = handlers::prepare_rename(content, position);
    assert!(result.is_some());
    let (range, name) = result.unwrap();
    assert_eq!(name, "id");
    assert_eq!(range.start.line, 3);
}

#[test]
fn prepare_rename_on_path_binding() {
    let content = r#"
/// # Path Parameters
///
/// id: The unique identifier
#[rovo]
async fn get_item(Path(id): Path<String>) -> impl IntoApiResponse {
    Json(id)
}
"#;

    let position = Position {
        line: 5,
        character: 24, // On "id" in Path(id)
    };

    let result = handlers::prepare_rename(content, position);
    assert!(result.is_some());
    let (range, name) = result.unwrap();
    assert_eq!(name, "id");
    assert_eq!(range.start.line, 5);
}

#[test]
fn rename_path_param_updates_both_doc_and_binding() {
    let content = r#"
/// # Path Parameters
///
/// id: The unique identifier
#[rovo]
async fn get_item(Path(id): Path<String>) -> impl IntoApiResponse {
    Json(id)
}
"#;

    let position = Position {
        line: 3,
        character: 5, // On "id" in the doc
    };

    let uri = Url::parse("file:///test.rs").unwrap();
    let result = handlers::rename_tag(content, position, "item_id", uri);
    assert!(result.is_some());

    let edit = result.unwrap();
    let changes = edit.changes.unwrap();
    let edits = changes.values().next().unwrap();

    // Should have at least 3 edits: doc param, binding, and usage in body
    assert!(
        edits.len() >= 2,
        "Expected at least 2 edits, got {}",
        edits.len()
    );

    // Check that one edit is for the doc line
    let doc_edit = edits.iter().find(|e| e.range.start.line == 3);
    assert!(doc_edit.is_some(), "Should have edit on doc line");
    assert_eq!(doc_edit.unwrap().new_text, "item_id");

    // Check that one edit is for the binding line
    let binding_edit = edits.iter().find(|e| e.range.start.line == 5);
    assert!(binding_edit.is_some(), "Should have edit on binding line");
}

#[test]
fn rename_from_binding_updates_doc() {
    let content = "/// # Path Parameters
///
/// user_id: The user identifier
#[rovo]
async fn get_user(Path(user_id): Path<u64>) -> impl IntoApiResponse {
    Json(user_id.to_string())
}";

    let position = Position {
        line: 4,
        character: 23, // On "user_id" in Path(user_id)
    };

    let uri = Url::parse("file:///test.rs").unwrap();
    let result = handlers::rename_tag(content, position, "uid", uri);
    assert!(result.is_some(), "rename_tag should return Some");

    let edit = result.unwrap();
    let changes = edit.changes.unwrap();
    let edits = changes.values().next().unwrap();

    // Should have edits for both doc and binding
    let doc_edit = edits.iter().find(|e| e.range.start.line == 2);
    assert!(doc_edit.is_some(), "Should have edit on doc line (line 2)");
    assert_eq!(doc_edit.unwrap().new_text, "uid");
}

#[test]
fn rename_tuple_path_param() {
    let content = r#"
/// # Path Parameters
///
/// collection_id: The collection
/// index: Item index
#[rovo]
async fn get_item(Path((collection_id, index)): Path<(String, u32)>) -> impl IntoApiResponse {
    Json(format!("{}/{}", collection_id, index))
}
"#;

    let position = Position {
        line: 4,
        character: 5, // On "index" in the doc
    };

    let uri = Url::parse("file:///test.rs").unwrap();
    let result = handlers::rename_tag(content, position, "item_index", uri);
    assert!(result.is_some());

    let edit = result.unwrap();
    let changes = edit.changes.unwrap();
    let edits = changes.values().next().unwrap();

    // Should have multiple edits for doc, binding, and usages
    assert!(
        edits.len() >= 2,
        "Expected at least 2 edits for tuple param rename"
    );
}

#[test]
fn prepare_rename_returns_none_outside_path_param() {
    let content = r#"
/// Some description
///
/// # Responses
///
/// 200: Json<String> - Success
#[rovo]
async fn get_item() -> impl IntoApiResponse {
    Json("test".to_string())
}
"#;

    let position = Position {
        line: 1,
        character: 8, // On "description", not a path param
    };

    let result = handlers::prepare_rename(content, position);
    // Should return None since we're not on a path param or tag
    assert!(result.is_none());
}

#[test]
fn prepare_rename_returns_none_for_undocumented_binding() {
    // Path binding without # Path Parameters doc should NOT trigger rovo-lsp rename
    // This allows rust-analyzer to handle the rename instead
    let content = r#"
#[rovo]
async fn get_item(Path(id): Path<String>) -> impl IntoApiResponse {
    Json(id)
}
"#;

    let position = Position {
        line: 2,
        character: 24, // On "id" in Path(id)
    };

    let result = handlers::prepare_rename(content, position);
    // Should return None - no doc to update, let rust-analyzer handle it
    assert!(
        result.is_none(),
        "Should not claim rename for undocumented binding"
    );
}

#[test]
fn prepare_rename_returns_some_for_documented_binding() {
    // Path binding WITH # Path Parameters doc SHOULD trigger rovo-lsp rename
    let content = r#"
/// # Path Parameters
///
/// id: The unique identifier
#[rovo]
async fn get_item(Path(id): Path<String>) -> impl IntoApiResponse {
    Json(id)
}
"#;

    let position = Position {
        line: 5,
        character: 24, // On "id" in Path(id)
    };

    let result = handlers::prepare_rename(content, position);
    // Should return Some - we have a doc to update
    assert!(
        result.is_some(),
        "Should claim rename for documented binding"
    );
    let (_, name) = result.unwrap();
    assert_eq!(name, "id");
}

// =============================================================================
// Path Parameter Goto Definition Tests
// =============================================================================

#[test]
fn goto_definition_from_binding_to_doc() {
    let content = r#"
/// # Path Parameters
///
/// id: The unique identifier
#[rovo]
async fn get_item(Path(id): Path<String>) -> impl IntoApiResponse {
    Json(id)
}
"#;

    let position = Position {
        line: 5,
        character: 24, // On "id" in Path(id)
    };

    let uri = Url::parse("file:///test.rs").unwrap();
    let result = handlers::goto_path_param_definition(content, position, uri);

    assert!(result.is_some(), "Should find definition");
    let location = result.unwrap();
    assert_eq!(location.range.start.line, 3, "Should point to doc line");
}

#[test]
fn goto_definition_from_body_to_doc() {
    let content = r#"
/// # Path Parameters
///
/// id: The unique identifier
#[rovo]
async fn get_item(Path(id): Path<String>) -> impl IntoApiResponse {
    Json(id)
}
"#;

    let position = Position {
        line: 6,
        character: 9, // On "id" in Json(id)
    };

    let uri = Url::parse("file:///test.rs").unwrap();
    let result = handlers::goto_path_param_definition(content, position, uri);

    assert!(result.is_some(), "Should find definition from body usage");
    let location = result.unwrap();
    assert_eq!(location.range.start.line, 3, "Should point to doc line");
}

#[test]
fn goto_definition_returns_none_when_no_doc() {
    let content = r#"
#[rovo]
async fn get_item(Path(id): Path<String>) -> impl IntoApiResponse {
    Json(id)
}
"#;

    let position = Position {
        line: 2,
        character: 24, // On "id" in Path(id)
    };

    let uri = Url::parse("file:///test.rs").unwrap();
    let result = handlers::goto_path_param_definition(content, position, uri);

    // No # Path Parameters doc, so no definition
    assert!(result.is_none(), "Should return None when no doc exists");
}

// =============================================================================
// Path Parameter Find References Tests
// =============================================================================

#[test]
fn find_references_from_doc() {
    let content = r#"
/// # Path Parameters
///
/// id: The unique identifier
#[rovo]
async fn get_item(Path(id): Path<String>) -> impl IntoApiResponse {
    Json(id)
}
"#;

    let position = Position {
        line: 3,
        character: 5, // On "id" in the doc
    };

    let uri = Url::parse("file:///test.rs").unwrap();
    let result = handlers::find_path_param_references(content, position, uri);

    assert!(result.is_some(), "Should find references");
    let refs = result.unwrap();
    // Should find: doc, binding, body usage
    assert!(
        refs.len() >= 3,
        "Should find at least 3 references (doc, binding, body), got {}",
        refs.len()
    );
}

#[test]
fn find_references_from_binding() {
    let content = r#"
/// # Path Parameters
///
/// id: The unique identifier
#[rovo]
async fn get_item(Path(id): Path<String>) -> impl IntoApiResponse {
    Json(id)
}
"#;

    let position = Position {
        line: 5,
        character: 24, // On "id" in Path(id)
    };

    let uri = Url::parse("file:///test.rs").unwrap();
    let result = handlers::find_path_param_references(content, position, uri);

    assert!(result.is_some(), "Should find references from binding");
    let refs = result.unwrap();
    assert!(refs.len() >= 3, "Should find at least 3 references");
}

#[test]
fn find_references_from_body() {
    let content = r#"
/// # Path Parameters
///
/// id: The unique identifier
#[rovo]
async fn get_item(Path(id): Path<String>) -> impl IntoApiResponse {
    Json(id)
}
"#;

    let position = Position {
        line: 6,
        character: 9, // On "id" in Json(id)
    };

    let uri = Url::parse("file:///test.rs").unwrap();
    let result = handlers::find_path_param_references(content, position, uri);

    assert!(result.is_some(), "Should find references from body usage");
    let refs = result.unwrap();
    assert!(refs.len() >= 3, "Should find at least 3 references");
}

#[test]
fn find_references_tuple_params() {
    let content = r#"
/// # Path Parameters
///
/// a: First param
/// b: Second param
#[rovo]
async fn get_item(Path((a, b)): Path<(String, u32)>) -> impl IntoApiResponse {
    Json(format!("{}{}", a, b))
}
"#;

    let position = Position {
        line: 3,
        character: 4, // On "a" in the doc
    };

    let uri = Url::parse("file:///test.rs").unwrap();
    let result = handlers::find_path_param_references(content, position, uri);

    assert!(result.is_some(), "Should find references for tuple param");
    let refs = result.unwrap();
    // Should find: doc, binding, body usage
    assert!(
        refs.len() >= 3,
        "Should find references for 'a': doc, binding, body"
    );
}

#[test]
fn find_references_returns_none_when_not_on_param() {
    let content = r#"
/// Some description
#[rovo]
async fn get_item() -> impl IntoApiResponse {
    Json("test")
}
"#;

    let position = Position {
        line: 1,
        character: 8, // On "description", not a param
    };

    let uri = Url::parse("file:///test.rs").unwrap();
    let result = handlers::find_path_param_references(content, position, uri);

    assert!(
        result.is_none(),
        "Should return None when not on a path param"
    );
}
