use std::collections::BTreeMap;

use serde::Deserialize;
use serde_json::Value;
use tower_lsp::lsp_types::{Diagnostic, Range};

#[derive(Clone, Debug)]
pub struct FrontmatterDocument {
    pub block: Option<FrontmatterBlock>,
}

#[derive(Clone, Debug)]
pub struct FrontmatterBlock {
    pub block_range: Range,
    pub content_range: Range,
    pub content_start_offset: usize,
    pub content_end_offset: usize,
    pub value: Option<Value>,
    pub diagnostics: Vec<Diagnostic>,
    pub top_level_keys: Vec<TopLevelKey>,
}

#[derive(Clone, Debug)]
pub struct TopLevelKey {
    pub name: String,
    pub key_range: Range,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct FrontmatterSchema {
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub type_name: Option<String>,
    pub properties: BTreeMap<String, FrontmatterSchema>,
    pub required: Vec<String>,
    #[serde(rename = "enum")]
    pub enum_values: Vec<Value>,
    pub default: Option<Value>,
    pub items: Option<Box<FrontmatterSchema>>,
    #[serde(rename = "additionalProperties")]
    pub additional_properties: Option<bool>,
}

impl FrontmatterSchema {
    #[must_use]
    pub fn property(&self, name: &str) -> Option<&Self> {
        self.properties.get(name)
    }

    #[must_use]
    pub fn kind_label(&self) -> String {
        self.type_name.clone().unwrap_or_else(|| {
            if self.properties.is_empty() {
                "value".to_string()
            } else {
                "object".to_string()
            }
        })
    }
}
