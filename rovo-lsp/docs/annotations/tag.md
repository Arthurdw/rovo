# @tag

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
