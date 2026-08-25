use std::collections::HashMap;

use napi::Task;
use napi::bindgen_prelude::*;
use napi_derive::napi;
use ox_content_mdast::transfer::TransferPayloadKind;
use ox_content_parser::ParserOptions;

use crate::parser_options::JsParserOptions;
use crate::render_scratch;

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

/// How a specifier was imported.
#[napi(object)]
pub struct MdxImportSpecifier {
    /// Imported name (`default`, `*`, or the named export).
    pub imported: String,

    /// Local binding name.
    pub local: String,

    /// `default`, `named`, or `namespace`.
    #[napi(ts_type = "'default' | 'named' | 'namespace'")]
    pub kind: String,
}

/// One `import` statement collected from MDX ESM.
#[napi(object)]
pub struct MdxImport {
    /// Module specifier string.
    pub source: String,

    /// Bindings created by the import.
    pub specifiers: Vec<MdxImportSpecifier>,
}

/// Transform result containing HTML, frontmatter, TOC, and MDX metadata.
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

    /// MDX `import` statements (empty when MDX is off or no ESM nodes).
    pub imports: Vec<MdxImport>,

    /// Export names from MDX ESM (empty when MDX is off or no exports).
    pub exports: Vec<String>,

    /// Unique JSX component names in document order (empty when none).
    pub components: Vec<String>,
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
#[allow(clippy::disallowed_types)]
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
    ///
    /// Default: `false`.
    pub frontmatter: Option<bool>,
}

/// Parses Markdown source into an AST.
///
/// Returns the AST as a JSON string for compatibility-oriented JavaScript consumers.
#[napi]
pub fn parse(source: String, options: Option<JsParserOptions>) -> ParseResult {
    crate::ffi::recover(
        || {
            let parser_options = options.map(ParserOptions::from).unwrap_or_default();
            match render_scratch::parse_to_mdast_json(&source, parser_options) {
                Ok(ast) => ParseResult { ast, errors: vec![] },
                Err(error) => ParseResult { ast: String::new(), errors: vec![error] },
            }
        },
        || ParseResult {
            ast: String::new(),
            errors: vec![crate::ffi::UNEXPECTED_PANIC.to_string()],
        },
    )
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
    let payload_kind = TransferPayloadKind::from_str_opt(&kind).ok_or_else(|| {
        napi::Error::from_reason(format!("Unsupported transfer payload kind: {kind}"))
    })?;

    match payload_kind {
        TransferPayloadKind::Mdast => {
            // Raw mdast transfer serializes immediately after parsing, so it
            // has the same arena shape as `parse`/`parseAndRender` and shares
            // the same reused arena.
            let parser_options = options.map(ParserOptions::from).unwrap_or_default();
            render_scratch::parse_to_mdast_raw(&source, parser_options)
                .map(Uint8Array::new)
                .map_err(napi::Error::from_reason)
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
    crate::ffi::recover(
        || {
            let parser_options = options.map(ParserOptions::from).unwrap_or_default();
            match render_scratch::parse_and_render_html(&source, parser_options) {
                Ok(html) => RenderResult { html, errors: vec![] },
                Err(error) => RenderResult { html: String::new(), errors: vec![error] },
            }
        },
        || RenderResult {
            html: String::new(),
            errors: vec![crate::ffi::UNEXPECTED_PANIC.to_string()],
        },
    )
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
        let result = match render_scratch::parse_and_render_html(&self.source, self.options.clone())
        {
            Ok(html) => RenderResult { html, errors: vec![] },
            Err(error) => RenderResult { html: String::new(), errors: vec![error] },
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
