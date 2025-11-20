use axum::extract::State;
use axum::http::StatusCode;
use axum::response::Json;
use rovo::aide::openapi::OpenApi;
use rovo::routing::{get, post, put};
use rovo::schemars::JsonSchema;
use rovo::{rovo, Router};
use serde::Serialize;

#[derive(Clone)]
struct AppState;

#[derive(Serialize, JsonSchema, Default)]
struct Item {
    id: u64,
    name: String,
}

/// Get item
/// @response 200 Json<Item> Item retrieved
#[rovo]
async fn get_item(State(_state): State<AppState>) -> Json<Item> {
    Json(Item::default())
}

/// Create item
/// @response 201 Json<Item> Item created
#[rovo]
async fn create_item(State(_state): State<AppState>) -> Json<Item> {
    Json(Item::default())
}

/// Replace item
/// @response 200 Json<Item> Item replaced
#[rovo]
async fn replace_item(State(_state): State<AppState>) -> Json<Item> {
    Json(Item::default())
}

/// Update item
/// @response 200 Json<Item> Item updated
#[rovo]
async fn update_item(State(_state): State<AppState>) -> Json<Item> {
    Json(Item::default())
}

#[test]
fn test_router_default() {
    let router: Router<AppState> = Router::default();
    let state = AppState;
    let mut api = OpenApi::default();
    api.info.title = "Test".to_string();

    let _app = router
        .route("/test", get(get_item))
        .with_oas(api)
        .with_state(state);
}

#[test]
fn test_router_finish_without_state() {
    let mut api = OpenApi::default();
    api.info.title = "Test API".to_string();

    let _app = Router::new()
        .route("/items", get(get_item))
        .with_oas(api)
        .finish();
}

#[test]
fn test_router_into_inner() {
    let router = Router::new().route("/items", get(get_item));

    let _inner = router.into_inner();
}

#[test]
fn test_router_finish_api() {
    let router = Router::new().route("/items", get(get_item));

    let mut api = OpenApi::default();
    api.info.title = "Test API".to_string();

    let _app = router.finish_api(&mut api);
}

#[test]
fn test_router_finish_api_with_extension() {
    let router = Router::new().route("/items", get(get_item));

    let mut api = OpenApi::default();
    api.info.title = "Test API".to_string();

    let _app = router.finish_api_with_extension(api);
}

#[test]
fn test_put_routing() {
    let state = AppState;
    let mut api = OpenApi::default();
    api.info.title = "Test API".to_string();

    let app = Router::new()
        .route("/items/{id}", put(replace_item))
        .with_oas(api)
        .with_state(state);

    let spec = extract_openapi_from_router(app);
    let paths = &spec.paths.as_ref().unwrap().paths;
    let item_path = get_path_item(paths.get("/items/{id}").unwrap());

    assert!(item_path.put.is_some(), "Should have PUT method");
}

#[test]
fn test_method_chaining_with_get() {
    let state = AppState;
    let mut api = OpenApi::default();
    api.info.title = "Test API".to_string();

    let app = Router::new()
        .route("/items", post(create_item).get(get_item))
        .with_oas(api)
        .with_state(state);

    let spec = extract_openapi_from_router(app);
    let paths = &spec.paths.as_ref().unwrap().paths;
    let items_path = get_path_item(paths.get("/items").unwrap());

    assert!(
        items_path.get.is_some(),
        "Should have GET method from chaining"
    );
    assert!(items_path.post.is_some(), "Should have POST method");
}

#[test]
fn test_method_chaining_with_put() {
    let state = AppState;
    let mut api = OpenApi::default();
    api.info.title = "Test API".to_string();

    let app = Router::new()
        .route("/items", get(get_item).put(replace_item))
        .with_oas(api)
        .with_state(state);

    let spec = extract_openapi_from_router(app);
    let paths = &spec.paths.as_ref().unwrap().paths;
    let items_path = get_path_item(paths.get("/items").unwrap());

    assert!(items_path.get.is_some(), "Should have GET method");
    assert!(
        items_path.put.is_some(),
        "Should have PUT method from chaining"
    );
}

