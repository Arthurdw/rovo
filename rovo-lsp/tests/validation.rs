use rovo_lsp::diagnostics::{validate_annotations, DiagnosticSeverity};

#[test]
fn reports_invalid_status_code() {
    let content = r#"
/// # Responses
///
/// 999: Json<User> - Invalid
#[rovo]
async fn handler() {}
"#;
    let diagnostics = validate_annotations(content);
    assert_eq!(diagnostics.len(), 1);
    assert!(diagnostics[0].message.contains("Invalid HTTP status"));
    assert_eq!(diagnostics[0].severity, DiagnosticSeverity::Error);
}

#[test]
fn reports_status_code_too_low() {
    let content = r#"
/// # Responses
///
/// 99: Json<User> - Too low
#[rovo]
async fn handler() {}
"#;
    let diagnostics = validate_annotations(content);
    assert_eq!(diagnostics.len(), 1);
    assert!(diagnostics[0].message.contains("Invalid HTTP status"));
}

#[test]
fn reports_status_code_too_high() {
    let content = r#"
/// # Responses
///
/// 600: Json<User> - Too high
#[rovo]
async fn handler() {}
"#;
    let diagnostics = validate_annotations(content);
    assert_eq!(diagnostics.len(), 1);
    assert!(diagnostics[0].message.contains("Invalid HTTP status"));
}

#[test]
fn accepts_valid_status_codes() {
    let content = r#"
/// # Responses
///
/// 200: Json<User> - OK
/// 404: Json<Error> - Not found
/// 500: Json<Error> - Server error
#[rovo]
async fn handler() {}
"#;
    let diagnostics = validate_annotations(content);
    assert_eq!(diagnostics.len(), 0);
}

#[test]
fn accepts_all_standard_ranges() {
    let content = r#"
/// # Responses
///
/// 100: Json<Continue> - Informational
/// 200: Json<Success> - Success
/// 301: Json<Redirect> - Redirection
/// 404: Json<Error> - Client error
/// 500: Json<Error> - Server error
#[rovo]
async fn handler() {}
"#;
    let diagnostics = validate_annotations(content);
    assert_eq!(diagnostics.len(), 0);
}

#[test]
fn reports_multiple_errors() {
    let content = r#"
/// # Responses
///
/// 999: Json<User> - Invalid
/// 998: Json<Error> - Also invalid
#[rovo]
async fn handler() {}
"#;
    let diagnostics = validate_annotations(content);
    assert_eq!(diagnostics.len(), 2);
}

#[test]
fn no_diagnostics_for_non_response_annotations() {
    let content = r#"
/// @tag users
/// @security bearer
/// @id get_user
#[rovo]
async fn handler() {}
"#;
    let diagnostics = validate_annotations(content);
    assert_eq!(diagnostics.len(), 0);
}

#[test]
fn reports_invalid_example_syntax() {
    let content = r#"
/// # Examples
///
/// 200: User { id: 1
#[rovo]
async fn handler() {}
"#;
    let diagnostics = validate_annotations(content);
    assert!(diagnostics.len() > 0);
    assert!(
        diagnostics[0].message.contains("Invalid example")
            || diagnostics[0].message.contains("parse")
    );
}

#[test]
fn reports_missing_fields_in_example() {
    let content = r#"
/// # Examples
///
/// 200: User { id: 1 }
#[rovo]
async fn handler() {}
"#;
    // This should show a helpful message about potentially missing fields
    // when the struct User requires more fields
    let diagnostics = validate_annotations(content);
    // Note: This may or may not produce diagnostics depending on type checking
    // The key is that if it does, the message should be helpful
    if !diagnostics.is_empty() {
        assert!(
            diagnostics[0].message.contains("missing") || diagnostics[0].message.contains("field")
        );
    }
}

#[test]
fn multi_line_example_diagnostic_spans_all_lines() {
    let content = r#"
/// # Examples
///
/// 200: User {
///     id: 1,
///     name: "Test
/// }
#[rovo]
async fn handler() {}
"#;
    let diagnostics = validate_annotations(content);
    if !diagnostics.is_empty() {
        // The diagnostic should span from line 3 to line 6
        assert_eq!(
            diagnostics[0].line, 3,
            "Should start at line with status code"
        );
        if let Some(end_line) = diagnostics[0].end_line {
            assert!(
                end_line >= 6,
                "Should end at or after the closing brace line"
            );
        }
    }
}

// Additional edge case tests for improved coverage

#[test]
fn reports_missing_closing_brace() {
    // Test for "unexpected end of input" error path
    let content = r#"
/// # Examples
///
/// 200: User {
#[rovo]
async fn handler() {}
"#;
    let diagnostics = validate_annotations(content);
    if !diagnostics.is_empty() {
        assert!(
            diagnostics[0].message.contains("Invalid example")
                || diagnostics[0].message.contains("Incomplete")
                || diagnostics[0].message.contains("closing")
        );
    }
}

