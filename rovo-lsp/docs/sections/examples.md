# Examples Section

Provide example response data for each status code in your OpenAPI documentation.

## Format

```
# Examples

<status>: <rust_expression>
```

## Example

```rust
/// # Examples
///
/// 200: User { id: 1, name: "Alice".into(), email: "alice@example.com".into() }
/// 404: ()
```

## Multi-line Examples

Examples can span multiple lines for complex structures:

```rust
/// # Examples
///
/// 200: User {
///     id: 1,
///     name: "Alice".into(),
///     email: "alice@example.com".into()
/// }
```

## Notes

- Expressions must be valid Rust code
- The example should match the response type defined in the Responses section
- Use `.into()`, `.to_string()`, or similar for owned strings
- Primitive examples: `"success"`, `42`, `true`, `99.9`
