use once_cell::sync::Lazy;
use regex::Regex;

// Static regex patterns to avoid recompilation
static TAG_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"@tag\s+(\S+)").unwrap());
static SECURITY_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"@security\s+(\S+)").unwrap());
static ID_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"@id\s+(\S+)").unwrap());

/// Type of Rovo annotation
#[derive(Debug, Clone, PartialEq)]
pub enum AnnotationKind {
    /// Response entry from # Responses section
    Response,
    /// @tag - Group endpoints by tag
    Tag,
    /// @security - Specify security scheme
    Security,
    /// Example entry from # Examples section
    Example,
    /// @id - Set operation ID
    Id,
    /// @hidden - Mark endpoint as hidden from docs
    Hidden,
    /// # Responses section header
    ResponsesSection,
    /// # Examples section header
    ExamplesSection,
    /// # Metadata section header
    MetadataSection,
}

/// Parsed annotation from a doc comment
#[derive(Debug, Clone)]
pub struct Annotation {
    /// Type of annotation
    pub kind: AnnotationKind,
    /// Line number where annotation appears (0-indexed)
    pub line: usize,

    // Response fields (from # Responses section)
    /// HTTP status code for response entries
    pub status: Option<u16>,
    /// Response type (e.g., Json<User>)
    pub response_type: Option<String>,
    /// Description for response entries
    pub description: Option<String>,

    // Tag fields
    /// Tag name for @tag annotations
    pub tag_name: Option<String>,

    // Security fields
    /// Security scheme name for @security annotations
    pub security_scheme: Option<String>,

    // Example fields (from # Examples section)
    /// Example value
    pub example_value: Option<String>,

    // ID fields
    /// Operation ID for @id annotations
    pub operation_id: Option<String>,
}

impl Annotation {
    fn new(kind: AnnotationKind, line: usize) -> Self {
        Self {
            kind,
            line,
            status: None,
            response_type: None,
            description: None,
            tag_name: None,
            security_scheme: None,
            example_value: None,
            operation_id: None,
        }
    }
}

/// Check if a given position (line number) is near a #[rovo] attribute
pub fn is_near_rovo_attribute(content: &str, target_line: usize) -> bool {
    let lines: Vec<&str> = content.lines().collect();

    // Look ahead up to 20 lines to find a #[rovo] attribute
    for i in target_line..std::cmp::min(target_line + 20, lines.len()) {
        if lines[i].trim() == "#[rovo]" || lines[i].contains("#[") && lines[i].contains("rovo") {
            return true;
        }
        // Stop if we hit a non-comment, non-attribute line
        if !lines[i].trim().starts_with("///")
            && !lines[i].trim().starts_with("#[")
            && !lines[i].trim().is_empty()
        {
            break;
        }
    }

    false
}

/// Parse all Rovo annotations from source code content
///
/// Searches for #[rovo] attributes and extracts all @ annotations and markdown sections
/// from the doc comments immediately preceding them.
///
/// # Arguments
/// * `content` - The source code to parse
///
/// # Returns
/// A vector of parsed annotations in order of appearance
pub fn parse_annotations(content: &str) -> Vec<Annotation> {
    let lines: Vec<&str> = content.lines().collect();
    let mut annotations = Vec::new();

    // Find all #[rovo] attributes
    let mut rovo_positions = Vec::new();
    for (idx, line) in lines.iter().enumerate() {
        if line.trim() == "#[rovo]" {
            rovo_positions.push(idx);
        }
    }

    // For each #[rovo], collect doc comments and parse them
    for rovo_pos in rovo_positions {
        // First, collect all doc comment lines above #[rovo]
        let mut doc_lines = Vec::new();
        let mut i = rovo_pos;

        while i > 0 {
            i -= 1;
            let line = lines[i].trim();

            // Skip empty lines
            if line.is_empty() {
                continue;
            }

            // Stop if we hit a non-doc-comment line
            if !line.starts_with("///") {
                break;
            }

            let doc_content = line.trim_start_matches("///").trim();

            // Check for @rovo-ignore directive - stops scanning this doc block
            if doc_content.starts_with("@rovo-ignore") {
                break;
            }

            doc_lines.push((i, line));
        }

        // Reverse to process in forward order
        doc_lines.reverse();

        // Now parse the doc lines in forward order
        let mut current_section: Option<&str> = None;
        let mut idx = 0;

        while idx < doc_lines.len() {
            let (line_num, line) = doc_lines[idx];
            let doc_content = line.trim_start_matches("///").trim();

            // Check for markdown section headers
            if doc_content.starts_with("# ") {
                let section_name = doc_content.trim_start_matches("# ").trim();
                match section_name {
                    "Responses" => {
                        current_section = Some("responses");
                        annotations
                            .push(Annotation::new(AnnotationKind::ResponsesSection, line_num));
                    }
                    "Examples" => {
                        current_section = Some("examples");
                        annotations
                            .push(Annotation::new(AnnotationKind::ExamplesSection, line_num));
                    }
                    "Metadata" => {
                        current_section = Some("metadata");
                        annotations
                            .push(Annotation::new(AnnotationKind::MetadataSection, line_num));
                    }
                    _ => current_section = None,
                }
                idx += 1;
                continue;
            }

            // Parse content based on current section or annotation
            if let Some(section) = current_section {
                match section {
                    "responses" => {
                        if let Some(ann) = parse_response_line(doc_content, line_num) {
                            annotations.push(ann);
                        }
                        idx += 1;
                    }
                    "examples" => {
                        // Try to parse a multi-line example
                        if let Some((ann, lines_consumed)) =
                            parse_multiline_example(&doc_lines[idx..])
                        {
                            annotations.push(ann);
                            idx += lines_consumed;
                        } else {
                            idx += 1;
                        }
                    }
                    "metadata" => {
                        if let Some(ann) = parse_annotation_line(line, line_num) {
                            annotations.push(ann);
                        }
                        idx += 1;
                    }
                    _ => {
                        idx += 1;
                    }
                }
            } else {
                // Not in a section - parse old-style @ annotations
                if let Some(ann) = parse_annotation_line(line, line_num) {
                    annotations.push(ann);
                }
                idx += 1;
            }
        }
    }

    annotations
}

