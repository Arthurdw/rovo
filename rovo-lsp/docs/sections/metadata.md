# Metadata Section

Define additional metadata for your endpoint such as tags, security requirements, operation IDs, and visibility.

## Format

```
# Metadata

@tag <tag_name>
@security <scheme>
@id <operation_id>
@hidden
```

## Example

```rust
/// # Metadata
///
/// @tag users
/// @security bearer_auth
/// @id get_user_by_id
```

## Available Annotations

- **@tag**: Group endpoints in OpenAPI documentation
- **@security**: Specify required authentication schemes
- **@id**: Custom operation ID (default: function name)
- **@hidden**: Exclude endpoint from OpenAPI documentation

## Notes

- Tags help organize endpoints in API documentation
- Security schemes must be defined in your OpenAPI configuration
- Operation IDs must be valid identifiers (letters, numbers, underscores only)
- Multiple tags can be specified with multiple `@tag` annotations
