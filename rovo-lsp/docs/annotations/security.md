# @security

Specify the security scheme required for this endpoint.

## Syntax
```rust
/// @security SCHEME
```

## Parameters
- `SCHEME`: Security scheme name (e.g., `bearer`, `basic`, `apiKey`, `oauth2`)

## Usage

The `@security` annotation is now used within the `# Metadata` section:

```rust
/// Get the current authenticated user
///
/// # Responses
///
/// 200: Json<User> - Authenticated user details
/// 401: () - Unauthorized
///
/// # Metadata
///
/// @security bearer
/// @tag users
#[rovo]
async fn get_current_user() -> Json<User> { ... }
```

## Common Security Schemes

- `bearer`: Bearer token authentication (e.g., JWT)
- `basic`: Basic HTTP authentication
- `apiKey`: API key in header/query/cookie
- `oauth2`: OAuth 2.0 authentication

## Multiple Security Schemes

You can specify multiple security schemes if the endpoint supports multiple authentication methods:

```rust
/// # Metadata
///
/// @security bearer
/// @security apiKey
#[rovo]
async fn protected_endpoint() -> Json<Response> { ... }
```

## Notes

- Security schemes must be defined in your OpenAPI configuration
- The scheme name must match what's defined in your API's security definitions
- Endpoints without `@security` are considered public/unauthenticated
- Different schemes can have different requirements (header location, format, etc.)