/// Parse a response line from # Responses section
/// Format: STATUS: TYPE - DESCRIPTION
fn parse_response_line(content: &str, line_num: usize) -> Option<Annotation> {
    if let Some(colon_pos) = content.find(':') {
        let before_colon = &content[..colon_pos].trim();
        if before_colon.chars().all(|c| c.is_ascii_digit()) {
            let status: u16 = before_colon.parse().ok()?;
            let after_colon = content[colon_pos + 1..].trim();

            if let Some(dash_pos) = after_colon.find(" - ") {
                let response_type = after_colon[..dash_pos].trim().to_string();
                let description = after_colon[dash_pos + 3..].trim().to_string();

                let mut ann = Annotation::new(AnnotationKind::Response, line_num);
                ann.status = Some(status);
                ann.response_type = Some(response_type);
                ann.description = Some(description);
                return Some(ann);
            }
        }
    }
    None
}

/// Parse a potentially multi-line example from # Examples section
/// Returns the annotation and the number of lines consumed
fn parse_multiline_example(doc_lines: &[(usize, &str)]) -> Option<(Annotation, usize)> {
    if doc_lines.is_empty() {
        return None;
    }

    let (line_num, first_line) = doc_lines[0];
    let content = first_line.trim_start_matches("///").trim();

    // Check if this line starts with STATUS:
    let colon_pos = content.find(':')?;
    let before_colon = content[..colon_pos].trim();
    if !before_colon.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let status: u16 = before_colon.parse().ok()?;

    let after_colon = content[colon_pos + 1..].trim();
    let mut lines_consumed = 1;
    let mut example_lines = Vec::new();

    // Check if this is a code block format (STATUS: followed by ```)
    let is_code_block = if after_colon.is_empty() && lines_consumed < doc_lines.len() {
        let (_, next_line) = doc_lines[lines_consumed];
        let next_content = next_line.trim_start_matches("///").trim();
        next_content == "```" || next_content.starts_with("```")
    } else {
        false
    };

    if is_code_block {
        // Skip the opening ``` line
        lines_consumed += 1;

        // Collect lines until we hit closing ```
        while lines_consumed < doc_lines.len() {
            let (_, line) = doc_lines[lines_consumed];
            let line_content = line.trim_start_matches("///").trim();

            // Check for closing ```
            if line_content == "```" || line_content.starts_with("```") {
                lines_consumed += 1;
                break;
            }

            example_lines.push(line_content);
            lines_consumed += 1;
        }
    } else {
        // Inline format - collect the example value, potentially across multiple lines
        example_lines.push(after_colon);

        // Check if the expression needs more lines (count braces/brackets/parens)
        let mut brace_depth = 0i32;
        let mut bracket_depth = 0i32;
        let mut paren_depth = 0i32;

        for line_content in &example_lines {
            for ch in line_content.chars() {
                match ch {
                    '{' => brace_depth += 1,
                    '}' => brace_depth -= 1,
                    '[' => bracket_depth += 1,
                    ']' => bracket_depth -= 1,
                    '(' => paren_depth += 1,
                    ')' => paren_depth -= 1,
                    _ => {}
                }
            }
        }

        // Continue collecting lines if we have unclosed delimiters
        while (brace_depth > 0 || bracket_depth > 0 || paren_depth > 0)
            && lines_consumed < doc_lines.len()
        {
            let (_, next_line) = doc_lines[lines_consumed];
            let next_content = next_line.trim_start_matches("///").trim();

            // Stop if we hit a new section or annotation
            if next_content.starts_with('#')
                || next_content.starts_with('@')
                || next_content.is_empty()
            {
                break;
            }

            // Count delimiters in this line
            for ch in next_content.chars() {
                match ch {
                    '{' => brace_depth += 1,
                    '}' => brace_depth -= 1,
                    '[' => bracket_depth += 1,
                    ']' => bracket_depth -= 1,
                    '(' => paren_depth += 1,
                    ')' => paren_depth -= 1,
                    _ => {}
                }
            }

            example_lines.push(next_content);
            lines_consumed += 1;
        }
    }

    let example_value = example_lines.join("\n");

    let mut ann = Annotation::new(AnnotationKind::Example, line_num);
    ann.status = Some(status);
    ann.example_value = Some(example_value);

    Some((ann, lines_consumed))
}

