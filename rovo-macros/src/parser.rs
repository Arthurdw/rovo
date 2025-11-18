use proc_macro2::{Ident, Span, TokenStream, TokenTree};
use quote::ToTokens;
use std::fmt;

/// Calculate Levenshtein distance between two strings
fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let len1 = s1.len();
    let len2 = s2.len();
    let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

    for i in 0..=len1 {
        matrix[i][0] = i;
    }
    for j in 0..=len2 {
        matrix[0][j] = j;
    }

    for (i, c1) in s1.chars().enumerate() {
        for (j, c2) in s2.chars().enumerate() {
            let cost = if c1 == c2 { 0 } else { 1 };
            matrix[i + 1][j + 1] = (matrix[i][j + 1] + 1)
                .min(matrix[i + 1][j] + 1)
                .min(matrix[i][j] + cost);
        }
    }

    matrix[len1][len2]
}

/// Find the closest matching annotation
fn find_closest_annotation(input: &str) -> Option<&'static str> {
    const ANNOTATIONS: &[&str] = &[
        "response",
        "example",
        "tag",
        "security",
        "id",
        "hidden",
        "rovo-ignore",
    ];

    let input_lower = input.to_lowercase();
    let mut best_match = None;
    let mut best_distance = usize::MAX;

    for &annotation in ANNOTATIONS {
        let distance = levenshtein_distance(&input_lower, annotation);
        // Only suggest if distance is small (≤ 2 characters different)
        if distance < best_distance && distance <= 2 {
            best_distance = distance;
            best_match = Some(annotation);
        }
    }

    best_match
}

#[derive(Debug)]
pub struct ParseError {
    message: String,
    span: Option<Span>,
}

