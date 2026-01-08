use proc_macro2::TokenStream;

use super::types::PathParamInfo;

/// Extract path parameter information from function tokens
/// Looks for patterns like:
/// - `Path(id): Path<u64>` (single primitive)
/// - `Path((a, b)): Path<(Uuid, u32)>` (tuple)
/// - `Path(MyStruct { id }): Path<MyStruct>` (struct destructuring)
/// - Multiple Path extractors: `Path(id): Path<Uuid>, Path(name): Path<String>`
pub fn extract_path_info(tokens: &TokenStream) -> Option<PathParamInfo> {
    let token_str = tokens.to_string();

    let mut all_bindings = Vec::new();
    let mut all_types = Vec::new();
    let mut any_struct_pattern = false;
    let mut search_start = 0;

    // Find ALL Path() patterns in the token stream
    while let Some(rel_pos) = find_path_pattern(&token_str[search_start..]) {
        let path_pos = search_start + rel_pos;

        // Determine skip length based on whether there's a space
        let skip_len = if token_str[path_pos..].starts_with("Path(") {
            5 // "Path("
        } else {
            6 // "Path ("
        };

        let after_open = &token_str[path_pos + skip_len..];

        // Find matching closing paren for the binding
        let mut depth = 1;
        let mut close_pos = 0;
        for (i, ch) in after_open.chars().enumerate() {
            match ch {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        close_pos = i;
                        break;
                    }
                }
                _ => {}
            }
        }

        if close_pos == 0 && depth != 0 {
            search_start = path_pos + skip_len;
            continue;
        }

        let binding_content = after_open[..close_pos].trim();

        // Check if this is a struct destructuring pattern (contains '{')
        if binding_content.contains('{') {
            any_struct_pattern = true;
        }

        // Extract bindings from this Path extractor
        if !binding_content.contains('{') {
            if binding_content.starts_with('(') {
                // Tuple pattern like "(a, b)"
                let inner = binding_content
                    .trim_start_matches('(')
                    .trim_end_matches(')');
                for s in inner.split(',') {
                    let binding = s.trim().to_string();
                    if !binding.is_empty() && !all_bindings.contains(&binding) {
                        all_bindings.push(binding);
                    }
                }
            } else if !binding_content.is_empty() {
                // Single binding like "id"
                let binding = binding_content.to_string();
                if !all_bindings.contains(&binding) {
                    all_bindings.push(binding);
                }
            }
        }

        // Extract the type from Path<Type>
        let rest = &after_open[close_pos..];
        if let Some(type_start) = rest.find("Path") {
            let after_type_path = &rest[type_start + 4..];
            if let Some(angle_open) = after_type_path.find('<') {
                let after_angle = &after_type_path[angle_open + 1..];

                // Find matching closing angle bracket
                let mut depth = 1;
                let mut type_end = 0;
                for (i, ch) in after_angle.chars().enumerate() {
                    match ch {
                        '<' => depth += 1,
                        '>' => {
                            depth -= 1;
                            if depth == 0 {
                                type_end = i;
                                break;
                            }
                        }
                        _ => {}
                    }
                }

                if type_end > 0 || depth == 0 {
                    let inner_type = after_angle[..type_end].trim().to_string();
                    if !inner_type.is_empty() {
                        all_types.push(inner_type);
                    }
                }
            }
        }

        // Move past this Path() for the next iteration
        search_start = path_pos + skip_len + close_pos;
    }

    if all_bindings.is_empty() && !any_struct_pattern {
        return None;
    }

    // Combine types - use tuple format if multiple, single if one
    let inner_type = match all_types.len().cmp(&1) {
        std::cmp::Ordering::Equal => all_types.into_iter().next().unwrap_or_default(),
        std::cmp::Ordering::Greater => format!("({})", all_types.join(", ")),
        std::cmp::Ordering::Less => String::new(),
    };

    Some(PathParamInfo {
        bindings: all_bindings,
        inner_type,
        is_struct_pattern: any_struct_pattern,
    })
}

