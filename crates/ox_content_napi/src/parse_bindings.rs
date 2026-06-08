use std::collections::HashMap;

use napi::bindgen_prelude::*;
use napi::Task;
use napi_derive::napi;
use ox_content_parser::{Parser, ParserOptions};
use ox_content_renderer::HtmlRenderer;

use crate::{create_allocator_for_source, mdast, mdast_raw, transfer::TransferPayloadKind};

/// Parse result containing the AST as JSON.
#[napi(object)]
pub struct ParseResult {
    /// The AST as a JSON string.
    pub ast: String,
    /// Parse errors, if any.
    pub errors: Vec<String>,
}

/// Render result containing the HTML output.
#[napi(object)]
pub struct RenderResult {
    /// The rendered HTML.
    pub html: String,
    /// Render errors, if any.
    pub errors: Vec<String>,
}

/// Table of contents entry.
#[napi(object)]
#[derive(Clone)]
pub struct TocEntry {
    /// Heading depth (1-6).
    pub depth: u8,
    /// Heading text.
    pub text: String,
    /// URL-friendly slug.
    pub slug: String,
    /// Child entries.
    pub children: Vec<TocEntry>,
}

/// Transform result containing HTML, frontmatter, and TOC.
#[napi(object)]
pub struct TransformResult {
    /// The rendered HTML.
    pub html: String,
    /// Parsed frontmatter as JSON string.
    pub frontmatter: String,
    /// Table of contents entries.
    pub toc: Vec<TocEntry>,
    /// Parse/render errors, if any.
    pub errors: Vec<String>,
}

/// Source offset where prepared Markdown content begins in the original source.
#[napi(object)]
pub struct JsSourceOrigin {
    /// UTF-8 byte offset.
    pub byte_offset: u32,
    /// UTF-16 code-unit offset.
    pub offset: u32,
    /// 1-based line number.
    pub line: u32,
    /// 1-based column number.
    pub column: u32,
}

/// Prepared Markdown source with parsed frontmatter.
#[napi(object)]
pub struct PreparedSourceResult {
    /// Markdown content after optional frontmatter removal.
    pub content: String,
    /// Parsed frontmatter object.
    pub frontmatter: HashMap<String, serde_json::Value>,
    /// Source position where `content` starts in the original source.
    pub source_offset: JsSourceOrigin,
}

/// Source preparation options for JavaScript.
#[napi(object)]
#[derive(Default, Clone)]
pub struct JsSourceOptions {
    /// Parse YAML frontmatter before returning the content payload.
    pub frontmatter: Option<bool>,
}

/// Parser options for JavaScript.
#[napi(object)]
#[derive(Default, Clone)]
pub struct JsParserOptions {
    /// Enable GFM extensions.
    pub gfm: Option<bool>,
    /// Enable footnotes.
    pub footnotes: Option<bool>,
    /// Enable task lists.
    pub task_lists: Option<bool>,
    /// Enable tables.
    pub tables: Option<bool>,
    /// Enable strikethrough.
    pub strikethrough: Option<bool>,
    /// Enable autolinks.
    pub autolinks: Option<bool>,
}

impl From<JsParserOptions> for ParserOptions {
    fn from(opts: JsParserOptions) -> Self {
        let mut options =
            if opts.gfm.unwrap_or(false) { ParserOptions::gfm() } else { ParserOptions::default() };

        if let Some(v) = opts.footnotes {
            options.footnotes = v;
        }
        if let Some(v) = opts.task_lists {
            options.task_lists = v;
        }
        if let Some(v) = opts.tables {
            options.tables = v;
        }
        if let Some(v) = opts.strikethrough {
            options.strikethrough = v;
        }
        if let Some(v) = opts.autolinks {
            options.autolinks = v;
        }

        options
    }
}

/// Parses Markdown source into an AST.
///
/// Returns the AST as a JSON string for compatibility-oriented JavaScript consumers.
#[napi]
pub fn parse(source: String, options: Option<JsParserOptions>) -> ParseResult {
    let allocator = create_allocator_for_source(&source);
    let parser_options = options.map(ParserOptions::from).unwrap_or_default();
    let parser = Parser::with_options(&allocator, &source, parser_options);

    let result = parser.parse();
    match result {
        Ok(doc) => {
            let ast = mdast::to_mdast_json(&doc);
            ParseResult { ast, errors: vec![] }
        }
        Err(e) => ParseResult { ast: String::new(), errors: vec![e.to_string()] },
    }
}

