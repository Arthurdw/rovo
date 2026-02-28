# Rovo LSP for Zed

Language Server Protocol support for Rovo annotations in Zed.

## Features

- **Completions** - Intelligent suggestions for section headers, metadata annotations, status codes, and security schemes
- **Diagnostics** - Real-time validation of response/example syntax and metadata annotations
- **Hover Documentation** - Detailed docs for sections, annotations, status codes, and security schemes
- **Code Actions** - Quick fixes for adding sections, annotations, and macros
- **Go-to-Definition** - Navigate to type definitions from responses
- **Find References** - Find all usages of tags
- **Rename** - Rename tags across your project
- **Semantic Highlighting** - LSP semantic tokens for section headers, annotations, status codes, and more

## Installation

### Prerequisites

- **Rust toolchain** (cargo) must be installed: https://rustup.rs/
- **Zed** editor

### Step 1: Install the LSP Server

```bash
cargo install rovo-lsp
```

### Step 2: Install the Extension

#### From Zed Extension Marketplace

1. Open Zed
2. Open the Extensions panel (`zed: extensions` from the command palette)
3. Search for "Rovo"
4. Click **Install**

#### As a Dev Extension (for development/testing)

1. Clone the repository:
   ```bash
   git clone https://github.com/Arthurdw/rovo.git
   ```

2. In Zed, run the `zed: install dev extension` action from the command palette

3. Select the `zed-rovo` directory from the cloned repository

## Usage

The LSP activates automatically for Rust files. Type `/// #` in a doc comment above a `#[rovo]` function to see section completions. Within the `# Metadata` section, type `@` to see annotation completions.

### Example

```rust
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
async fn get_user(id: i32) -> Result<Json<User>, StatusCode> {
    // ...
}
```

## Configuration

### Custom Binary Path

If `rovo-lsp` is not on your PATH, configure it in your Zed settings (`settings.json`):

```json
{
  "lsp": {
    "rovo-lsp": {
      "binary": {
        "path": "/path/to/rovo-lsp"
      }
    }
  }
}
```

### Enabling Semantic Token Highlighting

Rovo LSP provides semantic tokens for syntax highlighting of annotations. To enable them in Zed, add to your settings:

```json
{
  "languages": {
    "Rust": {
      "semantic_tokens": "combined"
    }
  }
}
```

Options:
- `"off"` (default) - No LSP semantic tokens
- `"combined"` - LSP semantic tokens layered with tree-sitter highlighting
- `"full"` - LSP semantic tokens only

### Customizing Semantic Token Styles

You can customize how Rovo's semantic tokens are displayed:

```json
{
  "global_lsp_settings": {
    "semantic_token_rules": [
      {
        "token_type": "keyword",
        "style": ["syntax.title"]
      },
      {
        "token_type": "macro",
        "style": ["syntax.keyword"]
      },
      {
        "token_type": "number",
        "style": ["syntax.number"]
      },
      {
        "token_type": "enumMember",
        "style": ["syntax.constant"]
      },
      {
        "token_type": "string",
        "style": ["syntax.string"]
      }
    ]
  }
}
```

Rovo LSP provides these semantic token types:
- `keyword` - Section headers (`# Responses`, `# Examples`, `# Metadata`)
- `macro` - Metadata annotations (`@tag`, `@security`, `@id`, `@hidden`)
- `number` - HTTP status codes (200, 404, 500, etc.)
- `enumMember` - Security schemes (bearer, basic, apiKey, oauth2)
- `string` - Tag values
- `parameter` - Path parameter names

## Troubleshooting

**LSP not starting?**

- Verify installation: `which rovo-lsp`
- Check Zed logs: `zed: open log` from the command palette
- For verbose output, start Zed from the terminal with `zed --foreground`

**No completions?**

- Ensure you're in a doc comment (`///`)
- Type `#` to trigger section completions, or `@` within `# Metadata` for annotations
- Verify you're in a Rust workspace with `Cargo.toml`

**Semantic highlighting not visible?**

- Make sure `semantic_tokens` is set to `"combined"` or `"full"` in your Zed settings (see Configuration above)

## Compatibility

This extension works alongside Zed's built-in `rust-analyzer` support. Rovo LSP only handles Rovo-specific annotations and does not interfere with other Rust tooling.

## Support

- **Issues**: https://github.com/Arthurdw/rovo/issues
- **Documentation**: https://github.com/Arthurdw/rovo
- **Extension Source**: https://github.com/Arthurdw/rovo/tree/main/zed-rovo

## License

MIT
