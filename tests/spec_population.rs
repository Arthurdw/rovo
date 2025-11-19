use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json, Response};
use rovo::aide::openapi::OpenApi;
use rovo::routing::get;
use rovo::schemars::JsonSchema;
use rovo::{rovo, Router};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
struct AppState;

#[derive(Serialize, JsonSchema, Default)]
struct User {
    id: u64,
    name: String,
}

#[derive(Deserialize, JsonSchema)]
struct UserId {
    id: u64,
}

/// Get user by ID.
///
/// Returns user information for the specified ID.
///
/// @tag users
/// @tag accounts
/// @response 200 Json<User> User found successfully
/// @response 404 () User not found
/// @example 200 User::default()
#[rovo]
async fn get_user(State(_state): State<AppState>, Path(UserId { id }): Path<UserId>) -> Response {
    if id == 1 {
        Json(User {
            id: 1,
            name: "Alice".to_string(),
        })
        .into_response()
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}

/// List all users.
///
/// Returns a list of all users in the system.
///
/// @tag users
/// @response 200 Json<Vec<User>> List of all users
#[rovo]
async fn list_users(State(_state): State<AppState>) -> Json<Vec<User>> {
    Json(vec![User {
        id: 1,
        name: "Alice".to_string(),
    }])
}

#[test]
fn test_spec_contains_paths() {
    let state = AppState;
    let mut api = OpenApi::default();
    api.info.title = "Test API".to_string();

    let app = Router::new()
        .route("/users", get(list_users))
        .route("/users/{id}", get(get_user))
        .with_oas(api.clone())
        .with_state(state);

    // Extract the OpenAPI spec from the Extension layer
    let spec = extract_openapi_from_router(app);

    // Verify paths exist
    assert!(spec.paths.is_some(), "OpenAPI spec should have paths");
    let paths = &spec.paths.as_ref().unwrap().paths;

    assert!(paths.contains_key("/users"), "Should contain /users path");
    assert!(
        paths.contains_key("/users/{id}"),
        "Should contain /users/{{id}} path"
    );
}

#[test]
fn test_spec_contains_tags() {
    let state = AppState;
    let mut api = OpenApi::default();
    api.info.title = "Test API".to_string();

    let app = Router::new()
        .route("/users/{id}", get(get_user))
        .with_oas(api.clone())
        .with_state(state);

    let spec = extract_openapi_from_router(app);

    // Verify the operation has tags
    let paths = &spec.paths.as_ref().unwrap().paths;
    let user_path = get_path_item(paths.get("/users/{id}").unwrap());
    let get_op = user_path.get.as_ref().unwrap();

    assert!(
        get_op.tags.contains(&"users".to_string()),
        "Should contain 'users' tag"
    );
    assert!(
        get_op.tags.contains(&"accounts".to_string()),
        "Should contain 'accounts' tag"
    );
}

#[test]
fn test_spec_contains_descriptions() {
    let state = AppState;
    let mut api = OpenApi::default();
    api.info.title = "Test API".to_string();

    let app = Router::new()
        .route("/users/{id}", get(get_user))
        .with_oas(api.clone())
        .with_state(state);

    let spec = extract_openapi_from_router(app);

    // Verify operation has summary and description
    let paths = &spec.paths.as_ref().unwrap().paths;
    let user_path = get_path_item(paths.get("/users/{id}").unwrap());
    let get_op = user_path.get.as_ref().unwrap();

    assert!(get_op.summary.is_some(), "Should have summary");
    assert_eq!(get_op.summary.as_ref().unwrap(), "Get user by ID.");

    assert!(get_op.description.is_some(), "Should have description");
    assert!(get_op
        .description
        .as_ref()
        .unwrap()
        .contains("Returns user information"));
}

#[test]
fn test_spec_contains_parameters() {
    let state = AppState;
    let mut api = OpenApi::default();
    api.info.title = "Test API".to_string();

    let app = Router::new()
        .route("/users/{id}", get(get_user))
        .with_oas(api.clone())
        .with_state(state);

    let spec = extract_openapi_from_router(app);

    // Verify operation has path parameters
    let paths = &spec.paths.as_ref().unwrap().paths;
    let user_path = get_path_item(paths.get("/users/{id}").unwrap());
    let get_op = user_path.get.as_ref().unwrap();

    assert!(!get_op.parameters.is_empty(), "Should have parameters");

    // Find the 'id' parameter
    let id_param = get_op.parameters.iter().find(|p| {
        matches!(p,
            rovo::aide::openapi::ReferenceOr::Item(
                rovo::aide::openapi::Parameter::Path { parameter_data, .. }
            ) if parameter_data.name == "id"
        )
    });

    assert!(id_param.is_some(), "Should have 'id' path parameter");
}

#[test]
fn test_spec_contains_responses() {
    let state = AppState;
    let mut api = OpenApi::default();
    api.info.title = "Test API".to_string();

    let app = Router::new()
        .route("/users/{id}", get(get_user))
        .with_oas(api.clone())
        .with_state(state);

    let spec = extract_openapi_from_router(app);

    // Verify operation has responses
    let paths = &spec.paths.as_ref().unwrap().paths;
    let user_path = get_path_item(paths.get("/users/{id}").unwrap());
    let get_op = user_path.get.as_ref().unwrap();

    assert!(get_op.responses.is_some(), "Should have responses");
    let responses = get_op.responses.as_ref().unwrap();

    let status_200 = aide::openapi::StatusCode::Code(200);
    let status_404 = aide::openapi::StatusCode::Code(404);

    assert!(
        responses.responses.contains_key(&status_200),
        "Should have 200 response"
    );
    assert!(
        responses.responses.contains_key(&status_404),
        "Should have 404 response"
    );

    // Verify 200 response has description
    if let aide::openapi::ReferenceOr::Item(response_200) =
        responses.responses.get(&status_200).unwrap()
    {
        assert_eq!(response_200.description, "User found successfully");
    } else {
        panic!("200 response should be an Item, not a Reference");
    }
}

#[test]
fn test_spec_contains_examples() {
    let state = AppState;
    let mut api = OpenApi::default();
    api.info.title = "Test API".to_string();

    let app = Router::new()
        .route("/users/{id}", get(get_user))
        .with_oas(api.clone())
        .with_state(state);

    let spec = extract_openapi_from_router(app);

    // Verify operation has response examples
    let paths = &spec.paths.as_ref().unwrap().paths;
    let user_path = get_path_item(paths.get("/users/{id}").unwrap());
    let get_op = user_path.get.as_ref().unwrap();
    let responses = get_op.responses.as_ref().unwrap();

    let status_200 = aide::openapi::StatusCode::Code(200);
    if let aide::openapi::ReferenceOr::Item(response_200) =
        responses.responses.get(&status_200).unwrap()
    {
        assert!(
            response_200.content.contains_key("application/json"),
            "Should have JSON content"
        );

        let json_content = response_200.content.get("application/json").unwrap();
        assert!(json_content.example.is_some(), "Should have example");
    }
}

#[test]
fn test_spec_contains_operation_id() {
    let state = AppState;
    let mut api = OpenApi::default();
    api.info.title = "Test API".to_string();

    let app = Router::new()
        .route("/users/{id}", get(get_user))
        .with_oas(api.clone())
        .with_state(state);

    let spec = extract_openapi_from_router(app);

    // Verify operation has operation_id (defaults to function name)
    let paths = &spec.paths.as_ref().unwrap().paths;
    let user_path = get_path_item(paths.get("/users/{id}").unwrap());
    let get_op = user_path.get.as_ref().unwrap();

    assert!(get_op.operation_id.is_some(), "Should have operation_id");
    assert_eq!(get_op.operation_id.as_ref().unwrap(), "get_user");
}

#[test]
fn test_spec_contains_request_body() {
    use axum::response::Json;
    use rovo::aide::axum::IntoApiResponse;

    #[derive(Deserialize, JsonSchema)]
    struct CreateUserRequest {
        name: String,
    }

    /// Create a new user.
    ///
    /// @tag users
    /// @response 201 Json<User> User created successfully
    #[rovo]
    async fn create_user(
        State(_state): State<AppState>,
        Json(req): Json<CreateUserRequest>,
    ) -> impl IntoApiResponse {
        (
            StatusCode::CREATED,
            Json(User {
                id: 1,
                name: req.name,
            }),
        )
    }

    let state = AppState;
    let mut api = OpenApi::default();
    api.info.title = "Test API".to_string();

    let app = Router::new()
        .route("/users", rovo::routing::post(create_user))
        .with_oas(api.clone())
        .with_state(state);

    let spec = extract_openapi_from_router(app);

    // Verify operation has request body
    let paths = &spec.paths.as_ref().unwrap().paths;
    let users_path = get_path_item(paths.get("/users").unwrap());
    let post_op = users_path.post.as_ref().unwrap();

    assert!(post_op.request_body.is_some(), "Should have request body");
}

// Helper function to extract PathItem from ReferenceOr
fn get_path_item(
    path: &aide::openapi::ReferenceOr<aide::openapi::PathItem>,
) -> &aide::openapi::PathItem {
    match path {
        aide::openapi::ReferenceOr::Item(item) => item,
        _ => panic!("Expected PathItem, got Reference"),
    }
}

// Helper function to extract OpenAPI spec from the router
fn extract_openapi_from_router(app: ::axum::Router) -> OpenApi {
    use axum::body::Body;
    use axum::http::Request;
    use tower::util::ServiceExt;

    // Create a runtime to execute async code
    let rt = tokio::runtime::Runtime::new().unwrap();

    rt.block_on(async {
        // Make a request to the /api.json endpoint
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api.json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Extract the body
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();

        // Parse the JSON into OpenApi
        serde_json::from_slice(&body).unwrap()
    })
}
