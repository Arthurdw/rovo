# Path Parameters Section

Document path parameters for primitive types without needing wrapper structs.

## Format

```text
# Path Parameters

<param_name>: <description>
```

## Example

```rust
/// # Path Parameters
///
/// id: The unique identifier of the resource
/// index: Zero-based item index
```

## Supported Types

Works with primitive types that can be used directly in `Path<T>`:
- `String`
- `u64`, `u32`, `u16`, `u8`
- `i64`, `i32`, `i16`, `i8`
- `bool`
- `Uuid`

## Tuple Parameters

For routes with multiple path parameters using tuple extraction:

```rust
/// # Path Parameters
///
/// collection_id: The collection UUID
/// item_index: Index of the item in the collection
#[rovo]
async fn get_item(
    Path((collection_id, item_index)): Path<(Uuid, u32)>
) -> impl IntoApiResponse {
    // ...
}
```

## When to Use

- **Primitives**: Use `# Path Parameters` section for simple types
- **Complex types**: Use structs with `#[derive(JsonSchema)]` for richer documentation

## Notes

- Parameter names in docs must match the binding names in your function signature
- Each parameter gets its own line with `name: description` format
- Descriptions appear in the OpenAPI specification
