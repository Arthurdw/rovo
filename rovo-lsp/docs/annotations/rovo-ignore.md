# @rovo-ignore

Stop processing Rovo annotations and sections after this point.

## Syntax
```rust
/// @rovo-ignore
```

## Usage

The `@rovo-ignore` annotation is special - it can be placed anywhere in doc comments to tell Rovo to stop parsing:

```rust
/// Get a user by ID
///
/// # Responses
///
/// 200: Json<User> - User found
/// 404: () - User not found
///
/// @rovo-ignore
///
/// # Implementation Notes
///
/// This function queries the database and caches results.
/// The @tag annotation below won't be processed.
/// @tag this-will-be-ignored
#[rovo]
async fn get_user(id: i32) -> Json<User> { ... }
```

## Use Cases

Useful for:
- **Adding detailed documentation** after Rovo sections without interference
- **Writing examples** that contain @ symbols or # headers
- **Preventing annotation-like text** from being parsed
- **Separating API docs from implementation docs**

## Complete Example

```rust
/// Create a new user account
///
/// # Responses
///
/// 201: Json<User> - User created successfully
/// 400: Json<Error> - Invalid input
/// 409: Json<Error> - User already exists
///
/// # Examples
///
/// 201: User {
///     id: 1,
///     name: "Alice".into(),
///     email: "alice@example.com".into()
/// }
///
/// # Metadata
///
/// @tag users
/// @security bearer
///
/// @rovo-ignore
///
/// ## Implementation Details
///
/// This endpoint performs the following:
/// 1. Validates the input data (@validate annotation on struct)
/// 2. Checks for existing users
/// 3. Hashes the password using argon2
/// 4. Creates database record
///
/// You can use @annotations freely here without them being processed.
#[rovo]
async fn create_user(data: Json<CreateUser>) -> Result<Json<User>, Error> { ... }
```

## Notes

- Everything after `@rovo-ignore` is treated as regular documentation
- The annotation applies to the rest of the current doc comment block
- You can use any special characters or annotations after `@rovo-ignore` without causing parsing errors
- Particularly useful when you want to mix OpenAPI documentation with internal implementation notes
