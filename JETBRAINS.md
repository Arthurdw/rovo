# Rovo LSP for JetBrains IDEs

This guide covers using Rovo with JetBrains IDEs (RustRover, IntelliJ IDEA, CLion).

## Installation

### Prerequisites

- **Rust toolchain** (cargo) must be installed: https://rustup.rs/
- **JetBrains IDE** version 2023.2 or later
- One of the following IDEs:
  - RustRover (recommended for Rust development)
  - IntelliJ IDEA Ultimate/Community with Rust plugin
  - CLion with Rust plugin

### Installing the Plugin

#### Option 1: From JetBrains Marketplace (Coming Soon)

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

3. On first use, the plugin will automatically install the `rovo-lsp` server from crates.io

## Features

The Rovo LSP plugin provides intelligent support for Rovo framework annotations:

### 🎨 Syntax Highlighting

Custom syntax highlighting for Rovo annotations (context-aware, only near `#[rovo]` attributes):
- **Annotations** (`@response`, `@tag`, `@security`, etc.) - highlighted as keywords
- **HTTP status codes** (200, 404, 500, etc.) - highlighted as numbers
- **Security schemes** (bearer, basic, apiKey, oauth2) - highlighted as strings

The highlighting is smart and only activates in doc comments above `#[rovo]` functions, ensuring no conflicts with other Rust syntax.

### 🎯 Smart Completions

Type `/// @` to get completions for:
- `@response` - HTTP response definitions
- `@tag` - Endpoint categorization
- `@security` - Security requirements
- `@example` - Usage examples
- `@id` - Custom operation IDs
- `@hidden` - Hide from documentation

HTTP status codes and security schemes are also auto-completed.

### 📖 Hover Documentation

Hover over annotations, status codes, or security schemes to see:
- Annotation usage and syntax
- HTTP status code meanings
- Security scheme explanations
- Type definitions

### ⚡ Code Actions

Press **Alt+Enter** (or **Option+Return** on macOS) to:
- Add missing annotations
- Add `#[rovo]` macro to functions
- Add `JsonSchema` derive to structs
- Add common response sets
- Fix invalid status codes

### 🔍 Navigation

- **Go to Definition**: Navigate from annotation types to their definitions
- **Find Usages**: Find all references to `@tag` annotations
- **Rename**: Rename tags and update all references

### ✅ Real-time Diagnostics

Get instant feedback on:
- Invalid HTTP status codes
- Malformed annotations
- Missing required fields

## Usage Example

```rust
use rovo::prelude::*;

/// Get user by ID
///
/// @tag users
/// @response 200 User Successfully retrieved user
/// @response 404 NotFoundError User not found
/// @security bearer
#[rovo]
async fn get_user(id: i32) -> Json<User> {
    // Implementation
}
```

## Configuration

### Auto-Installation

On first use, the plugin will:
1. Check if `rovo-lsp` is already installed
2. If not found, automatically run `cargo install rovo-lsp`
3. Show a notification when installation completes

This process may take a few minutes on first run.

### Manual Installation

If you prefer to install manually:

```bash
cargo install rovo-lsp
```

The plugin will detect the manually installed binary.

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

- **IDE Version**: 2023.2 or later
- **Rust Version**: Any version supported by your IDE's Rust plugin
- **Platform**: Linux, macOS, Windows

## Support

- **Issues**: https://github.com/Arthurdw/rovo/issues
- **Documentation**: https://github.com/Arthurdw/rovo

## License

Same license as the Rovo framework.