/// Parses Markdown source into a raw mdast memory block for JavaScript-side deserialization.
#[napi]
pub fn parse_mdast_raw(source: String, options: Option<JsParserOptions>) -> Result<Uint8Array> {
    parse_transfer_raw(source, "mdast".to_string(), options)
}

/// Parses Markdown source into a transfer buffer identified by payload kind.
///
/// Today `mdast` is the primary supported payload. Future payload kinds such as
/// markdown-it token streams will share the same transfer envelope.
#[napi]
pub fn parse_transfer_raw(
    source: String,
    kind: String,
    options: Option<JsParserOptions>,
) -> Result<Uint8Array> {
    let payload_kind = TransferPayloadKind::from_str(&kind).ok_or_else(|| {
        napi::Error::from_reason(format!("Unsupported transfer payload kind: {kind}"))
    })?;

    match payload_kind {
        TransferPayloadKind::Mdast => {
            // Raw mdast transfer serializes immediately after parsing, so it
            // has the same arena shape as `parse`/`parseAndRender`; pre-sizing
            // keeps large transfer requests from paying bumpalo chunk growth.
            let allocator = create_allocator_for_source(&source);
            let parser_options = options.map(ParserOptions::from).unwrap_or_default();
            let parser = Parser::with_options(&allocator, &source, parser_options);
            let document =
                parser.parse().map_err(|error| napi::Error::from_reason(error.to_string()))?;
            mdast_raw::to_mdast_raw(&document)
        }
        TransferPayloadKind::MarkdownItTokens => Err(napi::Error::from_reason(
            "markdown-it token transfer is not implemented yet; mdast is the current baseline",
        )),
        TransferPayloadKind::PreparedSource => Err(napi::Error::from_reason(
            "prepared-source transfer is exposed through prepareSourceRaw, not parseTransferRaw",
        )),
    }
}

/// Parses Markdown and renders to HTML.
#[napi]
pub fn parse_and_render(source: String, options: Option<JsParserOptions>) -> RenderResult {
    let allocator = create_allocator_for_source(&source);
    let parser_options = options.map(ParserOptions::from).unwrap_or_default();
    let parser = Parser::with_options(&allocator, &source, parser_options);

    let result = parser.parse();
    match result {
        Ok(doc) => {
            let mut renderer = HtmlRenderer::new();
            let html = renderer.render(&doc);
            RenderResult { html, errors: vec![] }
        }
        Err(e) => RenderResult { html: String::new(), errors: vec![e.to_string()] },
    }
}

/// Renders an AST (provided as JSON) to HTML.
#[napi]
pub fn render(_ast_json: String) -> RenderResult {
    // In a production implementation, we would:
    // 1. Parse the JSON AST
    // 2. Convert to our internal AST format
    // 3. Render to HTML
    //
    // For now, return an error indicating this is not yet implemented
    RenderResult {
        html: String::new(),
        errors: vec!["render from JSON not yet implemented".to_string()],
    }
}

/// Returns the version of ox_content_napi.
#[napi]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Async task for parse_and_render.
pub struct ParseAndRenderTask {
    source: String,
    options: ParserOptions,
}

impl Task for ParseAndRenderTask {
    type Output = RenderResult;
    type JsValue = RenderResult;

    fn compute(&mut self) -> Result<Self::Output> {
        let allocator = create_allocator_for_source(&self.source);
        let parser = Parser::with_options(&allocator, &self.source, self.options.clone());

        let result = match parser.parse() {
            Ok(doc) => {
                let mut renderer = HtmlRenderer::new();
                let html = renderer.render(&doc);
                RenderResult { html, errors: vec![] }
            }
            Err(e) => RenderResult { html: String::new(), errors: vec![e.to_string()] },
        };
        Ok(result)
    }

    fn resolve(&mut self, _env: Env, output: Self::Output) -> Result<Self::JsValue> {
        Ok(output)
    }
}

/// Parses Markdown and renders to HTML asynchronously (runs on worker thread).
#[napi]
pub fn parse_and_render_async(
    source: String,
    options: Option<JsParserOptions>,
) -> AsyncTask<ParseAndRenderTask> {
    let parser_options = options.map(ParserOptions::from).unwrap_or_default();
    AsyncTask::new(ParseAndRenderTask { source, options: parser_options })
}