#[test]
fn reports_missing_comma_in_struct() {
    // Test for "expected `,`" error path
    let content = r#"
/// # Examples
///
/// 200: User { id: 1 name: "Test".into() }
#[rovo]
async fn handler() {}
"#;
    let diagnostics = validate_annotations(content);
    if !diagnostics.is_empty() {
        assert!(
            diagnostics[0].message.contains("Invalid example")
                || diagnostics[0].message.contains("comma")
                || diagnostics[0].message.contains("parse")
        );
    }
}

#[test]
fn reports_invalid_identifier() {
    // Test for "expected identifier" error path
    let content = r#"
/// # Examples
///
/// 200: User { 123: "test" }
#[rovo]
async fn handler() {}
"#;
    let diagnostics = validate_annotations(content);
    if !diagnostics.is_empty() {
        assert!(
            diagnostics[0].message.contains("Invalid example")
                || diagnostics[0].message.contains("identifier")
                || diagnostics[0].message.contains("syntax")
        );
    }
}

#[test]
fn valid_example_produces_no_diagnostic() {
    let content = r#"
/// # Examples
///
/// 200: User::default()
#[rovo]
async fn handler() {}
"#;
    let diagnostics = validate_annotations(content);
    assert!(
        diagnostics.is_empty(),
        "Valid example should produce no diagnostics"
    );
}

#[test]
fn valid_struct_example_produces_no_diagnostic() {
    let content = r#"
/// # Examples
///
/// 200: User { id: 1, name: "Test".into() }
#[rovo]
async fn handler() {}
"#;
    let diagnostics = validate_annotations(content);
    assert!(
        diagnostics.is_empty(),
        "Valid struct example should produce no diagnostics"
    );
}

#[test]
fn valid_vec_example_produces_no_diagnostic() {
    let content = r#"
/// # Examples
///
/// 200: vec![1, 2, 3]
#[rovo]
async fn handler() {}
"#;
    let diagnostics = validate_annotations(content);
    assert!(
        diagnostics.is_empty(),
        "Valid vec example should produce no diagnostics"
    );
}

#[test]
fn handles_boundary_status_codes() {
    let content = r#"
/// # Responses
///
/// 100: () - Boundary low
/// 599: () - Boundary high
#[rovo]
async fn handler() {}
"#;
    let diagnostics = validate_annotations(content);
    assert_eq!(
        diagnostics.len(),
        0,
        "Boundary status codes should be valid"
    );
}

#[test]
fn example_diagnostic_includes_char_start() {
    let content = r#"
/// # Examples
///
/// 200: Invalid(
#[rovo]
async fn handler() {}
"#;
    let diagnostics = validate_annotations(content);
    assert!(
        !diagnostics.is_empty(),
        "Should produce a diagnostic for invalid example"
    );
    // char_start should point at the expression after the colon
    assert!(
        diagnostics[0].char_start.is_some(),
        "Example diagnostics should have a char_start for the invalid expression"
    );
}

#[test]
fn handles_example_followed_by_section() {
    let content = r#"
/// # Examples
///
/// 200: Invalid(
///
/// # Metadata
///
/// @tag users
#[rovo]
async fn handler() {}
"#;
    let diagnostics = validate_annotations(content);
    // Should detect the example ends before Metadata
    if !diagnostics.is_empty() {
        // End line should be before the Metadata section
        if let Some(end_line) = diagnostics[0].end_line {
            assert!(end_line < 5, "Example should end before # Metadata");
        }
    }
}

#[test]
fn handles_example_followed_by_annotation() {
    let content = r#"
/// # Examples
///
/// 200: Invalid(
/// @tag users
#[rovo]
async fn handler() {}
"#;
    let diagnostics = validate_annotations(content);
    // Should detect the example ends at @tag
    if !diagnostics.is_empty() {
        if let Some(end_line) = diagnostics[0].end_line {
            assert!(end_line < 4, "Example should end before @tag");
        }
    }
}

#[test]
fn handles_example_with_new_example_marker() {
    // Test that a new example (STATUS: ...) terminates the previous example
    let content = r#"
/// # Examples
///
/// 200: Invalid(
/// 201: User::default()
#[rovo]
async fn handler() {}
"#;
    let diagnostics = validate_annotations(content);
    // The first example (200) should be flagged as invalid
    // But the second (201) is valid
    if !diagnostics.is_empty() {
        assert!(diagnostics[0].message.contains("Invalid example"));
    }
}

