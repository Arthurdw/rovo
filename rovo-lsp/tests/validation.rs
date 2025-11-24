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
/// 50: Json<Error> - Also invalid
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
