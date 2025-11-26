# @hidden

Hide this endpoint from the generated API documentation.

## Syntax
```rust
/// @hidden
```

## Usage

The `@hidden` annotation is now used within the `# Metadata` section:

```rust
/// Internal health check endpoint
///
/// # Responses
///
/// 200: Json<Health> - System is healthy
///
/// # Metadata
///
/// @hidden
#[rovo]
async fn internal_health_check() -> Json<Health> { ... }
```

## Use Cases

Useful for:
- **Internal endpoints**: Debug or monitoring endpoints not meant for external use
- **Deprecated endpoints**: Endpoints you want to keep functional but hide from docs
- **Development endpoints**: Testing or staging-only endpoints
- **Administrative endpoints**: Internal admin tools not for public consumption

## Complete Example

```rust
/// Debug endpoint that returns internal system state
///
/// # Responses
///
/// 200: Json<SystemState> - Current system state
///
/// # Examples
///
/// 200: SystemState::default()
///
/// # Metadata
///
/// @hidden
/// @tag debug
/// @security bearer
#[rovo]
async fn debug_system_state() -> Json<SystemState> { ... }
```

## Notes

- Hidden endpoints are still functional and accessible
- They simply don't appear in the generated OpenAPI documentation
- Useful during development or for maintaining backward compatibility
- Can be combined with other metadata annotations
