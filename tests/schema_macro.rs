#![allow(dead_code)]

use rovo::schemars::JsonSchema;

/// Verifies `#[derive(JsonSchema)]` works without any helper attributes.
#[derive(Debug, JsonSchema)]
struct BasicStruct {
    name: String,
    count: u32,
}

/// Verifies the derive works with enums.
#[derive(Debug, JsonSchema)]
enum MyEnum {
    A,
    B(String),
}

/// Verifies explicit `#[schemars(crate = "...")]` is respected.
#[derive(Debug, JsonSchema)]
#[schemars(crate = "::rovo::schemars")]
struct ExplicitCratePath {
    value: i64,
}

/// Verifies the derive works with generic types.
#[derive(Debug, JsonSchema)]
struct Wrapper<T> {
    inner: T,
}

#[test]
fn derive_produces_valid_json_schema() {
    let schema = rovo::schemars::SchemaGenerator::default().into_root_schema_for::<BasicStruct>();
    let json = serde_json::to_string(&schema).unwrap();
    assert!(json.contains("BasicStruct"));
}

#[test]
fn derive_works_for_enums() {
    let schema = rovo::schemars::SchemaGenerator::default().into_root_schema_for::<MyEnum>();
    let json = serde_json::to_string(&schema).unwrap();
    assert!(json.contains("MyEnum"));
}

#[test]
fn derive_respects_explicit_crate_path() {
    let schema =
        rovo::schemars::SchemaGenerator::default().into_root_schema_for::<ExplicitCratePath>();
    let json = serde_json::to_string(&schema).unwrap();
    assert!(json.contains("ExplicitCratePath"));
}

#[test]
fn derive_works_with_generics() {
    let schema =
        rovo::schemars::SchemaGenerator::default().into_root_schema_for::<Wrapper<String>>();
    let json = serde_json::to_string(&schema).unwrap();
    assert!(json.contains("Wrapper"));
}

fn schema_json<T: JsonSchema>() -> serde_json::Value {
    let schema = rovo::schemars::SchemaGenerator::default().into_root_schema_for::<T>();
    serde_json::to_value(schema).unwrap()
}

/// Helper attributes of other derives on the same type must stay visible to them (#32).
#[derive(Debug, thiserror::Error, JsonSchema)]
#[error("lookup failed: {reason}")]
struct LookupError {
    reason: String,
    #[source]
    #[schemars(skip)]
    cause: std::io::Error,
}

/// The std `#[default]` helper must keep working next to the derive (#32).
#[derive(Debug, Default, JsonSchema)]
enum Level {
    #[default]
    Low,
    High,
}

/// `#[serde(default)]` must use the type's own `Default` impl (#32).
#[derive(Debug, serde::Deserialize, JsonSchema)]
#[serde(default)]
struct Settings {
    retries: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self { retries: 3 }
    }
}

/// A recursive tree node.
#[derive(Debug, JsonSchema)]
struct Node {
    /// Child nodes.
    children: Vec<Self>,
}

// Like upstream, the generated impl refers to the deprecated type itself.
#[allow(deprecated)]
mod deprecated_type {
    use rovo::schemars::JsonSchema;

    #[deprecated = "use Settings instead"]
    #[derive(Debug, JsonSchema)]
    pub struct OldSettings {
        pub retries: u32,
    }
}

#[derive(Debug, rovo::schemars::JsonSchema_repr)]
#[repr(u8)]
enum Priority {
    Low = 1,
    High = 10,
}

#[test]
fn derive_keeps_foreign_helper_attributes() {
    let schema = schema_json::<LookupError>();
    assert_eq!(schema["title"], "LookupError");
    assert!(schema["properties"]["reason"].is_object());
    assert!(schema["properties"].get("cause").is_none());
    assert_eq!(
        LookupError {
            reason: "gone".into(),
            cause: std::io::Error::other("io"),
        }
        .to_string(),
        "lookup failed: gone"
    );
}

#[test]
fn derive_keeps_std_default_attribute() {
    assert!(matches!(Level::default(), Level::Low));
    assert_eq!(
        schema_json::<Level>()["enum"],
        serde_json::json!(["Low", "High"])
    );
}

#[test]
fn derive_uses_manual_default_impl_for_serde_default() {
    let schema = schema_json::<Settings>();
    assert_eq!(schema["properties"]["retries"]["default"], 3);
}

#[test]
fn derive_keeps_doc_descriptions() {
    let schema = schema_json::<Node>();
    assert_eq!(schema["description"], "A recursive tree node.");
    assert_eq!(
        schema["properties"]["children"]["description"],
        "Child nodes."
    );
}

#[test]
fn derive_names_self_references_after_the_type() {
    let schema = schema_json::<Node>();
    assert_eq!(schema["title"], "Node");
    assert_eq!(
        schema["properties"]["children"]["items"]["$ref"],
        "#/$defs/Node"
    );
    assert!(!schema.to_string().contains("__rovo"));
}

#[test]
#[allow(deprecated)]
fn derive_marks_deprecated_types() {
    let schema = schema_json::<deprecated_type::OldSettings>();
    assert_eq!(schema["deprecated"], true);
}

#[test]
fn derive_repr_uses_discriminants() {
    assert_eq!(
        schema_json::<Priority>()["enum"],
        serde_json::json!([1, 10])
    );
}
