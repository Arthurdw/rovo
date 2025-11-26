use rovo_lsp::parser::{parse_annotations, AnnotationKind};

#[test]
fn detects_response_annotation() {
    let content = r#"
/// # Responses
///
/// 200: Json<User> - Success
#[rovo]
async fn handler() {}
"#;
    let annotations = parse_annotations(content);
    // Returns 2: ResponsesSection header + Response entry
    assert_eq!(annotations.len(), 2);
    // First annotation is the section header
    assert_eq!(annotations[0].kind, AnnotationKind::ResponsesSection);
    // Second annotation is the response entry
    assert_eq!(annotations[1].kind, AnnotationKind::Response);
    assert_eq!(annotations[1].status, Some(200));
    assert_eq!(annotations[1].response_type, Some("Json<User>".to_string()));
    assert_eq!(annotations[1].description, Some("Success".to_string()));
}

#[test]
fn detects_tag_annotation() {
    let content = r#"
/// @tag users
#[rovo]
async fn handler() {}
"#;
    let annotations = parse_annotations(content);
    assert_eq!(annotations.len(), 1);
    assert_eq!(annotations[0].kind, AnnotationKind::Tag);
    assert_eq!(annotations[0].tag_name, Some("users".to_string()));
}

#[test]
fn detects_multiple_annotations() {
    let content = r#"
/// @tag users
///
/// # Responses
///
/// 200: Json<User> - Success
/// 404: Json<Error> - Not found
#[rovo]
async fn handler() {}
"#;
    let annotations = parse_annotations(content);
    // Returns 4: Tag + ResponsesSection header + 2 Response entries
    assert_eq!(annotations.len(), 4);
    assert_eq!(annotations[0].kind, AnnotationKind::Tag);
    assert_eq!(annotations[1].kind, AnnotationKind::ResponsesSection);
    assert_eq!(annotations[2].kind, AnnotationKind::Response);
    assert_eq!(annotations[3].kind, AnnotationKind::Response);
}

#[test]
fn detects_security_annotation() {
    let content = r#"
/// @security bearer
#[rovo]
async fn handler() {}
"#;
    let annotations = parse_annotations(content);
    assert_eq!(annotations.len(), 1);
    assert_eq!(annotations[0].kind, AnnotationKind::Security);
    assert_eq!(annotations[0].security_scheme, Some("bearer".to_string()));
}

#[test]
fn detects_example_annotation() {
    let content = r#"
/// # Responses
///
/// 200: Json<User> - Success
///
/// # Examples
///
/// 200: {"name": "John", "age": 30}
#[rovo]
async fn handler() {}
"#;
    let annotations = parse_annotations(content);
    // Returns 4: ResponsesSection + Response + ExamplesSection + Example
    assert_eq!(annotations.len(), 4);
    assert_eq!(annotations[0].kind, AnnotationKind::ResponsesSection);
    assert_eq!(annotations[1].kind, AnnotationKind::Response);
    assert_eq!(annotations[2].kind, AnnotationKind::ExamplesSection);
    assert_eq!(annotations[3].kind, AnnotationKind::Example);
    assert_eq!(annotations[3].status, Some(200));
    assert_eq!(
        annotations[3].example_value,
        Some(r#"{"name": "John", "age": 30}"#.to_string())
    );
}

#[test]
fn detects_id_annotation() {
    let content = r#"
/// @id get_user
#[rovo]
async fn handler() {}
"#;
    let annotations = parse_annotations(content);
    assert_eq!(annotations.len(), 1);
    assert_eq!(annotations[0].kind, AnnotationKind::Id);
    assert_eq!(annotations[0].operation_id, Some("get_user".to_string()));
}

#[test]
fn detects_hidden_annotation() {
    let content = r#"
/// @hidden
#[rovo]
async fn handler() {}
"#;
    let annotations = parse_annotations(content);
    assert_eq!(annotations.len(), 1);
    assert_eq!(annotations[0].kind, AnnotationKind::Hidden);
}

#[test]
fn ignores_non_annotation_comments() {
    let content = r#"
/// This is a regular comment
/// Another regular comment
#[rovo]
async fn handler() {}
"#;
    let annotations = parse_annotations(content);
    assert_eq!(annotations.len(), 0);
}

#[test]
fn parses_annotations_only_near_rovo_attribute() {
    let content = r#"
/// # Responses
///
/// 200: Json<User> - Success
#[rovo]
async fn handler1() {}

/// # Responses
///
/// 200: Json<Post> - Success
async fn handler2() {}
"#;
    let annotations = parse_annotations(content);
    // Should only find annotations near #[rovo]: ResponsesSection + Response
    assert_eq!(annotations.len(), 2);
}

#[test]
fn detects_multiple_rovo_blocks() {
    let content = r#"
/// # Responses
///
/// 200: Json<User> - Success
///
/// # Metadata
///
/// @tag users
#[rovo]
async fn get_user() {}

/// # Responses
///
/// 200: Json<Post> - Success
///
/// # Metadata
///
/// @tag posts
/// @security bearer
#[rovo]
async fn get_post() {}
"#;
    let annotations = parse_annotations(content);
    // Should find all annotations from both blocks:
    // Block 1: ResponsesSection + Response + MetadataSection + Tag
    // Block 2: ResponsesSection + Response + MetadataSection + Tag + Security
    assert_eq!(annotations.len(), 9);

    // Check both blocks have their annotations
    let user_response = annotations.iter().find(|a| {
        a.kind == AnnotationKind::Response
            && a.response_type
                .as_ref()
                .map(|t| t.contains("User"))
                .unwrap_or(false)
    });
    let user_tag = annotations.iter().find(|a| {
        a.kind == AnnotationKind::Tag && a.tag_name.as_ref() == Some(&"users".to_string())
    });
    let post_response = annotations.iter().find(|a| {
        a.kind == AnnotationKind::Response
            && a.response_type
                .as_ref()
                .map(|t| t.contains("Post"))
                .unwrap_or(false)
    });
    let post_tag = annotations.iter().find(|a| {
        a.kind == AnnotationKind::Tag && a.tag_name.as_ref() == Some(&"posts".to_string())
    });
    let security = annotations
        .iter()
        .find(|a| a.kind == AnnotationKind::Security);

    assert!(user_response.is_some(), "Should find User response");
    assert!(user_tag.is_some(), "Should find users tag");
    assert!(post_response.is_some(), "Should find Post response");
    assert!(post_tag.is_some(), "Should find posts tag");
    assert!(security.is_some(), "Should find security annotation");
}
