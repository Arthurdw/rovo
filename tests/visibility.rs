//! Handlers annotated with `#[rovo]` must accept any visibility qualifier.
//! See <https://github.com/Arthurdw/rovo/issues/34>.

use rovo::aide::axum::IntoApiResponse;
use rovo::response::Json;
use rovo::{routing::get, rovo, Router};

mod handlers {
    use super::*;

    pub mod nested {
        use super::*;

        /// Restricted to an explicit path
        #[rovo]
        pub(in crate::handlers) async fn pub_in_path() -> impl IntoApiResponse {
            Json(())
        }

        /// Restricted to the parent module
        #[rovo]
        pub(super) async fn pub_super() -> impl IntoApiResponse {
            Json(())
        }

        pub(in crate::handlers) fn routes() -> Router<()> {
            Router::<()>::new().route("/in-path", get(pub_in_path))
        }
    }

    /// Public handler
    #[rovo]
    pub async fn pub_plain() -> impl IntoApiResponse {
        Json(())
    }

    /// Crate-visible handler
    #[rovo]
    pub(crate) async fn pub_crate() -> impl IntoApiResponse {
        Json(())
    }

    /// Crate-visible handler without `async` directly after the visibility
    #[rovo]
    #[allow(clippy::manual_async_fn)]
    pub(crate) fn pub_crate_sync() -> impl std::future::Future<Output = Json<()>> {
        async { Json(()) }
    }

    /// Private handler
    #[rovo]
    async fn private() -> impl IntoApiResponse {
        Json(())
    }

    pub fn routes() -> Router<()> {
        Router::<()>::new()
            .route("/private", get(private))
            .route("/super", get(nested::pub_super))
            .nest("/nested", nested::routes())
    }
}

#[test]
fn handlers_with_visibility_qualifiers_compile() {
    let _router: ::axum::Router = Router::<()>::new()
        .route("/plain", get(handlers::pub_plain))
        .route("/crate", get(handlers::pub_crate))
        .route("/crate-sync", get(handlers::pub_crate_sync))
        .nest("/handlers", handlers::routes())
        .finish_api(&mut rovo::aide::openapi::OpenApi::default());
}
