#![warn(clippy::all)]
#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![warn(missing_docs)]
#![warn(rust_2018_idioms)]
#![deny(unsafe_code)]
// Allow some overly strict pedantic lints
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::similar_names)]

//! # Rovo
//!
//! A lightweight macro-based framework for generating `OpenAPI` documentation
//! from doc comments in Axum handlers.
//!
//! Rovo provides a declarative way to document your API endpoints using special
//! annotations in doc comments, automatically generating `OpenAPI` specifications
//! without manual schema definitions.
//!
//! ## Quick Start
//!
//! ```no_run
//! use rovo::{Router, rovo, routing::get, schemars::JsonSchema, aide::axum::IntoApiResponse};
//! use rovo::aide::openapi::OpenApi;
//! use axum::{extract::Path, response::Json};
//! use serde::Serialize;
//!
//! #[derive(Serialize, JsonSchema)]
//! struct User { id: u32, name: String }
//!
//! /// Get user by ID
//! ///
//! /// # Responses
//! ///
//! /// 200: Json<User> - User found successfully
//! /// 404: () - User not found
//! ///
//! /// # Metadata
//! ///
//! /// @tag users
//! #[rovo]
//! async fn get_user(Path(id): Path<u32>) -> impl IntoApiResponse {
//!     Json(User { id, name: "Alice".to_string() })
//! }
//!
//! # #[tokio::main]
//! # async fn main() {
//! let mut api = OpenApi::default();
//! api.info.title = "My API".to_string();
//!
//! let app = Router::new()
//!     .route("/users/{id}", get(get_user))
//!     .with_oas(api);
//! # }
//! ```
//!
//! ## Documentation Format
//!
//! Rovo uses Rust-style markdown sections in doc comments:
//!
//! ### Responses Section
//! Document HTTP response codes and their types:
//! ```text
//! /// # Responses
//! ///
//! /// 200: Json<User> - User found successfully
//! /// 404: () - User not found
//! ```
//!
//! ### Examples Section
//! Provide response examples with valid Rust expressions:
//! ```text
//! /// # Examples
//! ///
//! /// 200: User { id: 1, name: "Alice".into() }
//! /// 404: ()
//! ```
//!
//! ### Metadata Section
//! Add API metadata with annotations:
//! ```text
//! /// # Metadata
//! ///
//! /// @tag users
//! /// @security bearer_auth
//! /// @id getUserById
//! /// @hidden
//! ```
//!
//! **Available metadata annotations:**
//! - `@tag <name>` - Group endpoints by tags
//! - `@security <scheme>` - Specify security requirements
//! - `@id <operation_id>` - Set custom operation ID
//! - `@hidden` - Hide endpoint from documentation
//!
//! **Special directives:**
//! - `@rovo-ignore` - Stop processing annotations after this point

pub use rovo_macros::rovo;

// Re-export aide and schemars for convenience
pub use aide;
pub use schemars;

use ::axum::{response::IntoResponse, Extension};
use aide::axum::ApiRouter as AideApiRouter;
use aide::openapi::OpenApi;

/// A drop-in replacement for `axum::Router` that adds `OpenAPI` documentation support.
///
/// This Router works seamlessly with handlers decorated with `#[rovo]` and provides
/// a fluent API for building documented APIs with Swagger UI integration.
///
/// # Example
/// ```no_run
/// use rovo::{Router, rovo, routing::get, aide::axum::IntoApiResponse};
/// use rovo::aide::openapi::OpenApi;
/// use axum::response::Json;
///
/// #[rovo]
/// async fn documented_handler() -> impl IntoApiResponse {
///     Json(())
/// }
///
/// # #[tokio::main]
/// # async fn main() {
/// let mut api = OpenApi::default();
/// api.info.title = "My API".to_string();
///
/// let app = Router::new()
///     .route("/documented", get(documented_handler))
///     .with_oas(api);
/// # }
/// ```
pub struct Router<S = ()> {
    inner: AideApiRouter<S>,
    oas_spec: Option<OpenApi>,
    oas_route: String,
}