#[test]
fn test_with_oas_route_strips_json_extension() {
    let state = AppState;
    let mut api = OpenApi::default();
    api.info.title = "Test API".to_string();

    let app = Router::new()
        .route("/items", get(get_item))
        .with_oas_route(api, "/spec.json") // Should strip .json
        .with_state(state);

    let rt = tokio::runtime::Runtime::new().unwrap();

    rt.block_on(async {
        use axum::body::Body;
        use axum::http::Request;
        use tower::util::ServiceExt;

        // Should work at /spec.json
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/spec.json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // Should also have /spec.yaml
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/spec.yaml")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    });
}

#[test]
fn test_with_oas_route_strips_yaml_extension() {
    let state = AppState;
    let mut api = OpenApi::default();
    api.info.title = "Test API".to_string();

    let app = Router::new()
        .route("/items", get(get_item))
        .with_oas_route(api, "/spec.yaml") // Should strip .yaml
        .with_state(state);

    let rt = tokio::runtime::Runtime::new().unwrap();

    rt.block_on(async {
        use axum::body::Body;
        use axum::http::Request;
        use tower::util::ServiceExt;

        // Should work at /spec.json (base + .json)
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/spec.json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    });
}

#[test]
fn test_with_oas_route_strips_yml_extension() {
    let state = AppState;
    let mut api = OpenApi::default();
    api.info.title = "Test API".to_string();

    let app = Router::new()
        .route("/items", get(get_item))
        .with_oas_route(api, "/spec.yml") // Should strip .yml
        .with_state(state);

    let rt = tokio::runtime::Runtime::new().unwrap();

    rt.block_on(async {
        use axum::body::Body;
        use axum::http::Request;
        use tower::util::ServiceExt;

        // Should work at /spec.json (base + .json)
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/spec.json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    });
}

#[test]
fn test_nest_with_child_having_oas() {
    let state = AppState;
    let mut child_api = OpenApi::default();
    child_api.info.title = "Child API".to_string();

    // Parent has no OAS, child has OAS
    let app = Router::new()
        .route("/parent", get(get_item))
        .nest(
            "/child",
            Router::new()
                .route("/items", get(get_item))
                .with_oas(child_api),
        )
        .with_state(state);

    // Just ensure it compiles and doesn't panic
    let _spec = extract_openapi_from_router(app);
}

#[test]
fn test_nest_both_have_oas() {
    let state = AppState;
    let mut parent_api = OpenApi::default();
    parent_api.info.title = "Parent API".to_string();

    let mut child_api = OpenApi::default();
    child_api.info.title = "Child API".to_string();

    // Both have OAS - parent should take precedence
    let app = Router::new()
        .route("/parent", get(get_item))
        .with_oas(parent_api)
        .nest(
            "/child",
            Router::new()
                .route("/items", get(get_item))
                .with_oas(child_api),
        )
        .with_state(state);

    let spec = extract_openapi_from_router(app);
    assert_eq!(spec.info.title, "Parent API", "Parent OAS should be used");
}

#[test]
fn test_router_without_oas() {
    let state = AppState;

    // Router without OAS spec
    let app = Router::new()
        .route("/items", get(get_item))
        .with_state(state);

    // Should not have /api.json endpoint
    let rt = tokio::runtime::Runtime::new().unwrap();

    rt.block_on(async {
        use axum::body::Body;
        use axum::http::Request;
        use tower::util::ServiceExt;

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api.json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Should return 404 or similar since no OAS routes
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    });
}

/// GET handler
#[rovo]
async fn handler_get() -> StatusCode {
    StatusCode::OK
}

/// POST handler
#[rovo]
async fn handler_post() -> StatusCode {
    StatusCode::OK
}

/// PUT handler
#[rovo]
async fn handler_put() -> StatusCode {
    StatusCode::OK
}

/// PATCH handler
#[rovo]
async fn handler_patch() -> StatusCode {
    StatusCode::OK
}

/// DELETE handler
#[rovo]
async fn handler_delete() -> StatusCode {
    StatusCode::OK
}

#[test]
fn test_all_http_methods() {
    let mut api = OpenApi::default();
    api.info.title = "Test API".to_string();

    let app = Router::new()
        .route(
            "/all-methods",
            get(handler_get)
                .post(handler_post)
                .put(handler_put)
                .patch(handler_patch)
                .delete(handler_delete),
        )
        .with_oas(api)
        .with_state(());

    let spec = extract_openapi_from_router(app);
    let paths = &spec.paths.as_ref().unwrap().paths;
    let path = get_path_item(paths.get("/all-methods").unwrap());

    assert!(path.get.is_some(), "Should have GET");
    assert!(path.post.is_some(), "Should have POST");
    assert!(path.put.is_some(), "Should have PUT");
    assert!(path.patch.is_some(), "Should have PATCH");
    assert!(path.delete.is_some(), "Should have DELETE");
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
