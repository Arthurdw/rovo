<div align="center">
  <img src=".docs/rovo-banner.png" alt="Rovo - OpenAPI for Axum, the easy way" width="600">

  [![Crates.io](https://img.shields.io/crates/v/rovo.svg)](https://crates.io/crates/rovo)
  [![Documentation](https://docs.rs/rovo/badge.svg)](https://docs.rs/rovo)
  [![CI](https://github.com/arthurdw/rovo/actions/workflows/ci.yml/badge.svg)](https://github.com/arthurdw/rovo/actions)
  [![codecov](https://codecov.io/gh/arthurdw/rovo/branch/main/graph/badge.svg)](https://codecov.io/gh/arthurdw/rovo)
  [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
</div>

---

OpenAPI documentation for Axum using doc comments and macros.

Built on [aide](https://github.com/tamasfe/aide), Rovo provides a declarative approach to API documentation through special annotations in doc comments.

## Features

- Drop-in replacement for `axum::Router`
- Doc-comment driven documentation
- Compile-time validation of annotations
- Method chaining support (`.get()`, `.post()`, `.patch()`, `.delete()`)
- Built-in Swagger/Redoc/Scalar UI integration
- Type-safe response definitions
- Minimal runtime overhead
- **Editor Integration**:
  - **VSCode/VSCodium** - See [VSCODE.md](./VSCODE.md) for extension with auto-installation, completions, and syntax highlighting
  - **Neovim LSP** - See [NEOVIM.md](./NEOVIM.md) for editor support with completions, hover docs, and more
  - **JetBrains IDEs** - See [JETBRAINS.md](./JETBRAINS.md) for RustRover, IntelliJ IDEA, and CLion support
  - **Zed** - See [ZED.md](./ZED.md) for full Zed editor integration

## Quick Start

```rust
use rovo::{rovo, Router, routing::get};
use rovo::{schemars, schemars::JsonSchema};
use rovo::{aide, aide::{axum::IntoApiResponse, openapi::OpenApi}};
use rovo::{extract::State, response::Json};
use serde::Serialize;

#[derive(Clone)]
struct AppState {}

#[derive(Serialize, JsonSchema)]
struct User {
    id: u64,
    name: String,
}

/// Get user information.
///
/// Returns the current user's profile information.
///
/// # Responses
///
/// 200: Json<User> - User profile retrieved successfully
///
/// # Metadata
///
/// @tag users
#[rovo]
async fn get_user(State(_state): State<AppState>) -> impl IntoApiResponse {
    Json(User {
        id: 1,
        name: "Alice".to_string(),
    })
}

#[tokio::main]
async fn main() {
    let state = AppState {};

    let mut api = OpenApi::default();
    api.info.title = "My API".to_string();

    let app = Router::new()
        .route("/user", get(get_user))
        .with_oas(api)
        .with_swagger("/")
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}
```

## Installation

```toml
[dependencies]
rovo = { version = "0.3.7", features = ["swagger"] }
axum = "0.8"
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1", features = ["full"] }
```

> **Note:** Rovo re-exports `aide`, `schemars`, and common axum types (`extract`, `response`, `http`), so you can import them directly from rovo. The `axum` dependency is still needed for `axum::serve()`.

For detailed API documentation, see [docs.rs/rovo](https://docs.rs/rovo).

### Feature Flags

Choose one or more documentation UIs (none enabled by default):

- `swagger` - Swagger UI
- `redoc` - Redoc UI
- `scalar` - Scalar UI

## Documentation Format

Rovo uses Rust-style documentation with markdown sections and metadata annotations.

### Complete Example

```rust
/// Get a todo item by ID.
///
/// Retrieves a single todo item from the database. Returns 404
/// if the item doesn't exist.
///
/// # Path Parameters
///
/// id: The todo item's unique identifier
///
/// # Responses
///
/// 200: Json<TodoItem> - Successfully retrieved the todo item
/// 404: () - Todo item was not found
/// 500: Json<ErrorResponse> - Internal server error
///
/// # Examples
///
/// 200: TodoItem {
///   title: "Buy milk".into(),
///   ..Default::default()
/// }
/// 404: ()
///
/// # Metadata
///
/// @tag todos
/// @security bearer_auth
#[rovo]
async fn get_todo(Path(id): Path<i32>) -> impl IntoApiResponse {
    // ...
}
```

### Responses Section

Document HTTP responses with status codes, types, and descriptions:

```rust
/// # Responses
///
/// 200: Json<User> - User found successfully
/// 404: () - User not found
/// 500: Json<ErrorResponse> - Internal server error
```

**Format:** `<status_code>: <type> - <description>`

- Status codes must be valid HTTP codes (100-599)
- Type must be valid Rust syntax
- Description explains when this response occurs

### Path Parameters Section

Document path parameters for primitive types:

```rust
/// # Path Parameters
///
/// id: The user's unique identifier
/// index: Zero-based item index
```

**Format:** `<name>: <description>`

- Parameter names must match the variable bindings in your function signature
- Works with primitives: `String`, `u64`, `u32`, `i64`, `i32`, `bool`, `Uuid`, etc.
- For tuple paths like `Path((a, b)): Path<(Uuid, u32)>`, document each parameter

### Examples Section

Provide concrete response examples:

```rust
/// # Examples
///
/// 200: User { id: 1, name: "Alice".into(), email: "alice@example.com".into() }
/// 404: ()
```

**Format:** `<status_code>: <rust_expression>`

Examples should match the types defined in the Responses section.

### Metadata Section

Contains API metadata using `@` annotations:

#### `@tag`

Group operations by tags (can be used multiple times):

```rust
/// # Metadata
///
/// @tag users
/// @tag authentication
```

#### `@security`

Specify security requirements (can be used multiple times):

```rust
/// # Metadata
///
/// @security bearer_auth
```

Security schemes must be defined in your OpenAPI spec. See [Tips](#tips) for details.

#### `@id`

Set custom operation ID (defaults to function name):

```rust
/// # Metadata
///
/// @id getUserById
```

Must contain only alphanumeric characters and underscores.

#### `@hidden`

Hide an operation from documentation:

```rust
/// # Metadata
///
/// @hidden
```

### Special Directives

#### `#[deprecated]`

Mark endpoints as deprecated using Rust's built-in attribute:

```rust
#[deprecated]
#[rovo]
async fn old_handler() -> impl IntoApiResponse {
    // ...
}
```

#### `@rovo-ignore`

Stop processing annotations after this point (location-independent):

```rust
/// Get user information.
///
/// # Responses
///
/// 200: Json<User> - User found successfully
///
/// # Metadata
///
/// @tag users
///
/// @rovo-ignore
///
/// Additional documentation here won't be processed.
/// You can write @anything without causing errors.
#[rovo]
async fn handler() -> impl IntoApiResponse {
    // ...
}
```

## Router API

### Basic Usage

```rust
use rovo::Router;

let app = Router::new()
    .route("/path", get(handler))
    .with_state(state);
```

### Method Chaining

```rust
use rovo::routing::{get, post, patch, delete};

Router::new()
    .route("/items", get(list_items).post(create_item))
    .route("/items/{id}", get(get_item).patch(update_item).delete(delete_item))
```

### Nesting Routes

```rust
Router::new()
    .nest(
        "/api",
        Router::new()
            .route("/users", get(list_users))
            .route("/posts", get(list_posts))
    )
```

### Documentation UIs

```rust
Router::new()
    .route("/users", get(list_users))
    .with_oas(api)
    .with_swagger("/swagger")
    .with_redoc("/redoc")
    .with_scalar("/scalar")
    .with_state(state)
```

Use custom OAS route:

```rust
Router::new()
    .route("/users", get(list_users))
    .with_oas_route(api, "/openapi")
    .with_swagger("/")
    .with_state(state)
```

## OpenAPI Formats

Rovo automatically serves your OpenAPI specification in multiple formats:

- **JSON** - `/api.json` (default)
- **YAML** - `/api.yaml` or `/api.yml`

All formats are automatically available when you use `.with_oas()` or `.with_oas_route()`.

## Examples

See [examples/todo_api.rs](./examples/todo_api.rs) for a complete CRUD API.

Run with:

```bash
cargo run -F swagger --example todo_api
```

## Migration from Axum

Replace imports and add documentation:

```rust
// Before
use axum::{Router, response::IntoResponse, routing::get};

async fn handler() -> impl IntoResponse {
    Json(data)
}

// After
use rovo::{Router, routing::get, schemars::JsonSchema};
use rovo::{aide::axum::IntoApiResponse, response::Json};

/// Handler description
///
/// # Responses
///
/// 200: Json<Data> - Success
///
/// # Metadata
///
/// @tag category
#[rovo]
async fn handler() -> impl IntoApiResponse {
    Json(data)
}
```

Add OpenAPI setup in `main()`:

```rust
use rovo::aide::openapi::OpenApi;

let mut api = OpenApi::default();
api.info.title = "My API".to_string();

let app = Router::new()
    .route("/path", get(handler))
    .with_oas(api)
    .with_swagger("/")
    .with_state(state);
```

## Tips

### Path Parameters

For primitive types (`String`, `u64`, `Uuid`, `bool`, etc.), document parameters directly in doc comments:

```rust
/// Get user by ID.
///
/// # Path Parameters
///
/// id: The user's unique identifier
///
/// # Responses
///
/// 200: Json<User> - User found
#[rovo]
async fn get_user(Path(id): Path<u64>) -> impl IntoApiResponse {
    // ...
}
```

For tuple parameters:

```rust
/// Get item in collection.
///
/// # Path Parameters
///
/// collection_id: The collection UUID
/// index: Item index
///
/// # Responses
///
/// 200: Json<Item> - Item found
#[rovo]
async fn get_item(Path((collection_id, index)): Path<(Uuid, u32)>) -> impl IntoApiResponse {
    // ...
}
```

For complex types, use structs with `JsonSchema`:

```rust
use rovo::schemars::JsonSchema;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize, JsonSchema)]
struct UserId {
    /// The user's unique identifier
    id: Uuid,
}

#[rovo]
async fn get_user(Path(UserId { id }): Path<UserId>) -> impl IntoApiResponse {
    // ...
}
```

### Security Schemes

Define in OpenAPI object:

```rust
use rovo::aide::openapi::{SecurityScheme, SecuritySchemeData};

api.components.get_or_insert_default()
    .security_schemes
    .insert(
        "bearer_auth".to_string(),
        SecurityScheme {
            data: SecuritySchemeData::Http {
                scheme: "bearer".to_string(),
                bearer_format: Some("JWT".to_string()),
            },
            ..Default::default()
        },
    );
```

Reference in handlers:

```rust
/// Protected endpoint requiring authentication.
///
/// # Responses
///
/// 200: Json<Data> - Success
/// 401: () - Unauthorized
///
/// # Metadata
///
/// @security bearer_auth
#[rovo]
async fn protected_handler() -> impl IntoApiResponse {
    // ...
}
```

## Troubleshooting

### Handler doesn't implement required traits

Add the `#[rovo]` macro:

```rust
#[rovo]
async fn handler() -> impl IntoApiResponse {
    // ...
}
```

### Type mismatch with `.with_state()`

Add explicit type annotation:

```rust
let router: Router<()> = Router::<AppState>::new()
    .route("/path", get(handler))
    .with_state(state);
```

## Comparison with aide

| Feature | aide | rovo |
|---------|------|------|
| Documentation | Separate `_docs` function | Doc comments |
| Routing | `api_route()` | Native axum syntax |
| Method chaining | Custom | Standard axum |
| Lines per endpoint | ~15-20 | ~5-10 |

## Development

This project uses [just](https://github.com/casey/just) for common development tasks.

### Quick Start

```bash
# List all available commands
just

# Run all checks (format, clippy, tests)
just check

# Fix formatting and clippy issues
just fix

# Run tests
just test

# Check for outdated dependencies
just outdated

# Check for unused dependencies
just unused-deps

# Check for security vulnerabilities
just audit
```

### Pre-commit Hooks

Uses prek for git hooks:

```bash
prek install
prek run  # Run manually
```

### Available Commands

- `just test` - Run all tests
- `just lint` - Run clippy lints
- `just fmt` - Format code
- `just check` - Run all checks (fmt, clippy, test)
- `just fix` - Run all checks and fixes
- `just build` - Build the project
- `just example` - Run the todo_api example
- `just outdated` - Check for outdated dependencies
- `just unused-deps` - Check for unused dependencies
- `just audit` - Check for security vulnerabilities
- `just docs` - Build and open documentation
- `just pre-release` - Run all pre-release checks

See `just --list` for all available commands.

## Contributing

Contributions are welcome. Please submit a Pull Request.

## License

MIT
