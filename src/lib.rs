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
    pub fn route(mut self, path: &str, method_router: aide::axum::routing::ApiMethodRouter<S>) -> Self {
        self.inner = self.inner.api_route(path, method_router);
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

/// Helper macro to create a GET route with auto-generated docs.
/// Use this for handlers decorated with `#[rovo]`.
///
/// For handlers without `#[rovo]`, use `aide::axum::routing::get()` instead.
#[macro_export]
macro_rules! get {
    ($handler:ident) => {
        aide::axum::routing::get_with($handler, paste::paste! { [<$handler _docs>] })
    };
}

/// Helper macro to create a POST route with auto-generated docs.
#[macro_export]
macro_rules! post {
    ($handler:ident) => {
        aide::axum::routing::post_with($handler, paste::paste! { [<$handler _docs>] })
    };
}

/// Helper macro to create a PATCH route with auto-generated docs.
#[macro_export]
macro_rules! patch {
    ($handler:ident) => {
        aide::axum::routing::patch_with($handler, paste::paste! { [<$handler _docs>] })
    };
}

/// Helper macro to create a DELETE route with auto-generated docs.
#[macro_export]
macro_rules! delete {
    ($handler:ident) => {
        aide::axum::routing::delete_with($handler, paste::paste! { [<$handler _docs>] })
    };
}

/// Helper macro to create a PUT route with auto-generated docs.
#[macro_export]
macro_rules! put {
    ($handler:ident) => {
        aide::axum::routing::put_with($handler, paste::paste! { [<$handler _docs>] })
    };
}

pub mod axum {
    pub use aide::axum::{IntoApiResponse, ApiRouter};
}