impl<S> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    /// Create a new Router
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: AideApiRouter::new(),
            oas_spec: None,
            oas_route: "/api.json".to_string(),
        }
    }

    /// Add a route to the router
    pub fn route<M>(mut self, path: &str, method_router: M) -> Self
    where
        M: Into<aide::axum::routing::ApiMethodRouter<S>>,
    {
        self.inner = self.inner.api_route(path, method_router.into());
        self
    }

    /// Nest another router at the given path
    #[must_use]
    pub fn nest(mut self, path: &str, router: Self) -> Self {
        self.inner = self.inner.nest(path, router.inner);
        // Adopt nested router's OAS spec if parent doesn't have one
        if self.oas_spec.is_none() && router.oas_spec.is_some() {
            self.oas_spec = router.oas_spec;
            self.oas_route = router.oas_route;
        }
        self
    }

    /// Configure `OpenAPI` spec with default routes (/api.json and /api.yaml)
    ///
    /// This automatically sets up endpoints for both JSON and YAML formats.
    #[must_use]
    pub fn with_oas(mut self, api: OpenApi) -> Self {
        self.oas_spec = Some(api);
        self.oas_route = "/api.json".to_string();
        self
    }

    /// Configure `OpenAPI` spec with custom base route
    ///
    /// This sets up endpoints with the specified base route.
    /// For example, "/openapi" creates:
    /// - /openapi.json
    /// - /openapi.yaml
    pub fn with_oas_route(mut self, api: OpenApi, route: impl Into<String>) -> Self {
        self.oas_spec = Some(api);
        let route_str = route.into();
        // Remove extension if provided
        let base_route = route_str
            .strip_suffix(".json")
            .or_else(|| route_str.strip_suffix(".yaml"))
            .or_else(|| route_str.strip_suffix(".yml"))
            .unwrap_or(&route_str);
        self.oas_route = format!("{base_route}.json");
        self
    }

    /// Add Swagger UI route at the specified path
    #[cfg(feature = "swagger")]
    #[must_use]
    pub fn with_swagger(mut self, swagger_path: &str) -> Self
    where
        S: Clone + Send + Sync + 'static,
    {
        let api_route = self.oas_route.clone();
        self.inner = self.inner.route(
            swagger_path,
            aide::swagger::Swagger::new(&api_route).axum_route(),
        );
        self
    }

    /// Add Redoc UI route at the specified path
    #[cfg(feature = "redoc")]
    #[must_use]
    pub fn with_redoc(mut self, redoc_path: &str) -> Self
    where
        S: Clone + Send + Sync + 'static,
    {
        let api_route = self.oas_route.clone();
        self.inner = self
            .inner
            .route(redoc_path, aide::redoc::Redoc::new(&api_route).axum_route());
        self
    }

    /// Add Scalar UI route at the specified path
    #[cfg(feature = "scalar")]
    #[must_use]
    pub fn with_scalar(mut self, scalar_path: &str) -> Self
    where
        S: Clone + Send + Sync + 'static,
    {
        let api_route = self.oas_route.clone();
        self.inner = self.inner.route(
            scalar_path,
            aide::scalar::Scalar::new(&api_route).axum_route(),
        );
        self
    }

    /// Internal helper to wire up `OpenAPI` endpoints and extension
    fn wire_openapi_routes(self) -> (Option<::axum::Router<S>>, Option<AideApiRouter<S>>)
    where
        S: Clone + Send + Sync + 'static,
    {
        if let Some(api) = self.oas_spec {
            let oas_route = self.oas_route.clone();

            // Finish API first to populate it with routes
            let mut api_mut = api;
            let axum_router = self.inner.finish_api(&mut api_mut);

            // Now api_mut is populated with all the routes
            // Clone it for the JSON/YAML/YML handlers
            let api_for_json = api_mut.clone();
            let api_for_yaml = api_mut.clone();
            let api_for_yml = api_mut.clone();

            // Determine base route (without extension)
            let base_route = oas_route.strip_suffix(".json").unwrap_or(&oas_route);

            // Add JSON endpoint to the axum::Router
            let router_with_json = axum_router.route(
                &oas_route,
                ::axum::routing::get(move || {
                    let api = api_for_json.clone();
                    async move { ::axum::Json(api) }
                }),
            );

            // Add YAML endpoint
            let yaml_route = format!("{base_route}.yaml");
            let router_with_yaml = router_with_json.route(
                &yaml_route,
                ::axum::routing::get(move || {
                    let api = api_for_yaml.clone();
                    async move {
                        match serde_yaml::to_string(&api) {
                            Ok(yaml) => (
                                [(::axum::http::header::CONTENT_TYPE, "application/x-yaml")],
                                yaml,
                            )
                                .into_response(),
                            Err(e) => (
                                ::axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                                format!("Failed to serialize OpenAPI spec to YAML: {e}"),
                            )
                                .into_response(),
                        }
                    }
                }),
            );

            // Add YML endpoint (alias for YAML)
            let yml_route = format!("{base_route}.yml");
            let router_with_yml = router_with_yaml.route(
                &yml_route,
                ::axum::routing::get(move || {
                    let api = api_for_yml.clone();
                    async move {
                        match serde_yaml::to_string(&api) {
                            Ok(yaml) => (
                                [(::axum::http::header::CONTENT_TYPE, "application/x-yaml")],
                                yaml,
                            )
                                .into_response(),
                            Err(e) => (
                                ::axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                                format!("Failed to serialize OpenAPI spec to YAML: {e}"),
                            )
                                .into_response(),
                        }
                    }
                }),
            );

            // Add extension
            (Some(router_with_yml.layer(Extension(api_mut))), None)
        } else {
            // No OAS spec, return the inner router
            (None, Some(self.inner))
        }
    }

    /// Add state to the router and finalize the API
    pub fn with_state(self, state: S) -> ::axum::Router
    where
        S: Clone + Send + Sync + 'static,
    {
        let (with_oas, without_oas) = self.wire_openapi_routes();

        if let Some(router) = with_oas {
            router.with_state(state)
        } else if let Some(inner) = without_oas {
            inner.with_state(state).into()
        } else {
            unreachable!("Either with_oas or without_oas must be Some")
        }
    }

    /// Finalize the API without state
    pub fn finish(self) -> ::axum::Router<S>
    where
        S: Clone + Send + Sync + 'static,
    {
        let (with_oas, without_oas) = self.wire_openapi_routes();

        with_oas.unwrap_or_else(|| {
            without_oas.map_or_else(
                || unreachable!("Either with_oas or without_oas must be Some"),
                std::convert::Into::into,
            )
        })
    }

    /// Finish building the API and return an axum Router for further configuration
    pub fn finish_api(self, api: &mut aide::openapi::OpenApi) -> ::axum::Router<S> {
        self.inner.finish_api(api)
    }

    /// Finish the API with `OpenAPI` spec embedded via Extension layer
    pub fn finish_api_with_extension(self, api: aide::openapi::OpenApi) -> ::axum::Router<S>
    where
        S: Clone + Send + Sync + 'static,
    {
        let mut api_mut = api;
        self.inner
            .finish_api(&mut api_mut)
            .layer(Extension(api_mut))
    }

    /// Convert into the underlying aide `ApiRouter`
    pub fn into_inner(self) -> AideApiRouter<S> {
        self.inner
    }
}

