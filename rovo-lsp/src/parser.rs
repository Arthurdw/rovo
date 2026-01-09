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
    /// # Path Parameters section header
    PathParametersSection,
    /// Path parameter entry from # Path Parameters section
    PathParameter,
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

    // Path parameter fields (from # Path Parameters section)
    /// Path parameter name
    pub param_name: Option<String>,
    /// Path parameter description
    pub param_description: Option<String>,
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
            param_name: None,
            param_description: None,
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

/// Internal enum for tracking which documentation section we're parsing
#[derive(Clone, Copy, PartialEq)]
enum Section {
    Responses,
    Examples,
    Metadata,
    PathParameters,
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
    // Note: We only support the exact `#[rovo]` form for parsing, not variants like
    // `#[rovo(...)]`. This is intentional - the simpler form matches actual usage.
    // (is_near_rovo_attribute uses a broader check for LSP feature guards, but
    // parsing only needs to handle the exact attribute form.)
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

            // Check for @rovo-ignore directive - everything AFTER this line (closer to #[rovo])
            // should be ignored, so we clear doc_lines and continue collecting lines BEFORE it
            if doc_content.starts_with("@rovo-ignore") {
                doc_lines.clear();
                continue;
            }

            // Only add lines if we haven't hit @rovo-ignore yet, or if we're past it
            // Since we scan backwards, lines collected after clearing are BEFORE @rovo-ignore
            doc_lines.push((i, line));
        }

        // Reverse to process in forward order
        doc_lines.reverse();

        // Now parse the doc lines in forward order
        let mut current_section: Option<Section> = None;
        let mut idx = 0;

        while idx < doc_lines.len() {
            let (line_num, line) = doc_lines[idx];
            let doc_content = line.trim_start_matches("///").trim();

            // Check for markdown section headers
            if doc_content.starts_with("# ") {
                let section_name = doc_content.trim_start_matches("# ").trim();
                match section_name {
                    "Responses" => {
                        current_section = Some(Section::Responses);
                        annotations
                            .push(Annotation::new(AnnotationKind::ResponsesSection, line_num));
                    }
                    "Examples" => {
                        current_section = Some(Section::Examples);
                        annotations
                            .push(Annotation::new(AnnotationKind::ExamplesSection, line_num));
                    }
                    "Metadata" => {
                        current_section = Some(Section::Metadata);
                        annotations
                            .push(Annotation::new(AnnotationKind::MetadataSection, line_num));
                    }
                    "Path Parameters" => {
                        current_section = Some(Section::PathParameters);
                        annotations.push(Annotation::new(
                            AnnotationKind::PathParametersSection,
                            line_num,
                        ));
                    }
                    _ => current_section = None,
                }
                idx += 1;
                continue;
            }

            // Parse content based on current section or annotation
            match current_section {
                Some(Section::Responses) => {
                    // Try to parse a multi-line response
                    if let Some((ann, lines_consumed)) = parse_multiline_response(&doc_lines[idx..])
                    {
                        annotations.push(ann);
                        idx += lines_consumed;
                    } else {
                        idx += 1;
                    }
                }
                Some(Section::Examples) => {
                    // Try to parse a multi-line example
                    if let Some((ann, lines_consumed)) = parse_multiline_example(&doc_lines[idx..])
                    {
                        annotations.push(ann);
                        idx += lines_consumed;
                    } else {
                        idx += 1;
                    }
                }
                Some(Section::Metadata) => {
                    if let Some(ann) = parse_annotation_line(line, line_num) {
                        annotations.push(ann);
                    }
                    idx += 1;
                }
                Some(Section::PathParameters) => {
                    // Parse path parameter: "name: description"
                    if let Some(ann) = parse_path_parameter(line, line_num) {
                        annotations.push(ann);
                    }
                    idx += 1;
                }
                None => {
                    // Not in a section - parse old-style @ annotations
                    if let Some(ann) = parse_annotation_line(line, line_num) {
                        annotations.push(ann);
                    }
                    idx += 1;
                }
            }
        }
    }

    annotations
}