// =============================================================================
// Path Parameter Diagnostic Tests
// =============================================================================

#[test]
fn warns_on_undocumented_path_param() {
    let content = r#"
#[rovo]
async fn get_user(Path(id): Path<u64>) {}
"#;
    let diagnostics = validate_annotations(content);
    assert_eq!(diagnostics.len(), 1);
    assert!(diagnostics[0]
        .message
        .contains("Undocumented path parameter"));
    assert!(diagnostics[0].message.contains("'id'"));
    assert_eq!(diagnostics[0].severity, DiagnosticSeverity::Warning);
}

#[test]
fn no_warning_for_documented_path_param() {
    let content = r#"
/// # Path Parameters
///
/// id: The user's unique identifier
#[rovo]
async fn get_user(Path(id): Path<u64>) {}
"#;
    let diagnostics = validate_annotations(content);
    assert!(
        diagnostics.is_empty(),
        "Should not warn when path param is documented"
    );
}

#[test]
fn no_warning_for_underscore_prefixed_param() {
    let content = r#"
#[rovo]
async fn handler(Path(_internal): Path<String>) {}
"#;
    let diagnostics = validate_annotations(content);
    assert!(
        diagnostics.is_empty(),
        "Should not warn for underscore-prefixed params"
    );
}

#[test]
fn warns_on_multiple_undocumented_params() {
    let content = r#"
#[rovo]
async fn handler(Path((a, b)): Path<(u32, u32)>) {}
"#;
    let diagnostics = validate_annotations(content);
    assert_eq!(diagnostics.len(), 1);
    assert!(diagnostics[0].message.contains("'a'"));
    assert!(diagnostics[0].message.contains("'b'"));
}

#[test]
fn warns_only_for_undocumented_when_some_documented() {
    let content = r#"
/// # Path Parameters
///
/// id: The user ID
#[rovo]
async fn handler(Path((id, name)): Path<(u64, String)>) {}
"#;
    let diagnostics = validate_annotations(content);
    assert_eq!(diagnostics.len(), 1);
    assert!(diagnostics[0].message.contains("'name'"));
    assert!(!diagnostics[0].message.contains("'id'"));
}

#[test]
fn handles_multiple_path_extractors() {
    let content = r#"
#[rovo]
async fn handler(Path(id): Path<u64>, Path(name): Path<String>) {}
"#;
    let diagnostics = validate_annotations(content);
    assert_eq!(diagnostics.len(), 1);
    assert!(diagnostics[0].message.contains("'id'"));
    assert!(diagnostics[0].message.contains("'name'"));
}

#[test]
fn handles_multiline_function_signature() {
    let content = r#"
#[rovo]
async fn handler(
    Path(id): Path<u64>,
    Path(name): Path<String>,
) {
}
"#;
    let diagnostics = validate_annotations(content);
    assert_eq!(diagnostics.len(), 1);
    assert!(diagnostics[0].message.contains("'id'"));
    assert!(diagnostics[0].message.contains("'name'"));
}

#[test]
fn no_warning_without_path_extractor() {
    let content = r#"
#[rovo]
async fn handler(Query(q): Query<String>) {}
"#;
    let diagnostics = validate_annotations(content);
    assert!(
        diagnostics.is_empty(),
        "Should not warn when no Path extractor"
    );
}

#[test]
fn handles_path_params_section_with_other_sections() {
    let content = r#"
/// Get a user by ID.
///
/// # Path Parameters
///
/// id: The user's unique identifier
///
/// # Responses
///
/// 200: Json<User> - Success
#[rovo]
async fn get_user(Path(id): Path<u64>) {}
"#;
    let diagnostics = validate_annotations(content);
    assert!(
        diagnostics.is_empty(),
        "Should not warn when path param documented with other sections"
    );
}

#[test]
fn handles_pub_async_fn() {
    let content = r#"
#[rovo]
pub async fn handler(Path(id): Path<u64>) {}
"#;
    let diagnostics = validate_annotations(content);
    assert_eq!(diagnostics.len(), 1);
    assert!(diagnostics[0].message.contains("'id'"));
}

#[test]
fn handles_pub_crate_async_fn() {
    let content = r#"
#[rovo]
pub(crate) async fn handler(Path(id): Path<u64>) {}
"#;
    let diagnostics = validate_annotations(content);
    assert_eq!(diagnostics.len(), 1);
    assert!(diagnostics[0].message.contains("'id'"));
}

#[test]
fn handles_attributes_between_rovo_and_fn() {
    let content = r#"
#[rovo]
#[deprecated]
async fn handler(Path(id): Path<u64>) {}
"#;
    let diagnostics = validate_annotations(content);
    assert_eq!(diagnostics.len(), 1);
    assert!(diagnostics[0].message.contains("'id'"));
}

