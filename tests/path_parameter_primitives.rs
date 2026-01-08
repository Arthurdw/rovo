//! Tests for automatic path parameter schema generation from primitive types
//!
//! These tests verify that primitive path parameters (String, u64, Uuid, etc.)
//! automatically generate proper OpenAPI documentation without requiring
//! wrapper structs with JsonSchema derive.

use rovo::aide::openapi::OpenApi;
use rovo::extract::{Path, State};
use rovo::response::Json;
use rovo::routing::get;
use rovo::schemars::JsonSchema;
use rovo::{rovo, Router};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone)]
struct AppState;

#[derive(Serialize, JsonSchema, Default)]
struct Item {
    id: String,
    name: String,
}

// =============================================================================
// Test Case 1: Single u64 path parameter with description
// =============================================================================

/// Get user by numeric ID.
///
/// # Path Parameters
///
/// id: The user's numeric identifier
///
/// # Responses
///
/// 200: Json<String> - User found
#[rovo]
async fn get_user_by_u64(Path(id): Path<u64>) -> Json<String> {
    Json(format!("User {id}"))
}

#[test]
fn test_single_u64_path_parameter() {
    // First test: call __docs directly to see if it adds parameters
    use rovo::aide::openapi::Operation;
    use rovo::aide::transform::TransformOperation;

    let mut op = Operation::default();
    let transform = TransformOperation::new(&mut op);
    let _ = get_user_by_u64::__docs(transform);

    // Now test via router
    let mut api = OpenApi::default();
    api.info.title = "Test API".to_string();

    let app = Router::new()
        .route("/users/{id}", get(get_user_by_u64))
        .with_oas(api)
        .with_state(());

    let spec = extract_openapi_from_router(app);
    let paths = &spec.paths.as_ref().unwrap().paths;
    let user_path = get_path_item(paths.get("/users/{id}").unwrap());
    let get_op = user_path.get.as_ref().unwrap();

    // Should have the 'id' parameter
    let id_param = find_path_parameter(&get_op.parameters, "id");
    assert!(id_param.is_some(), "Should have 'id' path parameter");

    // Verify description from doc comment
    // Note: Apostrophes may be escaped as \' in the parsed description
    let param_data = get_parameter_data(id_param.unwrap());
    let desc = param_data.description.as_deref().unwrap_or("");
    assert!(
        desc == "The user's numeric identifier" || desc == "The user\\'s numeric identifier",
        "Should have description from doc comment, got: {:?}",
        desc
    );
}

// =============================================================================
// Test Case 2: Single String path parameter
// =============================================================================

/// Get user by username.
///
/// # Path Parameters
///
/// username: The username to look up
///
/// # Responses
///
/// 200: Json<String> - User found
#[rovo]
async fn get_user_by_string(Path(username): Path<String>) -> Json<String> {
    Json(format!("User: {username}"))
}

#[test]
fn test_single_string_path_parameter() {
    let mut api = OpenApi::default();
    api.info.title = "Test API".to_string();

    let app = Router::new()
        .route("/users/{username}", get(get_user_by_string))
        .with_oas(api)
        .with_state(());

    let spec = extract_openapi_from_router(app);
    let paths = &spec.paths.as_ref().unwrap().paths;
    let user_path = get_path_item(paths.get("/users/{username}").unwrap());
    let get_op = user_path.get.as_ref().unwrap();

    let username_param = find_path_parameter(&get_op.parameters, "username");
    assert!(
        username_param.is_some(),
        "Should have 'username' path parameter"
    );
}

// =============================================================================
// Test Case 3: Uuid path parameter
// =============================================================================

/// Get resource by UUID.
///
/// # Path Parameters
///
/// resource_id: The resource UUID
///
/// # Responses
///
/// 200: Json<String> - Resource found
#[rovo]
async fn get_by_uuid(Path(resource_id): Path<Uuid>) -> Json<String> {
    Json(format!("Resource: {resource_id}"))
}

