# Justfile for Rovo project
# Run `just` or `just --list` to see available commands

# Default recipe shows available commands
default:
    @just --list

# Run all tests
test:
    cargo test --all-features --workspace

# Run tests quietly
test-quiet:
    cargo test --all-features --workspace --quiet

# Run tests including ignored ones
test-all:
    cargo test --all-features --workspace -- --include-ignored

# Run clippy lints
lint:
    cargo clippy --all-targets --all-features

# Fix clippy warnings automatically
lint-fix:
    cargo clippy --fix --allow-dirty --allow-staged --all-targets --all-features

# Format code
fmt:
    cargo fmt --all

# Check formatting without making changes
fmt-check:
    cargo fmt --all -- --check

# Build the project
build:
    cargo build --all-features

# Build in release mode
build-release:
    cargo build --release --all-features

# Clean build artifacts
clean:
    cargo clean

# Run the todo_api example with swagger UI
example:
    cargo run --example todo_api --features swagger

# Run all checks (fmt, clippy, test)
check: fmt-check lint test

# Run all checks and fixes
fix: fmt lint-fix test

# Check for outdated dependencies
outdated:
    cargo outdated

# Check for security vulnerabilities
audit:
    cargo audit

# Check for unused dependencies
unused-deps:
    cargo machete

# Check licenses
licenses:
    cargo deny check licenses

# Run all deny checks (licenses, advisories, bans, sources)
deny-check:
    cargo deny check

# Update dependencies
update:
    cargo update

# Build documentation
docs:
    cargo doc --all-features --no-deps --open

# Build documentation without opening
docs-build:
    cargo doc --all-features --no-deps

# Run benchmarks (if any)
bench:
    cargo bench --all-features

# Install development tools
install-tools:
    cargo install cargo-outdated
    cargo install cargo-audit
    cargo install cargo-edit
    cargo install cargo-machete
    cargo install cargo-deny

# Prepare for release (run all checks)
pre-release: fmt lint test
    @echo "All checks passed! Ready for release."

# Watch for changes and run tests
watch:
    cargo watch -x test

# Watch for changes and run clippy
watch-lint:
    cargo watch -x clippy

# --- Coverage Commands ---

# Run tests with coverage report (HTML)
coverage:
    cargo llvm-cov --all-features --workspace --html

# Run tests with coverage report (terminal summary only)
coverage-summary:
    cargo llvm-cov --all-features --workspace --summary-only

# Run tests with coverage and open HTML report
coverage-open:
    cargo llvm-cov --all-features --workspace --html --open

# Generate lcov.info for CI/external tools
coverage-lcov:
    cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info

# Clean coverage artifacts
coverage-clean:
    cargo llvm-cov clean

# Install coverage tool
install-coverage:
    cargo install cargo-llvm-cov

# --- LSP-Specific Commands ---

# Test only the LSP crate
test-lsp:
    cargo test --package rovo-lsp --all-features

# Test LSP with coverage
coverage-lsp:
    cargo llvm-cov --package rovo-lsp --all-features --html --open

# Build the LSP binary
build-lsp:
    cargo build --package rovo-lsp --release

# Run LSP server (for manual testing)
run-lsp:
    cargo run --package rovo-lsp

# Test only code actions
test-code-actions:
    cargo test --package rovo-lsp --test code_actions_test

# Test only handlers
test-handlers:
    cargo test --package rovo-lsp --test handlers_test

# Watch LSP tests
watch-lsp:
    cargo watch -x "test --package rovo-lsp"

# --- Combined Commands ---

# Run all quality checks including coverage
check-all: fmt-check lint test coverage-summary
    @echo "All quality checks passed!"

# Quick check (no coverage)
quick-check: fmt-check lint test-quiet
    @echo "Quick checks passed!"

# --- VSCode Extension Commands ---

# Install VSCode extension dependencies
vscode-install:
    cd vscode-rovo && npm install

# Build VSCode extension
vscode-build:
    cd vscode-rovo && npm run compile

# Lint VSCode extension
vscode-lint:
    cd vscode-rovo && npm run lint

# Fix VSCode extension lint issues
vscode-lint-fix:
    cd vscode-rovo && npm run lint:fix

# Package VSCode extension
vscode-package:
    cd vscode-rovo && npm run package

# Publish VSCode extension (requires VSCE_PAT)
vscode-publish:
    cd vscode-rovo && npx vsce publish

# Install local VSCode extension for testing
vscode-install-local: vscode-package
    #!/usr/bin/env bash
    set -euo pipefail
    cd vscode-rovo
    VSIX=$(ls -t *.vsix | head -1)
    code --uninstall-extension arthurdw.rovo-lsp || true
    code --install-extension "$VSIX"
    echo "Installed $VSIX. Reload VSCode to activate."