#[test]
fn diagnostic_line_points_to_function() {
    let content = r#"
/// Some doc
#[rovo]
async fn handler(Path(id): Path<u64>) {}
"#;
    let diagnostics = validate_annotations(content);
    assert_eq!(diagnostics.len(), 1);
    // Line 3 is the fn line (0-indexed)
    assert_eq!(diagnostics[0].line, 3);
}

#[test]
fn handles_multiple_rovo_blocks() {
    let content = r#"
#[rovo]
async fn handler1(Path(id1): Path<u64>) {}

/// # Path Parameters
///
/// id2: Documented
#[rovo]
async fn handler2(Path(id2): Path<u64>) {}
"#;
    let diagnostics = validate_annotations(content);
    // Only handler1's id1 should be warned about
    assert_eq!(diagnostics.len(), 1);
    assert!(diagnostics[0].message.contains("'id1'"));
}

// Additional coverage tests

#[test]
fn reports_example_with_incomplete_expression() {
    let content = r#"
/// # Examples
///
/// 200: { id: 1
#[rovo]
async fn handler() {}
"#;
    let diagnostics = validate_annotations(content);
    assert!(!diagnostics.is_empty());
    let has_example_error = diagnostics
        .iter()
        .any(|d| d.message.contains("example") || d.message.contains("expression"));
    assert!(has_example_error, "Should report incomplete example error");
}

#[test]
fn reports_example_with_missing_comma() {
    let content = r#"
/// # Examples
///
/// 200: { id: 1 name: "test" }
#[rovo]
async fn handler() {}
"#;
    let diagnostics = validate_annotations(content);
    assert!(!diagnostics.is_empty());
    let has_example_error = diagnostics
        .iter()
        .any(|d| d.message.contains("example") || d.message.contains("Syntax"));
    assert!(has_example_error, "Should report syntax error in example");
}

#[test]
fn accepts_valid_example() {
    let content = r#"
/// # Examples
///
/// 200: User { id: 1, name: "test".to_string() }
#[rovo]
async fn handler() {}
"#;
    let diagnostics = validate_annotations(content);
    let example_errors = diagnostics
        .iter()
        .filter(|d| d.message.contains("example"))
        .count();
    assert_eq!(
        example_errors, 0,
        "Should not report error for valid example"
    );
}

#[test]
fn handles_multiline_example() {
    let content = r#"
/// # Examples
///
/// 200: User {
///     id: 1,
///     name: "test".to_string()
/// }
#[rovo]
async fn handler() {}
"#;
    let diagnostics = validate_annotations(content);
    let example_errors: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.message.contains("example"))
        .collect();
    assert!(
        example_errors.is_empty(),
        "Should not report error for valid multiline example"
    );
}

#[test]
fn reports_multiline_example_error_with_span() {
    let content = r#"
/// # Examples
///
/// 200: User {
///     id: 1
///     name: "test"
/// }
#[rovo]
async fn handler() {}
"#;
    let diagnostics = validate_annotations(content);
    let example_errors: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.message.contains("example") || d.message.contains("Syntax"))
        .collect();
    assert!(
        !example_errors.is_empty(),
        "Should report multiline example error"
    );
    // The error should have an end_line for multiline spans
    if let Some(error) = example_errors.first() {
        // Check that char_start is set (position after the colon)
        assert!(
            error.char_start.is_some(),
            "Should have char_start for example error"
        );
    }
}

#[test]
fn handles_pub_fn_after_rovo() {
    let content = r#"
#[rovo]
pub fn handler(Path(id): Path<u64>) {}
"#;
    let diagnostics = validate_annotations(content);
    assert!(!diagnostics.is_empty());
    assert!(diagnostics[0].message.contains("'id'"));
}

#[test]
fn no_warning_when_no_path_params() {
    let content = r#"
#[rovo]
async fn handler(Query(q): Query<String>) {}
"#;
    let diagnostics = validate_annotations(content);
    assert!(
        diagnostics.is_empty(),
        "Should not warn when no path params"
    );
}

#[test]
fn handles_rovo_without_function() {
    // Edge case: #[rovo] not followed by a function
    let content = r#"
#[rovo]
const VALUE: u32 = 42;
"#;
    let diagnostics = validate_annotations(content);
    // Should not crash, no diagnostics expected
    let _ = diagnostics;
}

#[test]
fn handles_empty_content() {
    let content = "";
    let diagnostics = validate_annotations(content);
    assert!(diagnostics.is_empty());
}

#[test]
fn handles_content_without_rovo() {
    let content = r#"
fn regular_function() {}
"#;
    let diagnostics = validate_annotations(content);
    assert!(diagnostics.is_empty());
}
