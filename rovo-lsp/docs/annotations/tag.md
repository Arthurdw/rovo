# @tag

Group related endpoints together in the API documentation.

## Syntax
```rust
/// @tag NAME
```

## Parameters
- `NAME`: Tag name (e.g., `users`, `posts`, `admin`)

## Usage

The `@tag` annotation is now used within the `# Metadata` section:

```rust
/// Get a list of all users
///
/// # Responses
///
/// 200: Json<Vec<User>> - List of users
///
/// # Metadata
///
/// @tag users
#[rovo]
async fn list_users() -> Json<Vec<User>> { ... }
```

## Multiple Tags

You can specify multiple tags for a single endpoint:

```rust
/// # Metadata
///
/// @tag users
/// @tag admin
#[rovo]
async fn manage_users() -> Json<Response> { ... }
```

## Notes

- Tags help organize your API documentation by grouping related endpoints
- Tag names are case-sensitive
- Tags appear in the generated OpenAPI specification
- All endpoints with the same tag will be grouped together in API documentation tools
