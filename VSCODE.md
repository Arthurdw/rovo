# Rovo LSP for VSCode

This guide covers using Rovo with Visual Studio Code.

## Installation

### Prerequisites

- **Rust toolchain** (cargo) must be installed: https://rustup.rs/
- **VSCode** version 1.95.0 or later

### Installing the Extension

#### Option 1: From Marketplace (Recommended)

**VS Code Marketplace**: https://marketplace.visualstudio.com/items?itemName=arthurdw.rovo-lsp

Quick install in VS Code - press `Ctrl+P` (or `Cmd+P` on macOS) and run:
```
ext install arthurdw.rovo-lsp
```

**Open VSX** (for VSCodium/code-oss users): https://open-vsx.org/extension/arthurdw/rovo-lsp

Quick install in VSCodium - press `Ctrl+P` (or `Cmd+P` on macOS) and run:
```
ext install arthurdw.rovo-lsp
```

**Cursor**: The extension may not appear in Cursor's built-in extension search. To install, open this link in your browser:

[cursor:extension/arthurdw.rovo-lsp](cursor:extension/arthurdw.rovo-lsp)

Or install via the Extensions panel:

1. Open your editor (VS Code, VSCodium, Cursor, or code-oss)
2. Go to **Extensions** (Ctrl+Shift+X / Cmd+Shift+X)
3. Search for "Rovo"
4. Click **Install**
5. Reload your editor if prompted

The extension will automatically install the `rovo-lsp` server on first use (requires Cargo).

#### Option 2: Manual LSP Installation

If you prefer to install the language server manually:

```bash
cargo install rovo-lsp
```

Then the extension will automatically detect it.

## Features

The Rovo LSP extension provides intelligent support for Rovo framework annotations:

### 🎨 Syntax Highlighting

Custom syntax highlighting for Rovo documentation (context-aware, only near `#[rovo]` attributes):
- **Section headers** (`# Responses`, `# Examples`, `# Metadata`) - highlighted as headings
- **Metadata annotations** (`@tag`, `@security`, `@id`, `@hidden`) - highlighted as keywords
- **HTTP status codes** (200, 404, 500, etc.) - highlighted as numbers
- **Security schemes** (bearer, basic, apiKey, oauth2) - highlighted as strings
- **Tag values** - highlighted distinctively

The highlighting is smart and only activates in doc comments above `#[rovo]` functions, ensuring no conflicts with rust-analyzer.

### 🎯 Smart Completions

Type `/// #` to get completions for section headers:
- `# Responses` - HTTP response definitions section
- `# Examples` - Response examples section
- `# Metadata` - API metadata section

Within the `# Metadata` section, type `@` for annotation completions:
- `@tag` - Endpoint categorization
- `@security` - Security requirements
- `@id` - Custom operation IDs
- `@hidden` - Hide from documentation
- `@rovo-ignore` - Stop processing annotations

HTTP status codes and security schemes are also auto-completed with descriptions.

### 📖 Hover Documentation

Hover over section headers, annotations, status codes, or security schemes to see:
- Section usage and format
- Annotation usage and syntax
- HTTP status code meanings
- Security scheme explanations
- Type definitions

### ⚡ Code Actions

Quick fixes to:
- Add missing `#[rovo]` macro to functions
- Add `JsonSchema` derive to response types
- Insert common annotation patterns

### 🔍 Navigation

- **Go to Definition**: Navigate from types in responses to their definitions
- **Find References**: Find all usages of specific tags
- **Rename**: Rename tags and update all references (F2 or right-click → Rename Symbol)

### ✅ Real-time Diagnostics

Get instant feedback on:
- Invalid HTTP status codes
- Malformed response/example syntax
- Invalid metadata annotations
- Section format errors

## Usage Example

```rust
use rovo::{rovo, Router, routing::get};
use rovo::{extract::State, response::Json};
use rovo::aide::axum::IntoApiResponse;

/// Get user by ID.
///
/// # Responses
///
/// 200: Json<User> - Successfully retrieved user
/// 404: Json<Error> - User not found
///
/// # Examples
///
/// 200: User { id: 1, name: "Alice".into() }
/// 404: Error { message: "User not found".into() }
///
/// # Metadata
///
/// @tag users
/// @security bearer
#[rovo]
async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<i64>
) -> impl IntoApiResponse {
    // Implementation
}
```

## Configuration

Configure the extension in VSCode settings (File → Preferences → Settings → search "rovo"):

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

Useful for debugging LSP issues.

## Troubleshooting

### Language Server Not Starting

1. Check the **Output** panel (View → Output → Rovo LSP) for error messages
2. Verify `rovo-lsp` is installed:
   ```bash
   cargo install rovo-lsp
   ```
3. Check your `rovo.serverPath` setting
4. Try reloading VSCode: **Developer: Reload Window** (Ctrl+Shift+P / Cmd+Shift+P)

### Auto-Installation Failing

1. Ensure Cargo is installed:
   ```bash
   cargo --version
   ```
2. Install manually:
   ```bash
   cargo install rovo-lsp
   ```
3. Check the Output panel for detailed error messages
4. Verify you have internet connectivity

### Features Not Working

Make sure:
- You're in a Rust file (`.rs` extension)
- There's a `Cargo.toml` in your project root
- You're working within `#[rovo]` annotated functions
- Documentation is in doc comments (`///`) above `#[rovo]`
- Metadata annotations are within the `# Metadata` section

### Syntax Highlighting Not Visible

If annotations aren't being highlighted:

1. **Verify annotations are near `#[rovo]`**: The extension only highlights doc comments that appear directly above `#[rovo]` attributes

2. **Check the Output panel**:
   - Open Output panel (View → Output)
   - Select "Rovo LSP" from the dropdown
   - Look for diagnostic messages

3. **Reload VSCode**: Try **Developer: Reload Window**

Note: The extension uses text decorations that work seamlessly alongside rust-analyzer - no configuration needed!

## Extension Compatibility

This extension works seamlessly alongside **rust-analyzer** with zero configuration required. It uses text decorations for syntax highlighting, which overlay on top of rust-analyzer's highlighting without conflicts.

Both extensions can run simultaneously without any issues.

## Performance

The Rovo LSP is lightweight and context-aware:
- Only activates near `#[rovo]` attributes
- Minimal memory footprint
- Fast response times
- No interference with other Rust tooling

## Support

- **Issues**: https://github.com/Arthurdw/rovo/issues
- **Documentation**: https://github.com/Arthurdw/rovo
- **Extension Source**: https://github.com/Arthurdw/rovo/tree/main/vscode-rovo

## License

MIT