/// Find the next "Path(" or "Path (" pattern in a string
fn find_path_pattern(s: &str) -> Option<usize> {
    // Find "Path(" or "Path (" but not "Path<" (which is the type, not binding)
    let mut search_from = 0;
    while let Some(pos) = s[search_from..].find("Path") {
        let abs_pos = search_from + pos;
        let after = &s[abs_pos + 4..];

        if after.starts_with('(') || after.starts_with(" (") {
            return Some(abs_pos);
        }

        // Skip this "Path" and continue searching
        search_from = abs_pos + 4;
    }
    None
}

/// Extract the state type from State<T> in function parameters
/// Returns None if no State extractor is found (meaning state type is ())
pub fn extract_state_type(tokens: &TokenStream) -> Option<TokenStream> {
    let token_str = tokens.to_string();

    // Look for pattern "State < SomeType >"
    // This is a simplified parser that looks for State<...>
    if let Some(state_pos) = token_str.find("State") {
        let after_state = &token_str[state_pos..];
        if let Some(open_bracket) = after_state.find('<') {
            // Find the matching closing bracket
            let after_open = &after_state[open_bracket + 1..];
            let mut depth = 1;
            let mut close_pos = 0;

            for (i, ch) in after_open.chars().enumerate() {
                if ch == '<' {
                    depth += 1;
                } else if ch == '>' {
                    depth -= 1;
                    if depth == 0 {
                        close_pos = i;
                        break;
                    }
                }
            }

            if close_pos > 0 {
                let state_type_str = &after_open[..close_pos].trim();
                // Parse the extracted string back into a TokenStream
                if let Ok(state_type) = state_type_str.parse::<TokenStream>() {
                    return Some(state_type);
                }
            }
        }
    }

    None
}

