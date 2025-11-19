# Rovo

OpenAPI documentation for Axum using doc comments and macros.

Built on [aide](https://github.com/tamasfe/aide), Rovo provides a declarative approach to API documentation through special annotations in doc comments.

## Features

- Drop-in replacement for `axum::Router`
- Doc-comment driven documentation
- Compile-time validation of annotations
- Method chaining support (`.post()`, `.patch()`, `.delete()`)
- Built-in Swagger/Redoc/Scalar UI integration
- Type-safe response definitions
- Minimal runtime overhead

## Installation

```toml
[dependencies]
rovo = { version = "0.1", features = ["swagger"] }
aide = { version = "0.15", features = ["axum"] }
axum = "0.8"
schemars = "0.8"
serde = { version = "1.0", features = ["derive"] }
```

### Feature Flags

Choose one or more documentation UIs (none enabled by default):

- `swagger` - Swagger UI
- `redoc` - Redoc UI
- `scalar` - Scalar UI

## OpenAPI Formats

Rovo automatically serves your OpenAPI specification in multiple formats:

- **JSON** - `/api.json` (default)
- **YAML** - `/api.yaml` or `/api.yml`

All formats are automatically available when you use `.with_oas()`.

## Quick Start

```rust
use aide::axum::IntoApiResponse;
use aide::openapi::OpenApi;
use axum::{extract::State, response::Json, Extension};
use rovo::{rovo, Router, routing::get};
use schemars::JsonSchema;
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
/// @tag users
/// @response 200 Json<User> User profile retrieved successfully.
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

## Annotations

### `@response <code> <type> <description>`

Document response status codes:

```rust
/// @response 200 Json<User> User found successfully
/// @response 404 () User not found
/// @response 500 Json<ErrorResponse> Internal server error
```

### `@example <code> <expression>`

Provide example responses:

```rust
/// @response 200 Json<User> User information
/// @example 200 User { id: 1, name: "Alice".into(), email: "alice@example.com".into() }
```

### `@tag <tag_name>`

Group operations by tags:

```rust
/// @tag users
/// @tag authentication
```

### `@security <scheme_name>`

Specify security requirements:

```rust
/// @security bearer_auth
```

Security schemes must be defined in your OpenAPI spec.

### `@id <operation_id>`

Set custom operation ID (defaults to function name):

```rust
/// @id getUserById
```

### `@hidden`

Hide an operation from documentation:

```rust
/// @hidden
```

### `#[deprecated]`

Mark endpoints as deprecated using Rust's built-in attribute:

```rust
#[deprecated]
#[rovo]
async fn old_handler() -> impl IntoApiResponse {
    // ...
}
```

### `@rovo-ignore`

Stop processing annotations after this point:

```rust
/// Get user information.
///
/// @tag users
/// @response 200 Json<User> User found successfully
/// @rovo-ignore
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
    .with_oas_route(api, "/openapi")  // Creates /openapi.json, /openapi.yaml, and /openapi.yml
    .with_swagger("/")
    .with_state(state)
```

The OpenAPI spec is automatically available in multiple formats:
- JSON: `/api.json` (or `/openapi.json` with custom route)
- YAML: `/api.yaml` or `/api.yml` (or `/openapi.yaml`/`/openapi.yml` with custom route)

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
use rovo::{Router, routing::get};
use aide::axum::IntoApiResponse;

/// Handler description
///
/// @tag category
/// @response 200 Json<Data> Success
#[rovo]
async fn handler() -> impl IntoApiResponse {
    Json(data)
}
```

Add OpenAPI setup in `main()`:

```rust
use aide::openapi::OpenApi;

let mut api = OpenApi::default();
api.info.title = "My API".to_string();

let app = Router::new()
    .route("/path", get(handler))
    .with_oas(api)
    .with_swagger("/")
    .with_state(state);
```

## Comparison with aide

| Feature | aide | rovo |
|---------|------|------|
| Documentation | Separate `_docs` function | Doc comments |
| Routing | `api_route()` | Native axum syntax |
| Method chaining | Custom | Standard axum |
| Lines per endpoint | ~15-20 | ~5-10 |

## Tips

### Path Parameters

Use structs with `JsonSchema`:

```rust
#[derive(Deserialize, JsonSchema)]
struct UserId {
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
use aide::openapi::{SecurityScheme, SecuritySchemeData};

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

## Contributing

Contributions are welcome. Please submit a Pull Request.

## License

GPL-3.0
