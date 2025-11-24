# @rovo-ignore

Stop processing Rovo annotations after this point.

## Syntax
```rust
/// @rovo-ignore
```

## Example
```rust
/// @tag users
/// @response 200 Json<User> User found
/// @rovo-ignore
/// This is regular documentation that won't be processed.
/// You can write @anything here without causing errors.
#[rovo]
async fn get_user() -> Json<User> { ... }
```

Useful for:
- Adding detailed documentation after annotations
- Writing examples that use @ symbols
- Preventing annotation-like text from being parsed
