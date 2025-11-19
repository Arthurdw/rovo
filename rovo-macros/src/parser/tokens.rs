use proc_macro2::TokenStream;

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
