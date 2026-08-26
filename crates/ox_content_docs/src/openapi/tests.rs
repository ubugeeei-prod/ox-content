use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use super::{OpenApiDocsOptions, OpenApiSpecInput, generate_openapi_docs};

#[test]
fn renders_openapi_30_json_operations_schemas_and_nav() {
    let root = temp_root("json");
    let spec = root.join("petstore.json");
    fs::write(
        &spec,
        r##"{
  "openapi": "3.0.3",
  "info": { "title": "Pet Store", "version": "1.0.0", "description": "Store API." },
  "servers": [{ "url": "https://api.example.com" }],
  "security": [{ "ApiKeyAuth": [] }],
  "paths": {
    "/pets/{id}": {
      "parameters": [{ "name": "id", "in": "path", "required": true, "schema": { "type": "string" } }],
      "get": {
        "operationId": "getPet",
        "summary": "Read one pet.",
        "tags": ["Pets"],
        "responses": {
          "200": {
            "description": "Pet found.",
            "content": {
              "application/json": {
                "schema": { "$ref": "#/components/schemas/Pet" },
                "example": { "id": "pet-1", "name": "Nori" }
              }
            }
          }
        }
      }
    }
  },
  "components": {
    "securitySchemes": { "ApiKeyAuth": { "type": "apiKey", "description": "Header key." } },
    "schemas": {
      "Pet": {
        "type": "object",
        "description": "A pet resource.",
        "required": ["id"],
        "properties": {
          "id": { "type": "string", "description": "Stable id." },
          "tags": { "type": "array", "items": { "type": "string" } }
        }
      }
    }
  }
}"##,
    )
    .unwrap();

    let generated = generate_openapi_docs(
        &[OpenApiSpecInput { path: PathBuf::from("petstore.json"), ..OpenApiSpecInput::default() }],
        &OpenApiDocsOptions { root: Some(root.clone()), base_path: Some("/reference".to_string()) },
    )
    .unwrap();

    let index = generated.pages.get("openapi/pet-store/index.md").unwrap();
    assert!(index.contains("# Pet Store"));
    assert!(index.contains("[GET /pets/{id}](./getpet.md)"));
    let operation = generated.pages.get("openapi/pet-store/getpet.md").unwrap();
    assert!(operation.contains("Method: `GET`"));
    assert!(operation.contains("| id | path | yes | string |"));
    assert!(operation.contains("| ApiKeyAuth | apiKey |"));
    assert!(operation.contains("| application/json | Pet |"));
    let schemas = generated.pages.get("openapi/pet-store/schemas.md").unwrap();
    assert!(schemas.contains("## Pet"));
    assert!(schemas.contains("| tags | no | array<string> |"));
    assert_eq!(generated.nav_items[0].path, "/reference/openapi/pet-store");
    let _ = fs::remove_dir_all(root);
}

#[test]
fn renders_openapi_31_yaml_and_dedupes_operation_ids() {
    let root = temp_root("yaml");
    fs::write(
        root.join("billing.yaml"),
        r#"
openapi: 3.1.0
info:
  title: Billing API
  version: 2026-08
paths:
  /invoices:
    get:
      operationId: list
      summary: List invoices.
      responses:
        "200":
          description: OK
  /payments:
    get:
      operationId: list
      summary: List payments.
      responses:
        "200":
          description: OK
"#,
    )
    .unwrap();

    let generated = generate_openapi_docs(
        &[OpenApiSpecInput { path: PathBuf::from("billing.yaml"), ..OpenApiSpecInput::default() }],
        &OpenApiDocsOptions { root: Some(root.clone()), base_path: None },
    )
    .unwrap();

    assert!(generated.pages.contains_key("openapi/billing-api/list.md"));
    assert!(generated.pages.contains_key("openapi/billing-api/list-2.md"));
    assert_eq!(generated.nav_items[0].path, "/api/openapi/billing-api");
    let _ = fs::remove_dir_all(root);
}

#[test]
fn unresolved_refs_fail_with_the_ref_name() {
    let root = temp_root("ref");
    fs::write(
        root.join("broken.json"),
        r##"{
  "openapi": "3.0.3",
  "info": { "title": "Broken", "version": "1" },
  "paths": {
    "/pets": {
      "get": {
        "responses": {
          "200": { "$ref": "#/components/responses/Missing" }
        }
      }
    }
  }
}"##,
    )
    .unwrap();

    let error = generate_openapi_docs(
        &[OpenApiSpecInput { path: PathBuf::from("broken.json"), ..OpenApiSpecInput::default() }],
        &OpenApiDocsOptions { root: Some(root.clone()), base_path: None },
    )
    .unwrap_err()
    .to_string();
    assert!(error.contains("#/components/responses/Missing"));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn rejects_paths_outside_the_configured_root() {
    let root = temp_root("root");
    let outside = temp_root("outside");
    let spec = outside.join("outside.json");
    fs::write(&spec, r#"{"openapi":"3.0.3","info":{"title":"Outside","version":"1"},"paths":{}}"#)
        .unwrap();

    let error = generate_openapi_docs(
        &[OpenApiSpecInput { path: spec, ..OpenApiSpecInput::default() }],
        &OpenApiDocsOptions { root: Some(root.clone()), base_path: None },
    )
    .unwrap_err()
    .to_string();
    assert!(error.contains("outside the configured root"));
    let _ = fs::remove_dir_all(root);
    let _ = fs::remove_dir_all(outside);
}

fn temp_root(suffix: &str) -> PathBuf {
    static TEMP_COUNTER: AtomicUsize = AtomicUsize::new(0);
    let mut name = String::from("ox-content-openapi-");
    name.push_str(suffix);
    name.push('-');
    name.push_str(&std::process::id().to_string());
    name.push('-');
    name.push_str(&TEMP_COUNTER.fetch_add(1, Ordering::Relaxed).to_string());
    let path = std::env::temp_dir().join(name);
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    path
}