/// Parse a potentially multi-line response from # Responses section
/// Format: STATUS: TYPE - DESCRIPTION (description can continue on following lines)
/// Returns the annotation and the number of lines consumed
fn parse_multiline_response(doc_lines: &[(usize, &str)]) -> Option<(Annotation, usize)> {
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

    // Optionally split off " - DESCRIPTION" (description is optional)
    let (response_type, mut description_parts) = if let Some(dash_pos) = after_colon.find(" - ") {
        (
            after_colon[..dash_pos].trim().to_string(),
            vec![after_colon[dash_pos + 3..].trim()],
        )
    } else {
        (after_colon.trim().to_string(), Vec::new())
    };

    let mut lines_consumed = 1;

    // Continue collecting description lines
    // A continuation line:
    // - Does NOT start with a digit (new response entry)
    // - Does NOT start with # (new section)
    // - Does NOT start with @ (annotation)
    // - Is NOT empty
    while lines_consumed < doc_lines.len() {
        let (_, next_line) = doc_lines[lines_consumed];
        let next_content = next_line.trim_start_matches("///").trim();

        // Empty line ends the description
        if next_content.is_empty() {
            break;
        }

        // New section or annotation ends the description
        if next_content.starts_with('#') || next_content.starts_with('@') {
            break;
        }

        // New response entry (valid STATUS: format) ends the description
        // Must look like a status code (3+ digits, parseable as u16)
        let starts_new_response = if let Some(colon_pos) = next_content.find(':') {
            let before_colon = next_content[..colon_pos].trim();
            before_colon.len() >= 3
                && before_colon.chars().all(|c| c.is_ascii_digit())
                && before_colon.parse::<u16>().is_ok()
        } else {
            false
        };

        if starts_new_response {
            break;
        }

        // This is a continuation line
        description_parts.push(next_content);
        lines_consumed += 1;
    }

    let mut ann = Annotation::new(AnnotationKind::Response, line_num);
    ann.status = Some(status);
    ann.response_type = Some(response_type);
    if !description_parts.is_empty() {
        ann.description = Some(description_parts.join(" "));
    }

    Some((ann, lines_consumed))
}

