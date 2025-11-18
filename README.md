# rovo

A drop-in replacement for axum's Router that adds automatic OpenAPI documentation using doc comments.

Built on top of [aide](https://github.com/tamasfe/aide), rovo provides a seamless way to document your axum APIs without writing separate documentation functions.

## Features

- 🎯 **Drop-in replacement**: Use `rovo::Router` instead of `axum::Router` with the exact same API
- 📝 **Doc-comment driven**: Write API docs as Rust doc comments with special annotations
- ✅ **Compile-time validation**: Catches documentation errors at compile time, not runtime
- 🔄 **Method chaining**: Supports `.post()`, `.patch()`, `.delete()` just like axum
- 🚀 **Simplified setup**: Helper methods for Swagger UI and OpenAPI JSON endpoints
- 🏷️ **Rich annotations**: Support for tags, security, deprecation, examples, and more
- ⚡ **Type-safe**: Full type checking for response types and examples
- 🪶 **Lightweight**: Minimal overhead over plain axum

## Installation

```toml
[dependencies]
rovo = "0.1"
aide = { version = "0.15", features = ["axum"] }
axum = "0.8"
schemars = "0.8"
serde = { version = "1.0", features = ["derive"] }
```

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

async fn serve_api(Extension(api): Extension<OpenApi>) -> axum::Json<OpenApi> {
    axum::Json(api)
}

#[tokio::main]
async fn main() {
    let state = AppState {};

    let mut api = OpenApi::default();
    api.info.title = "My API".to_string();

    let app = Router::new()
        .route("/user", get(get_user))
        .with_swagger("/", "/api.json")
        .with_api_json("/api.json", serve_api)
        .with_state(state)
        .finish_api_with_extension(api);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}
```

Visit `http://127.0.0.1:3000` to see your interactive Swagger UI documentation!

## Documentation Annotations

### Basic Structure

```rust
/// Title (first line becomes the summary)
///
/// Description paragraph can span multiple lines
/// and provides detailed information about the endpoint.
///
/// @tag category_name
/// @response 200 Json<ResponseType> Success description
/// @response 404 () Not found description
#[rovo]
async fn handler() -> impl IntoApiResponse {
    // ...
}
```

### Available Annotations

#### `@response <code> <type> <description>`

Document response status codes:

```rust
/// @response 200 Json<User> User found successfully
/// @response 404 () User not found
/// @response 500 Json<ErrorResponse> Internal server error
```

#### `@example <code> <expression>`

Provide example responses:

```rust
/// @response 200 Json<User> User information
/// @example 200 User::default()
```

Or with custom values:

```rust
/// @example 200 User { id: 1, name: "Alice".into(), email: "alice@example.com".into() }
```

#### `@tag <tag_name>`

Group operations by tags (can be used multiple times):

```rust
/// @tag users
/// @tag authentication
```

Tags help organize your API in Swagger UI by grouping related endpoints together.

#### `@security <scheme_name>`

Specify security requirements (can be used multiple times):

```rust
/// @security bearer_auth
/// @security api_key
```

Note: You need to define security schemes in your OpenAPI spec separately.

#### `@id <operation_id>`

Set a custom operation ID (defaults to function name):

```rust
/// @id getUserById
```

#### `@hidden`

Hide an operation from the documentation:

```rust
/// @hidden
```

#### `#[deprecated]`

Use Rust's built-in deprecation attribute to mark endpoints as deprecated:

```rust
/// Old endpoint, use /v2/users instead
#[deprecated]
#[rovo]
async fn old_handler() -> impl IntoApiResponse {
    // ...
}
```

## Router API

### Creating a Router

```rust
use rovo::Router;

let app = Router::new()
    .route("/path", get(handler))
    .with_state(state);
```

### Method Chaining

Rovo supports the same method chaining as axum:

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

### Adding Swagger UI

```rust
Router::new()
    .route("/users", get(list_users))
    .with_swagger("/docs", "/api.json")  // Swagger UI at /docs
    .with_api_json("/api.json", serve_api)
    .with_state(state)
    .finish_api_with_extension(api)
```

## Complete Example

See [examples/todo_api.rs](./examples/todo_api.rs) for a full CRUD API with:
- Create, Read, Update, Delete operations
- Swagger UI integration
- Proper error handling
- Request/response validation
- Nested routing

Run it with:

```bash
cargo run --example todo_api
# Visit http://127.0.0.1:3000 for Swagger UI
```

## Migration Guide

### From Axum 0.8+

Migrating an existing axum project to rovo is straightforward:

#### Step 1: Update Dependencies

```toml
[dependencies]
# Add these
rovo = "0.1"
aide = { version = "0.15", features = ["axum"] }
schemars = "0.8"

# Keep your existing axum
axum = "0.8"
```

#### Step 2: Replace Router Import

```rust
// Before
use axum::Router;

// After
use rovo::Router;
```

#### Step 3: Update Handler Return Types

```rust
// Before
use axum::response::IntoResponse;
async fn handler() -> impl IntoResponse {
    Json(data)
}

// After
use aide::axum::IntoApiResponse;
async fn handler() -> impl IntoApiResponse {
    Json(data)
}
```

#### Step 4: Add the #[rovo] Macro and Docs

```rust
// Before
async fn get_user(State(state): State<AppState>) -> impl IntoApiResponse {
    Json(user)
}

// After
/// Get user by ID
///
/// @tag users
/// @response 200 Json<User> User found
/// @response 404 () User not found
#[rovo]
async fn get_user(State(state): State<AppState>) -> impl IntoApiResponse {
    Json(user)
}
```

#### Step 5: Update Routing Imports

```rust
// Before
use axum::routing::{get, post};

// After
use rovo::routing::{get, post};
```

#### Step 6: Add OpenAPI Setup

```rust
use aide::openapi::OpenApi;
use axum::Extension;

async fn serve_api(Extension(api): Extension<OpenApi>) -> axum::Json<OpenApi> {
    axum::Json(api)
}

#[tokio::main]
async fn main() {
    let state = AppState::new();

    let mut api = OpenApi::default();
    api.info.title = "My API".to_string();
    api.info.description = Some("API description".to_string());

    let app = Router::new()
        .route("/users", get(list_users))
        // ... your other routes
        .with_swagger("/", "/api.json")
        .with_api_json("/api.json", serve_api)
        .with_state(state)
        .finish_api_with_extension(api);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}
```

### Migration Checklist

- [ ] Add `rovo` and `aide` dependencies
- [ ] Change `axum::Router` to `rovo::Router`
- [ ] Change `IntoResponse` to `IntoApiResponse`
- [ ] Add `#[rovo]` macro to handlers
- [ ] Add doc comments with `@response` annotations
- [ ] Change `axum::routing::*` to `rovo::routing::*`
- [ ] Add OpenAPI configuration
- [ ] Add Swagger UI setup
- [ ] Test all endpoints

### Incremental Migration

You can migrate gradually by mixing rovo and aide routing:

```rust
use rovo::routing::get as rovo_get;
use aide::axum::routing::get as aide_get;

Router::new()
    .route("/documented", rovo_get(documented_handler))  // Migrated with #[rovo]
    .route("/legacy", aide_get(legacy_handler))          // Not yet migrated
```

However, we recommend fully migrating to `#[rovo]` for all endpoints to maintain consistency.

## Comparison with aide

| Feature | aide | rovo |
|---------|------|------|
| Documentation location | Separate `_docs` function | With handler (doc comments) |
| Routing API | aide's `api_route()` | Drop-in axum replacement |
| Method chaining | Custom implementation | Native axum syntax |
| Setup complexity | Manual | Helper methods |
| Lines of code per endpoint | ~15-20 | ~5-10 |

## Tips and Best Practices

### Path Parameters

Use structs with `JsonSchema` for proper documentation:

```rust
#[derive(Deserialize, JsonSchema)]
struct UserId {
    /// The unique user identifier
    id: Uuid,
}

#[rovo]
async fn get_user(Path(UserId { id }): Path<UserId>) -> impl IntoApiResponse {
    // ...
}
```

### Complex Response Types

For handlers that return multiple types, use `impl IntoApiResponse`:

```rust
#[rovo]
async fn handler() -> impl IntoApiResponse {
    if condition {
        (StatusCode::OK, Json(data)).into_response()
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}
```

### Tags for Organization

Use consistent tags to organize your API:

```rust
/// @tag users
/// @tag admin
```

### Security Documentation

Define security schemes in your OpenAPI object:

```rust
use aide::openapi::{SecurityScheme, SecuritySchemeData};

let mut api = OpenApi::default();
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

Then reference it in handlers:

```rust
/// @security bearer_auth
#[rovo]
async fn protected_handler() -> impl IntoApiResponse {
    // ...
}
```

## Troubleshooting

### Handler doesn't implement required traits

**Error**: "doesn't implement `IntoApiMethodRouter`"

**Solution**: Make sure you added the `#[rovo]` macro to your handler:

```rust
#[rovo]  // Don't forget this!
async fn handler() -> impl IntoApiResponse {
    // ...
}
```

### Type mismatch errors with `.with_state()`

**Error**: Type mismatch when calling `.with_state()`

**Solution**: Add explicit type annotation:

```rust
let router: Router<()> = Router::<AppState>::new()
    .route("/path", get(handler))
    .with_state(state);
```

### Routes return 404

**Problem**: Routes work in axum but return 404 in rovo

**Solution**: Make sure you're using axum path syntax (`{id}`), not OpenAPI syntax:

```rust
// ✅ Correct
.route("/todos/{id}", get(get_todo))

// ❌ Wrong
.route("/todos/:id", get(get_todo))
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT OR Apache-2.0
