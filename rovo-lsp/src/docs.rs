/// Shared documentation for Rovo annotations

/// Get detailed documentation for a Rovo annotation
///
/// # Arguments
/// * `annotation` - The annotation name (e.g., "@response", "@tag")
///
/// # Returns
/// Markdown-formatted documentation string
pub fn get_annotation_documentation(annotation: &str) -> String {
    match annotation {
        "@response" => r#"# @response

Define an API response for different status codes.

## Syntax
```rust
/// @response STATUS TYPE DESCRIPTION
```

## Parameters
- `STATUS`: HTTP status code (100-599)
- `TYPE`: Response type (e.g., `Json<User>`, `Vec<Post>`)
- `DESCRIPTION`: Human-readable description

## Examples
```rust
/// @response 200 Json<User> Successfully retrieved user
/// @response 404 Json<Error> User not found
/// @response 500 Json<Error> Internal server error
```

## Common Status Codes
- `200`: OK - Request succeeded
- `201`: Created - Resource created
- `400`: Bad Request - Invalid input
- `401`: Unauthorized - Authentication required
- `403`: Forbidden - Insufficient permissions
- `404`: Not Found - Resource doesn't exist
- `500`: Internal Server Error - Server error
"#
        .to_string(),

        "@tag" => r#"# @tag

Group related endpoints together in the API documentation.

## Syntax
```rust
/// @tag NAME
```

## Parameters
- `NAME`: Tag name (e.g., `users`, `posts`, `admin`)

## Example
```rust
/// @tag users
/// @response 200 Json<Vec<User>> List of users
#[rovo]
async fn list_users() -> Json<Vec<User>> { ... }
```

Tags help organize your API documentation by grouping related endpoints.
"#
        .to_string(),

        "@security" => r#"# @security

Specify the security scheme required for this endpoint.

## Syntax
```rust
/// @security SCHEME
```

## Parameters
- `SCHEME`: Security scheme name (e.g., `bearer`, `basic`, `apiKey`, `oauth2`)

## Examples
```rust
/// @security bearer
/// @response 200 Json<User> Authenticated user
#[rovo]
async fn get_current_user() -> Json<User> { ... }
```

Common schemes:
- `bearer`: Bearer token authentication
- `basic`: Basic HTTP authentication
- `apiKey`: API key in header/query/cookie
- `oauth2`: OAuth 2.0 authentication
"#
        .to_string(),

        "@example" => r#"# @example

Provide an example response for a specific status code.

## Syntax
```rust
/// @example STATUS JSON
```

## Parameters
- `STATUS`: HTTP status code matching a `@response`
- `JSON`: Example JSON response

## Example
```rust
/// @response 200 Json<User> User found
/// @example 200 {"id": 1, "name": "John Doe", "email": "john@example.com"}
#[rovo]
async fn get_user() -> Json<User> { ... }
```

Examples appear in the generated API documentation and help users understand the response format.
"#
        .to_string(),

        "@id" => r#"# @id

Set a custom operation ID for this endpoint.

## Syntax
```rust
/// @id OPERATION_ID
```

## Parameters
- `OPERATION_ID`: Unique identifier for this operation

## Example
```rust
/// @id getUserById
/// @response 200 Json<User> User found
#[rovo]
async fn get_user(id: i32) -> Json<User> { ... }
```

Operation IDs are used:
- In generated client SDKs
- For linking to specific operations
- In API documentation navigation
"#
        .to_string(),

        "@hidden" => r#"# @hidden

Hide this endpoint from the generated API documentation.

## Syntax
```rust
/// @hidden
```

## Example
```rust
/// @hidden
#[rovo]
async fn internal_endpoint() -> String { ... }
```

Useful for:
- Internal/debug endpoints
- Deprecated endpoints you want to keep
- Endpoints not ready for public documentation
"#
        .to_string(),

        _ => format!("Unknown annotation: {}", annotation),
    }
}

/// Get a short summary for a Rovo annotation
///
/// # Arguments
/// * `annotation` - The annotation name (e.g., "@response", "@tag")
///
/// # Returns
/// A brief one-line description
pub fn get_annotation_summary(annotation: &str) -> &'static str {
    match annotation {
        "@response" => "Define an API response for different status codes",
        "@tag" => "Group related endpoints together in the API documentation",
        "@security" => "Specify the security scheme required for this endpoint",
        "@example" => "Provide an example response for a specific status code",
        "@id" => "Set a custom operation ID for this endpoint",
        "@hidden" => "Hide this endpoint from the generated API documentation",
        _ => "Unknown annotation",
    }
}