#[test]
fn test_uuid_path_parameter() {
    let mut api = OpenApi::default();
    api.info.title = "Test API".to_string();

    let app = Router::new()
        .route("/resources/{resource_id}", get(get_by_uuid))
        .with_oas(api)
        .with_state(());

    let spec = extract_openapi_from_router(app);
    let paths = &spec.paths.as_ref().unwrap().paths;
    let resource_path = get_path_item(paths.get("/resources/{resource_id}").unwrap());
    let get_op = resource_path.get.as_ref().unwrap();

    let uuid_param = find_path_parameter(&get_op.parameters, "resource_id");
    assert!(
        uuid_param.is_some(),
        "Should have 'resource_id' path parameter"
    );
}

// =============================================================================
// Test Case 4: Boolean path parameter
// =============================================================================

/// Get items by active status.
///
/// # Path Parameters
///
/// active: Whether to filter by active items
///
/// # Responses
///
/// 200: Json<String> - Items found
#[rovo]
async fn get_by_active(Path(active): Path<bool>) -> Json<String> {
    Json(format!("Active: {active}"))
}

#[test]
fn test_bool_path_parameter() {
    let mut api = OpenApi::default();
    api.info.title = "Test API".to_string();

    let app = Router::new()
        .route("/items/{active}", get(get_by_active))
        .with_oas(api)
        .with_state(());

    let spec = extract_openapi_from_router(app);
    let paths = &spec.paths.as_ref().unwrap().paths;
    let path = get_path_item(paths.get("/items/{active}").unwrap());
    let get_op = path.get.as_ref().unwrap();

    let active_param = find_path_parameter(&get_op.parameters, "active");
    assert!(
        active_param.is_some(),
        "Should have 'active' path parameter"
    );
}

// =============================================================================
// Test Case 5: Tuple path parameters
// =============================================================================

/// Get item at index in collection.
///
/// # Path Parameters
///
/// collection_id: The collection UUID
/// index: The item index within the collection
///
/// # Responses
///
/// 200: Json<String> - Item found
#[rovo]
async fn get_item_in_collection(Path((collection_id, index)): Path<(Uuid, u32)>) -> Json<String> {
    Json(format!("Collection {collection_id}, item {index}"))
}

#[test]
fn test_tuple_path_parameters() {
    let mut api = OpenApi::default();
    api.info.title = "Test API".to_string();

    let app = Router::new()
        .route(
            "/collections/{collection_id}/items/{index}",
            get(get_item_in_collection),
        )
        .with_oas(api)
        .with_state(());

    let spec = extract_openapi_from_router(app);
    let paths = &spec.paths.as_ref().unwrap().paths;
    let path = get_path_item(
        paths
            .get("/collections/{collection_id}/items/{index}")
            .unwrap(),
    );
    let get_op = path.get.as_ref().unwrap();

    // Should have two parameters
    let collection_param = find_path_parameter(&get_op.parameters, "collection_id");
    let index_param = find_path_parameter(&get_op.parameters, "index");

    assert!(
        collection_param.is_some(),
        "Should have 'collection_id' parameter"
    );
    assert!(index_param.is_some(), "Should have 'index' parameter");

    // Verify descriptions
    let collection_data = get_parameter_data(collection_param.unwrap());
    assert_eq!(
        collection_data.description.as_deref(),
        Some("The collection UUID"),
        "Should have collection_id description"
    );

    let index_data = get_parameter_data(index_param.unwrap());
    assert_eq!(
        index_data.description.as_deref(),
        Some("The item index within the collection"),
        "Should have index description"
    );
}

// =============================================================================
// Test Case 6: Path parameter without description (should still work)
// =============================================================================

/// Get item without parameter description.
///
/// # Path Parameters
///
/// item_id:
///
/// # Responses
///
/// 200: Json<String> - Item found
#[rovo]
async fn get_item_no_desc(Path(item_id): Path<i64>) -> Json<String> {
    Json(format!("Item {item_id}"))
}

#[test]
fn test_path_param_without_description() {
    let mut api = OpenApi::default();
    api.info.title = "Test API".to_string();

    let app = Router::new()
        .route("/items/{item_id}", get(get_item_no_desc))
        .with_oas(api)
        .with_state(());

    let spec = extract_openapi_from_router(app);
    let paths = &spec.paths.as_ref().unwrap().paths;
    let path = get_path_item(paths.get("/items/{item_id}").unwrap());
    let get_op = path.get.as_ref().unwrap();

    let item_param = find_path_parameter(&get_op.parameters, "item_id");
    assert!(item_param.is_some(), "Should have 'item_id' path parameter");

    // Verify no description (or empty)
    let param_data = get_parameter_data(item_param.unwrap());
    assert!(
        param_data.description.is_none() || param_data.description.as_deref() == Some(""),
        "Should have no description"
    );
}

