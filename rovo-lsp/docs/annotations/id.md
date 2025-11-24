# @id

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
