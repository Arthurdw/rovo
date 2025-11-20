use rovo_lsp::completion::{get_completions, Position};

#[test]
fn completes_annotation_keywords() {
    let content = "/// @r";
    let position = Position {
        line: 0,
        character: 6,
    };
    let completions = get_completions(content, position);
    assert!(completions.iter().any(|c| c.label == "@response"));
}

#[test]
fn completes_all_annotations_on_at_sign() {
    let content = "/// @";
    let position = Position {
        line: 0,
        character: 5,
    };
    let completions = get_completions(content, position);

    assert!(completions.iter().any(|c| c.label == "@response"));
    assert!(completions.iter().any(|c| c.label == "@tag"));
    assert!(completions.iter().any(|c| c.label == "@security"));
    assert!(completions.iter().any(|c| c.label == "@example"));
    assert!(completions.iter().any(|c| c.label == "@id"));
    assert!(completions.iter().any(|c| c.label == "@hidden"));
}

#[test]
fn includes_snippet_for_response() {
    let content = "/// @res";
    let position = Position {
        line: 0,
        character: 8,
    };
    let completions = get_completions(content, position);

    let response_completion = completions.iter().find(|c| c.label == "@response").unwrap();
    assert!(response_completion.insert_text.is_some());
    assert!(response_completion
        .insert_text
        .as_ref()
        .unwrap()
        .contains("${1:200}"));
}

#[test]
fn no_completions_outside_doc_comment() {
    let content = "@response";
    let position = Position {
        line: 0,
        character: 1,
    };
    let completions = get_completions(content, position);
    assert_eq!(completions.len(), 0);
}

#[test]
fn no_completions_in_regular_comment() {
    let content = "// @response";
    let position = Position {
        line: 0,
        character: 4,
    };
    let completions = get_completions(content, position);
    assert_eq!(completions.len(), 0);
}

#[test]
fn completes_tag_prefix() {
    let content = "/// @t";
    let position = Position {
        line: 0,
        character: 6,
    };
    let completions = get_completions(content, position);
    assert!(!completions.is_empty(), "Should have completions");
    assert!(
        completions.iter().any(|c| c.label == "@tag"),
        "Should include @tag completion"
    );
}

#[test]
fn completes_security_prefix() {
    let content = "/// @s";
    let position = Position {
        line: 0,
        character: 6,
    };
    let completions = get_completions(content, position);
    assert!(!completions.is_empty(), "Should have completions");
    assert!(
        completions.iter().any(|c| c.label == "@security"),
        "Should include @security completion"
    );
}

#[test]
fn completes_example_prefix() {
    let content = "/// @e";
    let position = Position {
        line: 0,
        character: 6,
    };
    let completions = get_completions(content, position);
    assert!(!completions.is_empty(), "Should have completions");
    assert!(
        completions.iter().any(|c| c.label == "@example"),
        "Should include @example completion"
    );
}

#[test]
fn completes_hidden_prefix() {
    let content = "/// @h";
    let position = Position {
        line: 0,
        character: 6,
    };
    let completions = get_completions(content, position);
    assert!(!completions.is_empty(), "Should have completions");
    assert!(
        completions.iter().any(|c| c.label == "@hidden"),
        "Should include @hidden completion"
    );
}

#[test]
fn handles_multiline_doc_comments() {
    let content = "/// This is a comment\n/// @r";
    let position = Position {
        line: 1,
        character: 6,
    };
    let completions = get_completions(content, position);
    assert!(completions.iter().any(|c| c.label == "@response"));
}
