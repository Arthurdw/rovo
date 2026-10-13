<div align="center">
  <img src=".docs/rovo-banner.png" alt="Rovo - OpenAPI for Axum, the easy way" width="600">

  [![Crates.io](https://img.shields.io/crates/v/rovo.svg)](https://crates.io/crates/rovo)
  [![Documentation](https://docs.rs/rovo/badge.svg)](https://docs.rs/rovo)
  [![CI](https://github.com/arthurdw/rovo/actions/workflows/ci.yml/badge.svg)](https://github.com/arthurdw/rovo/actions)
  [![codecov](https://codecov.io/gh/arthurdw/rovo/branch/main/graph/badge.svg)](https://codecov.io/gh/arthurdw/rovo)
  [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
</div>

---

OpenAPI documentation for Axum, written as doc comments.

Document an endpoint in the doc comment above the handler, add `#[rovo]`, and keep routing exactly as you would in axum. Request bodies, query parameters and schemas are picked up from your extractors and types; rovo checks the doc comment at compile time.

```rust
/// Get a user by ID.
///
/// # Path Parameters
///
/// id: The user's ID
///
/// # Responses
///
/// 200: Json<User> - User found
/// 404: () - User not found
#[rovo]
async fn get_user(Path(id): Path<u64>) -> impl IntoApiResponse { /* ... */ }

let app = Router::new().route("/users/{id}", get(get_user));
```

## Features

- Drop-in replacement for `axum::Router`, with the same `.route(path, get(handler))` syntax
- Doc-comment driven documentation, readable as plain rustdoc
- Compile-time validation of annotations
- Request bodies (`Json`, `Form`, `Multipart`) and query parameters documented from extractors
- Built-in Swagger/Redoc/Scalar UI integration
- Type-safe response definitions
- Minimal runtime overhead
- **Editor Integration** (completions, hover docs and diagnostics for doc annotations):
  - **VSCode/VSCodium** - See [VSCODE.md](./VSCODE.md) for extension with auto-installation, completions, and syntax highlighting
  - **Neovim LSP** - See [NEOVIM.md](./NEOVIM.md) for editor support with completions, hover docs, and more
  - **JetBrains IDEs** - See [JETBRAINS.md](./JETBRAINS.md) for RustRover, IntelliJ IDEA, and CLion support
  - **Zed** - See [ZED.md](./ZED.md) for full Zed editor integration

## Quick Start

```rust
use rovo::prelude::*;
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
        .with_state(state)
        .finish();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}
```

## Installation

```toml
[dependencies]
rovo = { version = "0.4.13", features = ["swagger"] }
axum = "0.8"
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1", features = ["full"] }
```

> **Note:** `rovo::prelude::*` brings in the macro, router, routing functions and the common extractors and response types. Rovo also re-exports `aide`, `schemars`, and axum's `extract`, `response` and `http` modules, so you don't need them as direct dependencies. The `axum` dependency is still needed for `axum::serve()`.

For detailed API documentation, see [docs.rs/rovo](https://docs.rs/rovo).

Using an AI coding assistant? Point it at [`llms.txt`](./llms.txt) for a compact reference, or [`llms-full.txt`](./llms-full.txt) for the README and all examples in one file. Both are also shipped in the published crate.

> **schemars version:** rovo uses schemars 0.9, because aide 0.15 does. If your project also uses schemars 1.x, derive `rovo::schemars::JsonSchema` on the types your handlers return; see [Troubleshooting](#jsonschema-is-not-implemented-with-schemars-1x-in-the-project).

### Feature Flags

Choose one or more documentation UIs (none enabled by default):

- `swagger` - Swagger UI
- `redoc` - Redoc UI
- `scalar` - Scalar UI

Optional extractor support:

- `multipart` - Document handlers that take `axum::extract::Multipart`
- `ws` - Allow handlers that take `axum::extract::WebSocketUpgrade`

## Compared to utoipa and aide

The same endpoint in each library: a `GET /users/{id}` with a described path parameter, a tag and two responses.

**utoipa** (with `utoipa-axum`):

```rust
#[derive(Serialize, ToSchema)]
struct User { id: u64, name: String }

/// Get a user by ID.
#[utoipa::path(
    get,
    path = "/users/{id}",
    tag = "users",
    params(("id" = u64, Path, description = "The user's ID")),
    responses(
        (status = 200, description = "User found", body = User),
        (status = 404, description = "User not found"),
    ),
)]
async fn get_user(Path(id): Path<u64>) -> Result<Json<User>, StatusCode> { /* ... */ }

let (router, api) = OpenApiRouter::new()
    .routes(routes!(get_user))
    .split_for_parts();
```

**aide**:

```rust
#[derive(Serialize, JsonSchema)]
struct User { id: u64, name: String }

#[derive(Deserialize, JsonSchema)]
struct UserPath {
    /// The user's ID
    id: u64,
}

async fn get_user(Path(UserPath { id }): Path<UserPath>) -> impl IntoApiResponse { /* ... */ }

fn get_user_docs(op: TransformOperation) -> TransformOperation {
    op.description("Get a user by ID.")
        .tag("users")
        .response_with::<200, Json<User>, _>(|res| res.description("User found"))
        .response_with::<404, (), _>(|res| res.description("User not found"))
}

let router = ApiRouter::new()
    .api_route("/users/{id}", get_with(get_user, get_user_docs));
```

**rovo**:

```rust
#[derive(Serialize, JsonSchema)]
struct User { id: u64, name: String }

/// Get a user by ID.
///
/// # Path Parameters
///
/// id: The user's ID
///
/// # Responses
///
/// 200: Json<User> - User found
/// 404: () - User not found
///
/// # Metadata
///
/// @tag users
#[rovo]
async fn get_user(Path(id): Path<u64>) -> impl IntoApiResponse { /* ... */ }

let router = Router::new().route("/users/{id}", get(get_user));
```

| | utoipa | aide | rovo |
|---|---|---|---|
| Where docs live | `#[utoipa::path(...)]` attribute | A separate `TransformOperation` function | The handler's doc comment |
| Routing | `OpenApiRouter` + `routes!` | `api_route` + `get_with` | axum's `.route(path, get(handler))` |
| Frameworks | axum, actix-web, rocket and more | axum | axum |
| Docs checked at compile time | Yes | Yes (plain Rust code) | Yes |
| Editor support for the docs | Standard Rust tooling | Standard Rust tooling | Language server with completions and hover |
| Maturity | Most widely used | Established | Pre-1.0, smaller community |

Rovo is built on aide, so everything aide generates (schemas, request bodies, query parameters) is available, and `Router::into_inner()` hands you the underlying aide `ApiRouter` when you need something rovo doesn't cover.

**Pick utoipa** if you're not on axum, or you want the largest ecosystem. **Pick aide** if you prefer documentation as plain Rust function calls. **Pick rovo** if you're on axum and want your API docs to read like normal doc comments next to the handler.

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

## Request Inputs

You don't document request inputs in the doc comment: rovo reads them from the handler's extractors.

| Extractor | Documented as |
|-----------|---------------|
| `Json<T>` | `application/json` request body with `T`'s schema |
| `Form<T>` | `application/x-www-form-urlencoded` request body with `T`'s schema |
| `Multipart` (feature `multipart`) | `multipart/form-data` request body |
| `Query<T>` | One query parameter per field of `T` |
| `Path<T>` | Path parameters (see [Path Parameters](#path-parameters-section)) |
| `State<T>`, `HeaderMap` | Nothing (not part of the API surface) |

Doc comments on struct fields become parameter and property descriptions, and `Option<_>` fields are marked as not required:

```rust
#[derive(Deserialize, JsonSchema)]
struct SearchParams {
    /// Maximum number of results to return
    limit: Option<u32>,
    /// Only return users whose name contains this string
    name: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
struct CreateUser {
    /// Display name
    name: String,
}

/// Search users.
///
/// # Responses
///
/// 200: Json<Vec<User>> - Matching users
#[rovo]
async fn search_users(Query(params): Query<SearchParams>) -> impl IntoApiResponse {
    // documented as GET with `limit` and `name` query parameters
}

/// Create a user.
///
/// # Responses
///
/// 201: Json<User> - User created
/// 422: Json<ErrorResponse> - Invalid input
#[rovo]
async fn create_user(Json(body): Json<CreateUser>) -> impl IntoApiResponse {
    // documented with a JSON request body
}
```

Headers read through `HeaderMap` aren't added to the spec. For authentication headers, declare a [security scheme](#security-schemes) and reference it with `@security`.

Schemas come from [schemars](https://docs.rs/schemars), so serde attributes are respected. For example, an enum with `#[serde(tag = "type")]` becomes a `oneOf` with one object per variant.

### Custom extractors and error types

Every extractor in a `#[rovo]` handler must implement `aide::OperationInput`, and every return type `aide::OperationOutput`. For your own types, an empty `OperationInput` impl and an `OperationOutput` impl naming the response body are usually enough, since the `# Responses` section documents the actual responses:

```rust
use rovo::aide::{OperationInput, OperationOutput};

struct AuthUser { /* ... */ }
impl FromRequestParts<AppState> for AuthUser { /* ... */ }
impl OperationInput for AuthUser {}

enum ApiError { /* ... */ }
impl IntoResponse for ApiError { /* ... */ }
impl OperationOutput for ApiError {
    type Inner = ErrorBody;
}
```

`#[rovo]` takes the router's state type from a `State<T>` parameter, or uses `()` if there is none. If a custom extractor needs state but the handler has no `State<T>` parameter, name the state type in the attribute:

```rust
#[rovo(state = AppState)]
async fn me(user: AuthUser) -> Result<Json<Profile>, ApiError> {
    // ...
}
```

See [examples/auth_api.rs](./examples/auth_api.rs) for a complete bearer-token example.

## Router API

### Basic Usage

```rust
use rovo::Router;

let app = Router::new()
    .route("/path", get(handler))
    .with_state(state)
    .finish();
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

Nest routers with different state types by calling `.with_state()` on each inner router:

```rust
let app = Router::new()
    .nest(
        "/api",
        Router::new()
            .route("/todos", get(list_todos))
            .with_state(todo_state),
    )
    .nest(
        "/meta",
        Router::new()
            .route("/health", get(health_check))
            .with_state(meta_state),
    )
    .with_oas(api)
    .finish();
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
    .finish()
```

Use custom OAS route:

```rust
Router::new()
    .route("/users", get(list_users))
    .with_oas_route(api, "/openapi")
    .with_swagger("/")
    .with_state(state)
    .finish()
```

## OpenAPI Formats

Rovo automatically serves your OpenAPI specification in multiple formats:

- **JSON** - `/api.json` (default)
- **YAML** - `/api.yaml` or `/api.yml`

All formats are automatically available when you use `.with_oas()` or `.with_oas_route()`.

## Examples

| Example | Shows |
|---------|-------|
| [hello_world](./examples/hello_world) | The smallest documented handler, as a standalone crate |
| [todo_api](./examples/todo_api.rs) | A CRUD API with path parameters and nested routers with different state types |
| [auth_api](./examples/auth_api.rs) | Bearer-token auth with a custom extractor, a custom error type, form login and `@security` |
| [catalog_api](./examples/catalog_api.rs) | Query-string pagination, a generic `Page<T>` response, a tagged enum (`oneOf`) and a multipart upload (needs `-F multipart`) |

Run one with:

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
use rovo::prelude::*;

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
let mut api = OpenApi::default();
api.info.title = "My API".to_string();

let app = Router::new()
    .route("/path", get(handler))
    .with_oas(api)
    .with_swagger("/")
    .with_state(state)
    .finish();
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
use rovo::aide::openapi::{ReferenceOr, SecurityScheme};

api.components.get_or_insert_default().security_schemes.insert(
    "bearer_auth".to_string(),
    ReferenceOr::Item(SecurityScheme::Http {
        scheme: "bearer".to_string(),
        bearer_format: Some("JWT".to_string()),
        description: None,
        extensions: Default::default(),
    }),
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

### `JsonSchema` is not implemented with schemars 1.x in the project

```text
error[E0277]: the trait bound `User: rovo::schemars::JsonSchema` is not satisfied
note: there are multiple different versions of crate `schemars` in the dependency graph
```

rovo uses schemars 0.9 (through aide 0.15), so a type that only derives `schemars::JsonSchema` from schemars 1.x doesn't implement the trait rovo needs. Both versions can live in one project, and one type can derive both:

```rust
#[derive(Serialize, rovo::schemars::JsonSchema, schemars::JsonSchema)]
struct User {
    /// Shared `#[schemars(...)]` attributes apply to both derives.
    #[schemars(range(min = 1))]
    id: u64,
}
```

### Using different state types

Call `.with_state()` on each nested router individually, then `.finish()` at the top level (see [Nesting Routes](#nesting-routes)).

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
