use napi_derive::napi;

#[napi(object)]
#[derive(Clone)]
pub struct JsSourceDocTag {
    pub tag: String,
    pub value: String,
}

/// Parameter documentation extracted from source code.
#[napi(object)]
#[derive(Clone)]
pub struct JsSourceDocParam {
    pub name: String,
    pub type_annotation: Option<String>,
    pub optional: bool,
    pub default_value: Option<String>,
    pub description: Option<String>,
}

/// Source documentation item extracted from a JS/TS file.
#[napi(object)]
#[derive(Clone)]
pub struct JsSourceDocItem {
    pub name: String,
    pub kind: String,
    pub doc: Option<String>,
    pub jsdoc: Option<String>,
    pub source_path: String,
    pub line: u32,
    pub end_line: u32,
    pub exported: bool,
    pub signature: Option<String>,
    pub extends: Option<Vec<String>>,
    pub implements: Option<Vec<String>>,
    pub params: Vec<JsSourceDocParam>,
    pub return_type: Option<String>,
    pub return_members: Option<Vec<JsSourceDocItem>>,
    pub members: Option<Vec<JsSourceDocItem>>,
    pub tags: Vec<JsSourceDocTag>,
}
