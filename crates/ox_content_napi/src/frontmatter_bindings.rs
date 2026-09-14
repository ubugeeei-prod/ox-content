use napi::bindgen_prelude::*;
use napi_derive::napi;
use ox_content_transform::transformer::{parse_frontmatter_ordered, stringify_ordered_frontmatter};
use serde_json::{Map, Value};

use crate::FrontmatterParseResult;

/// Parses one Markdown source's YAML frontmatter without loading the Vite plugin.
#[napi(js_name = "parseFrontmatter")]
pub fn parse_frontmatter_napi(source: String) -> FrontmatterParseResult {
    let (content, frontmatter) = parse_frontmatter_ordered(&source);

    FrontmatterParseResult { content, frontmatter }
}

/// Serializes frontmatter and Markdown content into one document.
#[napi(js_name = "stringifyFrontmatter")]
pub fn stringify_frontmatter_napi(
    frontmatter: Map<String, Value>,
    content: String,
) -> Result<String> {
    stringify_ordered_frontmatter(&frontmatter, &content).map_err(|error| {
        napi::Error::from_reason(format!("Failed to serialize frontmatter: {error}"))
    })
}
