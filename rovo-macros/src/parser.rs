use proc_macro2::{Ident, TokenStream, TokenTree};
use quote::ToTokens;
use std::fmt;

#[derive(Debug)]
pub struct ParseError {
    message: String,
}

impl ParseError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

#[derive(Debug, Clone)]
pub struct ResponseInfo {
    pub status_code: u16,
    pub response_type: TokenStream,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct ExampleInfo {
    pub status_code: u16,
    pub example_code: TokenStream,
}

#[derive(Debug, Clone, Default)]
pub struct DocInfo {
    pub title: Option<String>,
    pub description: Option<String>,
    pub responses: Vec<ResponseInfo>,
    pub examples: Vec<ExampleInfo>,
}

#[derive(Clone)]
pub struct FuncItem {
    pub name: Ident,
    pub tokens: TokenStream,
    pub state_type: Option<TokenStream>,
}

impl FuncItem {
    /// Generate a renamed version of this function with pub visibility
    pub fn with_renamed(&self, new_name: &Ident) -> TokenStream {
        let tokens: Vec<TokenTree> = self.tokens.clone().into_iter().collect();
        let mut result = Vec::new();
        let mut i = 0;
        let mut found_fn = false;
        let mut added_pub = false;

        while i < tokens.len() {
            if let TokenTree::Ident(ident) = &tokens[i] {
                // Check if we should add pub before async or fn
                if !added_pub && (ident.to_string() == "async" || ident.to_string() == "fn") {
                    // Look back to see if pub already exists
                    let has_pub = if i > 0 {
                        if let TokenTree::Ident(prev) = &tokens[i - 1] {
                            prev.to_string() == "pub"
                        } else {
                            false
                        }
                    } else {
                        false
                    };

                    if !has_pub {
                        let pub_ident = Ident::new("pub", ident.span());
                        result.push(TokenTree::Ident(pub_ident));
                    }
                    added_pub = true;
                }

                // Handle function name replacement
                if ident.to_string() == "fn" && !found_fn {
                    // Found 'fn', add it
                    result.push(tokens[i].clone());
                    i += 1;
                    found_fn = true;
                    // Skip the old name and add the new name
                    if i < tokens.len() {
                        if let TokenTree::Ident(_) = &tokens[i] {
                            result.push(TokenTree::Ident(new_name.clone()));
                            i += 1;
                            continue;
                        }
                    }
                    continue;
                }
            }
            result.push(tokens[i].clone());
            i += 1;
        }

        result.into_iter().collect()
    }
}

impl ToTokens for FuncItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.tokens.to_tokens(tokens);
    }
}

pub fn parse_rovo_function(input: TokenStream) -> Result<(FuncItem, DocInfo), ParseError> {
    let tokens: Vec<TokenTree> = input.clone().into_iter().collect();

    // Extract doc comments and function name
    let mut doc_lines = Vec::new();
    let mut func_name = None;
    let mut i = 0;

    while i < tokens.len() {
        match &tokens[i] {
            TokenTree::Punct(p) if p.as_char() == '#' => {
                // Check if this is a doc comment
                if i + 1 < tokens.len() {
                    if let TokenTree::Group(group) = &tokens[i + 1] {
                        let attr_content = group.stream().to_string();
                        if attr_content.starts_with("doc") {
                            // Extract the doc comment text
                            let doc_text = extract_doc_text(&attr_content);
                            doc_lines.push(doc_text);
                        }
                    }
                }
                i += 1;
            }
            TokenTree::Ident(ident) if ident.to_string() == "fn" => {
                // Next token should be the function name
                if i + 1 < tokens.len() {
                    if let TokenTree::Ident(name) = &tokens[i + 1] {
                        func_name = Some(name.clone());
                    }
                }
                break;
            }
            _ => i += 1,
        }
    }

    let func_name = func_name.ok_or_else(|| ParseError::new("Could not find function name"))?;

    // Extract state type from function parameters
    let state_type = extract_state_type(&input);

    // Parse doc comments
    let doc_info = parse_doc_comments(&doc_lines)?;

    let func_item = FuncItem {
        name: func_name,
        tokens: input,
        state_type,
    };

    Ok((func_item, doc_info))
}

/// Extract the state type from State<T> in function parameters
/// Returns None if no State extractor is found (meaning state type is ())
fn extract_state_type(tokens: &TokenStream) -> Option<TokenStream> {
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

fn extract_doc_text(attr: &str) -> String {
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

fn parse_doc_comments(lines: &[String]) -> Result<DocInfo, ParseError> {
    let mut doc_info = DocInfo::default();
    let mut description_lines = Vec::new();
    let mut in_description = false;
    let mut title_set = false;

    for line in lines {
        let trimmed = line.trim();

        if trimmed.starts_with("@response") {
            // Parse: @response 200 Json<TodoItem> A single Todo item.
            let parts: Vec<&str> = trimmed.splitn(4, ' ').collect();
            if parts.len() >= 4 {
                let status_code = parts[1]
                    .parse::<u16>()
                    .map_err(|_| ParseError::new(format!("Invalid status code: {}", parts[1])))?;
                let response_type_str = parts[2];
                let description = parts[3].to_string();

                // Parse the response type string into a TokenStream
                let response_type: TokenStream = response_type_str
                    .parse()
                    .map_err(|_| {
                        ParseError::new(format!("Invalid response type: {}", response_type_str))
                    })?;

                doc_info.responses.push(ResponseInfo {
                    status_code,
                    response_type,
                    description,
                });
            }
        } else if trimmed.starts_with("@example") {
            // Parse: @example 200 TodoItem::default()
            let parts: Vec<&str> = trimmed.splitn(3, ' ').collect();
            if parts.len() >= 3 {
                let status_code = parts[1]
                    .parse::<u16>()
                    .map_err(|_| ParseError::new(format!("Invalid status code: {}", parts[1])))?;
                let example_code_str = parts[2];

                // Parse the example code string into a TokenStream
                let example_code: TokenStream = example_code_str
                    .parse()
                    .map_err(|_| {
                        ParseError::new(format!("Invalid example code: {}", example_code_str))
                    })?;

                doc_info.examples.push(ExampleInfo {
                    status_code,
                    example_code,
                });
            }
        } else if !trimmed.is_empty() {
            if !title_set {
                doc_info.title = Some(trimmed.to_string());
                title_set = true;
            } else {
                in_description = true;
                description_lines.push(trimmed.to_string());
            }
        } else if in_description {
            // Empty line in description - continue collecting
            description_lines.push(String::new());
        }
    }

    if !description_lines.is_empty() {
        doc_info.description = Some(description_lines.join("\n").trim().to_string());
    }

    Ok(doc_info)
}
