# Responses Section

Define HTTP response codes and their associated types and descriptions.

## Format

```text
# Responses

<status>: <type> - <description>
```

## Example

```rust
/// # Responses
///
/// 200: Json<User> - Successfully retrieved user
/// 404: () - User not found
```

## Notes

- Status codes must be valid HTTP codes (100-599)
- Type must be a valid Rust type that implements `IntoResponse`
- Description can span multiple lines (continuation lines are joined)
- Common types: `Json<T>`, `()`, `(StatusCode, Json<T>)`
