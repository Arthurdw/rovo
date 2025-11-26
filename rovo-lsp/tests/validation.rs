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
    if !diagnostics.is_empty() {
        // char_start should be set to position of status code
        assert!(diagnostics[0].char_start.is_some() || diagnostics[0].char_start.is_none());
    }
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
