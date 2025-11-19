# Rovo - Neovim Integration

Language Server Protocol support for Rovo annotations in Neovim.

## Features

- **Completions** - Intelligent suggestions for annotations, status codes, and security schemes
- **Diagnostics** - Real-time validation of annotation syntax
- **Hover Documentation** - Detailed docs for annotations, status codes, and security schemes
- **Code Actions** - Quick fixes for adding annotations and macros
- **Go-to-Definition** - Navigate to type definitions from annotations
- **Find References** - Find all usages of tags
- **Syntax Highlighting** - Context-aware highlighting near `#[rovo]` attributes

## Installation

### lazy.nvim

```lua
{
  'Arthurdw/rovo',
  ft = 'rust',
  build = 'cargo install --path rovo-lsp',
  config = function()
    require('rovo').setup()
  end,
  dependencies = { 'neovim/nvim-lspconfig' },
}
```

### packer.nvim

```lua
use {
  'Arthurdw/rovo',
  ft = 'rust',
  run = 'cargo install --path rovo-lsp',
  config = function()
    require('rovo').setup()
  end,
  requires = { 'neovim/nvim-lspconfig' }
}
```

### Manual

Install the LSP server from crates.io:
```bash
cargo install rovo-lsp
```

Or from source:
```bash
cargo install --path rovo-lsp
```

Then add to your Neovim config:
```lua
require('rovo').setup()
```

## Usage

The LSP activates automatically for Rust files in a workspace with `Cargo.toml`.

Type `/// @` in a doc comment above a `#[rovo]` function to see completions.

### Example

```rust
/// @tag users
/// @response 200 Json<User> Successfully retrieved user
/// @response 404 Json<Error> User not found
#[rovo]
async fn get_user(id: i32) -> Result<Json<User>, StatusCode> {
    // ...
}
```

## Configuration

The plugin accepts standard LSP configuration options:

```lua
require('rovo').setup({
  on_attach = function(client, bufnr)
    -- Custom on_attach logic
  end,
  capabilities = vim.lsp.protocol.make_client_capabilities(),
})
```

### Highlight Groups

Customize colors by overriding these highlight groups:
- `RovoAnnotation` - Annotation keywords
- `RovoStatusCode` - HTTP status codes
- `RovoSecurityScheme` - Security schemes

## Troubleshooting

**LSP not starting?**
- Verify installation: `which rovo-lsp`
- Check logs: `:LspLog` in Neovim

**No completions?**
- Ensure you're in a doc comment (`///`)
- Type `@` to trigger completions
- Verify you're in a Rust workspace with `Cargo.toml`

## License

MIT
