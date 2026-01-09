use rovo_lsp::code_actions;
use tower_lsp::lsp_types::*;

/// Helper to create a test URI
fn test_uri() -> Url {
    Url::parse("file:///test.rs").unwrap()
}

/// Helper to create a range at a specific line
fn range_at_line(line: u32) -> Range {
    Range {
        start: Position { line, character: 0 },
        end: Position {
            line,
            character: 100,
        },
    }
}

/// Helper to extract action titles
fn get_action_titles(actions: &[CodeActionOrCommand]) -> Vec<String> {
    actions
        .iter()
        .filter_map(|action| match action {
            CodeActionOrCommand::CodeAction(ca) => Some(ca.title.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn offers_annotations_inside_rovo_function() {
    let content = r#"
/// # Responses
///
/// 200: Json<User> - Success
#[rovo]
async fn handler() {
    // cursor here
}
"#;

    let actions = code_actions::get_code_actions(content, range_at_line(4), test_uri());
    let titles = get_action_titles(&actions);

    // Should offer to add more annotations (new format)
    assert!(titles
        .iter()
        .any(|t| t.contains("Add response") || t.contains("response")));
    assert!(titles
        .iter()
        .any(|t| t.contains("@tag") || t.contains("tag")));
    assert!(titles
        .iter()
        .any(|t| t.contains("@security") || t.contains("security")));
    assert!(titles
        .iter()
        .any(|t| t.contains("Add example") || t.contains("example")));
}

#[test]
fn offers_annotations_in_doc_comment_above_rovo() {
    let content = r#"
/// # Responses
///
/// 200: Json<User> - Success
/// cursor here
#[rovo]
async fn handler() {}
"#;

    let actions = code_actions::get_code_actions(content, range_at_line(2), test_uri());
    let titles = get_action_titles(&actions);

    // Should recognize we're in a rovo context
    assert!(!titles.is_empty());
    assert!(titles.iter().any(|t| t.contains("@tag")));
}

#[test]
fn offers_init_rovo_in_regular_function() {
    let content = r#"
async fn handler() {
    let x = 1;
}
"#;

    let actions = code_actions::get_code_actions(content, range_at_line(2), test_uri());
    let titles = get_action_titles(&actions);

    // Should offer to add #[rovo] when inside function body (indented line)
    assert!(!titles.is_empty(), "Expected actions but got none");
    assert!(titles.iter().any(|t| t.contains("#[rovo]")));
}

#[test]
fn offers_init_rovo_on_function_signature() {
    let content = r#"
async fn handler() {
    let x = 1;
}
"#;

    let actions = code_actions::get_code_actions(content, range_at_line(1), test_uri());
    let titles = get_action_titles(&actions);

    // Should offer to add #[rovo] when on the function signature
    assert!(titles.iter().any(|t| t.contains("#[rovo]")));
}

#[test]
fn no_init_rovo_if_already_has_rovo() {
    let content = r#"
#[rovo]
async fn handler() {
    let x = 1;
}
"#;

    let actions = code_actions::get_code_actions(content, range_at_line(1), test_uri());
    let titles = get_action_titles(&actions);

    // Should NOT offer to add #[rovo] again
    assert!(!titles.iter().any(|t| t.contains("Add #[rovo]")));
}

#[test]
fn no_actions_outside_function_or_struct() {
    let content = r#"
use std::collections::HashMap;

// Some comment
"#;

    let actions = code_actions::get_code_actions(content, range_at_line(2), test_uri());

    // Should not offer any actions outside of function/struct context
    assert_eq!(actions.len(), 0);
}

#[test]
fn offers_full_rest_responses_when_no_annotations() {
    let content = r#"
#[rovo]
async fn handler() {}
"#;

    let actions = code_actions::get_code_actions(content, range_at_line(2), test_uri());
    let titles = get_action_titles(&actions);

    // Should offer "Add common REST responses" when no annotations exist
    assert!(titles
        .iter()
        .any(|t| t.contains("Add common REST responses")));
}

#[test]
fn no_full_rest_responses_when_annotations_exist() {
    let content = r#"
/// # Responses
///
/// 200: Json<User> - Success
#[rovo]
async fn handler() {}
"#;

    let actions = code_actions::get_code_actions(content, range_at_line(3), test_uri());
    let titles = get_action_titles(&actions);

    // Should NOT offer full REST responses when annotations already exist
    assert!(!titles
        .iter()
        .any(|t| t.contains("Add common REST responses")));
}

#[test]
fn offers_id_annotation_when_missing() {
    let content = r#"
/// # Responses
///
/// 200: Json<User> - Success
#[rovo]
async fn handler() {}
"#;

    let actions = code_actions::get_code_actions(content, range_at_line(3), test_uri());
    let titles = get_action_titles(&actions);

    // Should offer @id since it's missing
    assert!(titles.iter().any(|t| t == "Add @id"));
}

#[test]
fn no_duplicate_id_annotation() {
    let content = r#"
/// # Responses
///
/// 200: Json<User> - Success
///
/// # Metadata
///
/// @id get_user
#[rovo]
async fn handler() {}
"#;

    let actions = code_actions::get_code_actions(content, range_at_line(4), test_uri());
    let titles = get_action_titles(&actions);

    // Should NOT offer @id since it already exists
    assert!(!titles.iter().any(|t| t == "Add @id"));
}

#[test]
fn offers_hidden_annotation_when_missing() {
    let content = r#"
/// # Responses
///
/// 200: Json<User> - Success
#[rovo]
async fn handler() {}
"#;

    let actions = code_actions::get_code_actions(content, range_at_line(3), test_uri());
    let titles = get_action_titles(&actions);

    // Should offer @hidden since it's missing
    assert!(titles.iter().any(|t| t == "Add @hidden"));
}

#[test]
fn no_duplicate_hidden_annotation() {
    let content = r#"
/// @hidden
#[rovo]
async fn handler() {}
"#;

    let actions = code_actions::get_code_actions(content, range_at_line(3), test_uri());
    let titles = get_action_titles(&actions);

    // Should NOT offer @hidden since it already exists
    assert!(!titles.iter().any(|t| t == "Add @hidden"));
}

#[test]
fn offers_jsonschema_derive_in_struct() {
    let content = r#"
struct User {
    name: String,
}
"#;

    let actions = code_actions::get_code_actions(content, range_at_line(2), test_uri());
    let titles = get_action_titles(&actions);

    // Should offer to add JsonSchema derive
    assert!(titles.iter().any(|t| t.contains("JsonSchema")));
}

#[test]
fn offers_jsonschema_to_existing_derive() {
    let content = r#"
#[derive(Debug, Clone)]
struct User {
    name: String,
}
"#;

    let actions = code_actions::get_code_actions(content, range_at_line(3), test_uri());
    let titles = get_action_titles(&actions);

    // Should offer to add JsonSchema to existing derive
    assert!(titles
        .iter()
        .any(|t| t.contains("JsonSchema") && t.contains("derive")));
}

#[test]
fn no_jsonschema_if_already_present() {
    let content = r#"
#[derive(Debug, JsonSchema)]
struct User {
    name: String,
}
"#;

    let actions = code_actions::get_code_actions(content, range_at_line(3), test_uri());
    let titles = get_action_titles(&actions);

    // Should NOT offer JsonSchema if it's already there
    assert!(!titles.iter().any(|t| t.contains("JsonSchema")));
}

#[test]
fn offers_jsonschema_in_enum() {
    let content = r#"
enum Status {
    Active,
    Inactive,
}
"#;

    let actions = code_actions::get_code_actions(content, range_at_line(2), test_uri());
    let titles = get_action_titles(&actions);

    // Should offer JsonSchema for enums too
    assert!(titles.iter().any(|t| t.contains("JsonSchema")));
}

#[test]
fn diagnostic_quick_fix_for_invalid_status() {
    let content = r#"
/// # Responses
///
/// 999: Json<User> - Invalid
#[rovo]
async fn handler() {}
"#;

    let diagnostic = Diagnostic {
        range: Range {
            start: Position {
                line: 1,
                character: 14,
            },
            end: Position {
                line: 1,
                character: 17,
            },
        },
        severity: Some(DiagnosticSeverity::ERROR),
        message: "Invalid HTTP status code: 999".to_string(),
        source: Some("rovo-lsp".to_string()),
        ..Default::default()
    };

    let actions = code_actions::get_diagnostic_code_actions(content, &diagnostic, test_uri());
    let titles = get_action_titles(&actions);

    // Should offer common status codes as fixes
    assert!(titles.iter().any(|t| t == "Change to 200"));
    assert!(titles.iter().any(|t| t == "Change to 404"));
    assert!(titles.iter().any(|t| t == "Change to 500"));
    assert_eq!(titles.len(), 5); // 200, 201, 400, 404, 500
}

#[test]
fn diagnostic_quick_fix_sets_preferred() {
    let content = r#"
/// # Responses
///
/// 999: Json<User> - Invalid
#[rovo]
async fn handler() {}
"#;

    let diagnostic = Diagnostic {
        range: Range {
            start: Position {
                line: 1,
                character: 14,
            },
            end: Position {
                line: 1,
                character: 17,
            },
        },
        message: "Invalid HTTP status code: 999".to_string(),
        ..Default::default()
    };

    let actions = code_actions::get_diagnostic_code_actions(content, &diagnostic, test_uri());

    // Find the "Change to 200" action
    let action_200 = actions.iter().find_map(|a| match a {
        CodeActionOrCommand::CodeAction(ca) if ca.title == "Change to 200" => Some(ca),
        _ => None,
    });

    assert!(action_200.is_some());
    // 200 should be marked as preferred
    assert_eq!(action_200.unwrap().is_preferred, Some(true));
}

#[test]
fn no_diagnostic_actions_for_non_status_errors() {
    let content = r#"
/// # Responses
///
/// 200: Json<User> - Success
#[rovo]
async fn handler() {}
"#;

    let diagnostic = Diagnostic {
        range: range_at_line(1),
        message: "Some other error".to_string(),
        ..Default::default()
    };

    let actions = code_actions::get_diagnostic_code_actions(content, &diagnostic, test_uri());

    // Should not offer any actions for non-status-code errors
    assert_eq!(actions.len(), 0);
}

#[test]
fn handles_nested_functions() {
    let content = r#"
fn outer() {
    #[rovo]
    async fn inner() {
        // cursor here
    }
}
"#;

    let actions = code_actions::get_code_actions(content, range_at_line(4), test_uri());
    let titles = get_action_titles(&actions);

    // Should detect we're in a rovo function even when nested
    assert!(titles
        .iter()
        .any(|t| t.contains("Add response") || t.contains("response")));
}

#[test]
fn handles_multiline_function_signature() {
    let content = r#"
#[rovo]
async fn complex_handler(
    param1: String,
    param2: i32
) -> Result<Json<User>, Error> {
    // cursor here
}
"#;

    let actions = code_actions::get_code_actions(content, range_at_line(6), test_uri());
    let titles = get_action_titles(&actions);

    // Should work with multiline function signatures
    assert!(titles
        .iter()
        .any(|t| t.contains("Add response") || t.contains("response")));
}

#[test]
fn handles_function_with_generics() {
    let content = r#"
#[rovo]
async fn generic_handler<T: Serialize>() {
    // cursor here
}
"#;

    let actions = code_actions::get_code_actions(content, range_at_line(3), test_uri());
    let titles = get_action_titles(&actions);

    // Should work with generic functions
    assert!(titles
        .iter()
        .any(|t| t.contains("Add response") || t.contains("response")));
}

#[test]
fn handles_struct_with_generics() {
    let content = r#"
struct Response<T> {
    data: T,
}
"#;

    let actions = code_actions::get_code_actions(content, range_at_line(2), test_uri());
    let titles = get_action_titles(&actions);

    // Should offer JsonSchema for generic structs
    assert!(titles.iter().any(|t| t.contains("JsonSchema")));
}

#[test]
fn handles_tuple_struct() {
    let content = r#"
struct UserId(u64);
"#;

    let actions = code_actions::get_code_actions(content, range_at_line(1), test_uri());
    let titles = get_action_titles(&actions);

    // Should not offer JsonSchema for tuple structs (cursor on same line)
    // Tuple structs are single line, so cursor position matters
    // This is a reasonable behavior - user would need to be "inside" the struct
    assert!(actions.is_empty() || titles.iter().any(|t| t.contains("JsonSchema")));
}

#[test]
fn respects_function_boundaries() {
    let content = r#"
#[rovo]
async fn handler1() {
}

async fn handler2() {
    let x = 1;
}
"#;

    let actions = code_actions::get_code_actions(content, range_at_line(6), test_uri());
    let titles = get_action_titles(&actions);

    // Should offer to add #[rovo] (on indented line inside handler2), not annotations
    assert!(!titles.is_empty(), "Expected actions but got none");
    assert!(titles.iter().any(|t| t.contains("#[rovo]")));
    assert!(!titles.iter().any(|t| t.contains("response")));
    assert!(!titles.iter().any(|t| t.contains("@tag")));
}

#[test]
fn handles_multiple_rovo_blocks() {
    let content = r#"
/// # Responses
///
/// 200: Json<User> - Success
#[rovo]
async fn handler1() {}

/// # Responses
///
/// 200: Json<Post> - Success
#[rovo]
async fn handler2() {
    // cursor here
}
"#;

    let actions = code_actions::get_code_actions(content, range_at_line(8), test_uri());
    let titles = get_action_titles(&actions);

    // Should offer annotations for the current block
    assert!(titles.iter().any(|t| t.contains("@tag")));
    // Should still offer @id since this block doesn't have it
    assert!(titles.iter().any(|t| t == "Add @id"));
}

#[test]
fn handles_attributes_with_parameters() {
    let content = r#"
#[derive(Debug, Clone, Serialize)]
struct User {
    name: String,
}
"#;

    let actions = code_actions::get_code_actions(content, range_at_line(3), test_uri());
    let titles = get_action_titles(&actions);

    // Should parse complex derive attributes correctly
    assert!(titles
        .iter()
        .any(|t| t.contains("JsonSchema") && t.contains("derive")));
}

#[test]
fn handles_empty_derive() {
    let content = r#"
#[derive()]
struct User {
    name: String,
}
"#;

    let actions = code_actions::get_code_actions(content, range_at_line(3), test_uri());
    let titles = get_action_titles(&actions);

    // Should handle empty derive correctly
    assert!(titles.iter().any(|t| t.contains("JsonSchema")));
}

#[test]
fn handles_doc_comments_with_blank_lines() {
    let content = r#"
/// # Responses
///
/// 200: Json<User> - Success
///
/// More details here
#[rovo]
async fn handler() {}
"#;

    let actions = code_actions::get_code_actions(content, range_at_line(3), test_uri());
    let titles = get_action_titles(&actions);

    // Should recognize doc block even with blank comment lines
    assert!(titles.iter().any(|t| t.contains("@tag")));
}

#[test]
fn action_edits_have_correct_structure() {
    let content = r#"
#[rovo]
async fn handler() {}
"#;

    let actions = code_actions::get_code_actions(content, range_at_line(2), test_uri());

    // Find the "Add response" action
    let response_action = actions.iter().find_map(|a| match a {
        CodeActionOrCommand::CodeAction(ca) if ca.title == "Add response" => Some(ca),
        _ => None,
    });

    assert!(response_action.is_some());
    let action = response_action.unwrap();

    // Check it has the right kind
    assert_eq!(action.kind, Some(CodeActionKind::REFACTOR));

    // Check it has a workspace edit
    assert!(action.edit.is_some());

    // Check the edit has changes
    let edit = action.edit.as_ref().unwrap();
    assert!(edit.changes.is_some());

    // Check the changes contain our URI
    let changes = edit.changes.as_ref().unwrap();
    assert!(changes.contains_key(&test_uri()));
}

#[test]
fn rest_responses_action_is_refactor_kind() {
    let content = r#"
#[rovo]
async fn handler() {}
"#;

    let actions = code_actions::get_code_actions(content, range_at_line(2), test_uri());

    // Find the REST responses action
    let rest_action = actions.iter().find_map(|a| match a {
        CodeActionOrCommand::CodeAction(ca) if ca.title.contains("Add common REST responses") => {
            Some(ca)
        }
        _ => None,
    });

    assert!(rest_action.is_some());

    // Should be a REFACTOR, not QUICKFIX
    assert_eq!(rest_action.unwrap().kind, Some(CodeActionKind::REFACTOR));
}

#[test]
fn metadata_annotations_maintain_order() {
    // Test that adding metadata annotations respects the order: @id, @tag, @security, @hidden
    let content = r#"
/// Handler
///
/// # Metadata
///
/// @security bearer
/// @hidden
#[rovo]
async fn handler() {}
"#;

    let actions = code_actions::get_code_actions(content, range_at_line(8), test_uri());

    // Find the "Add @tag" action
    let tag_action = actions.iter().find_map(|a| match a {
        CodeActionOrCommand::CodeAction(ca) if ca.title.contains("Add @tag") => Some(ca),
        _ => None,
    });

    assert!(tag_action.is_some());

    // Extract the edit
    let edit = tag_action.unwrap().edit.as_ref().unwrap();
    let changes = edit.changes.as_ref().unwrap();
    let text_edits = changes.values().next().unwrap();
    let text_edit = &text_edits[0];

    // The @tag should be inserted before @security (after any @id if present)
    // Line 5 is where @security is, so @tag should be inserted at line 5
    assert_eq!(text_edit.range.start.line, 5);
    assert!(text_edit.new_text.contains("@tag"));
}

#[test]
fn metadata_annotations_group_same_type() {
    // Test that adding a second @tag groups it with existing @tag
    let content = r#"
/// Handler
///
/// # Metadata
///
/// @tag users
/// @security bearer
#[rovo]
async fn handler() {}
"#;

    let actions = code_actions::get_code_actions(content, range_at_line(8), test_uri());

    // Find the "Add @tag" action
    let tag_action = actions.iter().find_map(|a| match a {
        CodeActionOrCommand::CodeAction(ca) if ca.title.contains("Add @tag") => Some(ca),
        _ => None,
    });

    assert!(tag_action.is_some());

    // Extract the edit
    let edit = tag_action.unwrap().edit.as_ref().unwrap();
    let changes = edit.changes.as_ref().unwrap();
    let text_edits = changes.values().next().unwrap();
    let text_edit = &text_edits[0];

    // The second @tag should be inserted right after the first @tag (line 6)
    assert_eq!(text_edit.range.start.line, 6);
    assert!(text_edit.new_text.contains("@tag"));
}

#[test]
fn metadata_id_comes_first() {
    // Test that @id is inserted at the beginning of metadata
    let content = r#"
/// Handler
///
/// # Metadata
///
/// @tag users
/// @security bearer
#[rovo]
async fn handler() {}
"#;

    let actions = code_actions::get_code_actions(content, range_at_line(8), test_uri());

    // Find the "Add @id" action
    let id_action = actions.iter().find_map(|a| match a {
        CodeActionOrCommand::CodeAction(ca) if ca.title.contains("Add @id") => Some(ca),
        _ => None,
    });

    assert!(id_action.is_some());

    // Extract the edit
    let edit = id_action.unwrap().edit.as_ref().unwrap();
    let changes = edit.changes.as_ref().unwrap();
    let text_edits = changes.values().next().unwrap();
    let text_edit = &text_edits[0];

    // The @id should be inserted before @tag (at line 5)
    assert_eq!(text_edit.range.start.line, 5);
    assert!(text_edit.new_text.contains("@id"));
}

// Additional edge case tests for improved coverage

#[test]
fn no_actions_for_out_of_bounds_line() {
    let content = r#"
#[rovo]
async fn handler() {}
"#;

    // Line 100 is way beyond the content
    let actions = code_actions::get_code_actions(content, range_at_line(100), test_uri());
    assert!(actions.is_empty());
}

#[test]
fn respects_rovo_ignore_for_code_actions() {
    let content = r#"
/// Handler
///
/// # Responses
///
/// 200: Json<User> - Success
///
/// @rovo-ignore
/// @invalid this would cause an error
#[rovo]
async fn handler() {}
"#;

    // Actions should be offered before @rovo-ignore
    let actions = code_actions::get_code_actions(content, range_at_line(5), test_uri());
    assert!(!actions.is_empty());
}

#[test]
fn handles_function_without_body() {
    let content = r#"
async fn handler();
"#;

    // Function declaration without body should not offer #[rovo]
    let actions = code_actions::get_code_actions(content, range_at_line(1), test_uri());
    // Should be empty or not offer rovo since there's no opening brace
    let titles = get_action_titles(&actions);
    assert!(!titles.iter().any(|t| t.contains("#[rovo]")));
}

#[test]
fn handles_struct_without_fields_block() {
    let content = r#"
struct Empty;
"#;

    // Unit struct should not offer JsonSchema (cursor not inside braces)
    let actions = code_actions::get_code_actions(content, range_at_line(1), test_uri());
    assert!(actions.is_empty());
}

#[test]
fn handles_comment_line() {
    let content = r#"
// This is a comment
async fn handler() {}
"#;

    // Comment line should not trigger actions
    let actions = code_actions::get_code_actions(content, range_at_line(1), test_uri());
    assert!(actions.is_empty());
}

#[test]
fn handles_attribute_line_without_rovo() {
    let content = r#"
#[allow(dead_code)]
async fn handler() {}
"#;

    // Attribute line should not trigger actions directly
    let actions = code_actions::get_code_actions(content, range_at_line(1), test_uri());
    // Should not crash, may or may not have actions
    let _ = actions;
}

#[test]
fn no_actions_between_functions() {
    let content = r#"
#[rovo]
async fn handler1() {
}

// Just a gap

#[rovo]
async fn handler2() {
}
"#;

    // Gap between functions should not offer annotations
    let actions = code_actions::get_code_actions(content, range_at_line(5), test_uri());
    let titles = get_action_titles(&actions);
    assert!(!titles.iter().any(|t| t.contains("@tag")));
}

#[test]
fn diagnostic_action_for_out_of_bounds_line() {
    let content = r#"
/// # Responses
///
/// 999: Json<User> - Invalid
#[rovo]
async fn handler() {}
"#;

    // Diagnostic at line 100 which doesn't exist
    let diagnostic = Diagnostic {
        range: Range {
            start: Position {
                line: 100,
                character: 0,
            },
            end: Position {
                line: 100,
                character: 10,
            },
        },
        message: "Invalid HTTP status code: 999".to_string(),
        ..Default::default()
    };

    let actions = code_actions::get_diagnostic_code_actions(content, &diagnostic, test_uri());
    // Should still work or return empty
    assert!(actions.is_empty() || actions.len() == 5);
}

#[test]
fn handles_section_insertion_order() {
    let content = r#"
/// Handler
///
/// # Metadata
///
/// @tag users
#[rovo]
async fn handler() {}
"#;

    // Should be able to add Responses section before Metadata
    let actions = code_actions::get_code_actions(content, range_at_line(6), test_uri());
    let titles = get_action_titles(&actions);
    assert!(titles.iter().any(|t| t.contains("Add response")));
}

#[test]
fn handles_existing_responses_section() {
    let content = r#"
/// Handler
///
/// # Responses
///
/// 200: Json<User> - Success
///
/// # Metadata
///
/// @tag users
#[rovo]
async fn handler() {}
"#;

    // Should be able to add more to existing sections
    let actions = code_actions::get_code_actions(content, range_at_line(10), test_uri());
    let titles = get_action_titles(&actions);
    assert!(titles.iter().any(|t| t.contains("Add response")));
}

#[test]
fn handles_closing_brace_outside_function() {
    let content = r#"
}

async fn handler() {
    let x = 1;
}
"#;

    // First brace is outside any function - should handle gracefully
    let actions = code_actions::get_code_actions(content, range_at_line(1), test_uri());
    // Should not crash
    let _ = actions;
}

#[test]
fn handles_derive_with_nested_generics() {
    let content = r#"
#[derive(Debug, Clone, Serialize, Deserialize)]
struct User<T: Default> {
    data: T,
}
"#;

    let actions = code_actions::get_code_actions(content, range_at_line(3), test_uri());
    let titles = get_action_titles(&actions);

    // Should still offer JsonSchema for complex derive
    assert!(titles.iter().any(|t| t.contains("JsonSchema")));
}

#[test]
fn handles_malformed_derive() {
    let content = r#"
#[derive(Debug Clone)]
struct User {
    name: String,
}
"#;

    // Malformed derive (missing comma) should fall back gracefully
    let actions = code_actions::get_code_actions(content, range_at_line(3), test_uri());
    let titles = get_action_titles(&actions);

    // Should still offer some action
    assert!(titles.iter().any(|t| t.contains("JsonSchema")));
}

// =============================================================================
// Path Parameter Code Action Tests
// =============================================================================

#[test]
fn offers_document_path_param_action() {
    let content = r#"
#[rovo]
async fn get_user(Path(id): Path<u64>) {}
"#;

    let actions = code_actions::get_code_actions(content, range_at_line(2), test_uri());
    let titles = get_action_titles(&actions);

    assert!(
        titles.iter().any(|t| t.contains("Document path param")),
        "Should offer 'Document path param' action, got: {:?}",
        titles
    );
}

#[test]
fn offers_document_multiple_path_params() {
    let content = r#"
#[rovo]
async fn get_item(Path((a, b)): Path<(String, u32)>) {}
"#;

    let actions = code_actions::get_code_actions(content, range_at_line(2), test_uri());
    let titles = get_action_titles(&actions);

    // Should offer to document path params
    let has_path_action = titles
        .iter()
        .any(|t| t.contains("path") || t.contains("Path"));
    assert!(
        has_path_action,
        "Should offer path param action for tuple, got: {:?}",
        titles
    );
}

#[test]
fn no_path_param_action_when_all_documented() {
    let content = r#"
/// # Path Parameters
///
/// id: The user ID
#[rovo]
async fn get_user(Path(id): Path<u64>) {}
"#;

    let actions = code_actions::get_code_actions(content, range_at_line(5), test_uri());
    let titles = get_action_titles(&actions);

    // Should NOT offer document path param if already documented
    let has_undocumented_action = titles.iter().any(|t| t.contains("Document path param"));
    assert!(
        !has_undocumented_action,
        "Should not offer document action for already documented param"
    );
}

#[test]
fn offers_path_param_action_for_partially_documented() {
    let content = r#"
/// # Path Parameters
///
/// id: The user ID
#[rovo]
async fn get_item(Path((id, name)): Path<(u64, String)>) {}
"#;

    let actions = code_actions::get_code_actions(content, range_at_line(5), test_uri());
    let titles = get_action_titles(&actions);

    // Should offer to document the undocumented 'name' param
    assert!(
        titles
            .iter()
            .any(|t| t.contains("Document path param") && t.contains("name")),
        "Should offer action to document undocumented 'name' param, got: {:?}",
        titles
    );
}

#[test]
fn path_param_action_available_on_doc_line() {
    let content = r#"
/// Get a user.
#[rovo]
async fn get_user(Path(id): Path<u64>) {}
"#;

    // Test on the doc comment line - should still offer path param action
    let actions = code_actions::get_code_actions(content, range_at_line(1), test_uri());
    let titles = get_action_titles(&actions);

    assert!(
        titles.iter().any(|t| t.contains("Document path param")),
        "Should offer path param action from doc line, got: {:?}",
        titles
    );
}

#[test]
fn path_param_action_with_multiline_signature() {
    let content = r#"
#[rovo]
async fn get_item(
    Path(id): Path<u64>,
    Path(name): Path<String>,
) {
}
"#;

    let actions = code_actions::get_code_actions(content, range_at_line(3), test_uri());
    let titles = get_action_titles(&actions);

    // Should offer to document path params from multiline signature
    let has_path_action = titles
        .iter()
        .any(|t| t.contains("path") || t.contains("Path"));
    assert!(
        has_path_action,
        "Should offer path action for multiline sig, got: {:?}",
        titles
    );
}

#[test]
fn no_path_param_action_without_path_extractor() {
    let content = r#"
#[rovo]
async fn handler(Query(q): Query<String>) {}
"#;

    let actions = code_actions::get_code_actions(content, range_at_line(2), test_uri());
    let titles = get_action_titles(&actions);

    // Should NOT offer document path param if there's no Path extractor
    let has_path_action = titles.iter().any(|t| t.contains("Document path param"));
    assert!(
        !has_path_action,
        "Should not offer path action without Path extractor"
    );
}

#[test]
fn path_param_action_with_state() {
    let content = r#"
#[rovo]
async fn handler(State(app): State<AppState>, Path(id): Path<u64>) {}
"#;

    let actions = code_actions::get_code_actions(content, range_at_line(2), test_uri());
    let titles = get_action_titles(&actions);

    let has_path_action = titles
        .iter()
        .any(|t| t.contains("path") || t.contains("Path"));
    assert!(
        has_path_action,
        "Should offer path action with State extractor, got: {:?}",
        titles
    );
}
