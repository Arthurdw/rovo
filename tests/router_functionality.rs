use aide::openapi::OpenApi;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json, Response};
use rovo::routing::{delete, get, patch};
use rovo::{rovo, Router};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
struct AppState;

#[derive(Serialize, JsonSchema, Default)]
struct Item {
    id: u64,
    name: String,
}

#[derive(Deserialize, JsonSchema)]
struct CreateItem {
    #[allow(dead_code)]
    name: String,
}

/// List items
/// @tag items
/// @response 200 Json<Vec<Item>> List of items
#[rovo]
async fn list_items(State(_state): State<AppState>) -> Json<Vec<Item>> {
    Json(vec![])
}

/// Create item
/// @tag items
/// @response 201 Json<Item> Item created
#[rovo]
async fn create_item(
    State(_state): State<AppState>,
    Json(_req): Json<CreateItem>,
) -> Response {
    (StatusCode::CREATED, Json(Item::default())).into_response()
}

/// Update item
/// @tag items
/// @response 200 Json<Item> Item updated
#[rovo]
async fn update_item(
    State(_state): State<AppState>,
    Json(_req): Json<CreateItem>,
) -> Json<Item> {
    Json(Item::default())
}

/// Delete item
/// @tag items
/// @response 204 () Item deleted
#[rovo]
async fn delete_item(State(_state): State<AppState>) -> StatusCode {
    StatusCode::NO_CONTENT
}

#[test]
fn test_method_chaining() {
    let state = AppState;
    let mut api = OpenApi::default();
    api.info.title = "Test API".to_string();

    let app = Router::new()
        .route("/items", get(list_items).post(create_item))
        .route("/items/{id}", patch(update_item).delete(delete_item))
        .with_oas(api)
        .with_state(state);

    let spec = extract_openapi_from_router(app);
    let paths = &spec.paths.as_ref().unwrap().paths;

    // Verify /items has GET and POST
    let items_path = get_path_item(paths.get("/items").unwrap());
    assert!(items_path.get.is_some(), "Should have GET method");
    assert!(items_path.post.is_some(), "Should have POST method");

    // Verify /items/{id} has PATCH and DELETE
    let item_path = get_path_item(paths.get("/items/{id}").unwrap());
    assert!(item_path.patch.is_some(), "Should have PATCH method");
    assert!(item_path.delete.is_some(), "Should have DELETE method");
}