impl<S> Default for Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for handlers that carry their own documentation.
///
/// This trait is automatically implemented by the `#[rovo]` macro for decorated handlers.
/// It provides methods to convert the handler into documented route handlers for each HTTP method.
///
/// You typically won't implement this trait manually - instead, use the `#[rovo]` macro
/// on your handler functions.
pub trait IntoApiMethodRouter<S = ()> {
    /// Convert into a GET route with documentation
    fn into_get_route(self) -> aide::axum::routing::ApiMethodRouter<S>;
    /// Convert into a POST route with documentation
    fn into_post_route(self) -> aide::axum::routing::ApiMethodRouter<S>;
    /// Convert into a PATCH route with documentation
    fn into_patch_route(self) -> aide::axum::routing::ApiMethodRouter<S>;
    /// Convert into a DELETE route with documentation
    fn into_delete_route(self) -> aide::axum::routing::ApiMethodRouter<S>;
    /// Convert into a PUT route with documentation
    fn into_put_route(self) -> aide::axum::routing::ApiMethodRouter<S>;
}

/// Wrapper around `ApiMethodRouter` that provides method chaining for documented handlers.
///
/// This type is returned by routing functions like `get()`, `post()`, etc. and allows
/// chaining methods with the exact same names as axum (`.post()`, `.patch()`, etc.) while
/// accepting documented handlers decorated with `#[rovo]`.
///
/// # Example
/// ```no_run
/// use rovo::{Router, rovo, routing::{get, post, patch, delete}, aide::axum::IntoApiResponse};
/// use axum::response::Json;
///
/// #[rovo]
/// async fn list_items() -> impl IntoApiResponse { Json(()) }
/// #[rovo]
/// async fn create_item() -> impl IntoApiResponse { Json(()) }
/// #[rovo]
/// async fn get_item() -> impl IntoApiResponse { Json(()) }
/// #[rovo]
/// async fn update_item() -> impl IntoApiResponse { Json(()) }
/// #[rovo]
/// async fn delete_item() -> impl IntoApiResponse { Json(()) }
///
/// # #[tokio::main]
/// # async fn main() {
/// Router::new()
///     .route("/items", get(list_items).post(create_item))
///     .route("/items/{id}", get(get_item).patch(update_item).delete(delete_item));
/// # }
/// ```
pub struct ApiMethodRouter<S = ()> {
    inner: aide::axum::routing::ApiMethodRouter<S>,
}

