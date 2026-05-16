//! Node.js bindings for Ox Content.
//!
//! This crate provides NAPI bindings for using Ox Content from Node.js,
//! enabling zero-copy AST transfer and JavaScript interoperability.

mod highlight;
mod lint;
mod mdast;

use napi::bindgen_prelude::*;
use napi::Task;
use napi_derive::napi;
use std::collections::HashMap;
use std::path::Path;

use ox_content_allocator::Allocator;
use ox_content_ast::{Document, Heading, Node};
use ox_content_docs::{DocExtractor, DocItem, DocItemKind, DocTag, ParamDoc};
use ox_content_parser::{Parser, ParserOptions};
use ox_content_renderer::{HtmlRenderer, HtmlRendererOptions};
use ox_content_search::{DocumentIndexer, SearchIndex, SearchIndexBuilder, SearchOptions};

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

/// Parser options for JavaScript.
#[napi(object)]
#[derive(Default)]
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
/// Returns the AST as a JSON string for zero-copy transfer to JavaScript.
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
    let toc_max_depth = opts.toc_max_depth.unwrap_or(3);

    // Parse frontmatter
    let (content, frontmatter) = parse_frontmatter(&source);

    // Parse markdown
    let allocator = create_allocator_for_source(&content);
    let parser_options = transform_options_to_parser_options(&opts);
    let parser = Parser::with_options(&allocator, &content, parser_options);

    let result = parser.parse();
    match result {
        Ok(doc) => {
            // Extract TOC from headings
            let toc = extract_toc(&doc, toc_max_depth);

            // Render to HTML
            let renderer_options = transform_options_to_renderer_options(&opts);
            let mut renderer = HtmlRenderer::with_options(renderer_options);
            let html = renderer.render(&doc);

            TransformResult {
                html,
                frontmatter: serde_json::to_string(&frontmatter)
                    .unwrap_or_else(|_| "{}".to_string()),
                toc,
                errors: vec![],
            }
        }
        Err(e) => TransformResult {
            html: String::new(),
            frontmatter: "{}".to_string(),
            toc: vec![],
            errors: vec![e.to_string()],
        },
    }
}

/// Parses YAML frontmatter from Markdown content.
fn parse_frontmatter(source: &str) -> (String, HashMap<String, serde_json::Value>) {
    // Check for frontmatter delimiter
    if !source.starts_with("---") {
        return (source.to_string(), HashMap::new());
    }

    // Find the closing delimiter
    let rest = &source[3..];
    let Some(end_pos) = rest.find("\n---") else {
        return (source.to_string(), HashMap::new());
    };

    let frontmatter_str = rest[..end_pos].trim_start_matches('\n');
    let content = rest[end_pos + 4..].trim_start_matches('\n');
    let frontmatter = serde_yaml::from_str(frontmatter_str).unwrap_or_default();

    (content.to_string(), frontmatter)
}

/// Extracts table of contents from document headings.
fn extract_toc(doc: &Document, max_depth: u8) -> Vec<TocEntry> {
    let mut entries = Vec::new();
    let mut slug_counts = HashMap::new();

    for node in &doc.children {
        if let Node::Heading(heading) = node {
            if heading.depth <= max_depth {
                let text = extract_heading_text(heading);
                let slug = unique_slug(slugify(&text), &mut slug_counts);
                entries.push(TocEntry { depth: heading.depth, text, slug });
            }
        }
    }

    entries
}

/// Extracts plain text from a heading node.
fn extract_heading_text(heading: &Heading) -> String {
    let mut text = String::new();
    for child in &heading.children {
        collect_text(child, &mut text);
    }
    text
}

/// Recursively collects text from nodes.
fn collect_text(node: &Node, text: &mut String) {
    match node {
        Node::Text(t) => text.push_str(t.value),
        Node::Emphasis(e) => {
            for child in &e.children {
                collect_text(child, text);
            }
        }
        Node::Strong(s) => {
            for child in &s.children {
                collect_text(child, text);
            }
        }
        Node::InlineCode(c) => text.push_str(c.value),
        Node::Delete(d) => {
            for child in &d.children {
                collect_text(child, text);
            }
        }
        Node::Link(l) => {
            for child in &l.children {
                collect_text(child, text);
            }
        }
        _ => {}
    }
}

/// Converts text to URL-friendly slug.
fn slugify(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() || c == ' ' || c == '-' { c } else { ' ' })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("-")
}

fn unique_slug(slug: String, counts: &mut HashMap<String, usize>) -> String {
    let slug = if slug.is_empty() { "section".to_string() } else { slug };
    let count = counts.entry(slug.clone()).or_insert(0);
    let unique = if *count == 0 { slug } else { format!("{slug}-{count}") };
    *count += 1;
    unique
}