#[test]
fn test_all_formats_identical() {
    let state = AppState;
    let mut api = OpenApi::default();
    api.info.title = "Test API".to_string();

    let app = Router::new()
        .route("/items", get(list_items))
        .with_oas(api)
        .with_state(state);

    let rt = tokio::runtime::Runtime::new().unwrap();

    let (json_spec, yaml_spec, yml_spec) = rt.block_on(async {
        use axum::body::Body;
        use axum::http::Request;
        use tower::util::ServiceExt;

        // Get JSON
        let json_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api.json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let json_body = axum::body::to_bytes(json_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json_spec: OpenApi = serde_json::from_slice(&json_body).unwrap();

        // Get YAML
        let yaml_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api.yaml")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let yaml_body = axum::body::to_bytes(yaml_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let yaml_spec: OpenApi = serde_yaml::from_slice(&yaml_body).unwrap();

        // Get YML
        let yml_response = app
            .oneshot(
                Request::builder()
                    .uri("/api.yml")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let yml_body = axum::body::to_bytes(yml_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let yml_spec: OpenApi = serde_yaml::from_slice(&yml_body).unwrap();

        (json_spec, yaml_spec, yml_spec)
    });

    // Verify all three formats have the same content
    assert_eq!(
        json_spec.info.title, yaml_spec.info.title,
        "JSON and YAML should have same title"
    );
    assert_eq!(
        json_spec.info.title, yml_spec.info.title,
        "JSON and YML should have same title"
    );

    let json_paths = json_spec.paths.as_ref().unwrap().paths.len();
    let yaml_paths = yaml_spec.paths.as_ref().unwrap().paths.len();
    let yml_paths = yml_spec.paths.as_ref().unwrap().paths.len();

    assert_eq!(json_paths, yaml_paths, "Should have same number of paths");
    assert_eq!(json_paths, yml_paths, "Should have same number of paths");
}

#[test]
fn test_nested_routers() {
    let state = AppState;
    let mut api = OpenApi::default();
    api.info.title = "Test API".to_string();

    let app = Router::new()
        .nest(
            "/api",
            Router::new()
                .route("/items", get(list_items))
                .route("/items/{id}", delete(delete_item)),
        )
        .with_oas(api)
        .with_state(state);

    let spec = extract_openapi_from_router(app);
    let paths = &spec.paths.as_ref().unwrap().paths;

    // Verify nested paths exist with /api prefix
    assert!(
        paths.contains_key("/api/items"),
        "Should contain nested /api/items path"
    );
    assert!(
        paths.contains_key("/api/items/{id}"),
        "Should contain nested /api/items/{{id}} path"
    );
}

#[test]
fn test_custom_oas_route() {
    let state = AppState;
    let mut api = OpenApi::default();
    api.info.title = "Test API".to_string();

    let app = Router::new()
        .route("/items", get(list_items))
        .with_oas_route(api, "/openapi")
        .with_state(state);

    let rt = tokio::runtime::Runtime::new().unwrap();

    rt.block_on(async {
        use axum::body::Body;
        use axum::http::Request;
        use tower::util::ServiceExt;

        // Verify /openapi.json works
        let json_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/openapi.json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            json_response.status(),
            StatusCode::OK,
            "Should be able to fetch /openapi.json"
        );

        // Verify /openapi.yaml works
        let yaml_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/openapi.yaml")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            yaml_response.status(),
            StatusCode::OK,
            "Should be able to fetch /openapi.yaml"
        );

        // Verify /openapi.yml works
        let yml_response = app
            .oneshot(
                Request::builder()
                    .uri("/openapi.yml")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            yml_response.status(),
            StatusCode::OK,
            "Should be able to fetch /openapi.yml"
        );
    });
}

#[test]
#[allow(deprecated)]
fn test_deprecated_endpoint() {
    /// Old endpoint
    /// @tag items
    /// @response 200 Json<Vec<Item>> Old response
    #[deprecated]
    #[rovo]
    async fn old_list_items(State(_state): State<AppState>) -> Json<Vec<Item>> {
        Json(vec![])
    }

    let state = AppState;
    let mut api = OpenApi::default();
    api.info.title = "Test API".to_string();

    let app = Router::new()
        .route("/old-items", get(old_list_items))
        .with_oas(api)
        .with_state(state);

    let spec = extract_openapi_from_router(app);
    let paths = &spec.paths.as_ref().unwrap().paths;
    let old_items_path = get_path_item(paths.get("/old-items").unwrap());
    let get_op = old_items_path.get.as_ref().unwrap();

    assert!(
        get_op.deprecated,
        "Deprecated endpoint should be marked as deprecated"
    );
}

#[test]
fn test_security_annotation() {
    /// Protected endpoint
    /// @tag items
    /// @security bearer_auth
    /// @response 200 Json<Vec<Item>> Protected response
    #[rovo]
    async fn protected_items(State(_state): State<AppState>) -> Json<Vec<Item>> {
        Json(vec![])
    }

    let state = AppState;
    let mut api = OpenApi::default();
    api.info.title = "Test API".to_string();

    let app = Router::new()
        .route("/protected", get(protected_items))
        .with_oas(api)
        .with_state(state);

    let spec = extract_openapi_from_router(app);
    let paths = &spec.paths.as_ref().unwrap().paths;
    let protected_path = get_path_item(paths.get("/protected").unwrap());
    let get_op = protected_path.get.as_ref().unwrap();

    assert!(
        !get_op.security.is_empty(),
        "Should have security requirements"
    );

    // Verify security requirement contains bearer_auth
    let has_bearer_auth = get_op.security.iter().any(|sec| {
        sec.contains_key("bearer_auth")
    });
    assert!(has_bearer_auth, "Should have bearer_auth security requirement");
}

#[test]
fn test_custom_operation_id() {
    /// Get items
    /// @tag items
    /// @id getItemsList
    /// @response 200 Json<Vec<Item>> Items list
    #[rovo]
    async fn custom_id_items(State(_state): State<AppState>) -> Json<Vec<Item>> {
        Json(vec![])
    }

    let state = AppState;
    let mut api = OpenApi::default();
    api.info.title = "Test API".to_string();

    let app = Router::new()
        .route("/custom-id-items", get(custom_id_items))
        .with_oas(api)
        .with_state(state);

    let spec = extract_openapi_from_router(app);
    let paths = &spec.paths.as_ref().unwrap().paths;
    let items_path = get_path_item(paths.get("/custom-id-items").unwrap());
    let get_op = items_path.get.as_ref().unwrap();

    assert_eq!(
        get_op.operation_id.as_ref().unwrap(),
        "getItemsList",
        "Should have custom operation ID"
    );
}

#[test]
fn test_hidden_endpoint() {
    /// Hidden endpoint
    /// @hidden
    #[rovo]
    async fn hidden_endpoint(State(_state): State<AppState>) -> StatusCode {
        StatusCode::OK
    }

    let state = AppState;
    let mut api = OpenApi::default();
    api.info.title = "Test API".to_string();

    let app = Router::new()
        .route("/hidden", get(hidden_endpoint))
        .with_oas(api)
        .with_state(state);

    let spec = extract_openapi_from_router(app);
    let paths = &spec.paths.as_ref().unwrap().paths;

    // Hidden endpoints should not appear in the spec
    // Note: Checking if the endpoint is actually hidden or just has no documentation
    if paths.contains_key("/hidden") {
        let hidden_path = get_path_item(paths.get("/hidden").unwrap());
        // If the path exists, the GET operation should not exist
        assert!(
            hidden_path.get.is_none(),
            "Hidden endpoint's GET operation should not be in the spec"
        );
    }
}

#[test]
fn test_multiple_tags() {
    /// Multi-tagged endpoint
    /// @tag items
    /// @tag admin
    /// @tag deprecated
    /// @response 200 () Success
    #[rovo]
    async fn multi_tag_endpoint(State(_state): State<AppState>) -> StatusCode {
        StatusCode::OK
    }

    let state = AppState;
    let mut api = OpenApi::default();
    api.info.title = "Test API".to_string();

    let app = Router::new()
        .route("/multi-tag", get(multi_tag_endpoint))
        .with_oas(api)
        .with_state(state);

    let spec = extract_openapi_from_router(app);
    let paths = &spec.paths.as_ref().unwrap().paths;
    let endpoint_path = get_path_item(paths.get("/multi-tag").unwrap());
    let get_op = endpoint_path.get.as_ref().unwrap();

    assert_eq!(get_op.tags.len(), 3, "Should have 3 tags");
    assert!(get_op.tags.contains(&"items".to_string()));
    assert!(get_op.tags.contains(&"admin".to_string()));
    assert!(get_op.tags.contains(&"deprecated".to_string()));
}

// Helper functions
fn get_path_item(
    path: &aide::openapi::ReferenceOr<aide::openapi::PathItem>,
) -> &aide::openapi::PathItem {
    match path {
        aide::openapi::ReferenceOr::Item(item) => item,
        _ => panic!("Expected PathItem, got Reference"),
    }
}

fn extract_openapi_from_router(app: ::axum::Router) -> OpenApi {
    use axum::body::Body;
    use axum::http::Request;
    use tower::util::ServiceExt;

    let rt = tokio::runtime::Runtime::new().unwrap();

    rt.block_on(async {
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api.json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();

        serde_json::from_slice(&body).unwrap()
    })
}
