# @hidden

Hide this endpoint from the generated API documentation.

## Syntax
```rust
/// @hidden
```

## Example
```rust
/// @hidden
#[rovo]
async fn internal_endpoint() -> String { ... }
```

Useful for:
- Internal/debug endpoints
- Deprecated endpoints you want to keep
- Endpoints not ready for public documentation