fn parse_annotation_line(line: &str, line_num: usize) -> Option<Annotation> {
    // Remove /// prefix and trim
    let content = line.trim_start_matches("///").trim();

    // Check if it starts with @
    if !content.starts_with('@') {
        return None;
    }

    // Parse metadata annotations (@tag, @security, @id, @hidden)
    if content.starts_with("@tag") {
        parse_tag(content, line_num)
    } else if content.starts_with("@security") {
        parse_security(content, line_num)
    } else if content.starts_with("@id") {
        parse_id(content, line_num)
    } else if content.starts_with("@hidden") {
        Some(Annotation::new(AnnotationKind::Hidden, line_num))
    } else {
        None
    }
}

fn parse_tag(content: &str, line_num: usize) -> Option<Annotation> {
    // Format: @tag NAME
    if let Some(captures) = TAG_RE.captures(content) {
        let tag_name = captures.get(1)?.as_str().to_string();

        let mut ann = Annotation::new(AnnotationKind::Tag, line_num);
        ann.tag_name = Some(tag_name);

        Some(ann)
    } else {
        None
    }
}

fn parse_security(content: &str, line_num: usize) -> Option<Annotation> {
    // Format: @security SCHEME
    if let Some(captures) = SECURITY_RE.captures(content) {
        let security_scheme = captures.get(1)?.as_str().to_string();

        let mut ann = Annotation::new(AnnotationKind::Security, line_num);
        ann.security_scheme = Some(security_scheme);

        Some(ann)
    } else {
        None
    }
}

