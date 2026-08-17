//! WebAssembly bindings for Ox Content.
//!
//! This crate provides WASM bindings for using Ox Content in browsers
//! and other WebAssembly environments.

use rustc_hash::FxHashMap;
use serde::Serialize as _;

use wasm_bindgen::prelude::*;

use ox_content_parser::{Parser, ParserOptions};
use scratch::{with_scratch, RendererKey};

use frontmatter::parse_frontmatter;
use toc::extract_toc;
pub use toc::TocEntry;

mod frontmatter;
mod scratch;
mod toc;

/// Transform result containing HTML, frontmatter, and TOC.
#[derive(serde::Serialize)]
pub struct TransformResult {
    pub html: String,
    pub frontmatter: FxHashMap<String, serde_json::Value>,
    pub toc: Vec<TocEntry>,
    pub errors: Vec<String>,
}

/// Parser and renderer options exposed to JavaScript.
///
/// `new WasmParserOptions()` disables optional Markdown extensions by default
/// and uses renderer defaults for TOC and auto-link handling.
#[wasm_bindgen]
#[derive(Default)]
pub struct WasmParserOptions {
    gfm: bool,
    // The extension flags are tri-state: `None` means "not set from JS",
    // which lets the `gfm` profile supply its own defaults instead of the
    // field defaults silently overwriting them (see the `From` impl).
    footnotes: Option<bool>,
    task_lists: Option<bool>,
    tables: Option<bool>,
    strikethrough: Option<bool>,
    autolinks: Option<bool>,
    toc_max_depth: u8,
    autolink_urls: bool,
    autolink_patterns: Vec<String>,
    autolink_target_blank: bool,
}

#[wasm_bindgen]
impl WasmParserOptions {
    /// Creates options with all Markdown extension flags disabled.
    ///
    /// Defaults: `gfm = false`, `tocMaxDepth = 3`, `autolinkUrls = true`,
    /// `autolinkPatterns = ["http://", "https://"]`, and
    /// `autolinkTargetBlank = true`.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            gfm: false,
            footnotes: None,
            task_lists: None,
            tables: None,
            strikethrough: None,
            autolinks: None,
            toc_max_depth: 3,
            autolink_urls: true,
            autolink_patterns: vec!["http://".to_string(), "https://".to_string()],
            autolink_target_blank: true,
        }
    }

    /// Enables the GFM convenience profile.
    ///
    /// Default: `true`.
    #[wasm_bindgen(setter)]
    pub fn set_gfm(&mut self, value: bool) {
        self.gfm = value;
    }

    /// Enables footnote references and definitions.
    ///
    /// Default: unset — follows the `gfm` profile (`false` without it).
    #[wasm_bindgen(setter)]
    pub fn set_footnotes(&mut self, value: bool) {
        self.footnotes = Some(value);
    }

    /// Enables GFM task-list item markers such as `- [x]`.
    ///
    /// Default: unset — follows the `gfm` profile (`false` without it).
    #[wasm_bindgen(setter = taskLists)]
    pub fn set_task_lists(&mut self, value: bool) {
        self.task_lists = Some(value);
    }

    /// Enables GFM pipe tables.
    ///
    /// Default: unset — follows the `gfm` profile (`false` without it).
    #[wasm_bindgen(setter)]
    pub fn set_tables(&mut self, value: bool) {
        self.tables = Some(value);
    }

    /// Enables GFM strikethrough spans.
    ///
    /// Default: unset — follows the `gfm` profile (`false` without it).
    #[wasm_bindgen(setter)]
    pub fn set_strikethrough(&mut self, value: bool) {
        self.strikethrough = Some(value);
    }

    /// Enables GFM autolinks in the parser.
    ///
    /// Default: unset — follows the `gfm` profile (`false` without it).
    #[wasm_bindgen(setter)]
    pub fn set_autolinks(&mut self, value: bool) {
        self.autolinks = Some(value);
    }

    /// Sets the maximum heading depth included in inline TOCs.
    ///
    /// Default: `3`.
    #[wasm_bindgen(setter = tocMaxDepth)]
    pub fn set_toc_max_depth(&mut self, value: u8) {
        self.toc_max_depth = value;
    }

    /// Enables the renderer's URL auto-linking builtin. Bare URLs matching
    /// any registered pattern are wrapped in an `<a>` tag.
    ///
    /// Default: `false`.
    #[wasm_bindgen(setter = autolinkUrls)]
    pub fn set_autolink_urls(&mut self, value: bool) {
        self.autolink_urls = value;
    }

    /// Replaces the URL prefix patterns used by auto-linking. Pass a JS
    /// array of strings such as `["http://", "https://", "ftp://"]`.
    ///
    /// Default: `["http://", "https://"]`.
    #[wasm_bindgen(setter = autolinkPatterns)]
    pub fn set_autolink_patterns(&mut self, value: Vec<String>) {
        self.autolink_patterns = value;
    }

    /// Toggles `target="_blank" rel="noopener noreferrer"` on auto-linked
    /// URLs. Has no effect when `autolinkUrls` is off.
    ///
    /// Default: `true`.
    #[wasm_bindgen(setter = autolinkTargetBlank)]
    pub fn set_autolink_target_blank(&mut self, value: bool) {
        self.autolink_target_blank = value;
    }
}

impl From<&WasmParserOptions> for ParserOptions {
    fn from(opts: &WasmParserOptions) -> Self {
        let mut options = if opts.gfm { ParserOptions::gfm() } else { ParserOptions::default() };

        // Only apply flags JS actually set. Overwriting unconditionally
        // meant `gfm = true` had its sub-features (tables, strikethrough,
        // autolinks, footnotes, task lists) immediately reset to the field
        // defaults, disabling the profile it had just enabled.
        if let Some(footnotes) = opts.footnotes {
            options.footnotes = footnotes;
        }
        if let Some(task_lists) = opts.task_lists {
            options.task_lists = task_lists;
        }
        if let Some(tables) = opts.tables {
            options.tables = tables;
        }
        if let Some(strikethrough) = opts.strikethrough {
            options.strikethrough = strikethrough;
        }
        if let Some(autolinks) = opts.autolinks {
            options.autolinks = autolinks;
        }

        options
    }
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
        autolink_patterns: opts.autolink_patterns,
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
        autolink_patterns: opts.autolink_patterns,
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
