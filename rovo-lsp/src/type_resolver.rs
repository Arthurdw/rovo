use regex::Regex;

/// Extract innermost type name from annotation response type by recursively unwrapping
///
/// Recursively unwraps known wrapper types (Json, Vec, Option, Result, Arc, Box, Rc)
/// until reaching the innermost non-wrapped type.
///
/// # Examples
/// - `"Json<TodoItem>"` -> `"TodoItem"`
/// - `"Json<Vec<TodoItem>>"` -> `"TodoItem"`
/// - `"Vec<Option<User>>"` -> `"User"`
/// - `"TodoItem"` -> `"TodoItem"`
pub fn extract_type_from_response(response_type: &str) -> Option<String> {
    let re = Regex::new(r"^(?:Json|Vec|Option|Result|Arc|Box|Rc)<(.*)>$").unwrap();

    let trimmed = response_type.trim();
    if let Some(captures) = re.captures(trimmed) {
        let inner = captures.get(1)?.as_str();
        // Recursively unwrap nested generics
        extract_type_from_response(inner)
    } else {
        // No wrapper found, return the trimmed type
        Some(trimmed.to_string())
    }
}

/// Find the definition line of a type in the content
pub fn find_type_definition(content: &str, type_name: &str) -> Option<usize> {
    let lines: Vec<&str> = content.lines().collect();

    // Look for struct, enum, or type alias definitions
    let struct_pattern = format!(r"^\s*(?:pub\s+)?struct\s+{}\b", regex::escape(type_name));
    let enum_pattern = format!(r"^\s*(?:pub\s+)?enum\s+{}\b", regex::escape(type_name));
    let type_pattern = format!(r"^\s*(?:pub\s+)?type\s+{}\b", regex::escape(type_name));

    let struct_re = Regex::new(&struct_pattern).ok()?;
    let enum_re = Regex::new(&enum_pattern).ok()?;
    let type_re = Regex::new(&type_pattern).ok()?;

    for (idx, line) in lines.iter().enumerate() {
        if struct_re.is_match(line) || enum_re.is_match(line) || type_re.is_match(line) {
            return Some(idx);
        }
    }

    None
}

/// Check if cursor is on a type in an annotation
pub fn get_type_at_position(line: &str, char_idx: usize) -> Option<(String, usize, usize)> {
    // Pattern: /// @response 200 Json<TodoItem> Description
    let re = Regex::new(r"///\s*@(?:response|example)\s+\d+\s+(\S+)").unwrap();

    if let Some(captures) = re.captures(line) {
        let response_type = captures.get(1)?.as_str();
        let start = captures.get(1)?.start();
        let end = captures.get(1)?.end();

        // Check if cursor is within this range
        if char_idx >= start && char_idx <= end {
            return Some((response_type.to_string(), start, end));
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_type_from_json() {
        assert_eq!(
            extract_type_from_response("Json<TodoItem>"),
            Some("TodoItem".to_string())
        );
    }

    #[test]
    fn test_extract_type_from_vec() {
        assert_eq!(
            extract_type_from_response("Vec<User>"),
            Some("User".to_string())
        );
    }

    #[test]
    fn test_extract_type_plain() {
        assert_eq!(
            extract_type_from_response("TodoItem"),
            Some("TodoItem".to_string())
        );
    }

    #[test]
    fn test_extract_type_nested_generics() {
        assert_eq!(
            extract_type_from_response("Json<Vec<TodoItem>>"),
            Some("TodoItem".to_string())
        );
    }

    #[test]
    fn test_extract_type_deeply_nested() {
        assert_eq!(
            extract_type_from_response("Option<Result<Arc<User>>>"),
            Some("User".to_string())
        );
    }

    #[test]
    fn test_find_struct_definition() {
        let content = r#"
struct TodoItem {
    id: u32,
}
"#;
        assert_eq!(find_type_definition(content, "TodoItem"), Some(1));
    }
}
