//! WebAssembly bindings for Ox Content.
//!
//! This crate provides WASM bindings for using Ox Content in browsers
//! and other WebAssembly environments.

use rustc_hash::FxHashMap;

use wasm_bindgen::prelude::*;

use ox_content_allocator::Allocator;
use ox_content_parser::{Parser, ParserOptions};
use ox_content_renderer::{HtmlRenderer, HtmlRendererOptions};

use frontmatter::parse_frontmatter;
use toc::extract_toc;
pub use toc::TocEntry;

mod frontmatter;
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
    footnotes: bool,
    task_lists: bool,
    tables: bool,
    strikethrough: bool,
    autolinks: bool,
    toc_max_depth: u8,
    autolink_urls: bool,
    autolink_patterns: Vec<String>,
    autolink_target_blank: bool,
}

#[wasm_bindgen]
impl WasmParserOptions {
    /// Creates options with all Markdown extension flags disabled.
    ///
    /// Defaults: `gfm = false`, `tocMaxDepth = 3`, `autolinkUrls = false`,
    /// `autolinkPatterns = ["http://", "https://"]`, and
    /// `autolinkTargetBlank = true`.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            gfm: false,
            footnotes: false,
            task_lists: false,
            tables: false,
            strikethrough: false,
            autolinks: false,
            toc_max_depth: 3,
            autolink_urls: false,
            autolink_patterns: vec!["http://".to_string(), "https://".to_string()],
            autolink_target_blank: true,
        }
    }

    /// Enables the GFM convenience profile.
    ///
    /// Default: `false`.
    #[wasm_bindgen(setter)]
    pub fn set_gfm(&mut self, value: bool) {
        self.gfm = value;
    }

    /// Enables footnote references and definitions.
    ///
    /// Default: `false`.
    #[wasm_bindgen(setter)]
    pub fn set_footnotes(&mut self, value: bool) {
        self.footnotes = value;
    }

    /// Enables GFM task-list item markers such as `- [x]`.
    ///
    /// Default: `false`.
    #[wasm_bindgen(setter = taskLists)]
    pub fn set_task_lists(&mut self, value: bool) {
        self.task_lists = value;
    }

    /// Enables GFM pipe tables.
    ///
    /// Default: `false`.
    #[wasm_bindgen(setter)]
    pub fn set_tables(&mut self, value: bool) {
        self.tables = value;
    }

    /// Enables GFM strikethrough spans.
    ///
    /// Default: `false`.
    #[wasm_bindgen(setter)]
    pub fn set_strikethrough(&mut self, value: bool) {
        self.strikethrough = value;
    }

    /// Enables GFM autolinks in the parser.
    ///
    /// Default: `false`.
    #[wasm_bindgen(setter)]
    pub fn set_autolinks(&mut self, value: bool) {
        self.autolinks = value;
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

        options.footnotes = opts.footnotes;
        options.task_lists = opts.task_lists;
        options.tables = opts.tables;
        options.strikethrough = opts.strikethrough;
        options.autolinks = opts.autolinks;

        options
    }
}

/// Parses Markdown and renders to HTML.
#[wasm_bindgen(js_name = parseAndRender)]
pub fn parse_and_render(source: &str, options: Option<WasmParserOptions>) -> JsValue {
    let opts = options.unwrap_or_default();
    let allocator = Allocator::new();
    let parser_options = ParserOptions::from(&opts);
    let parser = Parser::with_options(&allocator, source, parser_options);

    let result = parser.parse();
    match result {
        Ok(doc) => {
            let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
                toc_max_depth: opts.toc_max_depth,
                autolink_urls: opts.autolink_urls,
                autolink_target_blank: opts.autolink_target_blank,
                autolink_patterns: opts.autolink_patterns,
                ..Default::default()
            });
            let html = renderer.render(&doc);
            serde_wasm_bindgen::to_value(&serde_json::json!({
                "html": html,
                "errors": Vec::<String>::new()
            }))
            .unwrap_or(JsValue::NULL)
        }
        Err(e) => serde_wasm_bindgen::to_value(&serde_json::json!({
            "html": "",
            "errors": [e.to_string()]
        }))
        .unwrap_or(JsValue::NULL),
    }
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

    // Parse markdown
    let allocator = Allocator::new();
    let parser_options = ParserOptions::from(&opts);
    let parser = Parser::with_options(&allocator, &content, parser_options);

    let result = parser.parse();
    match result {
        Ok(doc) => {
            // Extract TOC from headings
            let toc = extract_toc(&doc, toc_max_depth);

            // Render to HTML
            let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
                toc_max_depth,
                autolink_urls: opts.autolink_urls,
                // `opts` is owned and unused after this literal, so move the
                // pattern Vec instead of deep-cloning it every call.
                autolink_patterns: opts.autolink_patterns,
                autolink_target_blank: opts.autolink_target_blank,
                ..Default::default()
            });
            let html = renderer.render(&doc);

            let transform_result = TransformResult { html, frontmatter, toc, errors: vec![] };

            serde_wasm_bindgen::to_value(&transform_result).unwrap_or(JsValue::NULL)
        }
        Err(e) => {
            let transform_result = TransformResult {
                html: String::new(),
                frontmatter: FxHashMap::default(),
                toc: vec![],
                errors: vec![e.to_string()],
            };

            serde_wasm_bindgen::to_value(&transform_result).unwrap_or(JsValue::NULL)
        }
    }
}

/// Returns the version of ox_content_wasm.
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
