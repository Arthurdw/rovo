//! Bearer-token authentication with a custom extractor and a custom error type.
//!
//! Run with `cargo run -F swagger --example auth_api`, then open http://127.0.0.1:3000
//! (set `PORT` to use another port).
//! Log in with `alice` / `hunter2` and paste the token into Swagger's "Authorize" dialog.

use rovo::aide::openapi::{ReferenceOr, SecurityScheme};
use rovo::aide::{OperationInput, OperationOutput};
use rovo::extract::FromRequestParts;
use rovo::http::{header, request::Parts};
use rovo::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone, Default)]
struct AppState {
    /// Issued tokens, mapped to the username they belong to.
    sessions: Arc<Mutex<HashMap<String, String>>>,
}

#[derive(Deserialize, JsonSchema)]
struct LoginForm {
    /// Account name.
    username: String,
    /// Account password.
    password: String,
}

#[derive(Serialize, JsonSchema)]
struct TokenResponse {
    /// Send as `Authorization: Bearer <token>`.
    token: String,
}

#[derive(Serialize, JsonSchema)]
struct Profile {
    username: String,
}

/// The JSON body of every error response.
#[derive(Serialize, JsonSchema)]
struct ErrorBody {
    /// Machine-readable error code.
    code: &'static str,
    /// Human-readable explanation.
    message: &'static str,
}

/// The error type shared by all handlers.
enum ApiError {
    InvalidCredentials,
    Unauthorized,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> rovo::response::Response {
        let (status, code, message) = match self {
            Self::InvalidCredentials => (
                StatusCode::UNAUTHORIZED,
                "invalid_credentials",
                "wrong username or password",
            ),
            Self::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "unauthorized",
                "missing or unknown bearer token",
            ),
        };
        (status, Json(ErrorBody { code, message })).into_response()
    }
}

// Lets handlers return `Result<_, ApiError>`. The error responses themselves are
// documented in each handler's `# Responses` section.
impl OperationOutput for ApiError {
    type Inner = ErrorBody;
}

/// A logged-in user, extracted from the `Authorization` header.
struct AuthUser {
    username: String,
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, ApiError> {
        let token = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))
            .ok_or(ApiError::Unauthorized)?;

        let sessions = state.sessions.lock().unwrap();
        let username = sessions.get(token).ok_or(ApiError::Unauthorized)?;
        Ok(AuthUser {
            username: username.clone(),
        })
    }
}

// Every extractor used in a `#[rovo]` handler must implement `OperationInput`.
// The empty impl adds nothing to the spec; the `@security` directive on the
// handler documents the requirement instead.
impl OperationInput for AuthUser {}

/// Log in.
///
/// Exchanges a username and password for a bearer token.
///
/// # Responses
///
/// 200: Json<TokenResponse> - Logged in
/// 401: Json<ErrorBody> - Wrong username or password
///
/// # Metadata
///
/// @tag auth
#[rovo]
async fn login(
    State(state): State<AppState>,
    Form(form): Form<LoginForm>,
) -> Result<Json<TokenResponse>, ApiError> {
    if form.username != "alice" || form.password != "hunter2" {
        return Err(ApiError::InvalidCredentials);
    }

    let token = format!("token-{}", form.username);
    state
        .sessions
        .lock()
        .unwrap()
        .insert(token.clone(), form.username);
    Ok(Json(TokenResponse { token }))
}

/// Get the current user.
///
/// # Responses
///
/// 200: Json<Profile> - The logged-in user
/// 401: Json<ErrorBody> - Missing or unknown bearer token
///
/// # Metadata
///
/// @tag auth
/// @security bearer_auth
#[rovo]
async fn me(
    // `#[rovo]` takes the router's state type from a `State<T>` parameter. `AuthUser`
    // needs `AppState`, so name it here even though the handler doesn't use it.
    _: State<AppState>,
    user: AuthUser,
) -> Result<Json<Profile>, ApiError> {
    Ok(Json(Profile {
        username: user.username,
    }))
}

#[tokio::main]
async fn main() {
    let mut api = OpenApi::default();
    api.info.title = "Auth API Example".to_string();
    api.components
        .get_or_insert_default()
        .security_schemes
        .insert(
            "bearer_auth".to_string(),
            ReferenceOr::Item(SecurityScheme::Http {
                scheme: "bearer".to_string(),
                bearer_format: None,
                description: Some("A token from `POST /login`.".to_string()),
                extensions: Default::default(),
            }),
        );

    // `with_oas` and `with_swagger` go before `with_state`.
    let app = Router::new()
        .route("/login", post(login))
        .route("/me", get(me))
        .with_oas(api)
        .with_swagger("/")
        .with_state(AppState::default())
        .finish();

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("127.0.0.1:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("Listening on http://{addr}");
    axum::serve(listener, app).await.unwrap();
}
