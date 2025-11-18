pub use rovo_macros::rovo;

// Re-export aide for convenience
pub use aide;

use aide::axum::ApiRouter as AideApiRouter;
use ::axum::Extension;

/// A drop-in replacement for axum::Router that adds OpenAPI documentation support.
///
/// This Router works seamlessly with handlers decorated with `#[rovo]` and provides
/// a fluent API for building documented APIs with Swagger UI integration.
///
/// # Example
/// ```ignore
/// use rovo::{Router, rovo, routing::get};
/// use aide::openapi::OpenApi;
///
/// #[rovo]
/// async fn documented_handler() -> impl IntoApiResponse { /* ... */ }
///
/// let mut api = OpenApi::default();
/// api.info.title = "My API".to_string();
///
/// let app = Router::new()
///     .route("/documented", get(documented_handler))
///     .with_swagger("/", "/api.json")
///     .with_api_json("/api.json", serve_api)
///     .with_state(state)
///     .finish_api_with_extension(api);
/// ```
pub struct Router<S = ()> {
    inner: AideApiRouter<S>,
}

impl<S> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    /// Create a new Router
    pub fn new() -> Self {
        Self {
            inner: AideApiRouter::new(),
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
    pub fn nest(mut self, path: &str, router: Router<S>) -> Self {
        self.inner = self.inner.nest(path, router.inner);
        self
    }

    /// Add Swagger UI route at the specified path
    pub fn with_swagger(mut self, swagger_path: &str, api_json_path: &str) -> Self
    where
        S: Clone + Send + Sync + 'static,
    {
        self.inner = self.inner.route(
            swagger_path,
            aide::swagger::Swagger::new(api_json_path).axum_route(),
        );
        self
    }

    /// Add the OpenAPI JSON endpoint
    pub fn with_api_json<H, T>(
        mut self,
        path: &str,
        handler: H,
    ) -> Self
    where
        H: ::axum::handler::Handler<T, S>,
        S: Clone + Send + Sync + 'static,
        T: 'static,
    {
        self.inner = self.inner.route(path, ::axum::routing::get(handler));
        self
    }

    /// Add state to the router
    pub fn with_state<S2>(self, state: S) -> Router<S2>
    where
        S2: Clone + Send + Sync + 'static,
    {
        Router {
            inner: self.inner.with_state(state),
        }
    }

    /// Finish building the API and return an axum Router for further configuration
    pub fn finish_api(self, api: &mut aide::openapi::OpenApi) -> ::axum::Router<S> {
        self.inner.finish_api(api)
    }

    /// Finish the API with OpenAPI spec embedded via Extension layer
    pub fn finish_api_with_extension(
        self,
        api: aide::openapi::OpenApi,
    ) -> ::axum::Router<S>
    where
        S: Clone + Send + Sync + 'static,
    {
        let mut api_mut = api;
        self.inner
            .finish_api(&mut api_mut)
            .layer(Extension(api_mut))
    }

    /// Convert into the underlying aide ApiRouter
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
/// ```ignore
/// use rovo::routing::get;
///
/// Router::new()
///     .route("/items", get(list_items).post(create_item))
///     .route("/items/{id}", get(get_item).patch(update_item).delete(delete_item))
/// ```
pub struct ApiMethodRouter<S = ()> {
    inner: aide::axum::routing::ApiMethodRouter<S>,
}

impl<S> ApiMethodRouter<S>
where
    S: Clone + Send + Sync + 'static,
{
    /// Create a new ApiMethodRouter from aide's ApiMethodRouter
    pub fn new(inner: aide::axum::routing::ApiMethodRouter<S>) -> Self {
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
/// ```ignore
/// use rovo::routing::{get, post};
///
/// Router::new()
///     .route("/items", get(list_items).post(create_item))
///     .route("/items/{id}", get(get_item).patch(update_item).delete(delete_item))
/// ```
pub mod routing {
    use super::*;

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

pub mod axum {
    pub use aide::axum::{IntoApiResponse, ApiRouter};
}
