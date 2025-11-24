# @example

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
