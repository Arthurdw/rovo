# Justfile for Rovo project
# Run `just` or `just --list` to see available commands

# Default recipe shows available commands
default:
    @just --list

# Run all tests
test:
    cargo test --all-features --all

# Run tests quietly
test-quiet:
    cargo test --all-features --all --quiet

# Run tests including ignored ones
test-all:
    cargo test --all-features --all -- --include-ignored

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

# Prepare for release (run all checks)
pre-release: fmt lint test
    @echo "All checks passed! Ready for release."

# Watch for changes and run tests
watch:
    cargo watch -x test

# Watch for changes and run clippy
watch-lint:
    cargo watch -x clippy
