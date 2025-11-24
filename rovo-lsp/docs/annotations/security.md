# @security

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
