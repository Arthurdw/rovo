# Rovo Language Support for VSCode

Language Server Protocol (LSP) support for [Rovo](https://github.com/arthurdw/rovo) annotations in Rust. Write cleaner, more maintainable OpenAPI-style documentation directly in your Rust code with intelligent IDE support.

## Features

### Intelligent Completions

Auto-complete for Rovo annotations with helpful snippets and documentation:

- `@response` - Define API response types with status codes
- `@tag` - Organize endpoints into logical groups
- `@security` - Specify authentication requirements
- `@example` - Add example values for documentation
- `@id` - Set custom operation IDs
- `@hidden` - Exclude endpoints from generated documentation

Context-aware completions include:
- HTTP status codes (200, 201, 400, 404, 500, etc.) with descriptions
- Security schemes (bearer, basic, apiKey, oauth2) with documentation

### Hover Information

Hover over annotations to see detailed documentation about:
- Annotation syntax and usage
- HTTP status code meanings
- Security scheme types and configurations

### Real-time Diagnostics

Get instant feedback on:
- Syntax errors in annotations
- Invalid status codes
- Malformed annotation syntax

### Code Actions

Quick fixes to:
- Add missing `#[rovo]` macro to functions
- Add `JsonSchema` derive to response types
- Insert common annotation patterns

### Go to Definition

Navigate from response type references to their definitions:
- Jump from `@response(200) -> User` to the `User` struct
- Works within the same file

### Find References

Find all usages of tags across your codebase:
- See where tags are referenced
- Understand endpoint organization

### Rename Support

Safely rename tags across your entire project:
- Rename with F2 or right-click → Rename Symbol
- Updates all references automatically
- Validates before renaming

### Syntax Highlighting

Distinctive colors for:
- Annotation keywords (@response, @tag, etc.)
- Status codes (200, 404, 500, etc.)
- Security schemes (bearer, oauth2, etc.)

## Installation

1. Install the extension from the VSCode Marketplace
2. Open a Rust project with Rovo annotations
3. The extension will automatically install `rovo-lsp` if not present (requires Cargo)

### Manual Installation

If you prefer to install the language server manually:

```bash
cargo install rovo-lsp
```

Then configure the extension to use your installed binary.

## Requirements

- VSCode 1.75.0 or higher
- Rust project with `Cargo.toml`
- (Optional) Cargo for auto-installation

## Configuration

Configure the extension in VSCode settings:

### `rovo.serverPath`

Path to the `rovo-lsp` executable. Default: `"rovo-lsp"` (uses PATH)

```json
{
  "rovo.serverPath": "/custom/path/to/rovo-lsp"
}
```

### `rovo.autoInstall`

Automatically install `rovo-lsp` via cargo if not found. Default: `true`

```json
{
  "rovo.autoInstall": false
}
```

### `rovo.trace.server`

Trace communication between VSCode and the language server. Default: `"off"`

Options: `"off"`, `"messages"`, `"verbose"`

```json
{
  "rovo.trace.server": "verbose"
}
```

## Usage

Add Rovo annotations to your Rust functions within the `#[rovo]` macro context:

```rust
#[rovo]
/// @tag Users
/// @security bearer
/// @response(200) -> User
/// @response(404) "User not found"
async fn get_user(id: i64) -> Result<User, Error> {
    // ...
}
```

The extension provides IDE support only near `#[rovo]` attributes, so it won't interfere with regular Rust development.

## Extension Compatibility

This extension works seamlessly alongside rust-analyzer with **zero configuration required**. It uses text decorations for syntax highlighting, which overlay on top of rust-analyzer's highlighting without conflicts.

## Troubleshooting

### Language server not starting

1. Check the Output panel (View → Output → Rovo LSP) for error messages
2. Verify `rovo-lsp` is installed: `cargo install rovo-lsp`
3. Check your `rovo.serverPath` setting
4. Try reloading VSCode (Developer: Reload Window)

### Auto-installation failing

1. Ensure Cargo is installed: `cargo --version`
2. Install manually: `cargo install rovo-lsp`
3. Check the Output panel for detailed error messages

### Features not working

1. Ensure you're editing a `.rs` file
2. Verify annotations are within a `#[rovo]` context
3. Check that the file is part of a Cargo workspace

### Syntax highlighting not visible

If annotations aren't being highlighted:

1. **Verify annotations are near `#[rovo]`**: The extension only highlights doc comments that appear directly above `#[rovo]` attributes

2. **Check the Output panel**:
   - Open Output panel (View → Output)
   - Select "Rovo LSP" from the dropdown
   - Look for "Applied Rovo text decorations" message

3. **Reload VSCode**: Try reloading the window (Developer: Reload Window)

Note: The extension uses text decorations that work seamlessly alongside rust-analyzer - no configuration needed!

## Contributing

This extension is part of the [Rovo](https://github.com/arthurdw/rovo) project. Contributions are welcome!

## License

MIT

## Related Projects

- [Rovo](https://github.com/arthurdw/rovo) - The main Rovo library and macro
- [rovo-lsp](https://crates.io/crates/rovo-lsp) - The language server implementation
