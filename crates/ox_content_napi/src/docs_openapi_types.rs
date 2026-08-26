use std::collections::HashMap;

use napi_derive::napi;

use crate::JsDocsNavItem;

/// One local OpenAPI file consumed by the generated docs pipeline.
#[napi(object)]
#[derive(Clone, Default)]
pub struct JsOpenApiDocsInput {
    pub path: String,
    pub name: Option<String>,
    pub fail_on_unresolved_refs: Option<bool>,
}

/// Options shared by all OpenAPI docs inputs.
#[napi(object)]
#[derive(Clone, Default)]
pub struct JsOpenApiDocsOptions {
    pub root: Option<String>,
    pub base_path: Option<String>,
}

/// Generated OpenAPI Markdown pages and navigation metadata.
#[napi(object)]
#[derive(Clone, Default)]
#[allow(clippy::disallowed_types)]
pub struct JsGeneratedOpenApiDocs {
    pub pages: HashMap<String, String>,
    pub nav: Vec<JsDocsNavItem>,
}