impl ParseError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            span: None,
        }
    }

    fn with_span(message: impl Into<String>, span: Span) -> Self {
        Self {
            message: message.into(),
            span: Some(span),
        }
    }

    pub fn span(&self) -> Option<Span> {
        self.span
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
    pub tags: Vec<String>,
    pub deprecated: bool,
    pub security_requirements: Vec<String>,
    pub operation_id: Option<String>,
    pub hidden: bool,
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

struct DocLine {
    text: String,
    span: Span,
}

pub fn parse_rovo_function(input: TokenStream) -> Result<(FuncItem, DocInfo), ParseError> {
    let tokens: Vec<TokenTree> = input.clone().into_iter().collect();

    // Extract doc comments, attributes, and function name
    let mut doc_lines = Vec::new();
    let mut func_name = None;
    let mut is_deprecated = false;
    let mut i = 0;

    while i < tokens.len() {
        match &tokens[i] {
            TokenTree::Punct(p) if p.as_char() == '#' => {
                // Check if this is an attribute
                if i + 1 < tokens.len() {
                    if let TokenTree::Group(group) = &tokens[i + 1] {
                        let attr_content = group.stream().to_string();
                        if attr_content.starts_with("doc") {
                            // Extract the doc comment text and preserve the span
                            let doc_text = extract_doc_text(&attr_content);
                            let span = group.span();
                            doc_lines.push(DocLine {
                                text: doc_text,
                                span,
                            });
                        } else if attr_content.starts_with("deprecated") {
                            // Mark as deprecated
                            is_deprecated = true;
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

    // Parse doc comments - pass function name and spans for better error messages
    let mut doc_info = parse_doc_comments(&doc_lines, &func_name.to_string())?;

    // Set deprecated flag from Rust attribute
    doc_info.deprecated = is_deprecated;

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

fn parse_doc_comments(lines: &[DocLine], _func_name: &str) -> Result<DocInfo, ParseError> {
    let mut doc_info = DocInfo::default();
    let mut description_lines = Vec::new();
    let mut in_description = false;
    let mut title_set = false;

    for doc_line in lines.iter() {
        let trimmed = doc_line.text.trim();
        let span = doc_line.span;

        if trimmed.starts_with("@response") {
            // Parse: @response 200 Json<TodoItem> A single Todo item.
            let parts: Vec<&str> = trimmed.splitn(4, ' ').collect();

            if parts.len() < 4 {
                return Err(ParseError::with_span(
                    format!(
                        "Invalid @response annotation format\n\
                         help: expected '@response <code> <type> <description>'\n\
                         note: example '@response 200 Json<User> Successfully retrieved user'"
                    ),
                    span,
                ));
            }

            let status_code = parts[1]
                .parse::<u16>()
                .map_err(|_| ParseError::with_span(
                    format!(
                        "Invalid status code '{}'\n\
                         help: status code must be a number between 100-599\n\
                         note: common codes: 200 (OK), 201 (Created), 400 (Bad Request), 404 (Not Found), 500 (Internal Error)",
                        parts[1]
                    ),
                    span
                ))?;

            // Validate status code is in valid HTTP range
            if !(100..=599).contains(&status_code) {
                return Err(ParseError::with_span(
                    format!(
                        "Status code {} is out of valid range\n\
                         help: HTTP status codes must be between 100-599\n\
                         note: common codes: 200 (OK), 201 (Created), 400 (Bad Request), 404 (Not Found), 500 (Internal Error)",
                        status_code
                    ),
                    span
                ));
            }

            let response_type_str = parts[2];
            let description = parts[3].to_string();

            if description.trim().is_empty() {
                return Err(ParseError::with_span(
                    format!(
                        "Missing description for @response\n\
                         help: add a description after the response type\n\
                         note: example '@response {} {} Successfully created resource'",
                        status_code, response_type_str
                    ),
                    span,
                ));
            }

            // Check if the "type" looks like it's actually part of the description
            // (common mistake: forgetting to include the type)
            let looks_like_description = {
                // List of common words that suggest this is a description, not a type
                let description_words = [
                    "item",
                    "deleted",
                    "successfully",
                    "created",
                    "updated",
                    "not",
                    "error",
                    "failed",
                    "success",
                    "the",
                    "a",
                    "an",
                    "user",
                    "data",
                    "resource",
                    "found",
                    "missing",
                    "invalid",
                    "request",
                    "response",
                ];

                let type_lower = response_type_str.to_lowercase();
                description_words.iter().any(|&word| type_lower == word)
                    // Or if it's a simple word (no type syntax) and description starts lowercase
                    || (!response_type_str.contains('<')
                        && !response_type_str.contains('(')
                        && !response_type_str.contains(')')
                        && !response_type_str.contains("::")
                        && description.chars().next().map_or(false, |c| c.is_lowercase()))
            };

            if looks_like_description {
                return Err(ParseError::with_span(
                    format!(
                        "Missing response type in @response annotation\n\
                         help: format is '@response <code> <type> <description>'\n\
                         note: did you forget to add the type? For example:\n\
                         note:   '@response {} () {}'\n\
                         note: common types: () for empty responses, Json<T> for JSON, (StatusCode, Json<T>) for custom status",
                        status_code,
                        trimmed.trim_start_matches("@response").trim_start_matches(char::is_whitespace).trim_start_matches(parts[1]).trim()
                    ),
                    span
                ));
            }

            // Parse the response type string into a TokenStream
            let response_type: TokenStream = response_type_str
                .parse()
                .map_err(|_| {
                    ParseError::with_span(
                        format!(
                            "Invalid response type '{}'\n\
                             help: response type must be valid Rust syntax\n\
                             note: common types: Json<T>, (), (StatusCode, Json<T>)\n\
                             note: if this is a description, you may have forgotten the type parameter",
                            response_type_str
                        ),
                        span
                    )
                })?;

            doc_info.responses.push(ResponseInfo {
                status_code,
                response_type,
                description,
            });
        } else if trimmed.starts_with("@example") {
            // Parse: @example 200 TodoItem::default()
            let parts: Vec<&str> = trimmed.splitn(3, ' ').collect();

            if parts.len() < 3 {
                return Err(ParseError::with_span(
                    format!(
                        "Invalid @example annotation format\n\
                         help: expected '@example <code> <expression>'\n\
                         note: example '@example 200 User::default()' or '@example 201 User {{ id: 1, name: \"Alice\".into() }}'"
                    ),
                    span
                ));
            }

            let status_code = parts[1]
                .parse::<u16>()
                .map_err(|_| ParseError::with_span(
                    format!(
                        "Invalid status code '{}'\n\
                         help: status code must be a number between 100-599\n\
                         note: common codes: 200 (OK), 201 (Created), 400 (Bad Request), 404 (Not Found), 500 (Internal Error)",
                        parts[1]
                    ),
                    span
                ))?;

            // Validate status code is in valid HTTP range
            if !(100..=599).contains(&status_code) {
                return Err(ParseError::with_span(
                    format!(
                        "Status code {} is out of valid range\n\
                         help: HTTP status codes must be between 100-599\n\
                         note: common codes: 200 (OK), 201 (Created), 400 (Bad Request), 404 (Not Found), 500 (Internal Error)",
                        status_code
                    ),
                    span
                ));
            }

            let example_code_str = parts[2];

            if example_code_str.trim().is_empty() {
                return Err(ParseError::with_span(
                    format!(
                        "Empty example expression in @example annotation\n\
                         help: provide a valid Rust expression after the status code\n\
                         note: example '@example {} User::default()' or '@example {} User {{ id: 1, name: \"Alice\".into() }}'",
                        status_code, status_code
                    ),
                    span
                ));
            }

            // Parse the example code string into a TokenStream
            let example_code: TokenStream = example_code_str
                .parse()
                .map_err(|_| {
                    ParseError::with_span(
                        format!(
                            "Invalid example expression '{}'\n\
                             help: expression must be valid Rust syntax\n\
                             note: examples: 'User::default()', 'User {{ id: 1, name: \"Alice\".into() }}', 'vec![1, 2, 3]'",
                            example_code_str
                        ),
                        span
                    )
                })?;

            doc_info.examples.push(ExampleInfo {
                status_code,
                example_code,
            });
        } else if trimmed.starts_with("@tag") {
            // Parse: @tag <tag_name>
            let parts: Vec<&str> = trimmed.splitn(2, ' ').collect();

            if parts.len() < 2 {
                return Err(ParseError::with_span(
                    format!(
                        "Invalid @tag annotation format\n\
                         help: expected '@tag <tag_name>'\n\
                         note: example '@tag users' or '@tag authentication'"
                    ),
                    span,
                ));
            }

            let tag = parts[1].trim();
            if tag.is_empty() {
                return Err(ParseError::with_span(
                    format!(
                        "Empty tag name in @tag annotation\n\
                         help: provide a tag name after @tag\n\
                         note: tags help organize endpoints in the OpenAPI documentation"
                    ),
                    span,
                ));
            }

            doc_info.tags.push(tag.to_string());
        } else if trimmed.starts_with("@security") {
            // Parse: @security <scheme_name>
            let parts: Vec<&str> = trimmed.splitn(2, ' ').collect();

            if parts.len() < 2 {
                return Err(ParseError::with_span(
                    format!(
                        "Invalid @security annotation format\n\
                         help: expected '@security <scheme_name>'\n\
                         note: example '@security bearer_auth' or '@security api_key'\n\
                         note: security schemes must be defined separately in your OpenAPI spec"
                    ),
                    span,
                ));
            }

            let scheme = parts[1].trim();
            if scheme.is_empty() {
                return Err(ParseError::with_span(
                    format!(
                        "Empty security scheme name in @security annotation\n\
                         help: provide a security scheme name after @security\n\
                         note: the scheme must match a security definition in your OpenAPI spec"
                    ),
                    span,
                ));
            }

            doc_info.security_requirements.push(scheme.to_string());
        } else if trimmed.starts_with("@id") {
            // Parse: @id <operation_id>
            let parts: Vec<&str> = trimmed.splitn(2, ' ').collect();

            if parts.len() < 2 {
                return Err(ParseError::with_span(
                    format!(
                        "Invalid @id annotation format\n\
                         help: expected '@id <operation_id>'\n\
                         note: example '@id getUserById' or '@id create_user'\n\
                         note: operation IDs help identify endpoints in generated clients"
                    ),
                    span,
                ));
            }

            let id = parts[1].trim();
            if id.is_empty() {
                return Err(ParseError::with_span(
                    format!(
                        "Empty operation ID in @id annotation\n\
                         help: provide an operation ID after @id\n\
                         note: operation IDs must be unique across all endpoints"
                    ),
                    span,
                ));
            }

            // Validate that operation ID is a valid identifier (alphanumeric + underscores)
            if !id.chars().all(|c| c.is_alphanumeric() || c == '_') {
                return Err(ParseError::with_span(
                    format!(
                        "Invalid operation ID '{}'\n\
                         help: operation IDs must contain only alphanumeric characters and underscores\n\
                         note: valid examples: 'getUserById', 'create_user', 'deleteItem123'",
                        id
                    ),
                    span
                ));
            }

            doc_info.operation_id = Some(id.to_string());
        } else if trimmed == "@hidden" {
            // Mark as hidden
            doc_info.hidden = true;
        } else if trimmed == "@rovo-ignore" {
            // Stop processing further doc comments
            break;
        } else if trimmed.starts_with('@') {
            // Unknown annotation
            let annotation = trimmed.split_whitespace().next().unwrap_or(trimmed);
            let annotation_name = annotation.strip_prefix('@').unwrap_or(annotation);

            let error_msg = if let Some(suggestion) = find_closest_annotation(annotation_name) {
                format!(
                    "Unknown annotation '{}'\n\
                     help: did you mean '@{}'?\n\
                     note: valid annotations are @response, @example, @tag, @security, @id, @hidden, @rovo-ignore",
                    annotation, suggestion
                )
            } else {
                format!(
                    "Unknown annotation '{}'\n\
                     note: valid annotations are @response, @example, @tag, @security, @id, @hidden, @rovo-ignore",
                    annotation
                )
            };

            return Err(ParseError::with_span(error_msg, span));
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
