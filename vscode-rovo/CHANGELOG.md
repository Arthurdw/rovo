# Change Log

All notable changes to the Rovo Language Support extension will be documented in this file.

## [0.1.4] - 2025-11-20

### Added

- Initial VSCode extension release
- Language Server Protocol client for rovo-lsp
- Automatic installation of rovo-lsp via cargo
- Intelligent completions for Rovo annotations (@response, @tag, @security, @example, @id, @hidden)
- Context-aware completions for HTTP status codes and security schemes
- Hover information for annotations with detailed documentation
- Real-time diagnostics for annotation syntax errors
- Code actions for quick fixes
- Go to definition for response types
- Find references for tags
- Rename support for tags with validation
- Syntax highlighting via TextMate grammar
- Configuration options:
  - `rovo.serverPath` - Custom path to rovo-lsp executable
  - `rovo.autoInstall` - Auto-install rovo-lsp when not found
  - `rovo.trace.server` - LSP communication tracing

### Features

- Full LSP feature parity with Neovim integration
- Works alongside rust-analyzer without conflicts
- Context-aware activation (only near `#[rovo]` attributes)
- Cross-platform support (Linux, macOS, Windows)
