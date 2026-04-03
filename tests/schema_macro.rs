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
