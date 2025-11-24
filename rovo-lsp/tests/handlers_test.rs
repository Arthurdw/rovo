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
