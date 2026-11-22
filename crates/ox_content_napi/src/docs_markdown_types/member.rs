use std::collections::BTreeMap;

use napi_derive::napi;

use super::{JsDocParam, JsDocReturn, JsDocThrows, JsDocsMarkdownTag, JsTypeParam};

/// Normalized member documentation used by generated API docs.
#[napi(object)]
#[derive(Clone)]
pub struct JsDocMember {
    pub name: String,
    pub kind: String,
    pub description: String,
    pub signature: Option<String>,
    pub r#type: Option<String>,
    pub r#default: Option<String>,
    pub params: Option<Vec<JsDocParam>>,
    pub type_parameters: Option<Vec<JsTypeParam>>,
    pub returns: Option<JsDocReturn>,
    pub throws: Option<Vec<JsDocThrows>>,
    pub examples: Option<Vec<String>>,
    pub members: Option<Vec<JsDocMember>>,
    pub optional: Option<bool>,
    pub readonly: Option<bool>,
    pub r#static: Option<bool>,
    pub private: Option<bool>,
    /// Custom JSDoc tags by name, with the first value of a repeated tag.
    pub tags: Option<BTreeMap<String, String>>,
    /// Every custom JSDoc tag in source order; `generateDocsMarkdown` prefers it.
    pub tag_list: Option<Vec<JsDocsMarkdownTag>>,
    pub implementation_of: Option<Vec<String>>,
    pub line: u32,
    pub end_line: u32,
}

impl Default for JsDocMember {
    fn default() -> Self {
        Self {
            name: String::new(),
            kind: String::new(),
            description: String::new(),
            signature: None,
            r#type: None,
            r#default: None,
            params: None,
            type_parameters: None,
            returns: None,
            throws: None,
            examples: None,
            members: None,
            optional: None,
            readonly: None,
            r#static: None,
            private: None,
            tags: None,
            tag_list: None,
            implementation_of: None,
            line: 1,
            end_line: 1,
        }
    }
}
