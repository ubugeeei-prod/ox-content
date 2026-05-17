//! Node.js bindings for Ox Content.
//!
//! This crate provides NAPI bindings for using Ox Content from Node.js,
//! including raw-buffer AST transfer for JavaScript interoperability.

mod highlight;
mod lint;
mod mdast;
mod mdast_raw;
mod transfer;
mod transformer;

use napi::bindgen_prelude::*;
use napi::Task;
use napi_derive::napi;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use ox_content_allocator::Allocator;
use ox_content_docs::{
    generate_markdown, generate_nav_code, generate_nav_metadata, normalize_doc_items, ApiDocEntry,
    ApiDocModule, ApiDocTag, ApiParamDoc, ApiReturnDoc, DocExtractor, DocItem, DocItemKind, DocTag,
    DocsNavItem, MarkdownDocsOptions, NormalizedDocEntry, NormalizedParamDoc, NormalizedReturnDoc,
    ParamDoc,
};
use ox_content_parser::{Parser, ParserOptions};
use ox_content_renderer::HtmlRenderer;
use ox_content_search::{
    DocumentIndexer, SearchDocument, SearchIndex, SearchIndexBuilder, SearchOptions,
};
use transfer::TransferPayloadKind;
use transformer::{parse_frontmatter, MarkdownTransformer};

const ALLOCATOR_BYTES_PER_INPUT_BYTE: usize = 8;
const MIN_ALLOCATOR_CAPACITY: usize = 4 * 1024;

fn create_allocator_for_source(source: &str) -> Allocator {
    let capacity =
        source.len().saturating_mul(ALLOCATOR_BYTES_PER_INPUT_BYTE).max(MIN_ALLOCATOR_CAPACITY);
    Allocator::with_capacity(capacity)
}

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

/// Raw JSDoc tag extracted from source code.
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
    pub params: Vec<JsSourceDocParam>,
    pub return_type: Option<String>,
    pub tags: Vec<JsSourceDocTag>,
}

/// Normalized parameter documentation used by generated API docs.
#[napi(object)]
#[derive(Clone)]
pub struct JsDocParam {
    pub name: String,
    pub r#type: String,
    pub description: String,
    pub optional: Option<bool>,
    pub r#default: Option<String>,
}

/// Normalized return documentation used by generated API docs.
#[napi(object)]
#[derive(Clone)]
pub struct JsDocReturn {
    pub r#type: String,
    pub description: String,
}

/// Normalized documentation entry used by generated API docs.
#[napi(object)]
#[derive(Clone)]
pub struct JsDocEntry {
    pub name: String,
    pub kind: String,
    pub description: String,
    pub params: Option<Vec<JsDocParam>>,
    pub returns: Option<JsDocReturn>,
    pub examples: Option<Vec<String>>,
    pub tags: Option<HashMap<String, String>>,
    pub private: bool,
    pub file: String,
    pub line: u32,
    pub end_line: u32,
    pub signature: Option<String>,
}

/// Navigation item emitted for generated documentation.
#[napi(object)]
#[derive(Clone)]
pub struct JsDocsNavItem {
    pub title: String,
    pub path: String,
    pub children: Option<Vec<JsDocsNavItem>>,
}

/// Ordered JSDoc tag used by generated API Markdown.
#[napi(object)]
#[derive(Clone)]
pub struct JsDocsMarkdownTag {
    pub tag: String,
    pub value: String,
}

/// Documentation entry used by generated API Markdown.
#[napi(object)]
#[derive(Clone)]
pub struct JsDocsMarkdownEntry {
    pub name: String,
    pub kind: String,
    pub description: String,
    pub params: Option<Vec<JsDocParam>>,
    pub returns: Option<JsDocReturn>,
    pub examples: Option<Vec<String>>,
    pub tags: Option<Vec<JsDocsMarkdownTag>>,
    pub private: bool,
    pub file: String,
    pub line: u32,
    pub end_line: u32,
    pub signature: Option<String>,
}

/// Extracted docs for one source file used by generated API Markdown.
#[napi(object)]
#[derive(Clone)]
pub struct JsDocsMarkdownModule {
    pub file: String,
    pub entries: Vec<JsDocsMarkdownEntry>,
}

/// Options for generated API Markdown.
#[napi(object)]
#[derive(Clone)]
pub struct JsDocsMarkdownOptions {
    pub group_by: Option<String>,
    pub github_url: Option<String>,
}

