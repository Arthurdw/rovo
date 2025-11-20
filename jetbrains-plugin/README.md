# Rovo JetBrains Plugin

JetBrains IDE plugin for Rovo LSP support (RustRover, IntelliJ IDEA, CLion).

## Building the Plugin

### Prerequisites

- Java 17 or later
- Gradle 8.5 or later (or use the wrapper)

### Build Commands

```bash
# Using Gradle wrapper (recommended)
./gradlew buildPlugin

# Or with system Gradle
gradle buildPlugin
```

The plugin will be built to `build/distributions/rovo-jetbrains-plugin-*.zip`.

### Development

```bash
# Run plugin in test IDE
./gradlew runIde

# Run tests
./gradlew test

# Verify plugin structure
./gradlew verifyPlugin
```

## Installation

See [JETBRAINS.md](../JETBRAINS.md) for installation and usage instructions.

## Project Structure

```
src/main/
├── kotlin/com/rovo/lsp/
│   ├── RovoLspServerFactory.kt     # LSP server factory
│   ├── RovoLspServerDescriptor.kt  # Server configuration
│   ├── RovoLspInstaller.kt         # Auto-installation logic
│   └── RovoNotifications.kt        # User notifications
└── resources/META-INF/
    └── plugin.xml                   # Plugin manifest
```

## Features

- Auto-installs `rovo-lsp` from crates.io on first use
- Detects existing `rovo-lsp` installation
- Context-aware activation (only in Rust files with Cargo.toml)
- Error handling and user notifications
- Compatible with RustRover, IntelliJ IDEA, and CLion

## Development Notes

### Testing

To test the plugin locally:

1. Build the plugin: `./gradlew buildPlugin`
2. Install in IDE: Settings → Plugins → Install from disk
3. Restart IDE
4. Open a Rust project with Rovo annotations

### LSP4IJ Dependency

This plugin depends on the LSP4IJ plugin for LSP integration. Make sure it's available in the target IDE or bundled appropriately.

## License

Same as the main Rovo project.