/// Count delimiter depths while ignoring delimiters inside string/char literals
/// Handles regular strings, raw strings (r"...", r#"..."#), and char literals
/// Returns (brace_depth, bracket_depth, paren_depth)
fn count_delimiters(content: &str) -> (i32, i32, i32) {
    let mut brace_depth = 0i32;
    let mut bracket_depth = 0i32;
    let mut paren_depth = 0i32;

    let chars: Vec<char> = content.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let ch = chars[i];

        // Check for raw string literal: r"..." or r#"..."# (with any number of #)
        if ch == 'r' && i + 1 < chars.len() {
            let mut hash_count = 0;
            let mut j = i + 1;

            // Count leading #s
            while j < chars.len() && chars[j] == '#' {
                hash_count += 1;
                j += 1;
            }

            // Check for opening quote
            if j < chars.len() && chars[j] == '"' {
                j += 1; // Skip opening quote
                        // Find closing quote followed by same number of #s
                while j < chars.len() {
                    if chars[j] == '"' {
                        // Check if followed by enough #s
                        let mut trailing_hashes = 0;
                        let mut k = j + 1;
                        while k < chars.len() && chars[k] == '#' && trailing_hashes < hash_count {
                            trailing_hashes += 1;
                            k += 1;
                        }
                        if trailing_hashes == hash_count {
                            i = k;
                            break;
                        }
                    }
                    j += 1;
                }
                if j >= chars.len() {
                    i = j;
                }
                continue;
            }
        }

        // Check for regular string literal
        if ch == '"' {
            i += 1;
            while i < chars.len() {
                if chars[i] == '\\' && i + 1 < chars.len() {
                    i += 2; // Skip escape sequence
                } else if chars[i] == '"' {
                    i += 1;
                    break;
                } else {
                    i += 1;
                }
            }
            continue;
        }

        // Check for char literal
        if ch == '\'' {
            i += 1;
            if i < chars.len() && chars[i] == '\\' && i + 2 < chars.len() {
                i += 3; // Skip escape + char + closing quote
            } else if i + 1 < chars.len() {
                i += 2; // Skip char + closing quote
            }
            continue;
        }

        // Count delimiters outside of strings/chars
        match ch {
            '{' => brace_depth += 1,
            '}' => brace_depth -= 1,
            '[' => bracket_depth += 1,
            ']' => bracket_depth -= 1,
            '(' => paren_depth += 1,
            ')' => paren_depth -= 1,
            _ => {}
        }

        i += 1;
    }

    (brace_depth, bracket_depth, paren_depth)
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

        // Collect lines until we hit closing ``` or a safety boundary
        while lines_consumed < doc_lines.len() {
            let (_, line) = doc_lines[lines_consumed];
            let line_content = line.trim_start_matches("///").trim();

            // Check for closing ```
            if line_content == "```" || line_content.starts_with("```") {
                lines_consumed += 1;
                break;
            }

            // Safety cutoff: stop at section headers or annotations if code block is unterminated
            // This prevents consuming the rest of the doc block on missing closing fence
            if line_content.starts_with("# ") || line_content.starts_with('@') {
                break;
            }

            example_lines.push(line_content);
            lines_consumed += 1;
        }
    } else {
        // Inline format - collect the example value, potentially across multiple lines
        example_lines.push(after_colon);

        // Check if the expression needs more lines (count braces/brackets/parens)
        // Uses string-aware counting to ignore delimiters inside literals
        let (mut brace_depth, mut bracket_depth, mut paren_depth) =
            count_delimiters(example_lines.join("\n").as_str());

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

            example_lines.push(next_content);
            lines_consumed += 1;

            // Recount delimiters with all collected content
            (brace_depth, bracket_depth, paren_depth) =
                count_delimiters(example_lines.join("\n").as_str());
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

/// Parse a path parameter entry from # Path Parameters section
/// Format: "name: description"
fn parse_path_parameter(line: &str, line_num: usize) -> Option<Annotation> {
    let content = line.trim_start_matches("///").trim();

    // Skip empty lines
    if content.is_empty() {
        return None;
    }

    // Look for "name: description" format
    let colon_pos = content.find(':')?;
    let name = content[..colon_pos].trim();

    // Skip if the name part looks like a status code (all digits)
    if name.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }

    // Skip if it starts with # (section header) or @ (annotation)
    if name.starts_with('#') || name.starts_with('@') {
        return None;
    }

    let description = content[colon_pos + 1..].trim();

    let mut ann = Annotation::new(AnnotationKind::PathParameter, line_num);
    ann.param_name = Some(name.to_string());
    ann.param_description = Some(description.to_string());

    Some(ann)
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

    #[test]
    fn test_rovo_ignore_stops_parsing_after() {
        // @rovo-ignore should stop processing everything AFTER it (closer to #[rovo])
        // Content BEFORE @rovo-ignore should still be parsed
        let content = r#"
/// Get user information.
///
/// # Responses
///
/// 200: Json<User> - Success
///
/// # Metadata
///
/// @tag users
///
/// @rovo-ignore
///
/// @invalid_annotation this won't cause errors
/// @tag ignored_tag
#[rovo]
async fn handler() {}
"#;
        let annotations = parse_annotations(content);

        // Should have ResponsesSection, Response, MetadataSection, Tag (users) - all BEFORE @rovo-ignore
        assert!(
            annotations
                .iter()
                .any(|a| a.kind == AnnotationKind::ResponsesSection),
            "Should have ResponsesSection"
        );
        assert!(
            annotations
                .iter()
                .any(|a| a.kind == AnnotationKind::Response),
            "Should have Response"
        );
        assert!(
            annotations
                .iter()
                .any(|a| a.kind == AnnotationKind::MetadataSection),
            "Should have MetadataSection"
        );

        // Check that we have the "users" tag but NOT "ignored_tag"
        let tags: Vec<_> = annotations
            .iter()
            .filter(|a| a.kind == AnnotationKind::Tag)
            .collect();
        assert_eq!(tags.len(), 1, "Should have exactly one tag");
        assert_eq!(
            tags[0].tag_name,
            Some("users".to_string()),
            "Tag should be 'users'"
        );
    }

    #[test]
    fn test_rovo_ignore_ignores_everything_after() {
        // Everything AFTER @rovo-ignore should be completely ignored
        let content = r#"
/// @rovo-ignore
///
/// # Responses
///
/// 200: Json<User> - This should be ignored
///
/// @tag ignored
#[rovo]
async fn handler() {}
"#;
        let annotations = parse_annotations(content);

        // Should have nothing - everything is after @rovo-ignore
        assert!(
            annotations.is_empty(),
            "All annotations should be ignored when after @rovo-ignore"
        );
    }

    #[test]
    fn test_parse_multiline_response_descriptions() {
        // Multi-line response descriptions should be joined with spaces
        let content = r#"
/// # Responses
///
/// 200: Json<User> - Successfully retrieved the user from the
///      database with all associated metadata
/// 404: () - User not found in the database or has been
///      deleted by another user
#[rovo]
async fn handler() {}
"#;
        let annotations = parse_annotations(content);

        let responses: Vec<_> = annotations
            .iter()
            .filter(|a| a.kind == AnnotationKind::Response)
            .collect();

        assert_eq!(responses.len(), 2, "Should have exactly 2 responses");

        assert_eq!(responses[0].status, Some(200));
        assert_eq!(responses[0].response_type, Some("Json<User>".to_string()));
        assert_eq!(
            responses[0].description,
            Some(
                "Successfully retrieved the user from the database with all associated metadata"
                    .to_string()
            )
        );

        assert_eq!(responses[1].status, Some(404));
        assert_eq!(responses[1].response_type, Some("()".to_string()));
        assert_eq!(
            responses[1].description,
            Some("User not found in the database or has been deleted by another user".to_string())
        );
    }

    #[test]
    fn test_parse_single_line_response() {
        // Single-line responses should still work
        let content = r#"
/// # Responses
///
/// 200: Json<User> - User found
/// 404: () - Not found
#[rovo]
async fn handler() {}
"#;
        let annotations = parse_annotations(content);

        let responses: Vec<_> = annotations
            .iter()
            .filter(|a| a.kind == AnnotationKind::Response)
            .collect();

        assert_eq!(responses.len(), 2);
        assert_eq!(responses[0].description, Some("User found".to_string()));
        assert_eq!(responses[1].description, Some("Not found".to_string()));
    }

    #[test]
    fn test_count_delimiters_with_raw_strings() {
        // Regular string with brace inside - should not count
        assert_eq!(count_delimiters(r#"let x = "{ brace";"#), (0, 0, 0));

        // Raw string with brace inside - should not count
        assert_eq!(count_delimiters(r#"let x = r"{ brace";"#), (0, 0, 0));

        // Raw string with hashes - should not count delimiters inside
        assert_eq!(count_delimiters(r##"let x = r#"{ brace"#;"##), (0, 0, 0));

        // Multiple hash raw string
        assert_eq!(
            count_delimiters(r###"let x = r##"{ [ ( )"##;"###),
            (0, 0, 0)
        );

        // Mixed: real brace + string with brace
        assert_eq!(count_delimiters(r#"Foo { name: "{ inner }" }"#), (0, 0, 0));

        // Actual unclosed brace
        assert_eq!(count_delimiters(r#"Foo { name: "test" "#), (1, 0, 0));

        // Char literal with bracket
        assert_eq!(count_delimiters("let x = '[';"), (0, 0, 0));
    }

    #[test]
    fn test_parse_response_without_description() {
        // Responses without descriptions should still be parsed
        let content = r#"
/// # Responses
///
/// 200: Json<User>
/// 404: ()
#[rovo]
async fn handler() {}
"#;
        let annotations = parse_annotations(content);

        let responses: Vec<_> = annotations
            .iter()
            .filter(|a| a.kind == AnnotationKind::Response)
            .collect();

        assert_eq!(responses.len(), 2, "Should have exactly 2 responses");

        assert_eq!(responses[0].status, Some(200));
        assert_eq!(responses[0].response_type, Some("Json<User>".to_string()));
        assert_eq!(responses[0].description, None);

        assert_eq!(responses[1].status, Some(404));
        assert_eq!(responses[1].response_type, Some("()".to_string()));
        assert_eq!(responses[1].description, None);
    }

    #[test]
    fn test_parse_path_parameters_section() {
        let content = r#"
/// Get user by ID.
///
/// # Path Parameters
///
/// id: The user's unique identifier
/// index: Zero-based item index
///
/// # Responses
///
/// 200: Json<User> - User found
#[rovo]
async fn handler() {}
"#;
        let annotations = parse_annotations(content);

        // Should have PathParametersSection
        assert!(
            annotations
                .iter()
                .any(|a| a.kind == AnnotationKind::PathParametersSection),
            "Should have PathParametersSection"
        );

        // Should have 2 PathParameter annotations
        let params: Vec<_> = annotations
            .iter()
            .filter(|a| a.kind == AnnotationKind::PathParameter)
            .collect();

        assert_eq!(params.len(), 2, "Should have exactly 2 path parameters");

        assert_eq!(params[0].param_name, Some("id".to_string()));
        assert_eq!(
            params[0].param_description,
            Some("The user's unique identifier".to_string())
        );

        assert_eq!(params[1].param_name, Some("index".to_string()));
        assert_eq!(
            params[1].param_description,
            Some("Zero-based item index".to_string())
        );
    }
}
