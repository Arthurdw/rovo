use regex::Regex;

/// Type of Rovo annotation
#[derive(Debug, Clone, PartialEq)]
pub enum AnnotationKind {
    /// @response - Define API response for a status code
    Response,
    /// @tag - Group endpoints by tag
    Tag,
    /// @security - Specify security scheme
    Security,
    /// @example - Provide example response
    Example,
    /// @id - Set operation ID
    Id,
    /// @hidden - Mark endpoint as hidden from docs
    Hidden,
}

/// Parsed annotation from a doc comment
#[derive(Debug, Clone)]
pub struct Annotation {
    /// Type of annotation
    pub kind: AnnotationKind,
    /// Line number where annotation appears (0-indexed)
    pub line: usize,

    // Response fields
    /// HTTP status code for @response annotations
    pub status: Option<u16>,
    /// Response type for @response annotations (e.g., Json<User>)
    pub response_type: Option<String>,
    /// Description for @response annotations
    pub description: Option<String>,

    // Tag fields
    /// Tag name for @tag annotations
    pub tag_name: Option<String>,

    // Security fields
    /// Security scheme name for @security annotations
    pub security_scheme: Option<String>,

    // Example fields
    /// Example JSON value for @example annotations
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
/// Searches for #[rovo] attributes and extracts all @ annotations from the doc comments
/// immediately preceding them.
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

    // For each #[rovo], look backwards for doc comments
    for rovo_pos in rovo_positions {
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

            // Parse annotation
            if let Some(ann) = parse_annotation_line(line, i) {
                annotations.insert(0, ann);
            }
        }
    }

    annotations
}

fn parse_annotation_line(line: &str, line_num: usize) -> Option<Annotation> {
    // Remove /// prefix and trim
    let content = line.trim_start_matches("///").trim();

    // Check if it starts with @
    if !content.starts_with('@') {
        return None;
    }

    // Parse different annotation types
    if content.starts_with("@response") {
        parse_response(content, line_num)
    } else if content.starts_with("@tag") {
        parse_tag(content, line_num)
    } else if content.starts_with("@security") {
        parse_security(content, line_num)
    } else if content.starts_with("@example") {
        parse_example(content, line_num)
    } else if content.starts_with("@id") {
        parse_id(content, line_num)
    } else if content.starts_with("@hidden") {
        Some(Annotation::new(AnnotationKind::Hidden, line_num))
    } else {
        None
    }
}

fn parse_response(content: &str, line_num: usize) -> Option<Annotation> {
    // Format: @response STATUS TYPE DESCRIPTION
    let re = Regex::new(r"@response\s+(\d+)\s+(\S+)\s*(.*)").unwrap();

    if let Some(captures) = re.captures(content) {
        let status: u16 = captures.get(1)?.as_str().parse().ok()?;
        let response_type = captures.get(2)?.as_str().to_string();
        let description = captures.get(3).map(|m| m.as_str().to_string());

        let mut ann = Annotation::new(AnnotationKind::Response, line_num);
        ann.status = Some(status);
        ann.response_type = Some(response_type);
        ann.description = description;

        Some(ann)
    } else {
        None
    }
}

fn parse_tag(content: &str, line_num: usize) -> Option<Annotation> {
    // Format: @tag NAME
    let re = Regex::new(r"@tag\s+(\S+)").unwrap();

    if let Some(captures) = re.captures(content) {
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
    let re = Regex::new(r"@security\s+(\S+)").unwrap();

    if let Some(captures) = re.captures(content) {
        let security_scheme = captures.get(1)?.as_str().to_string();

        let mut ann = Annotation::new(AnnotationKind::Security, line_num);
        ann.security_scheme = Some(security_scheme);

        Some(ann)
    } else {
        None
    }
}

fn parse_example(content: &str, line_num: usize) -> Option<Annotation> {
    // Format: @example STATUS JSON
    let re = Regex::new(r"@example\s+(\d+)\s+(.+)").unwrap();

    if let Some(captures) = re.captures(content) {
        let status: u16 = captures.get(1)?.as_str().parse().ok()?;
        let example_value = captures.get(2)?.as_str().to_string();

        let mut ann = Annotation::new(AnnotationKind::Example, line_num);
        ann.status = Some(status);
        ann.example_value = Some(example_value);

        Some(ann)
    } else {
        None
    }
}

fn parse_id(content: &str, line_num: usize) -> Option<Annotation> {
    // Format: @id OPERATION_ID
    let re = Regex::new(r"@id\s+(\S+)").unwrap();

    if let Some(captures) = re.captures(content) {
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
    fn test_parse_response() {
        let line = "/// @response 200 Json<User> Success";
        let ann = parse_annotation_line(line, 0).unwrap();
        assert_eq!(ann.kind, AnnotationKind::Response);
        assert_eq!(ann.status, Some(200));
        assert_eq!(ann.response_type, Some("Json<User>".to_string()));
        assert_eq!(ann.description, Some("Success".to_string()));
    }

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
    fn test_parse_example() {
        let line = r#"/// @example 200 {"id": 1}"#;
        let ann = parse_annotation_line(line, 0).unwrap();
        assert_eq!(ann.kind, AnnotationKind::Example);
        assert_eq!(ann.status, Some(200));
        assert_eq!(ann.example_value, Some(r#"{"id": 1}"#.to_string()));
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
    fn test_parse_annotations_with_blank_lines() {
        let content = r#"
/// @response 200 Json<User> Success
///
/// @tag users
#[rovo]
async fn handler() {}
"#;
        let annotations = parse_annotations(content);
        assert_eq!(annotations.len(), 2);
        assert_eq!(annotations[0].kind, AnnotationKind::Response);
        assert_eq!(annotations[1].kind, AnnotationKind::Tag);
    }
}