/// Transform options for JavaScript.
#[napi(object)]
#[derive(Default, Clone)]
pub struct JsTransformOptions {
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
    /// Parse YAML frontmatter before transforming.
    pub frontmatter: Option<bool>,
    /// Maximum TOC depth (1-6).
    pub toc_max_depth: Option<u8>,
    /// Convert `.md` links to `.html` links for SSG output.
    pub convert_md_links: Option<bool>,
    /// Base URL for absolute link conversion (e.g., "/" or "/docs/").
    pub base_url: Option<String>,
    /// Source file path for relative link resolution.
    pub source_path: Option<String>,
    /// Enable line annotations for code blocks using fence meta.
    pub code_annotations: Option<bool>,
    /// Fence meta key used to read code annotations.
    pub code_annotation_meta_key: Option<String>,
    /// Code annotation syntax mode.
    pub code_annotation_syntax: Option<String>,
    /// Enable line numbers for all code blocks by default.
    pub code_annotation_default_line_numbers: Option<bool>,
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
            let allocator = Allocator::new();
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

fn doc_item_kind_to_string(kind: DocItemKind) -> String {
    match kind {
        DocItemKind::Module => "module",
        DocItemKind::Function => "function",
        DocItemKind::Class => "class",
        DocItemKind::Interface => "interface",
        DocItemKind::Type => "type",
        DocItemKind::Enum => "enum",
        DocItemKind::Variable => "variable",
        DocItemKind::Method => "method",
        DocItemKind::Property => "property",
        DocItemKind::Constructor => "constructor",
        DocItemKind::Getter => "getter",
        DocItemKind::Setter => "setter",
    }
    .to_string()
}

fn map_doc_tag(tag: DocTag) -> JsSourceDocTag {
    JsSourceDocTag { tag: tag.tag, value: tag.value }
}

fn map_param_doc(param: ParamDoc) -> JsSourceDocParam {
    JsSourceDocParam {
        name: param.name,
        type_annotation: param.type_annotation,
        optional: param.optional,
        default_value: param.default_value,
        description: param.description,
    }
}

fn map_doc_item(item: DocItem) -> JsSourceDocItem {
    JsSourceDocItem {
        name: item.name,
        kind: doc_item_kind_to_string(item.kind),
        doc: item.doc,
        jsdoc: item.jsdoc,
        source_path: item.source_path,
        line: item.line,
        end_line: item.end_line,
        exported: item.exported,
        signature: item.signature,
        params: item.params.into_iter().map(map_param_doc).collect(),
        return_type: item.return_type,
        tags: item.tags.into_iter().map(map_doc_tag).collect(),
    }
}

fn map_normalized_param_doc(param: NormalizedParamDoc) -> JsDocParam {
    JsDocParam {
        name: param.name,
        r#type: param.type_annotation,
        description: param.description,
        optional: param.optional.then_some(true),
        r#default: param.default_value,
    }
}

fn map_normalized_return_doc(return_doc: NormalizedReturnDoc) -> JsDocReturn {
    JsDocReturn { r#type: return_doc.type_annotation, description: return_doc.description }
}

fn map_normalized_doc_entry(entry: NormalizedDocEntry) -> JsDocEntry {
    JsDocEntry {
        name: entry.name,
        kind: entry.kind.as_str().to_string(),
        description: entry.description,
        params: (!entry.params.is_empty())
            .then(|| entry.params.into_iter().map(map_normalized_param_doc).collect()),
        returns: entry.returns.map(map_normalized_return_doc),
        examples: (!entry.examples.is_empty()).then_some(entry.examples),
        tags: (!entry.tags.is_empty()).then(|| entry.tags.into_iter().collect()),
        private: entry.private,
        file: entry.file,
        line: entry.line,
        end_line: entry.end_line,
        signature: entry.signature,
    }
}

fn map_docs_nav_item(item: DocsNavItem) -> JsDocsNavItem {
    JsDocsNavItem {
        title: item.title,
        path: item.path,
        children: item.children.map(|children| {
            children.into_iter().map(map_docs_nav_item).collect::<Vec<JsDocsNavItem>>()
        }),
    }
}

fn convert_docs_nav_item(item: JsDocsNavItem) -> DocsNavItem {
    DocsNavItem {
        title: item.title,
        path: item.path,
        children: item.children.map(|children| {
            children.into_iter().map(convert_docs_nav_item).collect::<Vec<DocsNavItem>>()
        }),
    }
}

fn convert_markdown_param(param: JsDocParam) -> ApiParamDoc {
    ApiParamDoc {
        name: param.name,
        type_annotation: param.r#type,
        description: param.description,
        optional: param.optional.unwrap_or(false),
        default_value: param.r#default,
    }
}

fn convert_markdown_return(return_doc: JsDocReturn) -> ApiReturnDoc {
    ApiReturnDoc { type_annotation: return_doc.r#type, description: return_doc.description }
}

fn convert_markdown_tag(tag: JsDocsMarkdownTag) -> ApiDocTag {
    ApiDocTag { tag: tag.tag, value: tag.value }
}

fn convert_markdown_entry(entry: JsDocsMarkdownEntry) -> ApiDocEntry {
    ApiDocEntry {
        name: entry.name,
        kind: entry.kind,
        description: entry.description,
        params: entry.params.unwrap_or_default().into_iter().map(convert_markdown_param).collect(),
        returns: entry.returns.map(convert_markdown_return),
        examples: entry.examples.unwrap_or_default(),
        tags: entry.tags.unwrap_or_default().into_iter().map(convert_markdown_tag).collect(),
        private: entry.private,
        file: entry.file,
        line: entry.line,
        end_line: entry.end_line,
        signature: entry.signature,
    }
}

fn convert_markdown_module(module: JsDocsMarkdownModule) -> ApiDocModule {
    ApiDocModule {
        file: module.file,
        entries: module.entries.into_iter().map(convert_markdown_entry).collect(),
    }
}

/// Extracts documented declarations from a JavaScript/TypeScript file using Oxc.
#[napi]
pub fn extract_file_docs(
    file_path: String,
    include_private: Option<bool>,
) -> Result<Vec<JsSourceDocItem>> {
    let extractor = DocExtractor::with_private(include_private.unwrap_or(false));
    let items = extractor
        .extract_file(Path::new(&file_path))
        .map_err(|err| Error::from_reason(err.to_string()))?;

    Ok(items.into_iter().map(map_doc_item).collect())
}

/// Extracts normalized documentation entries from a JavaScript/TypeScript file using Oxc.
#[napi(js_name = "extractFileDocEntries")]
pub fn extract_file_doc_entries(
    file_path: String,
    include_private: Option<bool>,
) -> Result<Vec<JsDocEntry>> {
    let extractor = DocExtractor::with_private(include_private.unwrap_or(false));
    let items = extractor
        .extract_file(Path::new(&file_path))
        .map_err(|err| Error::from_reason(err.to_string()))?;

    Ok(normalize_doc_items(items).into_iter().map(map_normalized_doc_entry).collect())
}

/// Generates sidebar navigation metadata from documentation file paths.
#[napi(js_name = "generateDocsNavMetadata")]
pub fn generate_docs_nav_metadata(
    files: Vec<String>,
    base_path: Option<String>,
) -> Vec<JsDocsNavItem> {
    generate_nav_metadata(&files, base_path.as_deref()).into_iter().map(map_docs_nav_item).collect()
}

/// Generates TypeScript source code for documentation navigation metadata.
#[napi(js_name = "generateDocsNavCode")]
pub fn generate_docs_nav_code(
    nav_items: Vec<JsDocsNavItem>,
    export_name: Option<String>,
) -> String {
    let nav_items = nav_items.into_iter().map(convert_docs_nav_item).collect::<Vec<_>>();
    generate_nav_code(&nav_items, export_name.as_deref())
}

/// Collects source files for generated API documentation.
#[napi(js_name = "collectDocsSourceFiles")]
pub fn collect_docs_source_files(
    src_dir: String,
    include: Vec<String>,
    exclude: Vec<String>,
) -> Vec<String> {
    ox_content_docs::collect_source_files(&src_dir, &include, &exclude)
}

/// Generates Markdown API reference pages from extracted documentation entries.
#[napi(js_name = "generateDocsMarkdown")]
pub fn generate_docs_markdown(
    docs: Vec<JsDocsMarkdownModule>,
    options: Option<JsDocsMarkdownOptions>,
) -> HashMap<String, String> {
    let options =
        options.map_or_else(MarkdownDocsOptions::default, |options| MarkdownDocsOptions {
            group_by: options.group_by.unwrap_or_else(|| "file".to_string()),
            github_url: options.github_url,
        });
    generate_markdown(&docs.into_iter().map(convert_markdown_module).collect::<Vec<_>>(), &options)
        .into_iter()
        .collect()
}

/// Restores code block metadata after JavaScript-side syntax highlighting.
#[napi]
pub fn merge_highlighted_code_blocks(original_html: String, highlighted_html: String) -> String {
    highlight::merge_highlighted_code_blocks(&original_html, &highlighted_html)
}

/// Transforms Markdown source into HTML, frontmatter, and TOC.
///
/// This is the main entry point for @ox-content/unplugin.
#[napi]
pub fn transform(source: String, options: Option<JsTransformOptions>) -> TransformResult {
    let opts = options.unwrap_or_default();
    MarkdownTransformer::from_options(&opts).transform(&source)
}

/// Transforms Markdown into a raw mdast transfer buffer.
///
/// This keeps frontmatter parsing and mdast generation on the Rust side and
/// transfers a single external memory block to JavaScript for deserialization.
#[napi]
pub fn transform_mdast_raw(
    source: String,
    options: Option<JsTransformOptions>,
) -> Result<Uint8Array> {
    let opts = options.unwrap_or_default();
    MarkdownTransformer::from_options(&opts).transform_mdast_raw(&source)
}

/// Splits Markdown source into content and frontmatter in a raw transfer buffer.
///
/// This is used by JavaScript-side markdown-it and custom unified parser paths so
/// frontmatter stripping can stay on the Rust side even when parsing continues in JS.
#[napi]
pub fn prepare_source_raw(source: String, options: Option<JsSourceOptions>) -> Result<Uint8Array> {
    let frontmatter = options.unwrap_or_default().frontmatter.unwrap_or(true);
    MarkdownTransformer::with_frontmatter(frontmatter).prepare_source_raw(&source)
}

// =============================================================================
// Async (Multi-threaded) API
// =============================================================================

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

/// Async task for transform.
pub struct TransformTask {
    source: String,
    options: JsTransformOptions,
}

impl Task for TransformTask {
    type Output = TransformResult;
    type JsValue = TransformResult;

    fn compute(&mut self) -> Result<Self::Output> {
        Ok(MarkdownTransformer::from_options(&self.options).transform(&self.source))
    }

    fn resolve(&mut self, _env: Env, output: Self::Output) -> Result<Self::JsValue> {
        Ok(output)
    }
}

/// Transforms Markdown source asynchronously (runs on worker thread).
#[napi]
pub fn transform_async(
    source: String,
    options: Option<JsTransformOptions>,
) -> AsyncTask<TransformTask> {
    let opts = options.unwrap_or_default();
    AsyncTask::new(TransformTask { source, options: opts })
}

// =============================================================================
// OG Image Generation API
// =============================================================================

/// OG image configuration for JavaScript.
#[napi(object)]
#[derive(Default, Clone)]
pub struct JsOgImageConfig {
    /// Image width in pixels.
    pub width: Option<u32>,
    /// Image height in pixels.
    pub height: Option<u32>,
    /// Background color (hex).
    pub background_color: Option<String>,
    /// Text color (hex).
    pub text_color: Option<String>,
    /// Title font size.
    pub title_font_size: Option<u32>,
    /// Description font size.
    pub description_font_size: Option<u32>,
}

/// OG image data for JavaScript.
#[napi(object)]
pub struct JsOgImageData {
    /// Page title.
    pub title: String,
    /// Page description.
    pub description: Option<String>,
    /// Site name.
    pub site_name: Option<String>,
    /// Author name.
    pub author: Option<String>,
}

/// Generates an OG image as SVG.
///
/// This function generates an SVG representation of an OG image
/// that can be used for social media previews.
#[napi]
pub fn generate_og_image_svg(data: JsOgImageData, config: Option<JsOgImageConfig>) -> String {
    use ox_content_og_image::{OgImageConfig, OgImageData, OgImageGenerator};

    let cfg = config.unwrap_or_default();
    let mut og_config = OgImageConfig::default();

    if let Some(w) = cfg.width {
        og_config.width = w;
    }
    if let Some(h) = cfg.height {
        og_config.height = h;
    }
    if let Some(ref bg) = cfg.background_color {
        og_config.background_color.clone_from(bg);
    }
    if let Some(ref tc) = cfg.text_color {
        og_config.text_color.clone_from(tc);
    }
    if let Some(ts) = cfg.title_font_size {
        og_config.title_font_size = ts;
    }
    if let Some(ds) = cfg.description_font_size {
        og_config.description_font_size = ds;
    }

    let og_data = OgImageData {
        title: data.title,
        description: data.description,
        site_name: data.site_name,
        author: data.author,
        date: None,
        tags: vec![],
    };

    let generator = OgImageGenerator::new(og_config);
    generator.generate_svg(&og_data)
}

// =============================================================================
// Full-text Search API
// =============================================================================

/// Search document for JavaScript.
#[napi(object)]
#[derive(Clone)]
pub struct JsSearchDocument {
    /// Unique document identifier.
    pub id: String,
    /// Document title.
    pub title: String,
    /// Document URL.
    pub url: String,
    /// Document body text.
    pub body: String,
    /// Document headings.
    pub headings: Vec<String>,
    /// Code snippets.
    pub code: Vec<String>,
}

fn map_search_document(doc: SearchDocument) -> JsSearchDocument {
    JsSearchDocument {
        id: doc.id,
        title: doc.title,
        url: doc.url,
        body: doc.body,
        headings: doc.headings,
        code: doc.code,
    }
}

fn extract_search_document_from_source(
    source: &str,
    id: String,
    url: String,
    parser_options: ParserOptions,
) -> SearchDocument {
    let (content, frontmatter) = parse_frontmatter(source);
    let frontmatter_title = frontmatter.get("title").and_then(|v| v.as_str()).map(String::from);
    let allocator = create_allocator_for_source(&content);
    let parser = Parser::with_options(&allocator, &content, parser_options);

    let result = parser.parse();
    let document = match &result {
        Ok(doc) => {
            let mut indexer = DocumentIndexer::new();
            indexer.extract(doc);

            SearchDocument {
                id,
                title: frontmatter_title
                    .unwrap_or_else(|| indexer.title().map(String::from).unwrap_or_default()),
                url,
                body: indexer.body().to_string(),
                headings: indexer.headings().to_vec(),
                code: indexer.code().to_vec(),
            }
        }
        Err(_) => SearchDocument {
            id,
            title: frontmatter_title.unwrap_or_default(),
            url,
            body: String::new(),
            headings: Vec::new(),
            code: Vec::new(),
        },
    };
    drop(result);

    document
}

fn build_search_index_json(documents: impl IntoIterator<Item = SearchDocument>) -> String {
    let mut builder = SearchIndexBuilder::new();

    for doc in documents {
        builder.add_document(doc);
    }

    builder.build().to_json()
}

fn search_document_id(src_dir: &Path, file: &str, extensions: &[String]) -> String {
    let file_path = Path::new(file);
    let relative_path = file_path.strip_prefix(src_dir).unwrap_or(file_path);
    let relative_path = relative_path.to_string_lossy().replace('\\', "/");

    ox_content_search::strip_markdown_extension(&relative_path, extensions)
}

/// Search result for JavaScript.
#[napi(object)]
pub struct JsSearchResult {
    /// Document ID.
    pub id: String,
    /// Document title.
    pub title: String,
    /// Document URL.
    pub url: String,
    /// Relevance score.
    pub score: f64,
    /// Matched terms.
    pub matches: Vec<String>,
    /// Content snippet.
    pub snippet: String,
}

/// Search query split into free text and scope prefixes.
#[napi(object)]
pub struct JsScopedSearchQuery {
    /// Free-text terms after removing scope prefixes.
    pub text: String,
    /// Deduplicated lowercase scopes.
    pub scopes: Vec<String>,
}

/// Search options for JavaScript.
#[napi(object)]
#[derive(Default, Clone)]
pub struct JsSearchOptions {
    /// Maximum number of results.
    pub limit: Option<u32>,
    /// Enable prefix matching.
    pub prefix: Option<bool>,
    /// Enable fuzzy matching.
    pub fuzzy: Option<bool>,
    /// Minimum score threshold.
    pub threshold: Option<f64>,
}

impl From<JsSearchOptions> for SearchOptions {
    fn from(opts: JsSearchOptions) -> Self {
        Self {
            limit: opts.limit.unwrap_or(10) as usize,
            prefix: opts.prefix.unwrap_or(true),
            fuzzy: opts.fuzzy.unwrap_or(false),
            threshold: opts.threshold.unwrap_or(0.0),
        }
    }
}

/// Builds a search index from documents.
///
/// Takes an array of documents and returns a serialized search index as JSON.
#[napi]
pub fn build_search_index(documents: Vec<JsSearchDocument>) -> String {
    build_search_index_json(documents.into_iter().map(|doc| SearchDocument {
        id: doc.id,
        title: doc.title,
        url: doc.url,
        body: doc.body,
        headings: doc.headings,
        code: doc.code,
    }))
}

/// Builds a search index directly from Markdown files under a source directory.
///
/// File discovery, Markdown parsing, search document extraction, and index
/// construction all run on the Rust side.
#[napi(js_name = "buildSearchIndexFromDirectory")]
pub fn build_search_index_from_directory(
    src_dir: String,
    base: String,
    extensions: Vec<String>,
) -> String {
    let src_path = Path::new(&src_dir);
    let parser_options = ParserOptions::gfm();
    let documents = ox_content_search::collect_markdown_files(&src_dir, &extensions)
        .into_iter()
        .filter_map(|file| {
            let source = fs::read_to_string(&file).ok()?;
            let id = search_document_id(src_path, &file, &extensions);
            let url = format!("{base}{id}");

            Some(extract_search_document_from_source(&source, id, url, parser_options.clone()))
        });

    build_search_index_json(documents)
}

/// Searches a serialized index.
///
/// Takes a JSON-serialized index, query string, and options.
/// Returns an array of search results.
#[napi]
pub fn search_index(
    index_json: String,
    query: String,
    options: Option<JsSearchOptions>,
) -> Vec<JsSearchResult> {
    let Ok(index) = SearchIndex::from_json(&index_json) else {
        return Vec::new();
    };

    let opts = options.map(SearchOptions::from).unwrap_or_default();
    let results = index.search(&query, &opts);

    results
        .into_iter()
        .map(|r| JsSearchResult {
            id: r.id,
            title: r.title,
            url: r.url,
            score: r.score,
            matches: r.matches,
            snippet: r.snippet,
        })
        .collect()
}

/// Splits a search query into free-text terms and `@scope` prefixes.
#[napi(js_name = "parseScopedSearchQuery")]
pub fn parse_scoped_search_query(query: String) -> JsScopedSearchQuery {
    let parsed = ox_content_search::parse_scoped_search_query(&query);
    JsScopedSearchQuery { text: parsed.text, scopes: parsed.scopes }
}

/// Derives hierarchical search scopes from a document id or URL.
#[napi(js_name = "getSearchDocumentScopes")]
pub fn get_search_document_scopes(id: String, url: String) -> Vec<String> {
    ox_content_search::get_search_document_scopes(&id, &url)
}

/// Returns true when a document belongs to at least one requested search scope.
#[napi(js_name = "matchesSearchScopes")]
pub fn matches_search_scopes(id: String, url: String, scopes: Vec<String>) -> bool {
    ox_content_search::matches_search_scopes(&id, &url, &scopes)
}

/// Generates the client-side search runtime module.
#[napi(js_name = "generateSearchModule")]
pub fn generate_search_module(options_json: String, index_path: String) -> String {
    ox_content_search::generate_search_module(&options_json, &index_path)
}

/// Collects Markdown files for search indexing from a source directory.
#[napi(js_name = "collectSearchMarkdownFiles")]
pub fn collect_search_markdown_files(src_dir: String, extensions: Vec<String>) -> Vec<String> {
    ox_content_search::collect_markdown_files(&src_dir, &extensions)
}

// =============================================================================
// SSG HTML Generation API
// =============================================================================

/// Navigation item for SSG.
#[napi(object)]
#[derive(Clone)]
pub struct JsSsgNavItem {
    /// Display title.
    pub title: String,
    /// URL path.
    pub path: String,
    /// Full href.
    pub href: String,
    pub children: Option<Vec<JsSsgNavItem>>,
    pub collapsed: Option<bool>,
}

/// Navigation group for SSG.
#[napi(object)]
#[derive(Clone)]
pub struct JsSsgNavGroup {
    /// Group title.
    pub title: String,
    /// Navigation items.
    pub items: Vec<JsSsgNavItem>,
    pub collapsed: Option<bool>,
}

/// Resolved SSG output and public route paths.
#[napi(object)]
pub struct JsSsgRoutePaths {
    /// HTML output file path.
    pub output_path: String,
    /// Route path without extension.
    pub url_path: String,
    /// Public HTML href.
    pub href: String,
    /// OG image output file path.
    pub og_image_path: String,
    /// OG image public URL.
    pub og_image_url: String,
}

/// Theme sidebar item for SSG navigation generation.
#[napi(object)]
#[derive(Clone, Default)]
pub struct JsSsgSidebarItem {
    /// Display text.
    pub text: Option<String>,
    /// Link URL or route path.
    pub link: Option<String>,
    /// Child sidebar items.
    pub items: Option<Vec<JsSsgSidebarItem>>,
    /// Whether this group is collapsed by default.
    pub collapsed: Option<bool>,
}

/// Manual SSG navigation item supplied by user configuration.
#[napi(object)]
#[derive(Clone)]
pub struct JsSsgNavigationItem {
    pub title: String,
    pub path: Option<String>,
    pub href: Option<String>,
}

/// Manual SSG navigation group supplied by user configuration.
#[napi(object)]
#[derive(Clone)]
pub struct JsSsgNavigationGroup {
    pub title: String,
    pub items: Vec<JsSsgNavigationItem>,
}

/// Generated SSG HTML page for shared asset extraction.
#[napi(object)]
#[derive(Clone)]
pub struct JsSsgGeneratedHtmlPage {
    /// Source Markdown path.
    pub input_path: String,
    /// Output HTML path.
    pub output_path: String,
    /// HTML content.
    pub html: String,
}

/// Shared SSG asset extracted from generated pages.
#[napi(object)]
#[derive(Clone)]
pub struct JsSsgSharedAsset {
    /// Output file path.
    pub output_path: String,
    /// Public URL path used from HTML.
    pub public_path: String,
    /// Asset content.
    pub content: String,
}

/// Result of SSG shared asset extraction.
#[napi(object)]
pub struct JsSsgExternalizedAssets {
    /// HTML pages with inline assets replaced.
    pub pages: Vec<JsSsgGeneratedHtmlPage>,
    /// Extracted shared assets.
    pub assets: Vec<JsSsgSharedAsset>,
}

/// Hero action for entry page.
#[napi(object)]
#[derive(Clone, Default)]
pub struct JsHeroAction {
    /// Button theme: "brand" or "alt".
    pub theme: Option<String>,
    /// Button text.
    pub text: String,
    /// Link URL.
    pub link: String,
}

/// Hero image for entry page.
#[napi(object)]
#[derive(Clone, Default)]
pub struct JsHeroImage {
    /// Image source URL.
    pub src: String,
    /// Light mode image source URL.
    pub light_src: Option<String>,
    /// Dark mode image source URL.
    pub dark_src: Option<String>,
    /// Alt text.
    pub alt: Option<String>,
    /// Image width.
    pub width: Option<u32>,
    /// Image height.
    pub height: Option<u32>,
}

/// Hero notice for entry page.
#[napi(object)]
#[derive(Clone, Default)]
pub struct JsHeroNotice {
    /// Notice title.
    pub title: Option<String>,
    /// Notice paragraphs.
    pub body: Option<Vec<String>>,
}

/// Hero section configuration for entry page.
#[napi(object)]
#[derive(Clone, Default)]
pub struct JsHeroConfig {
    /// Main title (large, gradient text).
    pub name: Option<String>,
    /// Secondary text.
    pub text: Option<String>,
    /// Tagline.
    pub tagline: Option<String>,
    /// Optional notice shown in the hero.
    pub notice: Option<JsHeroNotice>,
    /// Hero image.
    pub image: Option<JsHeroImage>,
    /// Action buttons.
    pub actions: Option<Vec<JsHeroAction>>,
}

/// Feature card for entry page.
#[napi(object)]
#[derive(Clone, Default)]
pub struct JsFeatureConfig {
    /// Icon - supports: "mdi:icon-name" (Iconify), image URL, or emoji.
    pub icon: Option<String>,
    /// Feature title.
    pub title: String,
    /// Feature description.
    pub details: Option<String>,
    /// Optional link.
    pub link: Option<String>,
    /// Link text.
    pub link_text: Option<String>,
}

/// Entry page configuration.
#[napi(object)]
#[derive(Clone, Default)]
pub struct JsEntryPageConfig {
    /// Hero section.
    pub hero: Option<JsHeroConfig>,
    /// Feature cards.
    pub features: Option<Vec<JsFeatureConfig>>,
}

/// Page data for SSG.
#[napi(object)]
pub struct JsSsgPageData {
    /// Page title.
    pub title: String,
    /// Page description.
    pub description: Option<String>,
    /// Page content HTML.
    pub content: String,
    /// Table of contents entries.
    pub toc: Vec<TocEntry>,
    /// Last updated timestamp in milliseconds since the Unix epoch.
    pub last_updated: Option<f64>,
    /// URL path.
    pub path: String,
    /// Entry page configuration (if layout: entry).
    pub entry_page: Option<JsEntryPageConfig>,
}

/// Returns the last git commit timestamp for a file in milliseconds.
#[napi]
pub fn get_git_last_updated(file_path: String, root: Option<String>) -> Option<f64> {
    let root = root.map(PathBuf::from)?;
    let file = PathBuf::from(&file_path);
    let pathspec = file.strip_prefix(&root).ok().and_then(|p| p.to_str()).unwrap_or(&file_path);
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["log", "-1", "--format=%ct", "--"])
        .arg(pathspec)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let seconds = String::from_utf8(output.stdout).ok()?.trim().parse::<f64>().ok()?;
    Some(seconds * 1_000.0)
}

// =============================================================================
// Theme Configuration Types for NAPI
// =============================================================================

/// Theme colors for JavaScript.
#[napi(object)]
#[derive(Clone, Default)]
pub struct JsThemeColors {
    /// Primary accent color.
    pub primary: Option<String>,
    /// Primary color on hover.
    pub primary_hover: Option<String>,
    /// Background color.
    pub background: Option<String>,
    /// Alternative background color.
    pub background_alt: Option<String>,
    /// Main text color.
    pub text: Option<String>,
    /// Muted text color.
    pub text_muted: Option<String>,
    /// Border color.
    pub border: Option<String>,
    /// Code block background color.
    pub code_background: Option<String>,
    /// Code block text color.
    pub code_text: Option<String>,
}

/// Theme fonts for JavaScript.
#[napi(object)]
#[derive(Clone, Default)]
pub struct JsThemeFonts {
    /// Sans-serif font stack.
    pub sans: Option<String>,
    /// Monospace font stack.
    pub mono: Option<String>,
}

/// Entry page theme configuration for JavaScript.
#[napi(object)]
#[derive(Clone, Default)]
pub struct JsThemeEntryPage {
    /// Landing page presentation mode.
    pub mode: Option<String>,
}

/// Theme layout for JavaScript.
#[napi(object)]
#[derive(Clone, Default)]
pub struct JsThemeLayout {
    /// Sidebar width (CSS value).
    pub sidebar_width: Option<String>,
    /// Header height (CSS value).
    pub header_height: Option<String>,
    /// Maximum content width (CSS value).
    pub max_content_width: Option<String>,
}

/// Theme header for JavaScript.
#[napi(object)]
#[derive(Clone, Default)]
pub struct JsThemeHeader {
    /// Logo image URL.
    pub logo: Option<String>,
    /// Light mode logo image URL.
    pub logo_light: Option<String>,
    /// Dark mode logo image URL.
    pub logo_dark: Option<String>,
    /// Whether to render the site name text next to the logo.
    pub show_site_name_text: Option<bool>,
    /// Logo width in pixels.
    pub logo_width: Option<u32>,
    /// Logo height in pixels.
    pub logo_height: Option<u32>,
}

/// Theme footer for JavaScript.
#[napi(object)]
#[derive(Clone, Default)]
pub struct JsThemeFooter {
    /// Footer message (supports HTML).
    pub message: Option<String>,
    /// Copyright text (supports HTML).
    pub copyright: Option<String>,
}

/// Social links for JavaScript.
#[napi(object)]
#[derive(Clone, Default)]
pub struct JsSocialLinks {
    /// GitHub URL.
    pub github: Option<String>,
    /// Twitter/X URL.
    pub twitter: Option<String>,
    /// Discord URL.
    pub discord: Option<String>,
    /// Custom social links.
    pub links: Option<Vec<JsSocialLink>>,
}

/// Custom social link for JavaScript.
#[napi(object)]
#[derive(Clone)]
pub struct JsSocialLink {
    /// Icon label.
    pub icon: Option<String>,
    /// Inline SVG icon.
    pub icon_svg: Option<String>,
    /// Link URL.
    pub link: String,
    /// Accessible label.
    pub aria_label: Option<String>,
}

/// Embedded HTML content for specific positions.
#[napi(object)]
#[derive(Clone, Default)]
pub struct JsThemeEmbed {
    /// Content to embed into `<head>`.
    pub head: Option<String>,
    /// Content before header.
    pub header_before: Option<String>,
    /// Content after header.
    pub header_after: Option<String>,
    /// Content before sidebar navigation.
    pub sidebar_before: Option<String>,
    /// Content after sidebar navigation.
    pub sidebar_after: Option<String>,
    /// Content before main content.
    pub content_before: Option<String>,
    /// Content after main content.
    pub content_after: Option<String>,
    /// Content before footer.
    pub footer_before: Option<String>,
    /// Custom footer content.
    pub footer: Option<String>,
}

/// Theme configuration for JavaScript.
#[napi(object)]
#[derive(Clone, Default)]
pub struct JsThemeConfig {
    /// Light mode colors.
    pub colors: Option<JsThemeColors>,
    /// Dark mode colors.
    pub dark_colors: Option<JsThemeColors>,
    /// Font configuration.
    pub fonts: Option<JsThemeFonts>,
    /// Entry page configuration.
    pub entry_page: Option<JsThemeEntryPage>,
    /// Layout configuration.
    pub layout: Option<JsThemeLayout>,
    /// Header configuration.
    pub header: Option<JsThemeHeader>,
    /// Footer configuration.
    pub footer: Option<JsThemeFooter>,
    /// Social links configuration.
    pub social_links: Option<JsSocialLinks>,
    /// Embedded HTML content at specific positions.
    pub embed: Option<JsThemeEmbed>,
    /// Additional custom CSS.
    pub css: Option<String>,
    /// Additional custom JavaScript.
    pub js: Option<String>,
}

/// SSG configuration.
#[napi(object)]
#[derive(Clone)]
pub struct JsSsgConfig {
    /// Site name.
    pub site_name: String,
    /// Base URL path.
    pub base: String,
    /// OG image URL.
    pub og_image: Option<String>,
    /// Theme configuration.
    pub theme: Option<JsThemeConfig>,
    /// Current locale for this page.
    pub locale: Option<String>,
    /// Available locales for locale switcher.
    pub available_locales: Option<Vec<JsLocaleInfo>>,
}

/// Locale information for the locale switcher.
#[napi(object)]
#[derive(Clone)]
pub struct JsLocaleInfo {
    /// BCP 47 locale tag.
    pub code: String,
    /// Display name.
    pub name: String,
    /// Text direction.
    pub dir: String,
}

/// Converts JsThemeColors to ox_content_ssg::ThemeColors.
fn convert_theme_colors(colors: Option<JsThemeColors>) -> Option<ox_content_ssg::ThemeColors> {
    colors.map(|c| ox_content_ssg::ThemeColors {
        primary: c.primary,
        primary_hover: c.primary_hover,
        background: c.background,
        background_alt: c.background_alt,
        text: c.text,
        text_muted: c.text_muted,
        border: c.border,
        code_background: c.code_background,
        code_text: c.code_text,
    })
}

/// Converts JsThemeConfig to ox_content_ssg::ThemeConfig.
fn convert_theme_config(theme: Option<JsThemeConfig>) -> Option<ox_content_ssg::ThemeConfig> {
    theme.map(|t| ox_content_ssg::ThemeConfig {
        colors: convert_theme_colors(t.colors),
        dark_colors: convert_theme_colors(t.dark_colors),
        fonts: t.fonts.map(|f| ox_content_ssg::ThemeFonts { sans: f.sans, mono: f.mono }),
        entry_page: t.entry_page.map(|entry| ox_content_ssg::ThemeEntryPage { mode: entry.mode }),
        layout: t.layout.map(|l| ox_content_ssg::ThemeLayout {
            sidebar_width: l.sidebar_width,
            header_height: l.header_height,
            max_content_width: l.max_content_width,
        }),
        header: t.header.map(|h| ox_content_ssg::ThemeHeader {
            logo: h.logo,
            logo_light: h.logo_light,
            logo_dark: h.logo_dark,
            show_site_name_text: h.show_site_name_text,
            logo_width: h.logo_width,
            logo_height: h.logo_height,
        }),
        footer: t
            .footer
            .map(|f| ox_content_ssg::ThemeFooter { message: f.message, copyright: f.copyright }),
        social_links: t.social_links.map(|s| ox_content_ssg::SocialLinks {
            github: s.github,
            twitter: s.twitter,
            discord: s.discord,
            links: s.links.map(|links| {
                links
                    .into_iter()
                    .map(|l| ox_content_ssg::SocialLink {
                        icon: l.icon,
                        icon_svg: l.icon_svg,
                        link: l.link,
                        aria_label: l.aria_label,
                    })
                    .collect()
            }),
        }),
        embed: t.embed.map(|e| ox_content_ssg::ThemeEmbed {
            head: e.head,
            header_before: e.header_before,
            header_after: e.header_after,
            sidebar_before: e.sidebar_before,
            sidebar_after: e.sidebar_after,
            content_before: e.content_before,
            content_after: e.content_after,
            footer_before: e.footer_before,
            footer: e.footer,
        }),
        css: t.css,
        js: t.js,
    })
}

/// Converts JsEntryPageConfig to ox_content_ssg::EntryPageConfig.
fn convert_entry_page_config(
    entry: Option<JsEntryPageConfig>,
) -> Option<ox_content_ssg::EntryPageConfig> {
    entry.map(|e| ox_content_ssg::EntryPageConfig {
        hero: e.hero.map(|h| ox_content_ssg::HeroConfig {
            name: h.name,
            text: h.text,
            tagline: h.tagline,
            notice: h
                .notice
                .map(|n| ox_content_ssg::HeroNoticeConfig { title: n.title, body: n.body }),
            image: h.image.map(|i| ox_content_ssg::HeroImage {
                src: i.src,
                light_src: i.light_src,
                dark_src: i.dark_src,
                alt: i.alt,
                width: i.width,
                height: i.height,
            }),
            actions: h.actions.map(|actions| {
                actions
                    .into_iter()
                    .map(|a| ox_content_ssg::HeroAction {
                        theme: a.theme,
                        text: a.text,
                        link: a.link,
                    })
                    .collect()
            }),
        }),
        features: e.features.map(|features| {
            features
                .into_iter()
                .map(|f| ox_content_ssg::FeatureConfig {
                    icon: f.icon,
                    title: f.title,
                    details: f.details,
                    link: f.link,
                    link_text: f.link_text,
                })
                .collect()
        }),
    })
}

fn convert_nav_item(item: JsSsgNavItem) -> ox_content_ssg::NavItem {
    ox_content_ssg::NavItem {
        title: item.title,
        path: item.path,
        href: item.href,
        children: item.children.unwrap_or_default().into_iter().map(convert_nav_item).collect(),
        collapsed: item.collapsed,
    }
}

fn map_nav_item(item: ox_content_ssg::NavItem) -> JsSsgNavItem {
    JsSsgNavItem {
        title: item.title,
        path: item.path,
        href: item.href,
        children: if item.children.is_empty() {
            None
        } else {
            Some(item.children.into_iter().map(map_nav_item).collect())
        },
        collapsed: item.collapsed,
    }
}

fn map_nav_group(group: ox_content_ssg::NavGroup) -> JsSsgNavGroup {
    JsSsgNavGroup {
        title: group.title,
        items: group.items.into_iter().map(map_nav_item).collect(),
        collapsed: group.collapsed,
    }
}

fn convert_sidebar_item(item: JsSsgSidebarItem) -> ox_content_ssg::SidebarItem {
    ox_content_ssg::SidebarItem {
        text: item.text,
        link: item.link,
        items: item.items.unwrap_or_default().into_iter().map(convert_sidebar_item).collect(),
        collapsed: item.collapsed,
    }
}

fn convert_navigation_item(item: JsSsgNavigationItem) -> ox_content_ssg::ManualNavigationItem {
    ox_content_ssg::ManualNavigationItem { title: item.title, path: item.path, href: item.href }
}

fn convert_navigation_group(group: JsSsgNavigationGroup) -> ox_content_ssg::ManualNavigationGroup {
    ox_content_ssg::ManualNavigationGroup {
        title: group.title,
        items: group.items.into_iter().map(convert_navigation_item).collect(),
    }
}

fn map_route_paths(paths: ox_content_ssg::RoutePaths) -> JsSsgRoutePaths {
    JsSsgRoutePaths {
        output_path: paths.output_path,
        url_path: paths.url_path,
        href: paths.href,
        og_image_path: paths.og_image_path,
        og_image_url: paths.og_image_url,
    }
}

fn convert_generated_html_page(page: JsSsgGeneratedHtmlPage) -> ox_content_ssg::GeneratedHtmlPage {
    ox_content_ssg::GeneratedHtmlPage {
        input_path: page.input_path,
        output_path: page.output_path,
        html: page.html,
    }
}

fn map_generated_html_page(page: ox_content_ssg::GeneratedHtmlPage) -> JsSsgGeneratedHtmlPage {
    JsSsgGeneratedHtmlPage {
        input_path: page.input_path,
        output_path: page.output_path,
        html: page.html,
    }
}

fn map_shared_asset(asset: ox_content_ssg::SharedAsset) -> JsSsgSharedAsset {
    JsSsgSharedAsset {
        output_path: asset.output_path,
        public_path: asset.public_path,
        content: asset.content,
    }
}

/// Resolves all output and public route paths for an SSG page.
#[napi(js_name = "resolveSsgRoutePaths")]
pub fn resolve_ssg_route_paths(
    input_path: String,
    src_dir: String,
    out_dir: String,
    base: String,
    extension: String,
    site_url: Option<String>,
) -> JsSsgRoutePaths {
    map_route_paths(ox_content_ssg::resolve_route_paths(
        &input_path,
        &src_dir,
        &out_dir,
        &base,
        &extension,
        site_url.as_deref(),
    ))
}

/// Converts a markdown file path to its corresponding SSG HTML output path.
#[napi(js_name = "getSsgOutputPath")]
pub fn get_ssg_output_path(
    input_path: String,
    src_dir: String,
    out_dir: String,
    extension: String,
) -> String {
    ox_content_ssg::get_output_path(&input_path, &src_dir, &out_dir, &extension)
}

/// Converts a markdown file path to a relative SSG URL path.
#[napi(js_name = "getSsgUrlPath")]
pub fn get_ssg_url_path(input_path: String, src_dir: String) -> String {
    ox_content_ssg::get_url_path(&input_path, &src_dir)
}

/// Converts a markdown file path to an SSG href.
#[napi(js_name = "getSsgHref")]
pub fn get_ssg_href(
    input_path: String,
    src_dir: String,
    base: String,
    extension: String,
) -> String {
    ox_content_ssg::get_href(&input_path, &src_dir, &base, &extension)
}

/// Resolves a page locale from an SSG URL path and configured locale codes.
#[napi(js_name = "getSsgPageLocale")]
pub fn get_ssg_page_locale(
    url_path: String,
    default_locale: String,
    locale_codes: Vec<String>,
) -> Option<String> {
    ox_content_ssg::get_page_locale(&url_path, &default_locale, &locale_codes)
}

/// Extracts a page title from frontmatter title or rendered HTML.
#[napi(js_name = "extractSsgTitle")]
pub fn extract_ssg_title(content: String, frontmatter_title: Option<String>) -> String {
    ox_content_ssg::extract_title(&content, frontmatter_title.as_deref())
}

/// Formats a file or directory segment as an SSG title.
#[napi(js_name = "formatSsgTitle")]
pub fn format_ssg_title(name: String) -> String {
    ox_content_ssg::format_title(&name)
}

/// Builds SSG navigation groups from markdown files.
#[napi(js_name = "buildSsgNavItems")]
pub fn build_ssg_nav_items(
    markdown_files: Vec<String>,
    src_dir: String,
    base: String,
    extension: String,
) -> Vec<JsSsgNavGroup> {
    ox_content_ssg::build_nav_items(&markdown_files, &src_dir, &base, &extension)
        .into_iter()
        .map(map_nav_group)
        .collect()
}

/// Builds SSG navigation groups from an explicit theme sidebar tree.
#[napi(js_name = "buildSsgThemeNavItems")]
pub fn build_ssg_theme_nav_items(
    sidebar: Vec<JsSsgSidebarItem>,
    base: String,
    extension: String,
) -> Vec<JsSsgNavGroup> {
    let sidebar: Vec<ox_content_ssg::SidebarItem> =
        sidebar.into_iter().map(convert_sidebar_item).collect();
    ox_content_ssg::build_theme_nav_items(&sidebar, &base, &extension)
        .into_iter()
        .map(map_nav_group)
        .collect()
}

/// Resolves manual SSG navigation groups.
#[napi(js_name = "resolveSsgNavigationGroups")]
pub fn resolve_ssg_navigation_groups(
    navigation: Vec<JsSsgNavigationGroup>,
    base: String,
    extension: String,
) -> Vec<JsSsgNavGroup> {
    let navigation: Vec<ox_content_ssg::ManualNavigationGroup> =
        navigation.into_iter().map(convert_navigation_group).collect();
    ox_content_ssg::resolve_navigation_groups(&navigation, &base, &extension)
        .into_iter()
        .map(map_nav_group)
        .collect()
}

/// Collects Markdown files for SSG from a source directory.
#[napi(js_name = "collectSsgMarkdownFiles")]
pub fn collect_ssg_markdown_files(src_dir: String, extensions: Vec<String>) -> Vec<String> {
    ox_content_ssg::collect_markdown_files(&src_dir, &extensions)
}

/// Generates SSG HTML page with navigation and search.
#[napi]
pub fn generate_ssg_html(
    page_data: JsSsgPageData,
    nav_groups: Vec<JsSsgNavGroup>,
    config: JsSsgConfig,
) -> String {
    // Convert NAPI types to ox_content_ssg types
    let ssg_page_data = ox_content_ssg::PageData {
        title: page_data.title,
        description: page_data.description,
        content: page_data.content,
        toc: page_data
            .toc
            .into_iter()
            .map(|t| ox_content_ssg::TocEntry { depth: t.depth, text: t.text, slug: t.slug })
            .collect(),
        last_updated: page_data
            .last_updated
            .filter(|timestamp| timestamp.is_finite() && *timestamp >= 0.0)
            .map(|timestamp| timestamp as i64),
        path: page_data.path,
        entry_page: convert_entry_page_config(page_data.entry_page),
    };

    let ssg_nav_groups: Vec<ox_content_ssg::NavGroup> = nav_groups
        .into_iter()
        .map(|g| ox_content_ssg::NavGroup {
            title: g.title,
            items: g.items.into_iter().map(convert_nav_item).collect(),
            collapsed: g.collapsed,
        })
        .collect();

    let ssg_config = ox_content_ssg::SsgConfig {
        site_name: config.site_name,
        base: config.base,
        og_image: config.og_image,
        theme: convert_theme_config(config.theme),
        locale: config.locale,
        available_locales: config.available_locales.map(|locales| {
            locales
                .into_iter()
                .map(|l| ox_content_ssg::LocaleInfo { code: l.code, name: l.name, dir: l.dir })
                .collect()
        }),
    };

    ox_content_ssg::generate_html(&ssg_page_data, &ssg_nav_groups, &ssg_config)
}

/// Extracts shared CSS and JavaScript assets from generated SSG pages.
#[napi(js_name = "externalizeSsgAssets")]
pub fn externalize_ssg_assets(
    pages: Vec<JsSsgGeneratedHtmlPage>,
    out_dir: String,
    base: String,
) -> JsSsgExternalizedAssets {
    let result = ox_content_ssg::externalize_shared_page_assets(
        pages.into_iter().map(convert_generated_html_page).collect(),
        &out_dir,
        &base,
    );

    JsSsgExternalizedAssets {
        pages: result.pages.into_iter().map(map_generated_html_page).collect(),
        assets: result.assets.into_iter().map(map_shared_asset).collect(),
    }
}

/// Extracts searchable content from Markdown source.
///
/// Parses the Markdown and extracts title, body text, headings, and code.
#[napi]
pub fn extract_search_content(
    source: String,
    id: String,
    url: String,
    options: Option<JsParserOptions>,
) -> JsSearchDocument {
    let parser_options = options.map(ParserOptions::from).unwrap_or_default();
    map_search_document(extract_search_document_from_source(&source, id, url, parser_options))
}

// =============================================================================
// Mermaid Rendering API (mmdc CLI)
// =============================================================================

/// Mermaid transform result.
#[napi(object)]
pub struct MermaidTransformResult {
    /// The transformed HTML with mermaid code blocks replaced by rendered SVGs.
    pub html: String,
    /// Non-fatal errors encountered during rendering (per-diagram).
    pub errors: Vec<String>,
}

/// Transforms mermaid code blocks in HTML to rendered SVG diagrams.
///
/// Extracts `<pre><code class="language-mermaid">...</code></pre>` blocks,
/// renders each in parallel using the mmdc CLI, and replaces them with
/// `<div class="ox-mermaid">...</div>`.
#[napi]
pub fn transform_mermaid(html: String, mmdc_path: String) -> MermaidTransformResult {
    let blocks = extract_mermaid_blocks_from_html(&html);

    if blocks.is_empty() {
        return MermaidTransformResult { html, errors: vec![] };
    }

    // Render all diagrams in parallel using scoped threads.
    // The intermediate collect() is intentional: we must spawn ALL threads before
    // joining any, otherwise they would run sequentially instead of in parallel.
    #[allow(clippy::needless_collect)]
    let render_results: Vec<std::result::Result<String, String>> = std::thread::scope(|s| {
        let handles: Vec<_> = blocks
            .iter()
            .map(|block| {
                let source = &block.source;
                let path = &mmdc_path;
                s.spawn(move || render_mermaid_with_mmdc(source, path))
            })
            .collect();

        handles
            .into_iter()
            .map(|h| h.join().unwrap_or_else(|_| Err("Thread panicked".to_string())))
            .collect()
    });

    // Replace blocks in reverse order to preserve positions
    let mut result_html = html;
    let mut errors = Vec::new();

    for (i, block) in blocks.iter().enumerate().rev() {
        match &render_results[i] {
            Ok(svg) => {
                let replacement = format!(r#"<div class="ox-mermaid">{svg}</div>"#);
                result_html.replace_range(block.start..block.end, &replacement);
            }
            Err(e) => {
                errors.push(e.clone());
            }
        }
    }

    MermaidTransformResult { html: result_html, errors }
}

struct MermaidBlock {
    start: usize,
    end: usize,
    source: String,
}

fn extract_mermaid_blocks_from_html(html: &str) -> Vec<MermaidBlock> {
    let open = r#"<pre><code class="language-mermaid">"#;
    let close = "</code></pre>";
    let mut blocks = Vec::new();
    let mut cursor = 0;

    while let Some(rel) = html[cursor..].find(open) {
        let abs_start = cursor + rel;
        let content_start = abs_start + open.len();

        if let Some(rel_end) = html[content_start..].find(close) {
            let abs_end = content_start + rel_end + close.len();
            let raw = &html[content_start..content_start + rel_end];
            blocks.push(MermaidBlock {
                start: abs_start,
                end: abs_end,
                source: decode_html_entities_mermaid(raw),
            });
            cursor = abs_end;
        } else {
            break;
        }
    }

    blocks
}

fn decode_html_entities_mermaid(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        // Numeric character references (hex)
        .replace("&#x3C;", "<")
        .replace("&#x3c;", "<")
        .replace("&#x3E;", ">")
        .replace("&#x3e;", ">")
        .replace("&#x22;", "\"")
        .replace("&#x27;", "'")
        // Numeric character references (decimal)
        .replace("&#60;", "<")
        .replace("&#62;", ">")
        .replace("&#34;", "\"")
}

static MERMAID_FILE_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

fn render_mermaid_with_mmdc(source: &str, mmdc_path: &str) -> std::result::Result<String, String> {
    use std::sync::atomic::Ordering;

    let temp_dir = std::env::temp_dir();
    let id = MERMAID_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let pid = std::process::id();

    let input_path = temp_dir.join(format!("ox_mermaid_{pid}_{id}.mmd"));
    let output_path = temp_dir.join(format!("ox_mermaid_{pid}_{id}.svg"));
    let puppeteer_config_path = temp_dir.join(format!("ox_mermaid_{pid}_{id}_puppeteer.json"));

    // Write mermaid source to temp file
    std::fs::write(&input_path, source).map_err(|e| format!("Failed to write temp file: {e}"))?;

    // Write puppeteer config with --no-sandbox for CI environments
    std::fs::write(
        &puppeteer_config_path,
        r#"{"args":["--no-sandbox","--disable-setuid-sandbox"]}"#,
    )
    .map_err(|e| format!("Failed to write puppeteer config: {e}"))?;

    // Call mmdc CLI
    let output = std::process::Command::new(mmdc_path)
        .arg("-i")
        .arg(&input_path)
        .arg("-o")
        .arg(&output_path)
        .arg("-t")
        .arg("neutral")
        .arg("-q")
        .arg("-p")
        .arg(&puppeteer_config_path)
        .output()
        .map_err(|e| {
            format!("Failed to execute mmdc: {e}. Is @mermaid-js/mermaid-cli installed?")
        })?;

    // Clean up input and puppeteer config
    let _ = std::fs::remove_file(&input_path);
    let _ = std::fs::remove_file(&puppeteer_config_path);

    if !output.status.success() {
        let _ = std::fs::remove_file(&output_path);
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("mmdc failed: {stderr}"));
    }

    // Read rendered SVG
    let svg = std::fs::read_to_string(&output_path)
        .map_err(|e| format!("Failed to read SVG output: {e}"))?;

    let _ = std::fs::remove_file(&output_path);

    // Post-process SVG
    let svg = postprocess_mermaid_svg(&svg, id);

    Ok(svg)
}

/// Post-process mermaid SVG output:
/// - Replace `background-color: white` with `transparent` for dark mode compatibility
/// - Replace all `my-svg` references with unique IDs to avoid collisions between diagrams
///   (covers the SVG id, CSS selectors, and marker id prefixes like `my-svg_flowchart-v2-pointEnd`)
fn postprocess_mermaid_svg(svg: &str, id: u64) -> String {
    let unique_id = format!("ox-mermaid-{id}");

    svg.replace("background-color: white;", "background-color: transparent;")
        .replace("background-color:white;", "background-color:transparent;")
        .replace("my-svg", &unique_id)
}

// ── i18n ──────────────────────────────────────────────────────

/// Result of loading dictionaries.
#[napi(object)]
pub struct I18nLoadResult {
    /// Number of locales loaded.
    pub locale_count: u32,
    /// All locale tags.
    pub locales: Vec<String>,
    /// Errors encountered during loading.
    pub errors: Vec<String>,
}

/// Locale metadata for generated i18n runtime modules.
#[napi(object)]
#[derive(Clone)]
pub struct JsI18nRuntimeLocale {
    /// BCP 47 locale tag.
    pub code: String,
    /// Display name for this locale.
    pub name: String,
    /// Text direction.
    pub dir: Option<String>,
}

/// Configuration for generated i18n runtime modules.
#[napi(object)]
pub struct JsI18nRuntimeConfig {
    /// Default locale tag.
    pub default_locale: String,
    /// Available locales.
    pub locales: Vec<JsI18nRuntimeLocale>,
    /// Whether URLs should omit the default locale prefix.
    pub hide_default_locale: bool,
}

/// Result of MF2 validation.
#[napi(object)]
pub struct Mf2ValidateResult {
    /// Whether the message is valid.
    pub valid: bool,
    /// Validation errors.
    pub errors: Vec<String>,
    /// AST as JSON (if parsing succeeded).
    pub ast_json: Option<String>,
}

/// A single i18n diagnostic.
#[napi(object)]
pub struct I18nDiagnostic {
    /// Severity: "error", "warning", or "info".
    pub severity: String,
    /// Diagnostic message.
    pub message: String,
    /// Related translation key, if any.
    pub key: Option<String>,
    /// Related locale, if any.
    pub locale: Option<String>,
}

/// Result of i18n checking.
#[napi(object)]
pub struct I18nCheckResult {
    /// All diagnostics.
    pub diagnostics: Vec<I18nDiagnostic>,
    /// Number of errors.
    pub error_count: u32,
    /// Number of warnings.
    pub warning_count: u32,
}

/// Loads dictionaries from the given directory.
///
/// The directory should contain locale subdirectories (e.g., `en/`, `ja/`)
/// with JSON or YAML translation files.
#[napi]
pub fn load_dictionaries(dir: String) -> I18nLoadResult {
    let path = std::path::Path::new(&dir);
    match ox_content_i18n::dictionary::load_from_dir(path) {
        Ok(set) => {
            let locales: Vec<String> = set.locales().map(String::from).collect();
            I18nLoadResult { locale_count: locales.len() as u32, locales, errors: vec![] }
        }
        Err(e) => I18nLoadResult { locale_count: 0, locales: vec![], errors: vec![e.to_string()] },
    }
}

/// Loads dictionaries from the given directory and returns a flat key-value map per locale.
///
/// Each locale maps to a flat `{ "namespace.key": "value" }` structure.
/// Supports both JSON and YAML dictionary files.
#[napi]
pub fn load_dictionaries_flat(dir: String) -> HashMap<String, HashMap<String, String>> {
    let path = std::path::Path::new(&dir);
    ox_content_i18n::runtime::load_flat_dictionaries(path).unwrap_or_default()
}

/// Generates the `virtual:ox-content/i18n` runtime module.
#[napi(js_name = "generateI18nModule")]
pub fn generate_i18n_module(dict_dir: String, config: JsI18nRuntimeConfig) -> String {
    let config = ox_content_i18n::runtime::I18nRuntimeConfig {
        default_locale: config.default_locale,
        locales: config
            .locales
            .into_iter()
            .map(|locale| ox_content_i18n::runtime::I18nRuntimeLocale {
                code: locale.code,
                name: locale.name,
                dir: locale.dir,
            })
            .collect(),
        hide_default_locale: config.hide_default_locale,
    };
    let dictionaries =
        ox_content_i18n::runtime::load_flat_dictionaries(std::path::Path::new(&dict_dir))
            .unwrap_or_default();

    ox_content_i18n::runtime::generate_runtime_module(&config, &dictionaries)
}

/// Validates an MF2 message string.
///
/// Returns parsing and semantic validation results.
#[napi]
pub fn validate_mf2(message: String) -> Mf2ValidateResult {
    match ox_content_i18n::mf2::parse_and_validate(&message) {
        Ok((ast, validation_errors)) => {
            let ast_json = serde_json::to_string(&ast).ok();
            let errors: Vec<String> = validation_errors.iter().map(ToString::to_string).collect();
            Mf2ValidateResult { valid: errors.is_empty(), errors, ast_json }
        }
        Err(e) => Mf2ValidateResult { valid: false, errors: vec![e.to_string()], ast_json: None },
    }
}

/// Runs i18n checks on dictionaries against used translation keys.
///
/// `dict_dir` is the path to the i18n directory with locale subdirectories.
/// `used_keys` is a list of translation keys found in source code.
#[napi(js_name = "checkI18n")]
pub fn check_i18n(dict_dir: String, used_keys: Vec<String>) -> I18nCheckResult {
    let path = std::path::Path::new(&dict_dir);
    let dict_set = match ox_content_i18n::dictionary::load_from_dir(path) {
        Ok(set) => set,
        Err(e) => {
            return I18nCheckResult {
                diagnostics: vec![I18nDiagnostic {
                    severity: "error".to_string(),
                    message: e.to_string(),
                    key: None,
                    locale: None,
                }],
                error_count: 1,
                warning_count: 0,
            };
        }
    };

    let keys_set: std::collections::HashSet<String> = used_keys.into_iter().collect();
    let diagnostics = ox_content_i18n::checker::check_all(&keys_set, &dict_set);

    let mut error_count = 0u32;
    let mut warning_count = 0u32;
    let js_diagnostics: Vec<I18nDiagnostic> = diagnostics
        .into_iter()
        .map(|d| {
            let severity = match d.severity {
                ox_content_i18n::checker::Severity::Error => {
                    error_count += 1;
                    "error"
                }
                ox_content_i18n::checker::Severity::Warning => {
                    warning_count += 1;
                    "warning"
                }
                ox_content_i18n::checker::Severity::Info => "info",
            };
            I18nDiagnostic {
                severity: severity.to_string(),
                message: d.message,
                key: d.key,
                locale: d.locale,
            }
        })
        .collect();

    I18nCheckResult { diagnostics: js_diagnostics, error_count, warning_count }
}

/// A translation key usage found in source code.
#[napi(object)]
pub struct I18nKeyUsage {
    /// The translation key.
    pub key: String,
    /// Source file path.
    pub file_path: String,
    /// Line number.
    pub line: u32,
    /// Column number.
    pub column: u32,
    /// End column number.
    pub end_column: u32,
}

/// Extracts translation keys from a TypeScript/JavaScript source string.
///
/// Finds calls like `t('key')` and `$t('key')`.
#[napi]
pub fn extract_translation_keys(
    source: String,
    file_path: String,
    function_names: Option<Vec<String>>,
) -> Vec<I18nKeyUsage> {
    let collector = if let Some(names) = function_names {
        ox_content_i18n_checker::key_collector::KeyCollector::with_function_names(names)
    } else {
        ox_content_i18n_checker::key_collector::KeyCollector::new()
    };

    let source_type =
        oxc_span::SourceType::from_path(std::path::Path::new(&file_path)).unwrap_or_default();

    match collector.collect_source(&source, &file_path, source_type) {
        Ok(usages) => usages
            .into_iter()
            .map(|u| I18nKeyUsage {
                key: u.key,
                file_path: u.file_path,
                line: u.line,
                column: u.column,
                end_column: u.end_column,
            })
            .collect(),
        Err(_) => vec![],
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use std::fs;
    use std::process::Command;

    use super::get_git_last_updated;
    use super::transformer::parse_frontmatter;

    #[test]
    fn parses_nested_yaml_frontmatter() {
        let (content, frontmatter) = parse_frontmatter(
            "---\ntitle: Guide\nmeta:\n  tags:\n    - rust\n    - napi\n  draft: false\n---\n# Body",
        );

        assert_eq!(content, "# Body");
        assert_eq!(frontmatter.get("title"), Some(&json!("Guide")));
        assert_eq!(
            frontmatter.get("meta"),
            Some(&json!({"tags": ["rust", "napi"], "draft": false}))
        );
    }

    #[test]
    fn frontmatter_preserves_yaml_scalars_and_quoted_colons() {
        let (_, frontmatter) = parse_frontmatter(
            "---\ncount: 3\nratio: 1.5\ncanonical: \"https://example.com/a:b\"\n---\n",
        );

        assert_eq!(frontmatter.get("count"), Some(&json!(3)));
        assert_eq!(frontmatter.get("ratio"), Some(&json!(1.5)));
        assert_eq!(frontmatter.get("canonical"), Some(&json!("https://example.com/a:b")));
    }

    #[test]
    fn malformed_yaml_strips_block_and_returns_empty_frontmatter() {
        let (content, frontmatter) = parse_frontmatter("---\ntitle: [broken\n---\nBody");

        assert_eq!(content, "Body");
        assert!(frontmatter.is_empty());
    }

    #[test]
    fn transform_passes_toc_depth_to_inline_toc() {
        let result = super::transform(
            "[[toc]]\n\n## Intro\n### API".to_string(),
            Some(super::JsTransformOptions { toc_max_depth: Some(2), ..Default::default() }),
        );

        assert!(result.html.contains("href=\"#intro\""));
        assert!(!result.html.contains("href=\"#api\""));
    }

    #[test]
    fn builds_search_index_from_directory() {
        let root =
            std::env::temp_dir().join(format!("ox-content-napi-search-{}", std::process::id()));
        let docs_dir = root.join("docs");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(docs_dir.join("guide")).unwrap();
        fs::write(
            docs_dir.join("guide/intro.markdown"),
            "---\ntitle: Native Search\n---\n# Intro\n\nSearch body text.",
        )
        .unwrap();

        let index_json = super::build_search_index_from_directory(
            docs_dir.to_string_lossy().into_owned(),
            "/docs/".to_string(),
            vec![".md".to_string(), ".markdown".to_string()],
        );
        let index = ox_content_search::SearchIndex::from_json(&index_json).unwrap();

        assert_eq!(index.doc_count, 1);
        assert_eq!(index.documents[0].id, "guide/intro");
        assert_eq!(index.documents[0].title, "Native Search");
        assert_eq!(index.documents[0].url, "/docs/guide/intro");
        assert!(index.documents[0].body.contains("Search body text"));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn git_last_updated_uses_root_relative_path() {
        let root = std::env::temp_dir().join(format!("ox-content-git-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("docs")).unwrap();
        fs::write(root.join("docs/page.md"), "# Page").unwrap();

        for args in [
            vec!["init"],
            vec!["add", "docs/page.md"],
            vec![
                "-c",
                "user.name=Test",
                "-c",
                "user.email=test@example.com",
                "commit",
                "-m",
                "init",
            ],
        ] {
            let mut cmd = Command::new("git");
            cmd.arg("-C").arg(&root).args(args);
            cmd.env("GIT_AUTHOR_DATE", "@1234567890");
            cmd.env("GIT_COMMITTER_DATE", "@1234567890");
            assert!(cmd.status().unwrap().success());
        }

        let updated = get_git_last_updated(
            root.join("docs/page.md").to_string_lossy().into_owned(),
            Some(root.to_string_lossy().into_owned()),
        );
        assert_eq!(updated, Some(1_234_567_890_000.0));
        let _ = fs::remove_dir_all(root);
    }
}