fn parse_id(content: &str, line_num: usize) -> Option<Annotation> {
    // Format: @id OPERATION_ID
    if let Some(captures) = ID_RE.captures(content) {
        let operation_id = captures.get(1)?.as_str().to_string();

        let mut ann = Annotation::new(AnnotationKind::Id, line_num);
        ann.operation_id = Some(operation_id);

        Some(ann)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tag() {
        let line = "/// @tag users";
        let ann = parse_annotation_line(line, 0).unwrap();
        assert_eq!(ann.kind, AnnotationKind::Tag);
        assert_eq!(ann.tag_name, Some("users".to_string()));
    }

    #[test]
    fn test_parse_security() {
        let line = "/// @security bearer";
        let ann = parse_annotation_line(line, 0).unwrap();
        assert_eq!(ann.kind, AnnotationKind::Security);
        assert_eq!(ann.security_scheme, Some("bearer".to_string()));
    }

    #[test]
    fn test_parse_id() {
        let line = "/// @id getUserById";
        let ann = parse_annotation_line(line, 0).unwrap();
        assert_eq!(ann.kind, AnnotationKind::Id);
        assert_eq!(ann.operation_id, Some("getUserById".to_string()));
    }

    #[test]
    fn test_parse_hidden() {
        let line = "/// @hidden";
        let ann = parse_annotation_line(line, 0).unwrap();
        assert_eq!(ann.kind, AnnotationKind::Hidden);
    }

    #[test]
    fn test_parse_rust_style_responses() {
        let content = r#"
/// # Responses
///
/// 200: Json<User> - Successfully retrieved user
/// 404: () - User not found
#[rovo]
async fn handler() {}
"#;
        let annotations = parse_annotations(content);
        // Should have ResponsesSection + 2 Response annotations
        assert!(annotations
            .iter()
            .any(|a| a.kind == AnnotationKind::ResponsesSection));

        let responses: Vec<_> = annotations
            .iter()
            .filter(|a| a.kind == AnnotationKind::Response)
            .collect();
        assert_eq!(responses.len(), 2);
        assert_eq!(responses[0].status, Some(200));
        assert_eq!(responses[0].response_type, Some("Json<User>".to_string()));
        assert_eq!(
            responses[0].description,
            Some("Successfully retrieved user".to_string())
        );
    }

    #[test]
    fn test_parse_rust_style_examples() {
        let content = r#"
/// # Examples
///
/// 200: User::default()
/// 404: ()
#[rovo]
async fn handler() {}
"#;
        let annotations = parse_annotations(content);
        assert!(annotations
            .iter()
            .any(|a| a.kind == AnnotationKind::ExamplesSection));

        let examples: Vec<_> = annotations
            .iter()
            .filter(|a| a.kind == AnnotationKind::Example)
            .collect();
        assert_eq!(examples.len(), 2);
        assert_eq!(examples[0].status, Some(200));
        assert_eq!(
            examples[0].example_value,
            Some("User::default()".to_string())
        );
    }

    #[test]
    fn test_parse_rust_style_metadata() {
        let content = r#"
/// # Metadata
///
/// @tag users
/// @security bearer_auth
#[rovo]
async fn handler() {}
"#;
        let annotations = parse_annotations(content);
        assert!(annotations
            .iter()
            .any(|a| a.kind == AnnotationKind::MetadataSection));
        assert!(annotations.iter().any(|a| a.kind == AnnotationKind::Tag));
        assert!(annotations
            .iter()
            .any(|a| a.kind == AnnotationKind::Security));
    }

    #[test]
    fn test_parse_multiline_example() {
        let content = r#"
/// # Examples
///
/// 200: TodoItem {
///     id: Uuid::nil(),
///     description: "Buy milk".into(),
///     complete: false,
///     ..Default::default()
/// }
#[rovo]
async fn handler() {}
"#;
        let annotations = parse_annotations(content);
        let examples: Vec<_> = annotations
            .iter()
            .filter(|a| a.kind == AnnotationKind::Example)
            .collect();
        assert_eq!(examples.len(), 1);
        assert_eq!(examples[0].status, Some(200));
        let example_value = examples[0].example_value.as_ref().unwrap();
        assert!(example_value.contains("TodoItem"));
        assert!(example_value.contains("id: Uuid::nil()"));
        assert!(example_value.contains("..Default::default()"));
    }

    #[test]
    fn test_parse_multiline_example_with_code_blocks() {
        let content = r#"
/// # Examples
///
/// 200:
/// ```
/// TodoItem {
///     id: Uuid::nil(),
///     description: "Buy milk".into(),
///     ..Default::default()
/// }
/// ```
#[rovo]
async fn handler() {}
"#;
        let annotations = parse_annotations(content);
        let examples: Vec<_> = annotations
            .iter()
            .filter(|a| a.kind == AnnotationKind::Example)
            .collect();
        assert_eq!(examples.len(), 1, "Should have exactly one example");
        assert_eq!(examples[0].status, Some(200));
        let example_value = examples[0].example_value.as_ref().unwrap();
        assert!(
            example_value.contains("TodoItem"),
            "Example should contain 'TodoItem'"
        );
        assert!(
            example_value.contains("id: Uuid::nil()"),
            "Example should contain 'id: Uuid::nil()'"
        );
        assert!(
            example_value.contains("..Default::default()"),
            "Example should contain '..Default::default()'"
        );
        // Ensure we don't include the backticks
        assert!(
            !example_value.contains("```"),
            "Example should not contain backticks"
        );
    }

    #[test]
    fn test_parse_mixed_format() {
        let content = r#"
/// Get user by ID.
///
/// # Responses
///
/// 200: Json<User> - User found
///
/// # Metadata
///
/// @tag users
#[rovo]
async fn handler() {}
"#;
        let annotations = parse_annotations(content);
        assert!(annotations
            .iter()
            .any(|a| a.kind == AnnotationKind::ResponsesSection));
        assert!(annotations
            .iter()
            .any(|a| a.kind == AnnotationKind::MetadataSection));
        assert!(annotations
            .iter()
            .any(|a| a.kind == AnnotationKind::Response));
        assert!(annotations.iter().any(|a| a.kind == AnnotationKind::Tag));
    }
}
