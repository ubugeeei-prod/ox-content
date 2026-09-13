use std::collections::HashMap;

use napi::bindgen_prelude::*;
use napi_derive::napi;
use ox_content_transform::transformer::{parse_frontmatter, stringify_frontmatter};
use rustc_hash::FxHashMap;

use crate::FrontmatterParseResult;

/// Parses one Markdown source's YAML frontmatter without loading the Vite plugin.
#[napi(js_name = "parseFrontmatter")]
pub fn parse_frontmatter_napi(source: String) -> FrontmatterParseResult {
    let (content, frontmatter) = parse_frontmatter(&source);

    FrontmatterParseResult { content, frontmatter: frontmatter.into_iter().collect() }
}

/// Serializes frontmatter and Markdown content into one document.
#[napi(js_name = "stringifyFrontmatter")]
#[allow(clippy::disallowed_types, clippy::implicit_hasher)]
pub fn stringify_frontmatter_napi(
    frontmatter: HashMap<String, serde_json::Value>,
    content: String,
) -> Result<String> {
    let frontmatter = FxHashMap::from_iter(frontmatter);
    stringify_frontmatter(&frontmatter, &content).map_err(|error| {
        napi::Error::from_reason(format!("Failed to serialize frontmatter: {error}"))
    })
}
