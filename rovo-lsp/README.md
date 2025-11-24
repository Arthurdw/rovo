# Rovo LSP

Language Server Protocol implementation for Rovo annotation validation and completion.

## Features

- **Annotation Parsing**: Detects and parses Rovo annotations in doc comments
- **Diagnostics**: Real-time validation of annotation syntax (e.g., HTTP status codes must be 100-599)
- **Completions**: Intelligent completions for annotations, status codes, and security schemes
  - Auto-completion for common HTTP status codes (200, 201, 204, 400, 401, 403, 404, 409, 422, 500, 503)
  - Auto-completion for security schemes (bearer, basic, apiKey, oauth2)
  - Filters as you type (e.g., typing "2" shows 200, 201, 204)
- **Snippets**: Smart snippet support for common annotation patterns
- **Hover Documentation**: Rich markdown documentation for annotations, status codes, and security schemes
- **Code Actions**: Quick fixes and refactorings
  - Add missing sections (Responses, Examples, Metadata)
  - Add #[rovo] macro to functions
  - Add JsonSchema derive to structs
- **Go-to-Definition**: Navigate to type definitions from sections
- **Find References**: Find all references to a tag across the document
- **Context-Aware**: Features only activate near #[rovo] attributes

## Documentation Format

Rovo uses Rust-style documentation comments with special sections:

### Responses Section
Define HTTP response codes, types, and descriptions:
```rust
/// # Responses
///
/// 200: Json<User> - Successfully retrieved user
/// 404: () - User not found
```

### Examples Section
Provide example response data for each status code:
```rust
/// # Examples
///
/// 200: User { id: 1, name: "Alice".into() }
/// 404: ()
```

### Metadata Section
Specify tags, security, operation IDs, and visibility:
```rust
/// # Metadata
///
/// @tag users
/// @security bearer
/// @id getUserById
```

### Complete Example
```rust
/// Get a user by ID
///
/// # Responses
///
/// 200: Json<User> - User found
/// 404: () - User not found
///
/// # Examples
///
/// 200: User { id: 1, name: "Alice".into(), email: "alice@example.com".into() }
///
/// # Metadata
///
/// @tag users
/// @security bearer
#[rovo]
async fn get_user(id: i32) -> Json<User> { ... }
```

## Supported Metadata Annotations

- `@tag NAME` - Group endpoints in API documentation
- `@security SCHEME` - Specify security scheme (bearer, basic, apiKey, oauth2)
- `@id OPERATION_ID` - Set custom operation ID
- `@hidden` - Hide endpoint from documentation
- `@rovo-ignore` - Stop processing annotations (for regular doc comments)

## Installation

```bash
cargo install --path .
```

Or add to your `Cargo.toml`:

```toml
[dev-dependencies]
rovo-lsp = "0.1"
```

## Usage

The LSP server runs as a standalone binary that communicates via stdin/stdout following the LSP protocol.

### With Neovim

See [editors/nvim/README.md](../editors/nvim/README.md) for Neovim integration.

### With VSCode

Support for VSCode and other editors coming soon.

## Development

### Running Tests

```bash
# Run all tests
cargo test --package rovo-lsp

# Run specific test suite
cargo test --package rovo-lsp --test integration
cargo test --package rovo-lsp --test validation
cargo test --package rovo-lsp --test completion
```

### Manual Testing

1. Build the LSP server:
   ```bash
   cargo build --package rovo-lsp
   ```

2. Test with a fixture file:
   ```bash
   nvim rovo-lsp/tests/fixtures/test.rs
   ```

3. Try typing `/// @` to see completions

## Architecture

```
rovo-lsp/
├── src/
│   ├── main.rs         # LSP server entry point
│   ├── backend.rs      # LSP backend implementation
│   ├── handlers.rs     # LSP request handlers (hover, completion, references)
│   ├── parser.rs       # Annotation parser
│   ├── diagnostics.rs  # Validation logic
│   ├── completion.rs   # Completion provider with status codes and security schemes
│   ├── code_actions.rs # Code actions (add annotations, add #[rovo], add JsonSchema)
│   ├── type_resolver.rs # Type resolution and go-to-definition
│   └── docs.rs         # Shared documentation for annotations
└── tests/
    ├── integration.rs  # Parser integration tests
    ├── validation.rs   # Validation tests
    ├── completion.rs   # Completion tests
    └── fixtures/       # Test Rust files
```

## Contributing

Contributions are welcome! Please ensure all tests pass before submitting a PR:

```bash
cargo test --package rovo-lsp
```

## License

MIT
