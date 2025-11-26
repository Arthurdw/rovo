# @id

Set a custom operation ID for this endpoint.

## Syntax
```rust
/// @id OPERATION_ID
```

## Parameters
- `OPERATION_ID`: Unique identifier for this operation (letters, numbers, underscores only)

## Usage

The `@id` annotation is now used within the `# Metadata` section:

```rust
/// Get a user by their ID
///
/// # Responses
///
/// 200: Json<User> - User found
/// 404: () - User not found
///
/// # Metadata
///
/// @id getUserById
/// @tag users
#[rovo]
async fn get_user(id: i32) -> Json<User> { ... }
```

## Default Behavior

If no `@id` is specified, Rovo will automatically generate an operation ID based on the function name.

## Operation IDs are used for:

- Generated client SDKs (method names)
- Linking to specific operations in documentation
- API documentation navigation
- OpenAPI tooling and code generation

## Best Practices

- Use camelCase for consistency (e.g., `getUserById`, `createPost`)
- Make IDs descriptive and unique across your API
- Keep them concise but meaningful
- Avoid special characters (stick to alphanumeric and underscores)

## Example with Multiple Metadata

```rust
/// # Metadata
///
/// @id listAllActiveUsers
/// @tag users
/// @tag admin
/// @security bearer
#[rovo]
async fn list_users(active_only: bool) -> Json<Vec<User>> { ... }
```