/// Extract doc comment text from an attribute string
pub fn extract_doc_text(attr: &str) -> String {
    // Parse doc = "text" format
    if let Some(start) = attr.find('"') {
        if let Some(end) = attr.rfind('"') {
            if start < end {
                return attr[start + 1..end].to_string();
            }
        }
    }
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Path parameter extraction tests

    #[test]
    fn extracts_single_primitive_binding() {
        let tokens: TokenStream = "Path(id): Path<u64>".parse().unwrap();
        eprintln!("Token string: '{}'", tokens.to_string());
        let result = extract_path_info(&tokens);
        assert!(result.is_some());
        let info = result.unwrap();
        assert_eq!(info.bindings, vec!["id"]);
        assert_eq!(info.inner_type, "u64");
        assert!(!info.is_struct_pattern);
    }

    #[test]
    fn extracts_from_full_function() {
        let tokens: TokenStream =
            "async fn get_user_by_u64(Path(id): Path<u64>) -> Json<String> { }"
                .parse()
                .unwrap();
        eprintln!("Full function token string: '{}'", tokens.to_string());
        let result = extract_path_info(&tokens);
        assert!(
            result.is_some(),
            "Should extract path info from full function"
        );
        let info = result.unwrap();
        eprintln!("Bindings: {:?}", info.bindings);
        eprintln!("Inner type: '{}'", info.inner_type);
        eprintln!("Is struct pattern: {}", info.is_struct_pattern);
        assert_eq!(info.bindings, vec!["id"]);
        assert_eq!(info.inner_type, "u64");
    }

    #[test]
    fn extracts_from_function_with_docs() {
        // This simulates what the macro receives - doc comments are attributes
        let code = concat!(
            "#[doc = \"Get user by numeric ID.\"]",
            "#[doc = \"\"]",
            "#[doc = \"# Path Parameters\"]",
            "#[doc = \"\"]",
            "#[doc = \"id: The users numeric identifier\"]",
            "async fn get_user_by_u64(Path(id): Path<u64>) -> Json<String> { }"
        );
        let tokens: TokenStream = code.parse().unwrap();
        eprintln!("With docs token string: '{}'", tokens.to_string());
        let result = extract_path_info(&tokens);
        assert!(
            result.is_some(),
            "Should extract path info from function with docs"
        );
        let info = result.unwrap();
        eprintln!("Bindings: {:?}", info.bindings);
        eprintln!("Inner type: '{}'", info.inner_type);
        assert_eq!(info.bindings, vec!["id"]);
        assert_eq!(info.inner_type, "u64");
    }

    #[test]
    fn test_parse_rovo_function_returns_path_params() {
        use crate::parser::parse_rovo_function;

        let code = concat!(
            "#[doc = \"Get user by numeric ID.\"]",
            "#[doc = \"\"]",
            "#[doc = \"# Path Parameters\"]",
            "#[doc = \"\"]",
            "#[doc = \"id: The users numeric identifier\"]",
            "#[doc = \"\"]",
            "#[doc = \"# Responses\"]",
            "#[doc = \"\"]",
            "#[doc = \"200: Json<String> - User found\"]",
            "async fn get_user_by_u64(Path(id): Path<u64>) -> Json<String> { }"
        );
        let tokens: TokenStream = code.parse().unwrap();

        let result = parse_rovo_function(tokens);
        assert!(result.is_ok(), "Should parse successfully");

        let (func_item, doc_info) = result.unwrap();

        eprintln!("func_item.path_params: {:?}", func_item.path_params);
        eprintln!("doc_info.path_params: {:?}", doc_info.path_params);

        assert!(
            func_item.path_params.is_some(),
            "Should have path_params in func_item"
        );
        let path_info = func_item.path_params.unwrap();
        assert_eq!(path_info.bindings, vec!["id"]);
        assert!(!path_info.is_struct_pattern);

        assert_eq!(
            doc_info.path_params.len(),
            1,
            "Should have one path param doc"
        );
        assert_eq!(doc_info.path_params[0].name, "id");
    }

    #[test]
    fn extracts_string_path_binding() {
        let tokens: TokenStream = "Path(username): Path<String>".parse().unwrap();
        let result = extract_path_info(&tokens);
        assert!(result.is_some());
        let info = result.unwrap();
        assert_eq!(info.bindings, vec!["username"]);
        assert_eq!(info.inner_type, "String");
        assert!(!info.is_struct_pattern);
    }

    #[test]
    fn extracts_tuple_bindings() {
        let tokens: TokenStream = "Path((collection_id, index)): Path<(Uuid, u32)>"
            .parse()
            .unwrap();
        let result = extract_path_info(&tokens);
        assert!(result.is_some());
        let info = result.unwrap();
        assert_eq!(info.bindings, vec!["collection_id", "index"]);
        assert!(info.inner_type.contains("Uuid"));
        assert!(info.inner_type.contains("u32"));
        assert!(!info.is_struct_pattern);
    }

    #[test]
    fn detects_struct_pattern() {
        let tokens: TokenStream = "Path(UserId { id }): Path<UserId>".parse().unwrap();
        let result = extract_path_info(&tokens);
        assert!(result.is_some());
        let info = result.unwrap();
        assert!(info.is_struct_pattern);
        assert!(info.bindings.is_empty()); // Don't extract bindings for struct patterns
    }

    #[test]
    fn handles_path_with_state() {
        let tokens: TokenStream = "State(_state): State<AppState>, Path(id): Path<String>"
            .parse()
            .unwrap();
        let result = extract_path_info(&tokens);
        assert!(result.is_some());
        let info = result.unwrap();
        assert_eq!(info.bindings, vec!["id"]);
        assert_eq!(info.inner_type, "String");
    }

    #[test]
    fn handles_multiple_path_extractors() {
        let tokens: TokenStream = "Path(id): Path<Uuid>, Path(name): Path<String>"
            .parse()
            .unwrap();
        eprintln!("Multiple path extractors: '{}'", tokens.to_string());
        let result = extract_path_info(&tokens);
        assert!(result.is_some(), "Should extract multiple path extractors");
        let info = result.unwrap();
        eprintln!("Bindings: {:?}", info.bindings);
        assert!(
            info.bindings.contains(&"id".to_string()),
            "Should have 'id'"
        );
        assert!(
            info.bindings.contains(&"name".to_string()),
            "Should have 'name'"
        );
        assert_eq!(info.bindings.len(), 2);
    }

    #[test]
    fn handles_multiple_path_extractors_multiline() {
        let tokens: TokenStream = r#"
            async fn get_todo(
                Path(id): Path<Uuid>,
                Path(id2): Path<String>,
            ) -> impl IntoApiResponse { }
        "#
        .parse()
        .unwrap();
        eprintln!("Multiline path extractors: '{}'", tokens.to_string());
        let result = extract_path_info(&tokens);
        assert!(
            result.is_some(),
            "Should extract multiple path extractors from multiline"
        );
        let info = result.unwrap();
        eprintln!("Bindings: {:?}", info.bindings);
        assert!(
            info.bindings.contains(&"id".to_string()),
            "Should have 'id'"
        );
        assert!(
            info.bindings.contains(&"id2".to_string()),
            "Should have 'id2'"
        );
        assert_eq!(info.bindings.len(), 2);
    }

    // State type extraction tests

    #[test]
    fn extracts_simple_state_type() {
        let tokens: TokenStream = "State<AppState>".parse().unwrap();
        let result = extract_state_type(&tokens);
        assert!(result.is_some());
        assert_eq!(result.unwrap().to_string(), "AppState");
    }

    #[test]
    fn extracts_nested_state_type() {
        let tokens: TokenStream = "State<Arc<RwLock<Database>>>".parse().unwrap();
        let result = extract_state_type(&tokens);
        assert!(result.is_some());
        // Token spacing may vary
        let result_str = result.unwrap().to_string();
        assert!(result_str.contains("Arc"));
        assert!(result_str.contains("RwLock"));
        assert!(result_str.contains("Database"));
    }

    #[test]
    fn returns_none_when_no_state() {
        let tokens: TokenStream = "Path<u32>, Json<User>".parse().unwrap();
        let result = extract_state_type(&tokens);
        assert!(result.is_none());
    }

    #[test]
    fn handles_state_with_surrounding_params() {
        let tokens: TokenStream = "Path<u32>, State<AppState>, Json<User>".parse().unwrap();
        let result = extract_state_type(&tokens);
        assert!(result.is_some());
        assert_eq!(result.unwrap().to_string(), "AppState");
    }

    #[test]
    fn extracts_doc_text_simple() {
        let attr = r#"doc = "This is a comment""#;
        let result = extract_doc_text(attr);
        assert_eq!(result, "This is a comment");
    }

    #[test]
    fn extracts_doc_text_with_special_chars() {
        let attr = r#"doc = "Contains 'quotes' and \"escapes\"""#;
        let result = extract_doc_text(attr);
        assert_eq!(result, r#"Contains 'quotes' and \"escapes\""#);
    }

    #[test]
    fn returns_empty_string_when_no_quotes() {
        let attr = "doc = no quotes here";
        let result = extract_doc_text(attr);
        assert_eq!(result, "");
    }

    #[test]
    fn returns_empty_string_when_only_one_quote() {
        let attr = r#"doc = "incomplete"#;
        let result = extract_doc_text(attr);
        assert_eq!(result, "");
    }

    #[test]
    fn handles_empty_doc_text() {
        let attr = r#"doc = """#;
        let result = extract_doc_text(attr);
        assert_eq!(result, "");
    }

    #[test]
    fn extracts_multiline_doc_text() {
        let attr = r#"doc = "Line 1\nLine 2\nLine 3""#;
        let result = extract_doc_text(attr);
        assert_eq!(result, r"Line 1\nLine 2\nLine 3");
    }
}
