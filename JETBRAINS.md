# Rovo LSP for JetBrains IDEs

This guide covers using Rovo with JetBrains IDEs (RustRover, IntelliJ IDEA, CLion).

## Installation

### Prerequisites

- **Rust toolchain** (cargo) must be installed: https://rustup.rs/
- **JetBrains IDE** version 2024.3 or later
- One of the following IDEs:
  - RustRover (recommended for Rust development)
  - IntelliJ IDEA Ultimate/Community with Rust plugin
  - CLion with Rust plugin

### Installing the Plugin

#### Option 1: From JetBrains Marketplace

**Direct link**: https://plugins.jetbrains.com/plugin/29093-rovo-lsp

Or install from your IDE:

1. Open your IDE
2. Go to **Settings/Preferences → Plugins**
3. Search for "Rovo LSP"
4. Click **Install**
5. Restart the IDE

#### Option 2: From Disk (Current Method)

1. Build the plugin:
   ```bash
   cd jetbrains-plugin
   ./gradlew buildPlugin
   ```

2. Install the plugin:
   - Go to **Settings/Preferences → Plugins**
   - Click the gear icon ⚙️ → **Install Plugin from Disk...**
   - Select `jetbrains-plugin/build/distributions/rovo-jetbrains-plugin-*.zip`
   - Restart the IDE

3. Install the `rovo-lsp` server:
   ```bash
   cargo install rovo-lsp
   ```

## Features

The Rovo LSP plugin provides intelligent support for Rovo framework annotations:

### 🎨 Syntax Highlighting

Custom syntax highlighting for Rovo documentation (context-aware, only near `#[rovo]` attributes):
- **Section headers** (`# Responses`, `# Examples`, `# Metadata`) - highlighted as headings
- **Metadata annotations** (`@tag`, `@security`, `@id`, `@hidden`) - highlighted as keywords
- **HTTP status codes** (200, 404, 500, etc.) - highlighted as numbers
- **Security schemes** (bearer, basic, apiKey, oauth2) - highlighted as strings

The highlighting is smart and only activates in doc comments above `#[rovo]` functions, ensuring no conflicts with other Rust syntax.

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

HTTP status codes and security schemes are also auto-completed.

### 📖 Hover Documentation

Hover over section headers, annotations, status codes, or security schemes to see:
- Section usage and format
- Annotation usage and syntax
- HTTP status code meanings
- Security scheme explanations
- Type definitions

### ⚡ Code Actions

Press **Alt+Enter** (or **Option+Return** on macOS) to:
- Add missing sections (`# Responses`, `# Examples`, `# Metadata`)
- Add missing metadata annotations
- Add `#[rovo]` macro to functions
- Add `JsonSchema` derive to structs
- Add common response sets
- Fix invalid status codes

### 🔍 Navigation

- **Go to Definition**: Navigate from types in responses to their definitions
- **Find Usages**: Find all references to specific tags
- **Rename**: Rename tags and update all references

### ✅ Real-time Diagnostics

Get instant feedback on:
- Invalid HTTP status codes
- Malformed response/example syntax
- Invalid metadata annotations
- Section format errors
- Missing required fields

## Usage Example

```rust
use rovo::prelude::*;

/// Get user by ID.
///
/// # Responses
///
/// 200: Json<User> - Successfully retrieved user
/// 404: Json<NotFoundError> - User not found
///
/// # Examples
///
/// 200: User { id: 1, name: "Alice".into() }
/// 404: NotFoundError { message: "User not found".into() }
///
/// # Metadata
///
/// @tag users
/// @security bearer
#[rovo]
async fn get_user(id: i32) -> Json<User> {
    // Implementation
}
```

## Configuration

### LSP Server Installation

The plugin requires `rovo-lsp` to be installed:

```bash
cargo install rovo-lsp
```

The plugin will automatically detect the LSP server from common locations:
- `~/.cargo/bin/rovo-lsp`
- `/usr/local/bin/rovo-lsp`
- `/usr/bin/rovo-lsp`
- Or any location in your system PATH

### Troubleshooting

#### "Cargo Not Found" Error

If you see this error:
1. Install Rust from https://rustup.rs/
2. Ensure `cargo` is in your PATH
3. Restart the IDE

You can verify cargo is installed:
```bash
cargo --version
```

#### LSP Server Not Starting

1. Check if `rovo-lsp` is installed:
   ```bash
   which rovo-lsp  # macOS/Linux
   where rovo-lsp  # Windows
   ```

2. Try manual installation:
   ```bash
   cargo install rovo-lsp --force
   ```

3. Check IDE logs:
   - Go to **Help → Show Log in Finder/Explorer**
   - Look for errors related to "Rovo LSP"

#### Features Not Working

Make sure:
- You're in a Rust file (`.rs` extension)
- There's a `Cargo.toml` in your project
- You're working within `#[rovo]` annotated functions
- Documentation is in doc comments (`///`) above `#[rovo]`
- Metadata annotations are within the `# Metadata` section
- The file is properly saved

## IDE-Specific Notes

### RustRover

Works out of the box alongside RustRover's built-in Rust support. Rovo LSP only activates for Rovo-specific annotations.

### IntelliJ IDEA

Requires the Rust plugin to be installed separately:
1. Go to **Settings/Preferences → Plugins**
2. Install "Rust" plugin
3. Restart the IDE

### CLion

Same as IntelliJ IDEA - requires the Rust plugin.

## Performance

The Rovo LSP is lightweight and context-aware:
- Only activates near `#[rovo]` attributes
- Minimal memory footprint
- Fast response times
- No interference with other Rust tooling

## Compatibility

- **IDE Version**: 2024.3 or later
- **Rust Version**: Any version supported by your IDE's Rust plugin
- **Platform**: Linux, macOS, Windows

## Support

- **Issues**: https://github.com/Arthurdw/rovo/issues
- **Documentation**: https://github.com/Arthurdw/rovo

## License

Same license as the Rovo framework.
