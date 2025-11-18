pub use rovo_macros::rovo;

// Re-export aide for convenience
pub use aide;


use aide::axum::ApiRouter as AideApiRouter;

/// A drop-in replacement for axum::Router that adds OpenAPI documentation support.
///
/// Use this Router with aide's routing helpers for undocumented endpoints,
/// or with rovo's helper macros for endpoints decorated with `#[rovo]`.
///
/// # Example
/// ```ignore
/// use rovo::{Router, rovo};
/// use aide::axum::routing::{get, post};
///
/// #[rovo]
/// async fn documented_handler() -> impl IntoApiResponse { /* ... */ }
///
/// async fn regular_handler() -> impl IntoResponse { /* ... */ }
///
/// let app = Router::new()
///     .route("/documented", rovo::get!(documented_handler))
///     .route("/regular", get(regular_handler))
///     .with_state(state);
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
/// Automatically implemented by the `#[rovo]` macro.
pub trait IntoApiMethodRouter<S = ()> {
    fn into_get_route(self) -> aide::axum::routing::ApiMethodRouter<S>;
    fn into_post_route(self) -> aide::axum::routing::ApiMethodRouter<S>;
    fn into_patch_route(self) -> aide::axum::routing::ApiMethodRouter<S>;
    fn into_delete_route(self) -> aide::axum::routing::ApiMethodRouter<S>;
    fn into_put_route(self) -> aide::axum::routing::ApiMethodRouter<S>;
}

/// Wrapper around `ApiMethodRouter` that provides method chaining for documented handlers.
///
/// This type is returned by routing functions like `get()`, `post()`, etc. and allows
/// chaining methods with the same names as axum (`.post()`, `.patch()`, etc.) while
/// accepting documented handlers.
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
pub mod routing {
    use super::*;

    /// Create a GET route with documentation.
    pub fn get<S, H>(handler: H) -> ApiMethodRouter<S>
    where
        H: IntoApiMethodRouter<S>,
        S: Clone + Send + Sync + 'static,
    {
        ApiMethodRouter::new(handler.into_get_route())
    }

    /// Create a POST route with documentation.
    pub fn post<S, H>(handler: H) -> ApiMethodRouter<S>
    where
        H: IntoApiMethodRouter<S>,
        S: Clone + Send + Sync + 'static,
    {
        ApiMethodRouter::new(handler.into_post_route())
    }

    /// Create a PATCH route with documentation.
    pub fn patch<S, H>(handler: H) -> ApiMethodRouter<S>
    where
        H: IntoApiMethodRouter<S>,
        S: Clone + Send + Sync + 'static,
    {
        ApiMethodRouter::new(handler.into_patch_route())
    }

    /// Create a DELETE route with documentation.
    pub fn delete<S, H>(handler: H) -> ApiMethodRouter<S>
    where
        H: IntoApiMethodRouter<S>,
        S: Clone + Send + Sync + 'static,
    {
        ApiMethodRouter::new(handler.into_delete_route())
    }

    /// Create a PUT route with documentation.
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
