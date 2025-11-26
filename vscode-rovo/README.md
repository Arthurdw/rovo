# Rovo Language Support for VSCode

Language Server Protocol (LSP) support for [Rovo](https://github.com/arthurdw/rovo) annotations in Rust. Write cleaner, more maintainable OpenAPI-style documentation directly in your Rust code with intelligent IDE support.

## Features

- 🎯 **Smart Completions** - Auto-complete for Rovo annotations with helpful snippets
- 📖 **Hover Documentation** - Detailed information for annotations, status codes, and security schemes
- ✅ **Real-time Diagnostics** - Instant feedback on syntax errors and invalid annotations
- ⚡ **Code Actions** - Quick fixes for common issues
- 🔍 **Navigation** - Go to definition, find references, and rename support
- 🎨 **Syntax Highlighting** - Distinctive colors for annotations, status codes, and more

## Quick Start

1. Install the extension from the VSCode Marketplace
2. Open a Rust project with Rovo annotations
3. The extension will automatically install `rovo-lsp` if not present (requires Cargo)

## Usage Example

```rust
/// Get user by ID
///
/// # Responses
///
/// 200: Json<User> - Successfully retrieved user
/// 404: Json<Error> - User not found
///
/// # Metadata
///
/// @tag users
/// @security bearer
#[rovo]
async fn get_user(State(state): State<AppState>, Path(id): Path<i64>) -> impl IntoApiResponse {
    // Implementation
}
```

## Requirements

- VSCode 1.95.0 or higher
- Rust project with `Cargo.toml`
- (Optional) Cargo for auto-installation of `rovo-lsp`

## Documentation

For detailed installation instructions, configuration options, and troubleshooting:

📚 **[Full Documentation](https://github.com/arthurdw/rovo/blob/main/VSCODE.md)**

## Extension Compatibility

This extension works seamlessly alongside **rust-analyzer** with zero configuration required. It uses text decorations for syntax highlighting, which overlay without conflicts.

## Contributing

This extension is part of the [Rovo](https://github.com/arthurdw/rovo) project. Contributions are welcome!

## License

MIT

## Related Projects

- [Rovo](https://github.com/arthurdw/rovo) - The main Rovo library and macro
- [rovo-lsp](https://crates.io/crates/rovo-lsp) - The language server implementation
