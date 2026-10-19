//! WebAssembly bindings for Ox Content.
//!
//! This crate provides WASM bindings for using Ox Content in browsers
//! and other WebAssembly environments.

use rustc_hash::FxHashMap;
use serde::Serialize as _;

use wasm_bindgen::prelude::*;

use ox_content_parser::{Parser, ParserOptions};
use scratch::{RendererKey, with_scratch};

use frontmatter::parse_frontmatter;
pub use toc::TocEntry;
use toc::extract_toc;

mod frontmatter;
mod options;
mod scratch;
mod toc;

pub use options::WasmParserOptions;

/// Transform result containing HTML, frontmatter, and TOC.
#[derive(serde::Serialize)]
pub struct TransformResult {
    pub html: String,
    pub frontmatter: FxHashMap<String, serde_json::Value>,
    pub toc: Vec<TocEntry>,
    pub errors: Vec<String>,
}

/// Builds the `{ html, errors }` result object without a serde round-trip.
///
/// The old path serialized through a `serde_json::Value` tree and
/// `serde_wasm_bindgen`, which (a) copied the whole HTML string an extra
/// time and (b) produced a JS `Map` — so the documented `result.html`
/// access never actually worked. A plain object built directly is both the
/// documented shape and the cheap one.
fn render_result(html: &str, error: Option<String>) -> JsValue {
    let out = js_sys::Object::new();
    let errors = js_sys::Array::new();
    if let Some(error) = error {
        errors.push(&JsValue::from_str(&error));
    }
    // Reflect::set only fails on non-object targets; `out` is always an
    // object, so the results are ignorable.
    let _ = js_sys::Reflect::set(&out, &JsValue::from_str("html"), &JsValue::from_str(html));
    let _ = js_sys::Reflect::set(&out, &JsValue::from_str("errors"), &errors);
    out.into()
}

/// Parses Markdown and renders to HTML.
#[wasm_bindgen(js_name = parseAndRender)]
pub fn parse_and_render(source: &str, options: Option<WasmParserOptions>) -> JsValue {
    let opts = options.unwrap_or_default();
    let parser_options = ParserOptions::from(&opts);
    let renderer_key = RendererKey {
        toc_max_depth: opts.toc_max_depth,
        autolink_urls: opts.autolink_urls,
        autolink_target_blank: opts.autolink_target_blank,
        link_target_blank: opts.link_target_blank,
        source_spans: opts.source_spans,
        autolink_patterns: opts.autolink_patterns,
        semantic_footnotes: opts.semantic_footnotes,
        heading_permalinks: opts.heading_permalinks,
        sanitize: opts.sanitize,
    };

    // The arena and renderer are reused across calls (see `scratch`); on a
    // small document the fresh-per-call versions of both used to dominate
    // the entire call.
    with_scratch(source.len(), &renderer_key, |allocator, renderer| {
        match Parser::with_options(allocator, source, parser_options).parse() {
            Ok(doc) => render_result(renderer.render_borrowed(&doc), None),
            Err(e) => render_result("", Some(e.to_string())),
        }
    })
}

/// Transforms Markdown source into HTML, frontmatter, and TOC.
#[wasm_bindgen]
pub fn transform(source: &str, options: Option<WasmParserOptions>) -> JsValue {
    let opts = options.unwrap_or_default();
    let toc_max_depth = opts.toc_max_depth;

    // Parse frontmatter into a borrowed content slice. In the common "no
    // frontmatter" case this avoids allocating a second Markdown string before
    // handing the source to the parser.
    let (content, frontmatter) = parse_frontmatter(source);

    // Parse markdown with the reused arena + renderer (see `scratch`).
    let parser_options = ParserOptions::from(&opts);
    let renderer_key = RendererKey {
        toc_max_depth,
        autolink_urls: opts.autolink_urls,
        autolink_target_blank: opts.autolink_target_blank,
        link_target_blank: opts.link_target_blank,
        source_spans: opts.source_spans,
        autolink_patterns: opts.autolink_patterns,
        semantic_footnotes: opts.semantic_footnotes,
        heading_permalinks: opts.heading_permalinks,
        sanitize: opts.sanitize,
    };

    let transform_result = with_scratch(content.len(), &renderer_key, |allocator, renderer| {
        match Parser::with_options(allocator, &content, parser_options).parse() {
            Ok(doc) => {
                // Extract TOC from headings
                let toc = extract_toc(&doc, toc_max_depth);
                let html = renderer.render_borrowed(&doc).to_owned();
                TransformResult { html, frontmatter, toc, errors: vec![] }
            }
            Err(e) => TransformResult {
                html: String::new(),
                frontmatter: FxHashMap::default(),
                toc: vec![],
                errors: vec![e.to_string()],
            },
        }
    });

    // `json_compatible` produces plain JS objects instead of Maps, matching
    // the documented `result.html` / `result.frontmatter` access.
    transform_result
        .serialize(&serde_wasm_bindgen::Serializer::json_compatible())
        .unwrap_or(JsValue::NULL)
}

/// Returns the version of ox_content_wasm.
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
