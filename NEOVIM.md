# Rovo - Neovim Integration

Language Server Protocol support for Rovo annotations in Neovim.

<!--toc:start-->
- [Rovo - Neovim Integration](#rovo---neovim-integration)
  - [Features](#features)
  - [Installation](#installation)
    - [lazy.nvim](#lazynvim)
      - [Silencing LSP Notifications](#silencing-lsp-notifications)
    - [packer.nvim](#packernvim)
    - [Manual](#manual)
  - [Usage](#usage)
    - [Example](#example)
  - [Configuration](#configuration)
    - [Highlight Groups](#highlight-groups)
  - [Troubleshooting](#troubleshooting)
  - [License](#license)
<!--toc:end-->

## Features

- **Auto-Installation** - Automatically installs `rovo-lsp` if not found (requires Cargo)
- **Completions** - Intelligent suggestions for section headers, metadata annotations, status codes, and security schemes
- **Diagnostics** - Real-time validation of response/example syntax and metadata annotations
- **Hover Documentation** - Detailed docs for sections, annotations, status codes, and security schemes
- **Code Actions** - Quick fixes for adding sections, annotations, and macros
- **Go-to-Definition** - Navigate to type definitions from responses
- **Find References** - Find all usages of tags
- **Syntax Highlighting** - LSP semantic tokens for consistent highlighting (same as VSCode)

## Installation

The plugin will automatically offer to install `rovo-lsp` if it's not found (requires Cargo).

### lazy.nvim

```lua
{
  'Arthurdw/rovo',
  ft = 'rust',
  config = function()
    require('rovo').setup({
      auto_install = true,  -- Auto-install rovo-lsp if not found (default: true)
    })
  end,
  dependencies = { 'neovim/nvim-lspconfig' },
}
```

If you prefer to install the LSP server during plugin installation:

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

#### Silencing LSP Notifications

Noisy LSP notifications can be suppressed using Noice.nvim:

```lua
{
  "folke/noice.nvim",
  opts = {
    lsp = {
      hover = { silent = true },  -- Suppress double hover warnings
    },
    routes = {
      {
        filter = {
          any = {
            { find = "prepareRename" },      -- Suppress rename errors
            { find = "No references found" },
          },
        },
        opts = { skip = true },
      },
    },
  },
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

Type `/// #` in a doc comment above a `#[rovo]` function to see section completions.
Within the `# Metadata` section, type `@` to see annotation completions.

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
#[rovo]
async fn get_user(id: i32) -> Result<Json<User>, StatusCode> {
    // ...
}
```

## Configuration

The plugin accepts standard LSP configuration options:

```lua
require('rovo').setup({
  auto_install = true,  -- Auto-install rovo-lsp if not found (default: true)
  enable_highlighting = true,  -- Setup LSP semantic token highlights (default: true)
  on_attach = function(client, bufnr)
    -- Custom on_attach logic
  end,
  capabilities = vim.lsp.protocol.make_client_capabilities(),
})
```

### Configuration Options

- `auto_install` (boolean, default: `true`) - Automatically install `rovo-lsp` via cargo if not found
- `enable_highlighting` (boolean, default: `true`) - Setup LSP semantic token highlight groups
- `on_attach` (function, optional) - Custom LSP on_attach callback
- `cmd` (table, optional) - Override LSP server command (default: `{ 'rovo-lsp' }`)
- Plus any other standard `lspconfig` options

### Highlight Groups

Rovo uses LSP semantic tokens for syntax highlighting. Customize colors by linking these highlight groups:

```lua
-- Section headers (# Responses, # Examples, # Metadata)
vim.api.nvim_set_hl(0, '@lsp.type.keyword.rust', { link = 'Title' })

-- Metadata annotation keywords (@tag, @security, etc.)
vim.api.nvim_set_hl(0, '@lsp.type.macro.rust', { link = 'Macro' })

-- Tag values (e.g., "users" in @tag users)
vim.api.nvim_set_hl(0, '@lsp.type.string.rust', { link = 'String' })

-- Status codes (200, 404, etc.)
vim.api.nvim_set_hl(0, '@lsp.type.number.rust', { link = 'Number' })

-- Security schemes (bearer, oauth2, etc.)
vim.api.nvim_set_hl(0, '@lsp.type.enumMember.rust', { link = 'Constant' })
```

The plugin sets these by default, but you can override them in your config.

## Troubleshooting

**LSP not starting?**

- Verify installation: `which rovo-lsp`
- Check logs: `:LspLog` in Neovim

**No completions?**

- Ensure you're in a doc comment (`///`)
- Type `#` to trigger section completions, or `@` within `# Metadata` for annotations
- Verify you're in a Rust workspace with `Cargo.toml`

## License

MIT
