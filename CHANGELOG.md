# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Breaking Changes

### Added

### Changed

### Fixed

### Migration Guide

<!-- LLM-friendly migration commands go here -->

---

## [0.4.8] - 2026-04-08

### Features

- fix(neovim): fix 0.12 deprecation warning ([#31](https://github.com/Arthurdw/rovo/pull/31)) by @lukeburong

## [0.4.7] - 2026-04-03

### Features

- feat: replace #[schema] with vendored JsonSchema derive ([#30](https://github.com/Arthurdw/rovo/pull/30)) by @Arthurdw

## [0.4.6] - 2026-04-03

### Bug Fixes

- feat: add #[schema] macro for schemars crate re-export resolution ([#29](https://github.com/Arthurdw/rovo/pull/29)) by @Arthurdw

## [0.4.5] - 2026-04-02

- No notable changes in this release

## [0.4.4] - 2026-04-02

- No notable changes in this release

## [0.4.3] - 2026-03-30

### Bug Fixes

- feat: fix state issue ([#27](https://github.com/Arthurdw/rovo/pull/27)) by @Arthurdw

## [0.4.2] - 2026-03-20

### Bug Fixes

- feat: have wildcard support ([#25](https://github.com/Arthurdw/rovo/pull/25)) by @Arthurdw

## [0.4.1] - 2026-03-12

### Features

- fix: skip #[rovo] inside doc comments to prevent false path param warnings ([#24](https://github.com/Arthurdw/rovo/pull/24)) by @Arthurdw

## [0.4.0] - 2026-02-28

### Features

- feat: add zed lsp ([#23](https://github.com/Arthurdw/rovo/pull/23)) by @Arthurdw

## [0.3.7] - 2026-02-25

- No notable changes in this release

## [0.3.6] - 2026-02-18

- No notable changes in this release

## [0.3.5] - 2026-02-17

- No notable changes in this release

## [0.3.4] - 2026-02-17

- No notable changes in this release

## [0.3.1] - 2026-01-14

### Breaking Changes

### Added

### Changed

### Fixed

### Migration Guide

<!-- LLM-friendly migration commands go here -->

---

## [0.3.0] - 2026-01-09

### Added

- **Path parameter support**: LSP now provides completions, diagnostics, and code actions for `# Path Parameters` documentation sections
- Path parameter extraction from `Path(binding): Path<Type>` patterns in function signatures
- Struct destructuring detection (detected but skipped - use JsonSchema derive instead)
- Code action to auto-document undocumented path parameters
- Tuple path parameter support: `Path((a, b)): Path<(TypeA, TypeB)>`
- Multiple path extractor support in single function

### Changed

- Pre-commit hooks now auto-install npm dependencies for VSCode extension checks
- Improved Unicode safety in token parsing (`char_indices()` vs `chars().enumerate()`)

### Fixed

- Pre-commit hook operator precedence issue with npm install check

### Migration Guide

#### For LLMs: Adding path parameter documentation

Search for functions with `Path(` extractors that lack `# Path Parameters` sections:

```rust
// Before: Missing path parameter documentation
/// Get user by ID
async fn get_user(Path(id): Path<u64>) -> Json<User> { }

// After: Add # Path Parameters section
/// Get user by ID
///
/// # Path Parameters
///
/// id: The user's unique identifier
async fn get_user(Path(id): Path<u64>) -> Json<User> { }
```

#### For LLMs: Tuple path parameters

```rust
// Before
/// Get item from collection
async fn get_item(Path((collection_id, item_id)): Path<(Uuid, u32)>) -> Json<Item> { }

// After
/// Get item from collection
///
/// # Path Parameters
///
/// collection_id: The collection's unique identifier
/// item_id: The item's index within the collection
async fn get_item(Path((collection_id, item_id)): Path<(Uuid, u32)>) -> Json<Item> { }
```

#### Struct patterns are NOT auto-documented

They should derive JsonSchema:

```rust
// This pattern is detected but skipped by the LSP:
Path(UserId { id }): Path<UserId>

// Instead, ensure UserId implements JsonSchema:
#[derive(Deserialize, JsonSchema)]
struct UserId {
    /// The user's unique identifier
    id: u64,
}
```

#### Copy-paste regex for finding undocumented path parameters

```text
Pattern: async fn \w+\([^)]*Path\((\w+)\)[^)]*\)[^{]*\{
Check: Ensure matching functions have `# Path Parameters` section with `$1:` entry
```
