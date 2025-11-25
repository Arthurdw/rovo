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

/// Parse response from pre-parsed parts (for Rust-style sections)
pub fn parse_response_from_parts(
    response_type_str: &str,
    status_code: u16,
    description: &str,
    span: Span,
) -> Result<ResponseInfo, ParseError> {
    validate_status_code(status_code, span)?;

    if description.trim().is_empty() {
        return Err(ParseError::with_span(
            "Missing description for response\n\
             help: add a description after the response type\n\
             note: format is '<status>: <type> - <description>'",
            span,
        ));
    }

    let response_type: TokenStream = response_type_str.parse().map_err(|_| {
        ParseError::with_span(
            format!(
                "Invalid response type '{response_type_str}'\n\
                 help: response type must be valid Rust syntax\n\
                 note: common types: Json<T>, (), (StatusCode, Json<T>)"
            ),
            span,
        )
    })?;

    Ok(ResponseInfo {
        status_code,
        response_type,
        description: description.to_string(),
    })
}

/// Parse example from pre-parsed parts (for Rust-style sections)
pub fn parse_example_from_parts(
    status_code: u16,
    example_code_str: &str,
    span: Span,
) -> Result<ExampleInfo, ParseError> {
    validate_status_code(status_code, span)?;

    if example_code_str.trim().is_empty() {
        return Err(ParseError::with_span(
            "Empty example expression\n\
             help: provide a valid Rust expression\n\
             note: format is '<status>: <rust_expression>'",
            span,
        ));
    }

    // Unescape quotes that come from doc comments
    let unescaped = example_code_str.replace("\\\"", "\"");

    // First parse as TokenStream
    let example_code: TokenStream = unescaped.parse().map_err(|_| {
        ParseError::with_span(
            format!(
                "Invalid example expression '{example_code_str}'\n\
                 help: expression must be valid Rust syntax\n\
                 note: examples: 'User::default()', 'User {{ id: 1, name: \"Alice\".into() }}', 'vec![1, 2, 3]'"
            ),
            span,
        )
    })?;

    // Validate it's a valid expression using syn
    syn::parse2::<syn::Expr>(example_code.clone()).map_err(|e| {
        ParseError::with_span(
            format!(
                "Invalid example expression '{example_code_str}'\n\
                 help: expression must be valid Rust syntax\n\
                 note: parse error: {e}\n\
                 note: examples: 'User::default()', 'User {{ id: 1, name: \"Alice\".into() }}', 'vec![1, 2, 3]'"
            ),
            span,
        )
    })?;

    Ok(ExampleInfo {
        status_code,
        example_code,
        span,
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

#[cfg(test)]
mod tests {
    use super::*;

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

    // Tests for parse_response_from_parts

    #[test]
    fn response_from_parts_valid() {
        let result = parse_response_from_parts("Json<User>", 200, "Success", Span::call_site());
        assert!(result.is_ok());
        let info = result.unwrap();
        assert_eq!(info.status_code, 200);
        assert_eq!(info.description, "Success");
    }

    #[test]
    fn response_from_parts_empty_description() {
        let result = parse_response_from_parts("Json<User>", 200, "", Span::call_site());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Missing description"));
    }

    #[test]
    fn response_from_parts_whitespace_description() {
        let result = parse_response_from_parts("Json<User>", 200, "   ", Span::call_site());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Missing description"));
    }

    #[test]
    fn response_from_parts_invalid_status() {
        let result = parse_response_from_parts("Json<User>", 999, "Success", Span::call_site());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("out of valid range"));
    }

    #[test]
    fn response_from_parts_unit_type() {
        let result = parse_response_from_parts("()", 204, "No content", Span::call_site());
        assert!(result.is_ok());
    }

    // Tests for parse_example_from_parts

    #[test]
    fn example_from_parts_valid() {
        let result = parse_example_from_parts(200, "User::default()", Span::call_site());
        assert!(result.is_ok());
        let info = result.unwrap();
        assert_eq!(info.status_code, 200);
    }

    #[test]
    fn example_from_parts_empty_code() {
        let result = parse_example_from_parts(200, "", Span::call_site());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Empty example expression"));
    }

    #[test]
    fn example_from_parts_whitespace_code() {
        let result = parse_example_from_parts(200, "   ", Span::call_site());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Empty example expression"));
    }

    #[test]
    fn example_from_parts_invalid_status() {
        let result = parse_example_from_parts(999, "User::default()", Span::call_site());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("out of valid range"));
    }

    #[test]
    fn example_from_parts_invalid_syntax() {
        let result = parse_example_from_parts(200, "User{", Span::call_site());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid example expression"));
    }

    #[test]
    fn example_from_parts_struct_expression() {
        let result = parse_example_from_parts(
            200,
            "User { id: 1, name: \"Test\".into() }",
            Span::call_site(),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn example_from_parts_escaped_quotes() {
        let result =
            parse_example_from_parts(200, "User { name: \\\"Test\\\".into() }", Span::call_site());
        assert!(result.is_ok());
    }

    #[test]
    fn example_from_parts_vec_expression() {
        let result = parse_example_from_parts(200, "vec![1, 2, 3]", Span::call_site());
        assert!(result.is_ok());
    }

    #[test]
    fn example_from_parts_method_chain() {
        let result = parse_example_from_parts(200, "User::new().with_id(1)", Span::call_site());
        assert!(result.is_ok());
    }

    // Additional edge case tests for simple annotations

    #[test]
    fn tag_with_extra_spaces() {
        let result = parse_tag("@tag   users", Span::call_site());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "users");
    }

    #[test]
    fn security_with_extra_spaces() {
        let result = parse_security("@security   bearer", Span::call_site());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "bearer");
    }

    #[test]
    fn id_requires_value() {
        let result = parse_id("@id", Span::call_site());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid @id"));
    }

    #[test]
    fn id_rejects_empty_value() {
        let result = parse_id("@id  ", Span::call_site());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Empty"));
    }

    #[test]
    fn security_rejects_empty_value() {
        let result = parse_security("@security  ", Span::call_site());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Empty"));
    }
}
