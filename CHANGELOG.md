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

## [0.4.14] - 2026-10-15

### Features

- ci: harden workflows (Aikido findings) ([#59](https://github.com/Arthurdw/rovo/pull/59)) by @Arthurdw

### Bug Fixes

- fix: security hardening (LSP DoS, action pins, nvim installer) ([#89](https://github.com/Arthurdw/rovo/pull/89)) by @Arthurdw
- fix(security): [aikido] Fix shell command injection vulnerability in rovo.serverPath configuration ([#88](https://github.com/Arthurdw/rovo/pull/88)) by @aikido-autofix[bot]
- fix(security): [aikido] Fix 5 security issues in zerovec-derive, zerovec, smallvec and 2 more ([#65](https://github.com/Arthurdw/rovo/pull/65)) by @aikido-autofix[bot]
- fix(security): [aikido] Fix 5 security issues in http, smallvec, mio and 1 more ([#66](https://github.com/Arthurdw/rovo/pull/66)) by @aikido-autofix[bot]
- fix(security): [aikido] Restrict credential exposure in VS Code and JetBrains publish workflows ([#63](https://github.com/Arthurdw/rovo/pull/63)) by @aikido-autofix[bot]
- fix(security): [aikido] Isolate secrets from repository-controlled Gradle code in JetBrains publish workflow ([#62](https://github.com/Arthurdw/rovo/pull/62)) by @aikido-autofix[bot]

### Chores

- chore(deps): bump vscode-languageclient from 9.0.1 to 10.1.2 in /vscode-rovo ([#84](https://github.com/Arthurdw/rovo/pull/84)) by @dependabot[bot]
- chore(deps): bump the gradle-minor-patch group in /jetbrains-plugin with 3 updates ([#79](https://github.com/Arthurdw/rovo/pull/79)) by @dependabot[bot]
- chore(deps-dev): bump @vscode/vsce from 3.7.1 to 4.0.0 in /vscode-rovo ([#83](https://github.com/Arthurdw/rovo/pull/83)) by @dependabot[bot]
- chore(deps): bump gradle/actions/setup-gradle from 3.5.0 to 6.4.0 ([#87](https://github.com/Arthurdw/rovo/pull/87)) by @dependabot[bot]
- chore(deps): bump actions/upload-artifact from 4 to 7 ([#86](https://github.com/Arthurdw/rovo/pull/86)) by @dependabot[bot]
- chore(deps): bump actions/checkout from 4 to 7 ([#85](https://github.com/Arthurdw/rovo/pull/85)) by @dependabot[bot]
- chore(deps-dev): bump typescript from 5.9.3 to 6.0.3 in /vscode-rovo ([#82](https://github.com/Arthurdw/rovo/pull/82)) by @dependabot[bot]
- chore(deps-dev): bump eslint from 9.39.1 to 10.12.0 in /vscode-rovo ([#71](https://github.com/Arthurdw/rovo/pull/71)) by @dependabot[bot]
- ci: ignore @types/vscode updates in dependabot ([#81](https://github.com/Arthurdw/rovo/pull/81)) by @Arthurdw
- ci: harden weekly release checkout credentials and pin actions ([#80](https://github.com/Arthurdw/rovo/pull/80)) by @Arthurdw
- chore(deps-dev): bump the npm-minor-patch group across 1 directory with 4 updates ([#69](https://github.com/Arthurdw/rovo/pull/69)) by @dependabot[bot]
- chore(deps): bump actions/setup-java from 4 to 5 ([#77](https://github.com/Arthurdw/rovo/pull/77)) by @dependabot[bot]
- chore(deps): bump actions/download-artifact from 4 to 8 ([#78](https://github.com/Arthurdw/rovo/pull/78)) by @dependabot[bot]
- chore(deps): bump codecov/codecov-action from 5.5.5 to 7.1.1 ([#76](https://github.com/Arthurdw/rovo/pull/76)) by @dependabot[bot]
- chore(deps): bump actions/setup-node from 4 to 7 ([#75](https://github.com/Arthurdw/rovo/pull/75)) by @dependabot[bot]
- chore(deps): bump actions/dependency-review-action from 4 to 5 ([#74](https://github.com/Arthurdw/rovo/pull/74)) by @dependabot[bot]
- chore(deps-dev): bump @types/node from 22.19.1 to 26.6.5 in /vscode-rovo ([#72](https://github.com/Arthurdw/rovo/pull/72)) by @dependabot[bot]
- chore(deps-dev): bump ovsx from 0.10.9 to 1.2.0 in /vscode-rovo ([#73](https://github.com/Arthurdw/rovo/pull/73)) by @dependabot[bot]
- ci: add dependabot config for cargo, npm, gradle and actions ([#67](https://github.com/Arthurdw/rovo/pull/67)) by @Arthurdw
- chore(deps): bump the npm_and_yarn group across 1 directory with 2 updates ([#58](https://github.com/Arthurdw/rovo/pull/58)) by @dependabot[bot]
- chore(deps): bump the npm_and_yarn group across 1 directory with 15 updates ([#57](https://github.com/Arthurdw/rovo/pull/57)) by @dependabot[bot]
- ci: skip auto-fix for dependabot PRs ([#52](https://github.com/Arthurdw/rovo/pull/52)) by @Arthurdw

### Other Changes

- deps: update version of stuff ([#64](https://github.com/Arthurdw/rovo/pull/64)) by @Arthurdw
- [Aikido] Add 1 gradle.lockfile file for dependency locking ([#61](https://github.com/Arthurdw/rovo/pull/61)) by @aikido-autofix[bot]
- [Aikido] AI Fix for Potential file inclusion attack via reading file ([#60](https://github.com/Arthurdw/rovo/pull/60)) by @aikido-autofix[bot]
- [Aikido] Fix security issue in brace-expansion via minor version upgrade from 2.0.2 to 2.1.7 in vscode-rovo ([#55](https://github.com/Arthurdw/rovo/pull/55)) by @aikido-autofix[bot]
- [Aikido] Fix security issue in anyhow via minor version upgrade from 1.0.102 to 1.0.103 in zed-rovo ([#54](https://github.com/Arthurdw/rovo/pull/54)) by @aikido-autofix[bot]
- [Aikido] Fix security issue in bytes via minor version upgrade from 1.11.1 to 1.12.1 in examples ([#53](https://github.com/Arthurdw/rovo/pull/53)) by @aikido-autofix[bot]

## [0.4.13] - 2026-10-13

### Features

- fix: replace deprecated serde_yaml with serde-saphyr ([#40](https://github.com/Arthurdw/rovo/pull/40)) by @Arthurdw

### Bug Fixes

- fix(release): bump version in llms-full.txt ([#39](https://github.com/Arthurdw/rovo/pull/39)) by @Arthurdw

## [0.4.12] - 2026-10-11

### Documentation

- docs: add llms.txt and generated llms-full.txt ([#38](https://github.com/Arthurdw/rovo/pull/38)) by @Arthurdw

### Other Changes

- fix/docs: Docs overhaul, prelude, extractor fixes and #[rovo(state = T)] ([#37](https://github.com/Arthurdw/rovo/pull/37)) by @Arthurdw

## [0.4.11] - 2026-09-26

### Bug Fixes

- fix(schema): use forked schemars derive instead of proxy type ([#36](https://github.com/Arthurdw/rovo/pull/36)) by @Arthurdw

## [0.4.10] - 2026-09-17

### Bug Fixes

- fix(macros): preserve visibility qualifiers on rovo handlers ([#35](https://github.com/Arthurdw/rovo/pull/35)) by @Arthurdw

## [0.4.9] - 2026-08-26

### Other Changes

- Allow missing docs on generated items ([#33](https://github.com/Arthurdw/rovo/pull/33)) by @matthiasbeyer

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
