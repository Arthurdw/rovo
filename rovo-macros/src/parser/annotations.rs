use super::error::ParseError;
use super::types::{ExampleInfo, ResponseInfo};
use proc_macro2::{Span, TokenStream};

/// Macro to parse simple annotations with format: @name <value>
macro_rules! parse_simple_annotation {
    ($trimmed:expr, $span:expr, $name:literal, $help:literal, $example:literal) => {{
        let parts: Vec<&str> = $trimmed.splitn(2, ' ').collect();

        if parts.len() < 2 {
            return Err(ParseError::with_span(
                concat!(
                    "Invalid @",
                    $name,
                    " annotation format\n\
                     help: expected '@",
                    $name,
                    " ",
                    $help,
                    "'\n\
                     note: example '@",
                    $name,
                    " ",
                    $example,
                    "'"
                ),
                $span,
            ));
        }

        let value = parts[1].trim();
        if value.is_empty() {
            return Err(ParseError::with_span(
                concat!(
                    "Empty ",
                    $help,
                    " in @",
                    $name,
                    " annotation\n\
                     help: provide a ",
                    $help,
                    " after @",
                    $name
                ),
                $span,
            ));
        }

        value.to_string()
    }};
}

/// Macro to parse and validate status codes
macro_rules! parse_status_code {
    ($code_str:expr, $span:expr) => {{
        let status_code = $code_str.parse::<u16>().map_err(|_| {
            ParseError::with_span(
                format!(
                    "Invalid status code '{}'\n\
                     help: status code must be a number between 100-599\n\
                     note: common codes: 200 (OK), 201 (Created), 400 (Bad Request), 404 (Not Found), 500 (Internal Error)",
                    $code_str
                ),
                $span,
            )
        })?;

        validate_status_code(status_code, $span)?;
        status_code
    }};
}

/// Parse @response annotation
pub fn parse_response(trimmed: &str, span: Span) -> Result<ResponseInfo, ParseError> {
    let parts: Vec<&str> = trimmed.splitn(4, ' ').collect();

    if parts.len() < 4 {
        return Err(ParseError::with_span(
            "Invalid @response annotation format\n\
             help: expected '@response <code> <type> <description>'\n\
             note: example '@response 200 Json<User> Successfully retrieved user'",
            span,
        ));
    }

    let status_code = parse_status_code!(parts[1], span);

    let response_type_str = parts[2];
    let description = parts[3].to_string();

    if description.trim().is_empty() {
        return Err(ParseError::with_span(
            format!(
                "Missing description for @response\n\
                 help: add a description after the response type\n\
                 note: example '@response {status_code} {response_type_str} Successfully created resource'"
            ),
            span,
        ));
    }

    // Check if the "type" looks like it's actually part of the description
    if looks_like_description(response_type_str, &description) {
        let rest = trimmed
            .trim_start_matches("@response")
            .trim_start_matches(char::is_whitespace)
            .trim_start_matches(parts[1])
            .trim();
        return Err(ParseError::with_span(
            format!(
                "Missing response type in @response annotation\n\
                 help: format is '@response <code> <type> <description>'\n\
                 note: did you forget to add the type? For example:\n\
                 note:   '@response {status_code} () {rest}'\n\
                 note: common types: () for empty responses, Json<T> for JSON, (StatusCode, Json<T>) for custom status"
            ),
            span,
        ));
    }

    let response_type: TokenStream = response_type_str.parse().map_err(|_| {
        ParseError::with_span(
            format!(
                "Invalid response type '{response_type_str}'\n\
                 help: response type must be valid Rust syntax\n\
                 note: common types: Json<T>, (), (StatusCode, Json<T>)\n\
                 note: if this is a description, you may have forgotten the type parameter"
            ),
            span,
        )
    })?;

    Ok(ResponseInfo {
        status_code,
        response_type,
        description,
    })
}

/// Parse @example annotation
pub fn parse_example(trimmed: &str, span: Span) -> Result<ExampleInfo, ParseError> {
    let parts: Vec<&str> = trimmed.splitn(3, ' ').collect();

    if parts.len() < 3 {
        return Err(ParseError::with_span(
            "Invalid @example annotation format\n\
             help: expected '@example <code> <expression>'\n\
             note: example '@example 200 User::default()' or '@example 201 User {{ id: 1, name: \"Alice\".into() }}'",
            span,
        ));
    }

    let status_code = parse_status_code!(parts[1], span);

    let example_code_str = parts[2];

    if example_code_str.trim().is_empty() {
        return Err(ParseError::with_span(
            format!(
                "Empty example expression in @example annotation\n\
                 help: provide a valid Rust expression after the status code\n\
                 note: example '@example {status_code} User::default()' or '@example {status_code} User {{ id: 1, name: \"Alice\".into() }}'"
            ),
            span,
        ));
    }

    let example_code: TokenStream = example_code_str.parse().map_err(|_| {
        ParseError::with_span(
            format!(
                "Invalid example expression '{example_code_str}'\n\
                 help: expression must be valid Rust syntax\n\
                 note: examples: 'User::default()', 'User {{ id: 1, name: \"Alice\".into() }}', 'vec![1, 2, 3]'"
            ),
            span,
        )
    })?;

    Ok(ExampleInfo {
        status_code,
        example_code,
    })
}

/// Parse @tag annotation
pub fn parse_tag(trimmed: &str, span: Span) -> Result<String, ParseError> {
    Ok(parse_simple_annotation!(trimmed, span, "tag", "<tag_name>", "users"))
}

/// Parse @security annotation
pub fn parse_security(trimmed: &str, span: Span) -> Result<String, ParseError> {
    Ok(parse_simple_annotation!(trimmed, span, "security", "<scheme_name>", "bearer_auth"))
}

/// Parse @id annotation
pub fn parse_id(trimmed: &str, span: Span) -> Result<String, ParseError> {
    let id = parse_simple_annotation!(trimmed, span, "id", "<operation_id>", "getUserById");

    if !id.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err(ParseError::with_span(
            format!(
                "Invalid operation ID '{id}'\n\
                 help: operation IDs must contain only alphanumeric characters and underscores\n\
                 note: valid examples: 'getUserById', 'create_user', 'deleteItem123'"
            ),
            span,
        ));
    }

    Ok(id)
}

/// Validate HTTP status code
fn validate_status_code(status_code: u16, span: Span) -> Result<(), ParseError> {
    if (100..=599).contains(&status_code) {
        Ok(())
    } else {
        Err(ParseError::with_span(
            format!(
                "Status code {status_code} is out of valid range\n\
                 help: HTTP status codes must be between 100-599\n\
                 note: common codes: 200 (OK), 201 (Created), 400 (Bad Request), 404 (Not Found), 500 (Internal Error)"
            ),
            span,
        ))
    }
}

/// Check if a string looks like a description rather than a type
fn looks_like_description(response_type_str: &str, description: &str) -> bool {
    const DESCRIPTION_WORDS: &[&str] = &[
        "item", "deleted", "successfully", "created", "updated", "not", "error",
        "failed", "success", "the", "a", "an", "user", "data", "resource",
        "found", "missing", "invalid", "request", "response",
    ];

    let type_lower = response_type_str.to_lowercase();
    DESCRIPTION_WORDS.iter().any(|&word| type_lower == word)
        || (!response_type_str.contains('<')
            && !response_type_str.contains('(')
            && !response_type_str.contains(')')
            && !response_type_str.contains("::")
            && description.chars().next().is_some_and(char::is_lowercase))
}
