//! A product catalog: pagination, a generic response wrapper, a tagged enum and a file upload.
//!
//! Run with `cargo run -F swagger,multipart --example catalog_api`, then open http://127.0.0.1:3000
//! (set `PORT` to use another port).

use rovo::extract::Multipart;
use rovo::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

/// A product in the catalog.
#[derive(Clone, Serialize, JsonSchema)]
struct Product {
    id: u32,
    name: String,
    /// What kind of product this is.
    kind: ProductKind,
    /// Filename of the product image, if one was uploaded.
    image: Option<String>,
}

// Internally tagged, so the spec documents it as a `oneOf` with one object per
// variant, each with a `type` property.
/// The kind of product, with kind-specific details.
#[derive(Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ProductKind {
    /// A physical item that ships in a box.
    Physical {
        /// Shipping weight in grams.
        weight_grams: u32,
    },
    /// A download.
    Digital {
        /// Download size in bytes.
        size_bytes: u64,
    },
}

#[derive(Deserialize, JsonSchema)]
struct NewProduct {
    name: String,
    kind: ProductKind,
}

// Each field becomes a query parameter, with its doc comment, default and range.
/// Pagination for list endpoints.
#[derive(Deserialize, JsonSchema)]
struct Pagination {
    /// Page number, starting at 1.
    #[serde(default = "first_page")]
    #[schemars(range(min = 1))]
    page: u32,
    /// Items per page.
    #[serde(default = "default_per_page")]
    #[schemars(range(min = 1, max = 100))]
    per_page: u32,
}

fn first_page() -> u32 {
    1
}

fn default_per_page() -> u32 {
    20
}

// Every `Page<T>` used in a response gets its own schema (here `Page_for_Product`).
/// One page of results.
#[derive(Serialize, JsonSchema)]
struct Page<T> {
    items: Vec<T>,
    page: u32,
    per_page: u32,
    /// Total number of items across all pages.
    total: usize,
}

#[derive(Clone, Default)]
struct AppState {
    products: Arc<RwLock<Vec<Product>>>,
}

/// List products.
///
/// # Responses
///
/// 200: Json<Page<Product>> - One page of products
///
/// # Metadata
///
/// @tag products
#[rovo]
async fn list_products(
    State(state): State<AppState>,
    Query(pagination): Query<Pagination>,
) -> impl IntoApiResponse {
    let products = state.products.read().unwrap();
    let page = pagination.page.max(1);
    let per_page = pagination.per_page.clamp(1, 100);
    let items = products
        .iter()
        .skip(((page - 1) * per_page) as usize)
        .take(per_page as usize)
        .cloned()
        .collect();

    Json(Page {
        items,
        page,
        per_page,
        total: products.len(),
    })
}

/// Create a product.
///
/// # Responses
///
/// 201: Json<Product> - Product created
///
/// # Examples
///
/// 201: Product {
///     id: 1,
///     name: "Mechanical keyboard".into(),
///     kind: ProductKind::Physical { weight_grams: 900 },
///     image: None,
/// }
///
/// # Metadata
///
/// @tag products
#[rovo]
async fn create_product(
    State(state): State<AppState>,
    Json(new): Json<NewProduct>,
) -> impl IntoApiResponse {
    let mut products = state.products.write().unwrap();
    let product = Product {
        id: products.len() as u32 + 1,
        name: new.name,
        kind: new.kind,
        image: None,
    };
    products.push(product.clone());
    (StatusCode::CREATED, Json(product))
}

/// Upload a product image.
///
/// Send the image as the `image` field of a `multipart/form-data` request.
///
/// # Path Parameters
///
/// id: The product ID
///
/// # Responses
///
/// 200: Json<Product> - Image stored
/// 400: () - No `image` field in the request
/// 404: () - Product not found
///
/// # Metadata
///
/// @tag products
#[rovo]
async fn upload_image(
    State(state): State<AppState>,
    Path(id): Path<u32>,
    mut multipart: Multipart,
) -> impl IntoApiResponse {
    let mut filename = None;
    while let Ok(Some(field)) = multipart.next_field().await {
        if field.name() == Some("image") {
            filename = Some(field.file_name().unwrap_or("image").to_string());
            // A real service would store `field.bytes().await` somewhere.
            break;
        }
    }
    let Some(filename) = filename else {
        return StatusCode::BAD_REQUEST.into_response();
    };

    let mut products = state.products.write().unwrap();
    match products.iter_mut().find(|product| product.id == id) {
        Some(product) => {
            product.image = Some(filename);
            Json(product.clone()).into_response()
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

#[tokio::main]
async fn main() {
    let mut api = OpenApi::default();
    api.info.title = "Catalog API Example".to_string();

    let app = Router::new()
        .route("/products", get(list_products).post(create_product))
        .route("/products/{id}/image", post(upload_image))
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