impl<S> ApiMethodRouter<S>
where
    S: Clone + Send + Sync + 'static,
{
    /// Create a new `ApiMethodRouter` from aide's `ApiMethodRouter`
    #[must_use]
    pub const fn new(inner: aide::axum::routing::ApiMethodRouter<S>) -> Self {
        Self { inner }
    }

    /// Chain a POST handler
    pub fn post<H>(self, handler: H) -> Self
    where
        H: IntoApiMethodRouter<S>,
    {
        Self {
            inner: self.inner.merge(handler.into_post_route()),
        }
    }

    /// Chain a GET handler
    pub fn get<H>(self, handler: H) -> Self
    where
        H: IntoApiMethodRouter<S>,
    {
        Self {
            inner: self.inner.merge(handler.into_get_route()),
        }
    }

    /// Chain a PATCH handler
    pub fn patch<H>(self, handler: H) -> Self
    where
        H: IntoApiMethodRouter<S>,
    {
        Self {
            inner: self.inner.merge(handler.into_patch_route()),
        }
    }

    /// Chain a DELETE handler
    pub fn delete<H>(self, handler: H) -> Self
    where
        H: IntoApiMethodRouter<S>,
    {
        Self {
            inner: self.inner.merge(handler.into_delete_route()),
        }
    }

    /// Chain a PUT handler
    pub fn put<H>(self, handler: H) -> Self
    where
        H: IntoApiMethodRouter<S>,
    {
        Self {
            inner: self.inner.merge(handler.into_put_route()),
        }
    }
}

impl<S> From<ApiMethodRouter<S>> for aide::axum::routing::ApiMethodRouter<S> {
    fn from(router: ApiMethodRouter<S>) -> Self {
        router.inner
    }
}

/// Drop-in replacement routing functions that work with `#[rovo]` decorated handlers.
///
/// These functions provide the same API as axum's routing functions but accept
/// handlers decorated with `#[rovo]` and automatically include their documentation.
///
/// # Example
/// ```no_run
/// use rovo::{Router, rovo, routing::{get, post, patch, delete}, aide::axum::IntoApiResponse};
/// use axum::response::Json;
///
/// #[rovo]
/// async fn list_items() -> impl IntoApiResponse { Json(()) }
/// #[rovo]
/// async fn create_item() -> impl IntoApiResponse { Json(()) }
/// #[rovo]
/// async fn get_item() -> impl IntoApiResponse { Json(()) }
/// #[rovo]
/// async fn update_item() -> impl IntoApiResponse { Json(()) }
/// #[rovo]
/// async fn delete_item() -> impl IntoApiResponse { Json(()) }
///
/// # #[tokio::main]
/// # async fn main() {
/// Router::new()
///     .route("/items", get(list_items).post(create_item))
///     .route("/items/{id}", get(get_item).patch(update_item).delete(delete_item));
/// # }
/// ```
pub mod routing {
    use super::{ApiMethodRouter, IntoApiMethodRouter};

    /// Create a GET route with documentation from a `#[rovo]` decorated handler.
    pub fn get<S, H>(handler: H) -> ApiMethodRouter<S>
    where
        H: IntoApiMethodRouter<S>,
        S: Clone + Send + Sync + 'static,
    {
        ApiMethodRouter::new(handler.into_get_route())
    }

    /// Create a POST route with documentation from a `#[rovo]` decorated handler.
    pub fn post<S, H>(handler: H) -> ApiMethodRouter<S>
    where
        H: IntoApiMethodRouter<S>,
        S: Clone + Send + Sync + 'static,
    {
        ApiMethodRouter::new(handler.into_post_route())
    }

    /// Create a PATCH route with documentation from a `#[rovo]` decorated handler.
    pub fn patch<S, H>(handler: H) -> ApiMethodRouter<S>
    where
        H: IntoApiMethodRouter<S>,
        S: Clone + Send + Sync + 'static,
    {
        ApiMethodRouter::new(handler.into_patch_route())
    }

    /// Create a DELETE route with documentation from a `#[rovo]` decorated handler.
    pub fn delete<S, H>(handler: H) -> ApiMethodRouter<S>
    where
        H: IntoApiMethodRouter<S>,
        S: Clone + Send + Sync + 'static,
    {
        ApiMethodRouter::new(handler.into_delete_route())
    }

    /// Create a PUT route with documentation from a `#[rovo]` decorated handler.
    pub fn put<S, H>(handler: H) -> ApiMethodRouter<S>
    where
        H: IntoApiMethodRouter<S>,
        S: Clone + Send + Sync + 'static,
    {
        ApiMethodRouter::new(handler.into_put_route())
    }
}

/// Re-exports from aide's axum integration for convenience.
pub mod axum {
    pub use aide::axum::{ApiRouter, IntoApiResponse};
}