// =============================================================================
// Test Case 7: Backwards compatibility with struct-based Path
// =============================================================================

#[derive(Deserialize, JsonSchema)]
struct UserId {
    /// The user ID
    id: u64,
}

/// Get user (struct-based).
///
/// # Responses
///
/// 200: Json<String> - User found
#[rovo]
async fn get_user_struct(Path(UserId { id }): Path<UserId>) -> Json<String> {
    Json(format!("User {id}"))
}

#[test]
fn test_backwards_compat_struct_path() {
    let mut api = OpenApi::default();
    api.info.title = "Test API".to_string();

    let app = Router::new()
        .route("/users/{id}", get(get_user_struct))
        .with_oas(api)
        .with_state(());

    let spec = extract_openapi_from_router(app);
    let paths = &spec.paths.as_ref().unwrap().paths;
    let user_path = get_path_item(paths.get("/users/{id}").unwrap());
    let get_op = user_path.get.as_ref().unwrap();

    // Should still work with struct-based path (handled by aide via JsonSchema)
    let id_param = find_path_parameter(&get_op.parameters, "id");
    assert!(
        id_param.is_some(),
        "Should have 'id' path parameter from struct"
    );
}

// =============================================================================
// Test Case 8: Mixed - State and primitive Path
// =============================================================================

/// Get item with state.
///
/// # Path Parameters
///
/// id: The item identifier
///
/// # Responses
///
/// 200: Json<Item> - Item found
#[rovo]
async fn get_item_with_state(State(_state): State<AppState>, Path(id): Path<String>) -> Json<Item> {
    Json(Item {
        id,
        name: "Test".to_string(),
    })
}

#[test]
fn test_primitive_path_with_state() {
    let state = AppState;
    let mut api = OpenApi::default();
    api.info.title = "Test API".to_string();

    let app = Router::new()
        .route("/items/{id}", get(get_item_with_state))
        .with_oas(api)
        .with_state(state);

    let spec = extract_openapi_from_router(app);
    let paths = &spec.paths.as_ref().unwrap().paths;
    let path = get_path_item(paths.get("/items/{id}").unwrap());
    let get_op = path.get.as_ref().unwrap();

    let id_param = find_path_parameter(&get_op.parameters, "id");
    assert!(id_param.is_some(), "Should have 'id' path parameter");

    let param_data = get_parameter_data(id_param.unwrap());
    assert_eq!(
        param_data.description.as_deref(),
        Some("The item identifier"),
        "Should have description"
    );
}

// =============================================================================
// Helper functions
// =============================================================================

fn get_path_item(
    path: &rovo::aide::openapi::ReferenceOr<rovo::aide::openapi::PathItem>,
) -> &rovo::aide::openapi::PathItem {
    match path {
        rovo::aide::openapi::ReferenceOr::Item(item) => item,
        _ => panic!("Expected PathItem, got Reference"),
    }
}

fn find_path_parameter<'a>(
    parameters: &'a [rovo::aide::openapi::ReferenceOr<rovo::aide::openapi::Parameter>],
    name: &str,
) -> Option<&'a rovo::aide::openapi::Parameter> {
    parameters.iter().find_map(|p| {
        if let rovo::aide::openapi::ReferenceOr::Item(param) = p {
            if let rovo::aide::openapi::Parameter::Path { parameter_data, .. } = param {
                if parameter_data.name == name {
                    return Some(param);
                }
            }
        }
        None
    })
}

fn get_parameter_data(
    param: &rovo::aide::openapi::Parameter,
) -> &rovo::aide::openapi::ParameterData {
    match param {
        rovo::aide::openapi::Parameter::Path { parameter_data, .. } => parameter_data,
        rovo::aide::openapi::Parameter::Query { parameter_data, .. } => parameter_data,
        rovo::aide::openapi::Parameter::Header { parameter_data, .. } => parameter_data,
        rovo::aide::openapi::Parameter::Cookie { parameter_data, .. } => parameter_data,
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