/// Converts transform options to parser options.
fn transform_options_to_parser_options(opts: &JsTransformOptions) -> ParserOptions {
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

/// Converts transform options to renderer options.
fn transform_options_to_renderer_options(opts: &JsTransformOptions) -> HtmlRendererOptions {
    let mut options = HtmlRendererOptions::new();

    options.toc_max_depth = opts.toc_max_depth.unwrap_or(options.toc_max_depth);

    if let Some(v) = opts.convert_md_links {
        options.convert_md_links = v;
    }
    if let Some(ref v) = opts.base_url {
        options.base_url.clone_from(v);
    }
    if let Some(ref v) = opts.source_path {
        options.source_path.clone_from(v);
    }
    if let Some(v) = opts.code_annotations {
        options.code_annotations = v;
    }
    if let Some(ref v) = opts.code_annotation_meta_key {
        options.code_annotation_meta_key.clone_from(v);
    }
    if let Some(ref v) = opts.code_annotation_syntax {
        options.code_annotation_syntax = match v.as_str() {
            "vitepress" => ox_content_renderer::CodeAnnotationSyntax::VitePress,
            "both" => ox_content_renderer::CodeAnnotationSyntax::Both,
            _ => ox_content_renderer::CodeAnnotationSyntax::Attribute,
        };
    }
    if let Some(v) = opts.code_annotation_default_line_numbers {
        options.code_annotation_default_line_numbers = v;
    }

    options
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
        let toc_max_depth = self.options.toc_max_depth.unwrap_or(3);

        // Parse frontmatter
        let (content, frontmatter) = parse_frontmatter(&self.source);

        // Parse markdown
        let allocator = create_allocator_for_source(&content);
        let parser_options = transform_options_to_parser_options(&self.options);
        let parser = Parser::with_options(&allocator, &content, parser_options);

        let result = match parser.parse() {
            Ok(doc) => {
                let toc = extract_toc(&doc, toc_max_depth);
                let renderer_options = transform_options_to_renderer_options(&self.options);
                let mut renderer = HtmlRenderer::with_options(renderer_options);
                let html = renderer.render(&doc);

                TransformResult {
                    html,
                    frontmatter: serde_json::to_string(&frontmatter)
                        .unwrap_or_else(|_| "{}".to_string()),
                    toc,
                    errors: vec![],
                }
            }
            Err(e) => TransformResult {
                html: String::new(),
                frontmatter: "{}".to_string(),
                toc: vec![],
                errors: vec![e.to_string()],
            },
        };
        Ok(result)
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
    let mut builder = SearchIndexBuilder::new();

    for doc in documents {
        builder.add_document(ox_content_search::SearchDocument {
            id: doc.id,
            title: doc.title,
            url: doc.url,
            body: doc.body,
            headings: doc.headings,
            code: doc.code,
        });
    }

    let index = builder.build();
    index.to_json()
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
}

/// Navigation group for SSG.
#[napi(object)]
#[derive(Clone)]
pub struct JsSsgNavGroup {
    /// Group title.
    pub title: String,
    /// Navigation items.
    pub items: Vec<JsSsgNavItem>,
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
    /// URL path.
    pub path: String,
    /// Entry page configuration (if layout: entry).
    pub entry_page: Option<JsEntryPageConfig>,
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
        path: page_data.path,
        entry_page: convert_entry_page_config(page_data.entry_page),
    };

    let ssg_nav_groups: Vec<ox_content_ssg::NavGroup> = nav_groups
        .into_iter()
        .map(|g| ox_content_ssg::NavGroup {
            title: g.title,
            items: g
                .items
                .into_iter()
                .map(|i| ox_content_ssg::NavItem { title: i.title, path: i.path, href: i.href })
                .collect(),
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
    // Parse frontmatter first
    let (content, frontmatter) = parse_frontmatter(&source);
    let allocator = create_allocator_for_source(&content);
    let parser_options = options.map(ParserOptions::from).unwrap_or_default();

    // Try to get title from frontmatter
    let frontmatter_title = frontmatter.get("title").and_then(|v| v.as_str()).map(String::from);

    let parser = Parser::with_options(&allocator, &content, parser_options);

    let result = parser.parse();
    let (title, body, headings, code) = if let Ok(ref doc) = result {
        let mut indexer = DocumentIndexer::new();
        indexer.extract(doc);

        let title = frontmatter_title
            .unwrap_or_else(|| indexer.title().map(String::from).unwrap_or_default());

        (title, indexer.body().to_string(), indexer.headings().to_vec(), indexer.code().to_vec())
    } else {
        (frontmatter_title.unwrap_or_default(), String::new(), Vec::new(), Vec::new())
    };
    // Explicitly drop the result to release the borrow
    drop(result);

    JsSearchDocument { id, title, url, body, headings, code }
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
    let Ok(set) = ox_content_i18n::dictionary::load_from_dir(path) else {
        return HashMap::new();
    };

    let mut result = HashMap::new();
    for locale in set.locales() {
        if let Some(dict) = set.get(locale) {
            let flat: HashMap<String, String> =
                dict.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
            result.insert(locale.to_string(), flat);
        }
    }
    result
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

    use ox_content_allocator::Allocator;
    use ox_content_parser::Parser;

    use super::{extract_toc, parse_frontmatter};

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
    fn toc_slugs_are_unique_and_match_heading_ids() {
        let allocator = Allocator::new();
        let doc = Parser::new(&allocator, "## Setup!\n## Setup?\n##").parse().unwrap();

        let toc = extract_toc(&doc, 3);

        assert_eq!(toc[0].slug, "setup");
        assert_eq!(toc[1].slug, "setup-1");
        assert_eq!(toc[2].slug, "section");
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
}
