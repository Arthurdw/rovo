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
    Ok(parse_simple_annotation!(
        trimmed,
        span,
        "tag",
        "<tag_name>",
        "users"
    ))
}

/// Parse @security annotation
pub fn parse_security(trimmed: &str, span: Span) -> Result<String, ParseError> {
    Ok(parse_simple_annotation!(
        trimmed,
        span,
        "security",
        "<scheme_name>",
        "bearer_auth"
    ))
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

    // Check if it exactly matches a description word (case-sensitive)
    DESCRIPTION_WORDS.contains(&response_type_str)
        || (!response_type_str.contains('<')
            && !response_type_str.contains('(')
            && !response_type_str.contains(')')
            && !response_type_str.contains("::")
            && response_type_str
                .chars()
                .next()
                .is_some_and(char::is_lowercase)
            && description.chars().next().is_some_and(char::is_lowercase))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_response() {
        let result = parse_response(
            "@response 200 Json<User> Successfully retrieved",
            Span::call_site(),
        );
        assert!(result.is_ok());
        let info = result.unwrap();
        assert_eq!(info.status_code, 200);
        assert_eq!(info.description, "Successfully retrieved");
    }

    #[test]
    fn response_requires_all_parts() {
        let result = parse_response("@response 200", Span::call_site());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid @response"));
    }

    #[test]
    fn response_validates_status_code() {
        let result = parse_response("@response 999 Json<User> Test", Span::call_site());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("out of valid range"));
    }

    #[test]
    fn response_rejects_empty_description() {
        let result = parse_response("@response 200 Json<User>  ", Span::call_site());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Missing description"));
    }

    #[test]
    fn response_detects_missing_type() {
        let result = parse_response(
            "@response 200 successfully retrieved user",
            Span::call_site(),
        );
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Missing response type"));
    }

    #[test]
    fn response_handles_complex_types() {
        let result = parse_response(
            "@response 200 (StatusCode,Json<Vec<User>>) Multiple users",
            Span::call_site(),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn parses_valid_example() {
        let result = parse_example("@example 200 User::default()", Span::call_site());
        assert!(result.is_ok());
        let info = result.unwrap();
        assert_eq!(info.status_code, 200);
    }

    #[test]
    fn example_requires_code_and_expression() {
        let result = parse_example("@example 200", Span::call_site());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid @example"));
    }

    #[test]
    fn example_validates_status_code() {
        let result = parse_example("@example 1000 User::default()", Span::call_site());
        assert!(result.is_err());
    }

    #[test]
    fn example_rejects_empty_expression() {
        let result = parse_example("@example 200  ", Span::call_site());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Empty example expression"));
    }

    #[test]
    fn parses_valid_tag() {
        let result = parse_tag("@tag users", Span::call_site());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "users");
    }

    #[test]
    fn tag_requires_value() {
        let result = parse_tag("@tag", Span::call_site());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid @tag"));
    }

    #[test]
    fn tag_rejects_empty_value() {
        let result = parse_tag("@tag  ", Span::call_site());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Empty"));
    }

    #[test]
    fn parses_valid_security() {
        let result = parse_security("@security bearer_auth", Span::call_site());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "bearer_auth");
    }

    #[test]
    fn security_requires_value() {
        let result = parse_security("@security", Span::call_site());
        assert!(result.is_err());
    }

    #[test]
    fn parses_valid_id() {
        let result = parse_id("@id getUserById", Span::call_site());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "getUserById");
    }

    #[test]
    fn id_allows_underscores() {
        let result = parse_id("@id get_user_by_id", Span::call_site());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "get_user_by_id");
    }

    #[test]
    fn id_allows_numbers() {
        let result = parse_id("@id getUser123", Span::call_site());
        assert!(result.is_ok());
    }

    #[test]
    fn id_rejects_special_characters() {
        let result = parse_id("@id get-user", Span::call_site());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid operation ID"));
    }

    #[test]
    fn id_rejects_spaces() {
        let result = parse_id("@id get user", Span::call_site());
        assert!(result.is_err());
    }

    #[test]
    fn validates_status_code_range() {
        assert!(validate_status_code(200, Span::call_site()).is_ok());
        assert!(validate_status_code(100, Span::call_site()).is_ok());
        assert!(validate_status_code(599, Span::call_site()).is_ok());
        assert!(validate_status_code(99, Span::call_site()).is_err());
        assert!(validate_status_code(600, Span::call_site()).is_err());
    }

    #[test]
    fn detects_description_words_as_non_types() {
        assert!(looks_like_description("successfully", "retrieved user"));
        assert!(looks_like_description("error", "occurred"));
        assert!(looks_like_description("user", "not found"));
        assert!(looks_like_description("the", "item was deleted"));
    }

    #[test]
    fn recognizes_valid_types() {
        assert!(!looks_like_description(
            "Json<User>",
            "Successfully retrieved"
        ));
        assert!(!looks_like_description("(StatusCode, Json<T>)", "Created"));
        assert!(!looks_like_description("User::Response", "Success"));
        assert!(!looks_like_description("()", "No content"));
    }

    #[test]
    fn detects_lowercase_start_as_description() {
        assert!(looks_like_description("item", "was created"));
        assert!(!looks_like_description("Item", "Was created")); // Uppercase start
    }

    #[test]
    fn handles_multiword_descriptions_in_response() {
        let result = parse_response(
            "@response 200 Json<User> Successfully retrieved the user data",
            Span::call_site(),
        );
        assert!(result.is_ok());
        let info = result.unwrap();
        assert_eq!(info.description, "Successfully retrieved the user data");
    }

    #[test]
    fn handles_complex_example_expressions() {
        let result = parse_example(
            "@example 201 User { id: 1, name: \"Alice\".into() }",
            Span::call_site(),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn response_handles_unit_type() {
        let result = parse_response("@response 204 () No content", Span::call_site());
        assert!(result.is_ok());
    }

    #[test]
    fn invalid_status_code_string() {
        let result = parse_response("@response abc Json<User> Test", Span::call_site());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid status code"));
    }
}
