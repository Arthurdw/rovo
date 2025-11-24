use rovo_lsp::completion::{get_completions, Position};

#[test]
fn completes_annotation_keywords() {
    let content = "/// @t";
    let position = Position {
        line: 0,
        character: 6,
    };
    let completions = get_completions(content, position);
    assert!(completions.iter().any(|c| c.label == "@tag"));
}

#[test]
fn completes_all_annotations_on_at_sign() {
    let content = "/// @";
    let position = Position {
        line: 0,
        character: 5,
    };
    let completions = get_completions(content, position);

    // Only metadata annotations (use sections for responses/examples)
    assert_eq!(completions.len(), 4);
    assert!(completions.iter().any(|c| c.label == "@tag"));
    assert!(completions.iter().any(|c| c.label == "@security"));
    assert!(completions.iter().any(|c| c.label == "@id"));
    assert!(completions.iter().any(|c| c.label == "@hidden"));
}

#[test]
fn includes_snippet_for_tag() {
    let content = "/// @t";
    let position = Position {
        line: 0,
        character: 6,
    };
    let completions = get_completions(content, position);

    let tag_completion = completions.iter().find(|c| c.label == "@tag").unwrap();
    assert!(tag_completion.insert_text.is_some());
    assert!(tag_completion
        .insert_text
        .as_ref()
        .unwrap()
        .contains("@tag"));
}

#[test]
fn no_completions_outside_doc_comment() {
    let content = "@tag";
    let position = Position {
        line: 0,
        character: 1,
    };
    let completions = get_completions(content, position);
    assert_eq!(completions.len(), 0);
}

#[test]
fn no_completions_in_regular_comment() {
    let content = "// @tag";
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
fn completes_id_prefix() {
    let content = "/// @i";
    let position = Position {
        line: 0,
        character: 6,
    };
    let completions = get_completions(content, position);
    assert!(!completions.is_empty(), "Should have completions");
    assert!(
        completions.iter().any(|c| c.label == "@id"),
        "Should include @id completion"
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
    let content = "/// This is a comment\n/// @t";
    let position = Position {
        line: 1,
        character: 6,
    };
    let completions = get_completions(content, position);
    assert!(completions.iter().any(|c| c.label == "@tag"));
}
