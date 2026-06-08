//! Node.js bindings for Ox Content.
//!
//! This crate provides NAPI bindings for using Ox Content from Node.js,
//! including raw-buffer AST transfer for JavaScript interoperability.

pub(crate) mod features;
mod highlight;
mod html_scan;
mod lint;
mod mdast;
mod mdast_raw;
mod media_embeds;
mod pm;
mod sanitize;
mod tabs;
mod transfer;
mod transformer;
mod youtube;

use napi::bindgen_prelude::*;
use napi::Task;
use napi_derive::napi;
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use ox_content_allocator::Allocator;
use ox_content_docs::{
    build_export_graph, extract_docs_from_directories, extract_docs_from_entry_points,
    generate_docs_data_json, generate_markdown, generate_nav_code, generate_nav_metadata,
    generate_nav_metadata_from_docs, normalize_doc_items, write_docs_output, ApiDocEntry,
    ApiDocMember, ApiDocModule, ApiDocTag, ApiParamDoc, ApiReturnDoc, ApiTypeParamDoc,
    DocExtractor, DocItem, DocItemKind, DocTag, DocsDiagnostic, DocsDiagnosticCode, DocsNavItem,
    DocsOutputOptions, EntryPointDocsOptions, EntryPointSpec, ExportGraph, ExportKind,
    ExportSource, ExternalDocsOptions, ExternalPackageSource, ExtractedDocModule, GraphOptions,
    MarkdownDisplayFormat, MarkdownDocsOptions, MarkdownLinkStyle, MarkdownPathStrategy,
    MarkdownRenderStyle, NormalizedDocEntry, NormalizedMember, NormalizedParamDoc,
    NormalizedReturnDoc, NormalizedTypeParam, ParamDoc, PublicExport,
};
use ox_content_parser::{Parser, ParserOptions};
use ox_content_renderer::HtmlRenderer;
use ox_content_search::{
    DocumentIndexer, SearchDocument, SearchIndex, SearchIndexBuilder, SearchOptions,
    SearchRuntimeOptions,
};
use transfer::TransferPayloadKind;
use transformer::{parse_frontmatter, MarkdownTransformer};

fn create_allocator_for_source(source: &str) -> Allocator {
    // NAPI parse/render calls know the full Markdown string length before
    // parsing. Use the shared source-length heuristic so synchronous native
    // calls start with one appropriately sized bump chunk instead of growing
    // from `Bump::new()` while JavaScript is blocked.
    Allocator::for_source_len(source.len())
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
    pub extends: Option<Vec<String>>,
    pub implements: Option<Vec<String>>,
    pub params: Vec<JsSourceDocParam>,
    pub return_type: Option<String>,
    pub return_members: Option<Vec<JsSourceDocItem>>,
    pub members: Option<Vec<JsSourceDocItem>>,
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
    pub members: Option<Vec<JsDocMember>>,
}

/// Type parameter documentation (`<T extends C = D>`) used by generated API docs.
#[napi(object)]
#[derive(Clone)]
pub struct JsTypeParam {
    pub name: String,
    pub constraint: Option<String>,
    pub r#default: Option<String>,
    pub description: String,
}

/// Normalized member documentation used by generated API docs.
#[napi(object)]
#[derive(Clone)]
pub struct JsDocMember {
    pub name: String,
    pub kind: String,
    pub description: String,
    pub signature: Option<String>,
    pub r#type: Option<String>,
    pub r#default: Option<String>,
    pub params: Option<Vec<JsDocParam>>,
    pub type_parameters: Option<Vec<JsTypeParam>>,
    pub returns: Option<JsDocReturn>,
    pub members: Option<Vec<JsDocMember>>,
    pub optional: Option<bool>,
    pub readonly: Option<bool>,
    pub r#static: Option<bool>,
    pub private: Option<bool>,
    pub tags: Option<HashMap<String, String>>,
    pub implementation_of: Option<Vec<String>>,
    pub line: u32,
    pub end_line: u32,
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
    pub extends: Option<Vec<String>>,
    pub implements: Option<Vec<String>>,
    /// Whether a function declaration carries an implementation body. `false` for
    /// overload signatures and ambient declarations.
    pub has_body: bool,
    pub members: Option<Vec<JsDocMember>>,
    pub type_parameters: Option<Vec<JsTypeParam>>,
}

/// Navigation item emitted for generated documentation.
#[napi(object)]
#[derive(Clone)]
pub struct JsDocsNavItem {
    pub title: String,
    pub path: String,
    pub children: Option<Vec<JsDocsNavItem>>,
}

/// Options for generating sidebar navigation metadata from extracted docs.
#[napi(object)]
#[derive(Default)]
pub struct JsDocsNavOptions {
    pub base_path: Option<String>,
    #[napi(ts_type = "'flat' | 'typedoc'")]
    pub path_strategy: Option<String>,
    /// TypeDoc-style group order for nav groups (matches `generateDocsMarkdown`'s
    /// `groupOrder` so the sidebar and page order stay in sync).
    pub group_order: Option<Vec<String>>,
    /// TypeDoc-style `sort`: ordered sort strategies for nav leaf entries (matches
    /// `generateDocsMarkdown`'s `sort`). Unsupported strategies are ignored.
    pub sort: Option<Vec<String>>,
    /// TypeDoc-style `sortEntryPoints`: when `false`, preserve the caller-provided
    /// module order instead of sorting alphabetically. Defaults to `true`.
    pub sort_entry_points: Option<bool>,
    /// TypeDoc-style `kindSortOrder`: kind ranking used for nav group order (before
    /// `groupOrder`) and the `kind` sort strategy.
    pub kind_sort_order: Option<Vec<String>>,
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
    pub extends: Option<Vec<String>>,
    pub implements: Option<Vec<String>>,
    /// Whether a function declaration carries an implementation body. Optional so
    /// callers that build entries by hand need not set it; defaults to `false`.
    /// Round-trips from `extractDocsFromEntryPoints` output unchanged.
    pub has_body: Option<bool>,
    pub members: Option<Vec<JsDocMember>>,
    pub type_parameters: Option<Vec<JsTypeParam>>,
}

/// Extracted docs for one source file used by generated API Markdown.
#[napi(object)]
#[derive(Clone)]
pub struct JsDocsMarkdownModule {
    pub file: String,
    /// Module-level description from the entry file's `@module` / leading JSDoc.
    pub description: Option<String>,
    /// Absolute source path of the entry point (from `extractDocsFromEntryPoints`'
    /// `sourcePath`). Optional; when provided, the TypeDoc path strategy places a
    /// re-exported symbol's canonical page under its defining module.
    pub source_path: Option<String>,
    /// Module-level example blocks from the entry file's `@module` / leading JSDoc.
    pub examples: Option<Vec<String>>,
    /// Module-level custom JSDoc tags.
    pub tags: Option<Vec<JsDocsMarkdownTag>>,
    pub entries: Vec<JsDocsMarkdownEntry>,
}

/// Extracted docs for one source file returned to JavaScript callers.
#[napi(object)]
#[derive(Clone)]
pub struct JsExtractedDocsModule {
    pub file: String,
    pub entries: Vec<JsDocEntry>,
}

/// Options for generated API Markdown.
#[napi(object)]
#[derive(Clone, Default)]
pub struct JsDocsMarkdownOptions {
    pub group_by: Option<String>,
    pub github_url: Option<String>,
    #[napi(ts_type = "'markdown' | 'clean'")]
    pub link_style: Option<String>,
    pub base_path: Option<String>,
    #[napi(ts_type = "'flat' | 'typedoc'")]
    pub path_strategy: Option<String>,
    #[napi(ts_type = "'html' | 'markdown'")]
    pub render_style: Option<String>,
    #[napi(ts_type = "'none' | 'list' | 'table'")]
    pub index_format: Option<String>,
    #[napi(ts_type = "'none' | 'list' | 'table'")]
    pub parameters_format: Option<String>,
    #[napi(ts_type = "'none' | 'list' | 'table'")]
    pub interface_properties_format: Option<String>,
    #[napi(ts_type = "'none' | 'list' | 'table'")]
    pub class_properties_format: Option<String>,
    #[napi(ts_type = "'none' | 'list' | 'table'")]
    pub type_alias_properties_format: Option<String>,
    #[napi(ts_type = "'none' | 'list' | 'table'")]
    pub enum_members_format: Option<String>,
    #[napi(ts_type = "'none' | 'list' | 'table'")]
    pub property_members_format: Option<String>,
    #[napi(ts_type = "'none' | 'list' | 'table'")]
    pub type_declaration_format: Option<String>,
    /// Emit the stats summary line on index pages (default: true).
    pub render_stats: Option<bool>,
    /// Emit the generated-by attribution on root index pages (default: true).
    pub render_generated_by: Option<bool>,
    /// TypeDoc-style group order for module index sections and nav groups. Unlisted
    /// groups are sorted alphabetically at `*` (or at the end when `*` is absent).
    pub group_order: Option<Vec<String>>,
    /// TypeDoc-style `sort`: ordered sort strategies applied to entries and members.
    /// Later strategies break ties left by earlier ones. Omit to keep the default
    /// (alphabetical, with enum members in declaration order). Unsupported
    /// strategies (e.g. `enum-value-*`, `documents-*`) are ignored.
    #[napi(
        ts_type = "Array<'source-order' | 'alphabetical' | 'alphabetical-ignoring-documents' | 'enum-value-ascending' | 'enum-value-descending' | 'static-first' | 'instance-first' | 'visibility' | 'required-first' | 'kind' | 'external-last' | 'documents-first' | 'documents-last'>"
    )]
    pub sort: Option<Vec<String>>,
    /// TypeDoc-style `sortEntryPoints`: when `false`, preserve the caller-provided
    /// module order instead of sorting alphabetically. Defaults to `true`.
    pub sort_entry_points: Option<bool>,
    /// TypeDoc-style `kindSortOrder`: declaration kind ranking used as the base order
    /// for module index sections / nav groups (before `groupOrder`) and the `kind`
    /// sort strategy.
    pub kind_sort_order: Option<Vec<String>>,
}

/// Options for writing generated API documentation files.
#[napi(object)]
#[derive(Clone, Default)]
pub struct JsDocsOutputOptions {
    pub generate_nav: Option<bool>,
    pub group_by: Option<String>,
    pub generated_at: Option<String>,
    pub base_path: Option<String>,
    #[napi(ts_type = "'flat' | 'typedoc'")]
    pub path_strategy: Option<String>,
    /// TypeDoc-style group order for generated nav groups.
    pub group_order: Option<Vec<String>>,
    /// TypeDoc-style sort strategies for generated nav leaf entries.
    pub sort: Option<Vec<String>>,
    /// TypeDoc-style `sortEntryPoints`: when `false`, preserve module order.
    pub sort_entry_points: Option<bool>,
    /// TypeDoc-style kind ranking for generated nav groups.
    pub kind_sort_order: Option<Vec<String>>,
}

/// Entry point used to group generated API docs.
#[napi(object)]
#[derive(Clone)]
pub struct JsEntryPointSpec {
    pub path: String,
    pub name: Option<String>,
}

/// Export graph resolution options.
#[napi(object)]
#[derive(Clone, Default)]
pub struct JsGraphOptions {
    pub root: Option<String>,
    pub tsconfig: Option<String>,
    pub external_docs: Option<bool>,
    pub external_package_sources: Option<Vec<JsExternalPackageSource>>,
}

/// Options for extracting docs grouped by entry point.
#[napi(object)]
#[derive(Clone, Default)]
pub struct JsEntryPointDocsOptions {
    pub root: Option<String>,
    pub tsconfig: Option<String>,
    pub private: Option<bool>,
    pub internal: Option<bool>,
    pub external_docs: Option<bool>,
    pub external_package_sources: Option<Vec<JsExternalPackageSource>>,
    /// Opt in to TSDoc-style type-parameter docs (`@typeParam` / `<T>` table).
    /// Off by default.
    pub type_parameters: Option<bool>,
}

/// Explicit source entry for an external package.
#[napi(object)]
#[derive(Clone)]
pub struct JsExternalPackageSource {
    pub package: String,
    pub entry: String,
}

/// Export source metadata.
#[napi(object)]
#[derive(Clone)]
pub struct JsExportSource {
    pub kind: String,
    pub module: Option<String>,
    pub package: Option<String>,
    pub specifier: Option<String>,
    pub original_name: String,
    pub type_only: bool,
}

/// Public export metadata.
#[napi(object)]
#[derive(Clone)]
pub struct JsPublicExport {
    pub name: String,
    pub kind: String,
    pub source: JsExportSource,
}

/// Public entry point module.
#[napi(object)]
#[derive(Clone)]
pub struct JsEntrypointModule {
    pub name: String,
    pub source_path: String,
    pub exports: Vec<JsPublicExport>,
}

/// Resolved source module.
#[napi(object)]
#[derive(Clone)]
pub struct JsResolvedModule {
    pub path: String,
    pub exports: Vec<JsPublicExport>,
}

/// Resolved export graph.
#[napi(object)]
#[derive(Clone)]
pub struct JsExportGraph {
    pub entrypoints: Vec<JsEntrypointModule>,
    pub modules: Vec<JsResolvedModule>,
}

/// Docs grouped by a public entry point.
#[napi(object)]
#[derive(Clone)]
pub struct JsEntrypointDocsModule {
    pub name: String,
    pub file: String,
    pub source_path: String,
    /// Module-level description from the entry file's `@module` / leading JSDoc.
    pub description: String,
    /// Module-level example blocks from the entry file's `@module` / leading JSDoc.
    pub examples: Vec<String>,
    /// Module-level custom JSDoc tags.
    pub tags: Vec<JsDocsMarkdownTag>,
    pub entries: Vec<JsDocEntry>,
    pub exports: Vec<JsPublicExport>,
    pub diagnostics: Vec<JsDocsDiagnostic>,
}

/// Diagnostic for an entry point export during docs extraction.
#[napi(object)]
#[derive(Clone)]
pub struct JsDocsDiagnostic {
    pub code: String,
    pub entrypoint: String,
    pub export_name: String,
    pub export_kind: String,
    pub source: JsExportSource,
    pub message: String,
}

/// Wiki-link transform options.
#[napi(object)]
#[derive(Default, Clone)]
pub struct JsWikiLinkOptions {
    /// Enable `[[target]]` and `[[target|label]]` expansion.
    pub enabled: Option<bool>,
    /// Base URL used for site-relative wiki links.
    pub base_url: Option<String>,
}

/// Emoji-shortcode transform options.
#[napi(object)]
#[derive(Default, Clone)]
pub struct JsEmojiShortcodeOptions {
    /// Enable `:shortcode:` expansion.
    pub enabled: Option<bool>,
    /// Custom shortcode map. Values are emitted verbatim.
    pub custom: Option<HashMap<String, String>>,
}

/// Attribute syntax transform options.
#[napi(object)]
#[derive(Default, Clone)]
pub struct JsAttrsOptions {
    /// Enable markdown-it-attrs style `{#id .class key=value}`.
    pub enabled: Option<bool>,
}

/// Code import / snippet injection options.
#[napi(object)]
#[derive(Default, Clone)]
pub struct JsCodeImportOptions {
    /// Enable `<<< path{selector}` snippet injection.
    pub enabled: Option<bool>,
    /// Root directory used for `@/` and absolute snippet imports.
    pub root_dir: Option<String>,
}

/// HTML sanitizer options.
#[napi(object)]
#[derive(Default, Clone)]
pub struct JsSanitizeOptions {
    /// Enable sanitizer. When omitted, passing this object enables it.
    pub enabled: Option<bool>,
    /// Allowed tag names. Omit for safe defaults.
    pub allowed_tags: Option<Vec<String>>,
    /// Allowed attribute names. Omit for safe defaults.
    pub allowed_attributes: Option<Vec<String>>,
    /// Allowed URL schemes for URL-bearing attributes. Omit for safe defaults.
    pub allowed_url_schemes: Option<Vec<String>>,
}

/// Edit-this-page link options.
#[napi(object)]
#[derive(Default, Clone)]
pub struct JsEditThisPageOptions {
    /// Enable edit link generation.
    pub enabled: Option<bool>,
    /// GitHub repository URL, e.g. `https://github.com/owner/repo`.
    pub repo_url: Option<String>,
    /// Branch used in edit URLs.
    pub branch: Option<String>,
    /// Root directory used to relativize `sourcePath`.
    pub root_dir: Option<String>,
    /// Link label.
    pub label: Option<String>,
}

/// Code block linting options.
#[napi(object)]
#[derive(Default, Clone)]
pub struct JsCodeBlockLintOptions {
    /// Enable code block linting.
    pub enabled: Option<bool>,
    /// Restrict linting to these language identifiers.
    pub languages: Option<Vec<String>>,
    /// Report fences without a language identifier.
    pub require_language: Option<bool>,
    /// Report trailing whitespace in code block lines.
    pub trailing_spaces: Option<bool>,
}

/// Docs-as-tests extraction options.
#[napi(object)]
#[derive(Default, Clone)]
pub struct JsDocsTestOptions {
    /// Enable docs test extraction.
    pub enabled: Option<bool>,
    /// Languages that can be emitted as test cases.
    pub languages: Option<Vec<String>>,
    /// Require fence meta such as `test`, `runnable`, or `vitest`.
    pub require_meta: Option<bool>,
}

/// Built-in media embed transform switches.
#[napi(object)]
#[derive(Default, Clone)]
pub struct JsMediaEmbedsOptions {
    /// Render `<Spotify>` embeds.
    pub spotify: Option<bool>,
    /// Render `<StackBlitz>` embeds.
    pub stack_blitz: Option<bool>,
    /// Render `<Tweet>` / `<XPost>` static cards.
    pub twitter: Option<bool>,
    /// Render `<Bluesky>` static cards.
    pub bluesky: Option<bool>,
    /// Render `<WebContainer>` lazy placeholder blocks.
    pub web_container: Option<bool>,
}

/// Extracted fenced code block.
#[napi(object)]
#[derive(Clone)]
pub struct JsCodeBlock {
    pub language: String,
    pub meta: String,
    pub code: String,
    pub start_line: u32,
    pub end_line: u32,
}

/// Diagnostic emitted by code block linting.
#[napi(object)]
#[derive(Clone)]
pub struct JsCodeBlockDiagnostic {
    pub rule_id: String,
    pub severity: String,
    pub message: String,
    pub line: u32,
    pub column: u32,
    pub end_line: u32,
    pub end_column: u32,
    pub language: Option<String>,
}

impl From<features::ExtractedCodeBlock> for JsCodeBlock {
    fn from(block: features::ExtractedCodeBlock) -> Self {
        Self {
            language: block.language,
            meta: block.meta,
            code: block.code,
            start_line: block.start_line,
            end_line: block.end_line,
        }
    }
}

impl From<features::CodeBlockDiagnostic> for JsCodeBlockDiagnostic {
    fn from(diagnostic: features::CodeBlockDiagnostic) -> Self {
        Self {
            rule_id: diagnostic.rule_id,
            severity: diagnostic.severity,
            message: diagnostic.message,
            line: diagnostic.line,
            column: diagnostic.column,
            end_line: diagnostic.end_line,
            end_column: diagnostic.end_column,
            language: diagnostic.language,
        }
    }
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
    /// Auto-link bare URLs in text. When enabled, the renderer wraps any
    /// text occurrence starting with a registered pattern (default `http://`
    /// and `https://`) in an `<a>` tag.
    pub autolink_urls: Option<bool>,
    /// URL prefix patterns for [`Self::autolink_urls`]. Overrides the
    /// default `["http://", "https://"]` when set.
    pub autolink_patterns: Option<Vec<String>>,
    /// Add `target="_blank" rel="noopener noreferrer"` to auto-linked URLs.
    /// Defaults to true; ignored when [`Self::autolink_urls`] is off.
    pub autolink_target_blank: Option<bool>,
    /// Opt-in Obsidian-style wiki links.
    pub wiki_links: Option<JsWikiLinkOptions>,
    /// Opt-in emoji shortcode expansion.
    pub emoji_shortcodes: Option<JsEmojiShortcodeOptions>,
    /// Opt-in markdown-it-attrs style attributes.
    pub attributes: Option<JsAttrsOptions>,
    /// Opt-in CJK emphasis compatibility flag. The parser is already CJK-friendly;
    /// this keeps the feature explicit in the public API.
    pub cjk_emphasis: Option<bool>,
    /// Opt-in VitePress-style code import/snippet injection.
    pub code_imports: Option<JsCodeImportOptions>,
    /// Opt-in HTML sanitizer.
    pub sanitize: Option<JsSanitizeOptions>,
    /// Opt-in edit-this-page link generation.
    pub edit_this_page: Option<JsEditThisPageOptions>,
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
        DocItemKind::EnumMember => "enumMember",
        DocItemKind::IndexSignature => "indexSignature",
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
    let return_members = (!item.return_members.is_empty())
        .then(|| item.return_members.into_iter().map(map_doc_item).collect());
    let members =
        (!item.children.is_empty()).then(|| item.children.into_iter().map(map_doc_item).collect());

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
        extends: (!item.extends.is_empty()).then_some(item.extends),
        implements: (!item.implements.is_empty()).then_some(item.implements),
        params: item.params.into_iter().map(map_param_doc).collect(),
        return_type: item.return_type,
        return_members,
        members,
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
    JsDocReturn {
        r#type: return_doc.type_annotation,
        description: return_doc.description,
        members: (!return_doc.members.is_empty())
            .then(|| return_doc.members.into_iter().map(map_normalized_member).collect()),
    }
}

fn map_normalized_member(member: NormalizedMember) -> JsDocMember {
    JsDocMember {
        name: member.name,
        kind: member.kind.as_str().to_string(),
        description: member.description,
        signature: member.signature,
        r#type: member.type_annotation,
        r#default: member.default_value,
        params: (!member.params.is_empty())
            .then(|| member.params.into_iter().map(map_normalized_param_doc).collect()),
        type_parameters: (!member.type_parameters.is_empty())
            .then(|| member.type_parameters.into_iter().map(map_normalized_type_param).collect()),
        returns: member.returns.map(map_normalized_return_doc),
        members: (!member.members.is_empty())
            .then(|| member.members.into_iter().map(map_normalized_member).collect()),
        optional: member.optional.then_some(true),
        readonly: member.readonly.then_some(true),
        r#static: member.r#static.then_some(true),
        private: member.private.then_some(true),
        tags: (!member.tags.is_empty()).then(|| member.tags.into_iter().collect()),
        implementation_of: None,
        line: member.line,
        end_line: member.end_line,
    }
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
        extends: (!entry.extends.is_empty()).then_some(entry.extends),
        implements: (!entry.implements.is_empty()).then_some(entry.implements),
        has_body: entry.has_body,
        members: (!entry.members.is_empty())
            .then(|| entry.members.into_iter().map(map_normalized_member).collect()),
        type_parameters: (!entry.type_parameters.is_empty())
            .then(|| entry.type_parameters.into_iter().map(map_normalized_type_param).collect()),
    }
}

fn map_normalized_type_param(type_param: NormalizedTypeParam) -> JsTypeParam {
    JsTypeParam {
        name: type_param.name,
        constraint: type_param.constraint,
        r#default: type_param.default,
        description: type_param.description,
    }
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

fn convert_entrypoint_spec(spec: JsEntryPointSpec) -> EntryPointSpec {
    EntryPointSpec { path: PathBuf::from(spec.path), name: spec.name }
}

fn convert_graph_options(options: Option<JsGraphOptions>) -> GraphOptions {
    let options = options.unwrap_or_default();
    GraphOptions {
        root: options.root.map(PathBuf::from),
        tsconfig: options.tsconfig.map(PathBuf::from),
        external_docs: ExternalDocsOptions {
            enabled: options.external_docs.unwrap_or(false),
            package_sources: convert_external_package_sources(options.external_package_sources),
        },
    }
}

fn convert_entrypoint_docs_options(
    options: Option<JsEntryPointDocsOptions>,
) -> EntryPointDocsOptions {
    let options = options.unwrap_or_default();
    EntryPointDocsOptions {
        graph: GraphOptions {
            root: options.root.map(PathBuf::from),
            tsconfig: options.tsconfig.map(PathBuf::from),
            external_docs: ExternalDocsOptions {
                enabled: options.external_docs.unwrap_or(false),
                package_sources: convert_external_package_sources(options.external_package_sources),
            },
        },
        include_private: options.private.unwrap_or(false),
        include_internal: options.internal.unwrap_or(false),
        type_parameters: options.type_parameters.unwrap_or(false),
    }
}

fn convert_external_package_sources(
    sources: Option<Vec<JsExternalPackageSource>>,
) -> Vec<ExternalPackageSource> {
    sources
        .unwrap_or_default()
        .into_iter()
        .map(|source| ExternalPackageSource {
            package: source.package,
            entry: PathBuf::from(source.entry),
        })
        .collect()
}

fn map_export_kind(kind: ExportKind) -> String {
    match kind {
        ExportKind::Value => "value",
        ExportKind::Type => "type",
        ExportKind::ValueAndType => "valueAndType",
        ExportKind::Namespace => "namespace",
        ExportKind::Default => "default",
    }
    .to_string()
}

fn map_export_source(source: ExportSource) -> JsExportSource {
    match source {
        ExportSource::Local { module, original_name } => JsExportSource {
            kind: "local".to_string(),
            module: Some(path_to_string(&module)),
            package: None,
            specifier: None,
            original_name,
            type_only: false,
        },
        ExportSource::External { package, specifier, module, original_name, type_only } => {
            JsExportSource {
                kind: "external".to_string(),
                module: module.as_ref().map(|module| path_to_string(module)),
                package: Some(package),
                specifier: (!specifier.is_empty()).then_some(specifier),
                original_name,
                type_only,
            }
        }
    }
}

fn map_public_export(export: PublicExport) -> JsPublicExport {
    JsPublicExport {
        name: export.name,
        kind: map_export_kind(export.kind),
        source: map_export_source(export.source),
    }
}

fn map_docs_diagnostic_code(code: DocsDiagnosticCode) -> String {
    match code {
        DocsDiagnosticCode::FilteredByVisibility => "filteredByVisibility",
        DocsDiagnosticCode::MissingDeclaration => "missingDeclaration",
        DocsDiagnosticCode::UnsupportedExport => "unsupportedExport",
        DocsDiagnosticCode::UnresolvedExternal => "unresolvedExternal",
    }
    .to_string()
}

fn map_docs_diagnostic(diagnostic: DocsDiagnostic) -> JsDocsDiagnostic {
    JsDocsDiagnostic {
        code: map_docs_diagnostic_code(diagnostic.code),
        entrypoint: diagnostic.entrypoint,
        export_name: diagnostic.export_name,
        export_kind: map_export_kind(diagnostic.export_kind),
        source: map_export_source(diagnostic.source),
        message: diagnostic.message,
    }
}

fn map_export_graph(graph: ExportGraph) -> JsExportGraph {
    JsExportGraph {
        entrypoints: graph
            .entrypoints
            .into_iter()
            .map(|entrypoint| JsEntrypointModule {
                name: entrypoint.name,
                source_path: path_to_string(&entrypoint.source_path),
                exports: entrypoint.exports.into_iter().map(map_public_export).collect(),
            })
            .collect(),
        modules: graph
            .modules
            .into_values()
            .map(|module| JsResolvedModule {
                path: path_to_string(&module.path),
                exports: module.exports.into_iter().map(map_public_export).collect(),
            })
            .collect(),
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
    ApiReturnDoc {
        type_annotation: return_doc.r#type,
        description: return_doc.description,
        members: return_doc
            .members
            .unwrap_or_default()
            .into_iter()
            .map(convert_markdown_member)
            .collect(),
    }
}

fn convert_markdown_tag(tag: JsDocsMarkdownTag) -> ApiDocTag {
    ApiDocTag { tag: tag.tag, value: tag.value }
}

fn map_api_doc_tag(tag: ApiDocTag) -> JsDocsMarkdownTag {
    JsDocsMarkdownTag { tag: tag.tag, value: tag.value }
}

fn convert_markdown_member(member: JsDocMember) -> ApiDocMember {
    ApiDocMember {
        name: member.name,
        kind: member.kind,
        description: member.description,
        signature: member.signature,
        type_annotation: member.r#type,
        default_value: member.r#default,
        params: member.params.unwrap_or_default().into_iter().map(convert_markdown_param).collect(),
        type_parameters: member
            .type_parameters
            .unwrap_or_default()
            .into_iter()
            .map(convert_markdown_type_param)
            .collect(),
        returns: member.returns.map(convert_markdown_return),
        members: member
            .members
            .unwrap_or_default()
            .into_iter()
            .map(convert_markdown_member)
            .collect(),
        optional: member.optional.unwrap_or(false),
        readonly: member.readonly.unwrap_or(false),
        r#static: member.r#static.unwrap_or(false),
        private: member.private.unwrap_or(false),
        tags: member
            .tags
            .unwrap_or_default()
            .into_iter()
            .map(|(tag, value)| ApiDocTag { tag, value })
            .collect(),
        implementation_of: member.implementation_of.unwrap_or_default(),
        line: member.line,
        end_line: member.end_line,
    }
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
        extends: entry.extends.unwrap_or_default(),
        implements: entry.implements.unwrap_or_default(),
        has_body: entry.has_body.unwrap_or(false),
        members: entry
            .members
            .unwrap_or_default()
            .into_iter()
            .map(convert_markdown_member)
            .collect(),
        type_parameters: entry
            .type_parameters
            .unwrap_or_default()
            .into_iter()
            .map(convert_markdown_type_param)
            .collect(),
    }
}

fn convert_markdown_type_param(type_param: JsTypeParam) -> ApiTypeParamDoc {
    ApiTypeParamDoc {
        name: type_param.name,
        constraint: type_param.constraint,
        default: type_param.r#default,
        description: type_param.description,
    }
}

fn convert_markdown_module(module: JsDocsMarkdownModule) -> ApiDocModule {
    ApiDocModule {
        file: module.file,
        description: module.description.unwrap_or_default(),
        source_path: module.source_path.unwrap_or_default(),
        examples: module.examples.unwrap_or_default(),
        tags: module.tags.unwrap_or_default().into_iter().map(convert_markdown_tag).collect(),
        entries: module.entries.into_iter().map(convert_markdown_entry).collect(),
    }
}

fn map_extracted_doc_module(module: ExtractedDocModule) -> JsExtractedDocsModule {
    JsExtractedDocsModule {
        file: module.file,
        entries: module.entries.into_iter().map(map_normalized_doc_entry).collect(),
    }
}

fn convert_docs_output_options(options: Option<JsDocsOutputOptions>) -> DocsOutputOptions {
    let options = options.unwrap_or_default();
    DocsOutputOptions {
        generate_nav: options.generate_nav.unwrap_or(false),
        group_by: options.group_by.unwrap_or_else(|| "file".to_string()),
        generated_at: options.generated_at.unwrap_or_default(),
        base_path: options.base_path,
        path_strategy: parse_markdown_path_strategy(options.path_strategy.as_deref()),
        group_order: options.group_order,
        sort: options.sort,
        sort_entry_points: options.sort_entry_points.unwrap_or(true),
        kind_sort_order: options.kind_sort_order,
    }
}

/// Extracts documented declarations from a JavaScript/TypeScript file using Oxc.
#[napi]
pub fn extract_file_docs(
    file_path: String,
    include_private: Option<bool>,
    include_internal: Option<bool>,
) -> Result<Vec<JsSourceDocItem>> {
    let extractor = DocExtractor::with_visibility(
        include_private.unwrap_or(false),
        include_internal.unwrap_or(false),
    );
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
    include_internal: Option<bool>,
    type_parameters: Option<bool>,
) -> Result<Vec<JsDocEntry>> {
    let extractor = DocExtractor::with_visibility(
        include_private.unwrap_or(false),
        include_internal.unwrap_or(false),
    );
    let items = extractor
        .extract_file(Path::new(&file_path))
        .map_err(|err| Error::from_reason(err.to_string()))?;

    Ok(normalize_doc_items(items, type_parameters.unwrap_or(false))
        .into_iter()
        .map(map_normalized_doc_entry)
        .collect())
}

/// Generates sidebar navigation metadata from documentation file paths.
#[napi(js_name = "generateDocsNavMetadata")]
pub fn generate_docs_nav_metadata(
    files: Vec<String>,
    base_path: Option<String>,
) -> Vec<JsDocsNavItem> {
    generate_nav_metadata(&files, base_path.as_deref()).into_iter().map(map_docs_nav_item).collect()
}

/// Generates sidebar navigation metadata from extracted documentation modules.
///
/// Use this when the output `pathStrategy` is `"typedoc"` so that the navigation
/// tree mirrors the nested module/category/symbol pages.
#[napi(js_name = "generateDocsNavMetadataFromDocs")]
pub fn generate_docs_nav_metadata_from_docs_napi(
    docs: Vec<JsDocsMarkdownModule>,
    options: Option<JsDocsNavOptions>,
) -> Vec<JsDocsNavItem> {
    let options = options.unwrap_or_default();
    let strategy = parse_markdown_path_strategy(options.path_strategy.as_deref());
    let modules = docs.into_iter().map(convert_markdown_module).collect::<Vec<_>>();
    generate_nav_metadata_from_docs(
        &modules,
        options.base_path.as_deref(),
        strategy,
        options.group_order.as_deref(),
        options.sort.as_deref(),
        options.sort_entry_points.unwrap_or(true),
        options.kind_sort_order.as_deref(),
    )
    .into_iter()
    .map(map_docs_nav_item)
    .collect()
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

/// Extracts normalized documentation entries from source directories using Oxc.
#[napi(js_name = "extractDocsFromDirectories")]
pub fn extract_docs_from_directories_napi(
    src_dirs: Vec<String>,
    include: Vec<String>,
    exclude: Vec<String>,
    include_private: Option<bool>,
    include_internal: Option<bool>,
    type_parameters: Option<bool>,
) -> Result<Vec<JsExtractedDocsModule>> {
    let modules = extract_docs_from_directories(
        &src_dirs,
        &include,
        &exclude,
        include_private.unwrap_or(false),
        include_internal.unwrap_or(false),
        type_parameters.unwrap_or(false),
    )
    .map_err(|err| Error::from_reason(err.to_string()))?;

    Ok(modules.into_iter().map(map_extracted_doc_module).collect())
}

/// Builds the public API export graph from entry points.
#[napi(js_name = "buildExportGraph")]
pub fn build_export_graph_napi(
    entry_points: Vec<JsEntryPointSpec>,
    options: Option<JsGraphOptions>,
) -> Result<JsExportGraph> {
    let entry_points = entry_points.into_iter().map(convert_entrypoint_spec).collect::<Vec<_>>();
    let graph = build_export_graph(&entry_points, &convert_graph_options(options))
        .map_err(|error| Error::from_reason(error.to_string()))?;
    Ok(map_export_graph(graph))
}

/// Extracts generated API docs grouped by public entry points.
#[napi(js_name = "extractDocsFromEntryPoints")]
pub fn extract_docs_from_entry_points_napi(
    entry_points: Vec<JsEntryPointSpec>,
    options: Option<JsEntryPointDocsOptions>,
) -> Result<Vec<JsEntrypointDocsModule>> {
    let entry_points = entry_points.into_iter().map(convert_entrypoint_spec).collect::<Vec<_>>();
    let modules =
        extract_docs_from_entry_points(&entry_points, &convert_entrypoint_docs_options(options))
            .map_err(|error| Error::from_reason(error.to_string()))?;

    Ok(modules
        .into_iter()
        .map(|module| JsEntrypointDocsModule {
            name: module.name,
            file: module.file,
            source_path: path_to_string(&module.source_path),
            description: module.description,
            examples: module.examples,
            tags: module.tags.into_iter().map(map_api_doc_tag).collect(),
            entries: module.entries.into_iter().map(map_normalized_doc_entry).collect(),
            exports: module.exports.into_iter().map(map_public_export).collect(),
            diagnostics: module.diagnostics.into_iter().map(map_docs_diagnostic).collect(),
        })
        .collect())
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
            link_style: parse_markdown_link_style(options.link_style.as_deref()),
            base_path: options.base_path,
            path_strategy: parse_markdown_path_strategy(options.path_strategy.as_deref()),
            render_style: parse_markdown_render_style(options.render_style.as_deref()),
            index_format: parse_markdown_display_format(options.index_format.as_deref()),
            parameters_format: parse_markdown_display_format(options.parameters_format.as_deref()),
            interface_properties_format: parse_markdown_display_format(
                options.interface_properties_format.as_deref(),
            ),
            class_properties_format: parse_markdown_display_format(
                options.class_properties_format.as_deref(),
            ),
            type_alias_properties_format: parse_markdown_display_format(
                options.type_alias_properties_format.as_deref(),
            ),
            enum_members_format: parse_markdown_display_format(
                options.enum_members_format.as_deref(),
            ),
            property_members_format: parse_markdown_display_format(
                options.property_members_format.as_deref(),
            ),
            type_declaration_format: parse_markdown_display_format(
                options.type_declaration_format.as_deref(),
            ),
            render_stats: options.render_stats.unwrap_or(true),
            render_generated_by: options.render_generated_by.unwrap_or(true),
            group_order: options.group_order,
            sort: options.sort,
            sort_entry_points: options.sort_entry_points.unwrap_or(true),
            kind_sort_order: options.kind_sort_order,
        });
    generate_markdown(&docs.into_iter().map(convert_markdown_module).collect::<Vec<_>>(), &options)
        .into_iter()
        .collect()
}

fn parse_markdown_link_style(link_style: Option<&str>) -> MarkdownLinkStyle {
    match link_style {
        Some("clean") => MarkdownLinkStyle::Clean,
        _ => MarkdownLinkStyle::Markdown,
    }
}

fn parse_markdown_path_strategy(path_strategy: Option<&str>) -> MarkdownPathStrategy {
    match path_strategy {
        Some("typedoc") => MarkdownPathStrategy::TypeDoc,
        _ => MarkdownPathStrategy::Flat,
    }
}

fn parse_markdown_render_style(render_style: Option<&str>) -> MarkdownRenderStyle {
    match render_style {
        Some("markdown") => MarkdownRenderStyle::Markdown,
        _ => MarkdownRenderStyle::Html,
    }
}

fn parse_markdown_display_format(format: Option<&str>) -> MarkdownDisplayFormat {
    match format {
        Some("list") => MarkdownDisplayFormat::List,
        Some("table") => MarkdownDisplayFormat::Table,
        _ => MarkdownDisplayFormat::None,
    }
}

/// Generates the machine-readable docs data JSON payload.
#[napi(js_name = "generateDocsDataJson")]
pub fn generate_docs_data_json_napi(
    docs: Vec<JsDocsMarkdownModule>,
    generated_at: String,
) -> Result<String> {
    generate_docs_data_json(
        &docs.into_iter().map(convert_markdown_module).collect::<Vec<_>>(),
        &generated_at,
    )
    .map_err(|error| Error::from_reason(error.to_string()))
}

/// Writes generated API documentation files and native sidecars.
#[napi(js_name = "writeGeneratedDocs")]
#[allow(clippy::implicit_hasher)]
pub fn write_generated_docs(
    docs: HashMap<String, String>,
    out_dir: String,
    extracted_docs: Option<Vec<JsDocsMarkdownModule>>,
    options: Option<JsDocsOutputOptions>,
) -> Result<()> {
    let docs = docs.into_iter().collect::<BTreeMap<_, _>>();
    let extracted_docs = extracted_docs
        .map(|docs| docs.into_iter().map(convert_markdown_module).collect::<Vec<_>>());
    let options = convert_docs_output_options(options);

    write_docs_output(&docs, Path::new(&out_dir), extracted_docs.as_deref(), &options)
        .map_err(|error| Error::from_reason(error.to_string()))
}

/// Restores code block metadata after JavaScript-side syntax highlighting.
#[napi]
pub fn merge_highlighted_code_blocks(original_html: String, highlighted_html: String) -> String {
    highlight::merge_highlighted_code_blocks(&original_html, &highlighted_html)
}

/// Options for [`transform_youtube_embeds`]; all optional, matching the TS
/// `YouTubeOptions` defaults when omitted.
#[napi(object)]
pub struct JsYouTubeOptions {
    /// Use privacy-enhanced mode (youtube-nocookie.com). Default: true.
    pub privacy_enhanced: Option<bool>,
    /// Default aspect ratio. Default: "16/9".
    pub aspect_ratio: Option<String>,
    /// Allow fullscreen. Default: true.
    pub allow_fullscreen: Option<bool>,
    /// Lazy-load the iframe. Default: true.
    pub lazy_load: Option<bool>,
}

/// Rewrites `<youtube …>` elements in rendered HTML into responsive,
/// privacy-enhanced iframe embeds. Rust port of the TS `transformYouTube`.
#[napi]
pub fn transform_youtube_embeds(html: String, options: Option<JsYouTubeOptions>) -> String {
    let defaults = youtube::YouTubeEmbedOptions::default();
    let resolved = match options {
        Some(options) => youtube::YouTubeEmbedOptions {
            privacy_enhanced: options.privacy_enhanced.unwrap_or(defaults.privacy_enhanced),
            aspect_ratio: options.aspect_ratio.unwrap_or(defaults.aspect_ratio),
            allow_fullscreen: options.allow_fullscreen.unwrap_or(defaults.allow_fullscreen),
            lazy_load: options.lazy_load.unwrap_or(defaults.lazy_load),
        },
        None => defaults,
    };
    youtube::transform_youtube(&html, &resolved)
}

/// Result of [`transform_tabs_embeds`].
#[napi(object)]
pub struct JsTabsTransformResult {
    /// HTML with every `<tabs>` block expanded.
    pub html: String,
    /// Number of tab groups expanded; the caller advances its group counter by
    /// this amount so generated CSS covers exactly the emitted groups.
    pub group_count: u32,
}

/// Rewrites `<tabs><tab>…</tab></tabs>` blocks in rendered HTML into the no-JS
/// CSS tab widget plus a `<details>` fallback. Rust port of the TS
/// `transformTabs`. Groups are numbered from `start_group`.
#[napi]
pub fn transform_tabs_embeds(html: String, start_group: u32) -> JsTabsTransformResult {
    let result = tabs::transform_tabs(&html, start_group);
    JsTabsTransformResult { html: result.html, group_count: result.group_count }
}

/// Options for [`transform_pm_embeds`].
#[napi(object)]
pub struct JsPmOptions {
    /// Enable opt-in synced package-manager tab groups. When `true`, a
    /// `data-ox-tab-group="pkg-manager"` attribute is emitted so the client
    /// runtime keeps every pm tab group on the page in sync via `localStorage`.
    /// Off by default; when omitted/`false` the output has no group attribute
    /// and behaves exactly like a standalone tab group.
    pub sync: Option<bool>,
}

/// Result of [`transform_pm_embeds`].
#[napi(object)]
pub struct JsPmTransformResult {
    /// HTML with every `<pm>` block expanded into a package-manager tab widget.
    pub html: String,
    /// Number of tab groups expanded; the caller advances its shared tab-group
    /// counter by this amount.
    pub group_count: u32,
}

/// Expand `<pm>` blocks in rendered HTML into npm/pnpm/yarn/bun install tabs.
///
/// The single npm-style command inside each `<pm>` element is converted to the
/// equivalent command for every package manager and rendered into the shared
/// `ox-tabs` widget. Groups are numbered from `start_group`. Syncing is opt-in
/// via `options.sync` and off by default.
#[napi]
pub fn transform_pm_embeds(
    html: String,
    start_group: u32,
    options: Option<JsPmOptions>,
) -> JsPmTransformResult {
    let resolved =
        pm::PmOptions { sync: options.and_then(|options| options.sync).unwrap_or(false) };
    let result = pm::transform_pm(&html, start_group, resolved);
    JsPmTransformResult { html: result.html, group_count: result.group_count }
}

/// Transforms Markdown source into HTML, frontmatter, and TOC.
///
/// This is the main entry point for @ox-content/unplugin.
#[napi]
pub fn transform(source: String, options: Option<JsTransformOptions>) -> TransformResult {
    let opts = options.unwrap_or_default();
    MarkdownTransformer::from_options(&opts).transform(&source)
}

/// Sanitize an HTML string with safe defaults or an explicit allow-list.
#[napi(js_name = "sanitizeHtml")]
pub fn sanitize_html_binding(html: String, options: Option<JsSanitizeOptions>) -> String {
    sanitize::sanitize_html(&html, options.as_ref())
}

/// Transform opt-in static media embed components in already-rendered HTML.
#[napi(js_name = "transformMediaEmbeds")]
pub fn transform_media_embeds(html: String, options: Option<JsMediaEmbedsOptions>) -> String {
    media_embeds::transform_media_embeds(&html, options.as_ref())
}

/// Extract fenced code blocks from Markdown.
#[napi(js_name = "extractCodeBlocks")]
pub fn extract_code_blocks(source: String) -> Vec<JsCodeBlock> {
    features::extract_code_blocks(&source).into_iter().map(Into::into).collect()
}

/// Lint fenced code blocks in Markdown.
#[napi(js_name = "lintCodeBlocks")]
pub fn lint_code_blocks(
    source: String,
    options: Option<JsCodeBlockLintOptions>,
) -> Vec<JsCodeBlockDiagnostic> {
    features::lint_code_blocks(&source, options.as_ref()).into_iter().map(Into::into).collect()
}

/// Extract runnable documentation examples for Vitest harness generation.
#[napi(js_name = "extractDocsTests")]
pub fn extract_docs_tests(source: String, options: Option<JsDocsTestOptions>) -> Vec<JsCodeBlock> {
    features::extract_docs_tests(&source, options.as_ref()).into_iter().map(Into::into).collect()
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

/// Splits Markdown source into content and parsed frontmatter.
#[napi(js_name = "prepareSource")]
pub fn prepare_source(source: String, options: Option<JsSourceOptions>) -> PreparedSourceResult {
    let frontmatter = options.unwrap_or_default().frontmatter.unwrap_or(true);
    let prepared = MarkdownTransformer::with_frontmatter(frontmatter).prepare_source(&source);

    PreparedSourceResult {
        content: prepared.content,
        frontmatter: prepared.frontmatter,
        source_offset: JsSourceOrigin {
            byte_offset: prepared.source_origin.byte_offset,
            offset: prepared.source_origin.offset,
            line: prepared.source_origin.line,
            column: prepared.source_origin.column,
        },
    }
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

/// Resolved search runtime options for JavaScript.
#[napi(object)]
pub struct JsSearchRuntimeOptions {
    /// Whether search is enabled.
    pub enabled: bool,
    /// Maximum number of results.
    pub limit: u32,
    /// Enable prefix matching.
    pub prefix: bool,
    /// Search input placeholder.
    pub placeholder: String,
    /// Keyboard shortcut to focus search.
    pub hotkey: String,
}

impl From<JsSearchRuntimeOptions> for SearchRuntimeOptions {
    fn from(options: JsSearchRuntimeOptions) -> Self {
        Self {
            enabled: options.enabled,
            limit: options.limit,
            prefix: options.prefix,
            placeholder: options.placeholder,
            hotkey: options.hotkey,
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

/// Generates the client-side search runtime module from typed options.
#[napi(js_name = "generateSearchModuleFromOptions")]
pub fn generate_search_module_from_options(
    options: JsSearchRuntimeOptions,
    index_path: String,
) -> String {
    ox_content_search::generate_search_module_with_options(&options.into(), &index_path)
}

/// Collects Markdown files for search indexing from a source directory.
#[napi(js_name = "collectSearchMarkdownFiles")]
pub fn collect_search_markdown_files(src_dir: String, extensions: Vec<String>) -> Vec<String> {
    ox_content_search::collect_markdown_files(&src_dir, &extensions)
}

/// Writes a serialized search index to `search-index.json` under an output directory.
#[napi(js_name = "writeSearchIndex")]
pub fn write_search_index(index_json: String, out_dir: String) -> Result<()> {
    ox_content_search::write_search_index(&index_json, &out_dir)
        .map_err(|err| Error::from_reason(format!("failed to write search index: {err}")))?;
    Ok(())
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

/// Normalizes VitePress-specific frontmatter into ox-content's entry-page shape.
#[napi(js_name = "normalizeVitePressFrontmatter")]
pub fn normalize_vitepress_frontmatter(frontmatter: serde_json::Value) -> serde_json::Value {
    ox_content_ssg::normalize_vitepress_frontmatter(frontmatter)
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
        toc: flatten_toc_entries(page_data.toc),
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

fn flatten_toc_entries(entries: Vec<TocEntry>) -> Vec<ox_content_ssg::TocEntry> {
    let mut flat = Vec::new();
    for entry in entries {
        flat.push(ox_content_ssg::TocEntry {
            depth: entry.depth,
            text: entry.text,
            slug: entry.slug,
        });
        flat.extend(flatten_toc_entries(entry.children));
    }
    flat
}

/// Generates a bare SSG HTML page without navigation or styles.
#[napi(js_name = "generateSsgBareHtml")]
pub fn generate_ssg_bare_html(content: String, title: String) -> String {
    ox_content_ssg::generate_bare_html(&content, &title)
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

fn i18n_check_result_from_diagnostics(
    diagnostics: Vec<ox_content_i18n::checker::Diagnostic>,
) -> I18nCheckResult {
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

fn i18n_check_error(message: String) -> I18nCheckResult {
    I18nCheckResult {
        diagnostics: vec![I18nDiagnostic {
            severity: "error".to_string(),
            message,
            key: None,
            locale: None,
        }],
        error_count: 1,
        warning_count: 0,
    }
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
        Err(e) => return i18n_check_error(e.to_string()),
    };

    let keys_set: std::collections::HashSet<String> = used_keys.into_iter().collect();
    let diagnostics = ox_content_i18n::checker::check_all(&keys_set, &dict_set);

    i18n_check_result_from_diagnostics(diagnostics)
}

/// Runs project-level i18n checks by collecting source keys and validating dictionaries.
///
/// `dict_dir` is the path to the i18n directory with locale subdirectories.
/// `src_dirs` are source/content directories to scan recursively.
/// `function_names` are translation call names to collect from JS/TS source.
/// `default_locale` is used for dictionary fallback rules.
#[napi(js_name = "checkI18nProject")]
pub fn check_i18n_project(
    dict_dir: String,
    src_dirs: Vec<String>,
    function_names: Vec<String>,
    default_locale: String,
) -> I18nCheckResult {
    let config = ox_content_i18n_checker::CheckConfig {
        dict_dir,
        src_dirs,
        function_names,
        default_locale: Some(default_locale),
        ..Default::default()
    };

    match ox_content_i18n_checker::check(&config) {
        Ok(result) => i18n_check_result_from_diagnostics(result.diagnostics),
        Err(error) => i18n_check_error(error),
    }
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
    use ox_content_docs::{
        NormalizedDocEntry, NormalizedDocKind, NormalizedMember, NormalizedMemberKind,
        NormalizedReturnDoc, NormalizedTypeParam,
    };
    use serde_json::json;
    use std::collections::BTreeMap;
    use std::fs;
    use std::process::Command;

    use super::transformer::parse_frontmatter;
    use super::{
        convert_markdown_entry, extract_docs_from_entry_points_napi, extract_file_doc_entries,
        generate_docs_markdown, generate_docs_nav_metadata_from_docs_napi, get_git_last_updated,
        map_normalized_doc_entry, JsDocMember, JsDocParam, JsDocReturn, JsDocsMarkdownEntry,
        JsDocsMarkdownModule, JsDocsMarkdownOptions, JsDocsMarkdownTag, JsDocsNavOptions,
        JsEntryPointDocsOptions, JsEntryPointSpec, JsTypeParam,
    };

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
    fn normalized_doc_entry_maps_members_to_js_shape() {
        let entry = NormalizedDocEntry {
            name: "Command".to_string(),
            kind: NormalizedDocKind::Interface,
            description: "Runtime command.".to_string(),
            params: vec![],
            returns: None,
            examples: vec![],
            tags: BTreeMap::new(),
            private: false,
            file: "command.ts".to_string(),
            line: 1,
            end_line: 8,
            signature: Some("export interface Command".to_string()),
            extends: vec![],
            implements: vec![],
            has_body: false,
            members: vec![NormalizedMember {
                name: "name".to_string(),
                kind: NormalizedMemberKind::Property,
                description: "Command name.".to_string(),
                signature: None,
                type_annotation: Some("string".to_string()),
                default_value: Some("\"cli\"".to_string()),
                params: vec![],
                type_parameters: vec![NormalizedTypeParam {
                    name: "T".to_string(),
                    constraint: Some("Base".to_string()),
                    default: Some("Default".to_string()),
                    description: "Value type.".to_string(),
                }],
                returns: None,
                members: vec![NormalizedMember {
                    name: "timeout".to_string(),
                    kind: NormalizedMemberKind::Property,
                    description: "Request timeout.".to_string(),
                    signature: None,
                    type_annotation: Some("number".to_string()),
                    default_value: Some("5000".to_string()),
                    params: vec![],
                    type_parameters: vec![],
                    returns: None,
                    members: vec![],
                    optional: true,
                    readonly: false,
                    r#static: false,
                    private: false,
                    tags: BTreeMap::new(),
                    line: 5,
                    end_line: 5,
                }],
                optional: true,
                readonly: true,
                r#static: false,
                private: false,
                tags: BTreeMap::new(),
                line: 4,
                end_line: 4,
            }],
            type_parameters: vec![],
        };

        let js_entry = map_normalized_doc_entry(entry);
        let member = &js_entry.members.as_ref().unwrap()[0];

        assert_eq!(member.name, "name");
        assert_eq!(member.kind, "property");
        assert_eq!(member.r#type.as_deref(), Some("string"));
        assert_eq!(member.r#default.as_deref(), Some("\"cli\""));
        let type_param = &member.type_parameters.as_ref().unwrap()[0];
        assert_eq!(type_param.name, "T");
        assert_eq!(type_param.constraint.as_deref(), Some("Base"));
        assert_eq!(type_param.r#default.as_deref(), Some("Default"));
        assert_eq!(type_param.description, "Value type.");
        assert_eq!(member.optional, Some(true));
        assert_eq!(member.readonly, Some(true));
        let nested_member = &member.members.as_ref().unwrap()[0];
        assert_eq!(nested_member.name, "timeout");
        assert_eq!(nested_member.kind, "property");
        assert_eq!(nested_member.r#type.as_deref(), Some("number"));
        assert_eq!(nested_member.r#default.as_deref(), Some("5000"));
        assert_eq!(nested_member.description, "Request timeout.");
        assert_eq!(nested_member.optional, Some(true));
    }

    #[test]
    fn normalized_doc_entry_maps_index_signature_members_to_js_shape() {
        let entry = NormalizedDocEntry {
            name: "Args".to_string(),
            kind: NormalizedDocKind::Interface,
            description: "Arguments.".to_string(),
            params: vec![],
            returns: None,
            examples: vec![],
            tags: BTreeMap::new(),
            private: false,
            file: "args.ts".to_string(),
            line: 1,
            end_line: 5,
            signature: Some("export interface Args".to_string()),
            extends: vec![],
            implements: vec![],
            has_body: false,
            members: vec![NormalizedMember {
                name: "[option: string]".to_string(),
                kind: NormalizedMemberKind::IndexSignature,
                description: "Argument schema by option name.".to_string(),
                signature: Some("readonly [option: string]: ArgSchema".to_string()),
                type_annotation: Some("ArgSchema".to_string()),
                default_value: None,
                params: vec![ox_content_docs::NormalizedParamDoc {
                    name: "option".to_string(),
                    type_annotation: "string".to_string(),
                    description: String::new(),
                    optional: false,
                    default_value: None,
                }],
                type_parameters: vec![],
                returns: None,
                members: vec![],
                optional: false,
                readonly: true,
                r#static: false,
                private: false,
                tags: BTreeMap::new(),
                line: 4,
                end_line: 4,
            }],
            type_parameters: vec![],
        };

        let js_entry = map_normalized_doc_entry(entry);
        let member = &js_entry.members.as_ref().unwrap()[0];

        assert_eq!(member.name, "[option: string]");
        assert_eq!(member.kind, "indexSignature");
        assert_eq!(member.signature.as_deref(), Some("readonly [option: string]: ArgSchema"));
        assert_eq!(member.r#type.as_deref(), Some("ArgSchema"));
        assert_eq!(member.params.as_ref().unwrap()[0].name, "option");
        assert_eq!(member.params.as_ref().unwrap()[0].r#type, "string");
        assert_eq!(member.readonly, Some(true));
    }

    #[test]
    fn normalized_doc_entry_maps_heritage_to_js_shape() {
        let entry = NormalizedDocEntry {
            name: "DefaultTranslation".to_string(),
            kind: NormalizedDocKind::Class,
            description: "Default adapter.".to_string(),
            params: vec![],
            returns: None,
            examples: vec![],
            tags: BTreeMap::new(),
            private: false,
            file: "adapter.ts".to_string(),
            line: 1,
            end_line: 10,
            signature: Some("class DefaultTranslation implements TranslationAdapter".to_string()),
            extends: vec!["BaseTranslation".to_string()],
            implements: vec!["TranslationAdapter".to_string()],
            has_body: false,
            members: vec![],
            type_parameters: vec![],
        };

        let js_entry = map_normalized_doc_entry(entry);

        assert_eq!(js_entry.extends, Some(vec!["BaseTranslation".to_string()]));
        assert_eq!(js_entry.implements, Some(vec!["TranslationAdapter".to_string()]));
    }

    #[test]
    fn normalized_doc_entry_maps_return_members_to_js_shape() {
        let entry = NormalizedDocEntry {
            name: "resolveArgs".to_string(),
            kind: NormalizedDocKind::Function,
            description: "Resolve.".to_string(),
            params: vec![],
            returns: Some(NormalizedReturnDoc {
                type_annotation: "object".to_string(),
                description: "Resolved args.".to_string(),
                members: vec![NormalizedMember {
                    name: "values".to_string(),
                    kind: NormalizedMemberKind::Property,
                    description: String::new(),
                    signature: None,
                    type_annotation: Some("ArgValues<A>".to_string()),
                    default_value: None,
                    params: vec![],
                    type_parameters: vec![],
                    returns: None,
                    members: vec![],
                    optional: false,
                    readonly: false,
                    r#static: false,
                    private: false,
                    tags: BTreeMap::new(),
                    line: 3,
                    end_line: 3,
                }],
            }),
            examples: vec![],
            tags: BTreeMap::new(),
            private: false,
            file: "resolver.ts".to_string(),
            line: 1,
            end_line: 8,
            signature: Some("export function resolveArgs(): object".to_string()),
            extends: vec![],
            implements: vec![],
            has_body: false,
            members: vec![],
            type_parameters: vec![],
        };

        let js_entry = map_normalized_doc_entry(entry);
        let returns = js_entry.returns.as_ref().unwrap();
        let member = &returns.members.as_ref().unwrap()[0];

        assert_eq!(returns.r#type, "object");
        assert_eq!(member.name, "values");
        assert_eq!(member.r#type.as_deref(), Some("ArgValues<A>"));
    }

    #[test]
    fn extract_file_doc_entries_preserves_type_alias_return_without_returns_tag() {
        let unique =
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let root = std::env::temp_dir()
            .join(format!("ox-content-napi-type-alias-return-{}-{unique}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let file = root.join("plugin.ts");
        fs::write(
            &file,
            r"
/**
 * Plugin extension hook.
 *
 * @param ctx - The command context.
 * @param cmd - The command.
 */
export type OnPluginExtension<G> = (
    ctx: Readonly<CommandContext<G>>,
    cmd: Readonly<Command<G>>
) => Awaitable<void>;
",
        )
        .unwrap();

        let entries =
            extract_file_doc_entries(file.to_string_lossy().into_owned(), None, None, None)
                .unwrap();
        let entry = entries.iter().find(|entry| entry.name == "OnPluginExtension").unwrap();
        let params = entry.params.as_ref().unwrap();
        let returns = entry.returns.as_ref().unwrap();

        assert_eq!(params.len(), 2);
        assert_eq!(params[0].r#type, "Readonly<CommandContext<G>>");
        assert_eq!(params[0].description, "The command context.");
        assert_eq!(params[1].r#type, "Readonly<Command<G>>");
        assert_eq!(params[1].description, "The command.");
        assert_eq!(returns.r#type, "Awaitable<void>");
        assert_eq!(returns.description, "");

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn extract_file_doc_entries_preserves_object_literal_parameter_members() {
        let unique =
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let root = std::env::temp_dir()
            .join(format!("ox-content-napi-object-literal-param-{}-{unique}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let file = root.join("plugin.ts");
        fs::write(
            &file,
            r"
/**
 * Define a plugin.
 *
 * @param options - Plugin options.
 * @param options.id - Plugin id.
 * @param options.name - Plugin display name.
 */
export function plugin<Id, PluginExt>(options: {
    id: Id;
    name?: string;
    setup?: (
        ctx: Readonly<
            PluginContext
        >
    ) => Awaitable<void>;
    extension: PluginExt;
}): PluginWithExtension<PluginExt>;
",
        )
        .unwrap();

        let entries =
            extract_file_doc_entries(file.to_string_lossy().into_owned(), None, None, None)
                .unwrap();
        let entry = entries.iter().find(|entry| entry.name == "plugin").unwrap();
        let params = entry.params.as_ref().unwrap();

        assert_eq!(params.len(), 5);
        assert_eq!(params[0].name, "options");
        assert_ne!(params[0].r#type, "{ ... }");
        assert!(params[0].r#type.contains("id: Id"));
        assert!(params[0].r#type.contains("name?: string"));
        assert!(params[0]
            .r#type
            .contains("setup?: (ctx: Readonly<PluginContext>) => Awaitable<void>"));
        assert_eq!(params[0].description, "Plugin options.");
        assert_eq!(params[1].name, "options.id");
        assert_eq!(params[1].r#type, "Id");
        assert_eq!(params[1].description, "Plugin id.");
        assert_eq!(params[2].name, "options.name?");
        assert_eq!(params[2].description, "Plugin display name.");
        assert_eq!(params[2].optional, Some(true));
        assert_eq!(params[3].name, "options.setup?");
        assert_eq!(params[3].r#type, "(ctx: Readonly<PluginContext>) => Awaitable<void>");
        assert_eq!(params[4].name, "options.extension");
        assert_eq!(params[4].r#type, "PluginExt");

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn has_body_round_trips_from_extract_output_to_markdown_model() {
        let normalized = NormalizedDocEntry {
            name: "plugin".to_string(),
            kind: NormalizedDocKind::Function,
            description: String::new(),
            params: vec![],
            returns: None,
            examples: vec![],
            tags: BTreeMap::new(),
            private: false,
            file: "plugin.ts".to_string(),
            line: 1,
            end_line: 1,
            signature: Some("export function plugin(): void".to_string()),
            extends: vec![],
            implements: vec![],
            has_body: true,
            members: vec![],
            type_parameters: vec![],
        };

        // `extractDocsFromEntryPoints` output exposes the flag ...
        let js_entry = map_normalized_doc_entry(normalized);
        assert!(js_entry.has_body);

        // ... and `generateDocsMarkdown` input round-trips it back into the model
        // (gunshi's `{ ...entry }` spread carries it across the boundary).
        let markdown_entry = JsDocsMarkdownEntry {
            name: js_entry.name,
            kind: js_entry.kind,
            description: js_entry.description,
            params: None,
            returns: None,
            examples: None,
            tags: None,
            private: js_entry.private,
            file: js_entry.file,
            line: js_entry.line,
            end_line: js_entry.end_line,
            signature: js_entry.signature,
            extends: None,
            implements: None,
            has_body: Some(js_entry.has_body),
            members: None,
            type_parameters: None,
        };
        assert!(convert_markdown_entry(markdown_entry).has_body);
    }

    #[test]
    fn convert_markdown_entry_defaults_has_body_to_false_when_absent() {
        let entry = JsDocsMarkdownEntry {
            name: "Command".to_string(),
            kind: "interface".to_string(),
            description: String::new(),
            params: None,
            returns: None,
            examples: None,
            tags: None,
            private: false,
            file: "command.ts".to_string(),
            line: 1,
            end_line: 1,
            signature: Some("export interface Command".to_string()),
            extends: None,
            implements: None,
            has_body: None,
            members: None,
            type_parameters: None,
        };

        assert!(!convert_markdown_entry(entry).has_body);
    }

    #[test]
    fn convert_markdown_entry_preserves_heritage_and_implementation_metadata() {
        let entry = JsDocsMarkdownEntry {
            name: "DefaultTranslation".to_string(),
            kind: "class".to_string(),
            description: "Default adapter.".to_string(),
            params: None,
            returns: None,
            examples: None,
            tags: None,
            private: false,
            file: "adapter.ts".to_string(),
            line: 1,
            end_line: 10,
            signature: Some("class DefaultTranslation implements TranslationAdapter".to_string()),
            extends: Some(vec!["BaseTranslation".to_string()]),
            implements: Some(vec!["TranslationAdapter".to_string()]),
            has_body: None,
            members: Some(vec![JsDocMember {
                name: "getResource".to_string(),
                kind: "method".to_string(),
                description: "Gets a locale resource.".to_string(),
                signature: Some(
                    "getResource(locale: string): Record<string, string> | undefined".to_string(),
                ),
                r#type: None,
                r#default: Some("undefined".to_string()),
                params: None,
                type_parameters: Some(vec![JsTypeParam {
                    name: "L".to_string(),
                    constraint: Some("Base".to_string()),
                    r#default: Some("Default".to_string()),
                    description: "Locale type.".to_string(),
                }]),
                returns: None,
                members: None,
                optional: None,
                readonly: None,
                r#static: None,
                private: None,
                tags: None,
                implementation_of: Some(vec!["TranslationAdapter.getResource".to_string()]),
                line: 5,
                end_line: 8,
            }]),
            type_parameters: None,
        };

        let converted = convert_markdown_entry(entry);

        assert_eq!(converted.extends, vec!["BaseTranslation"]);
        assert_eq!(converted.implements, vec!["TranslationAdapter"]);
        assert_eq!(converted.members[0].implementation_of, vec!["TranslationAdapter.getResource"]);
        assert_eq!(converted.members[0].default_value.as_deref(), Some("undefined"));
        assert_eq!(converted.members[0].type_parameters[0].name, "L");
        assert_eq!(converted.members[0].type_parameters[0].constraint.as_deref(), Some("Base"));
    }

    #[test]
    fn generate_docs_markdown_accepts_clean_link_options() {
        let docs = vec![JsDocsMarkdownModule {
            description: None,
            file: "/repo/src/context.ts".to_string(),
            source_path: None,
            examples: None,
            tags: None,
            entries: vec![JsDocsMarkdownEntry {
                name: "CommandContext".to_string(),
                kind: "interface".to_string(),
                description: "Runtime context.".to_string(),
                params: None,
                returns: None,
                examples: None,
                tags: None,
                private: false,
                file: "/repo/src/context.ts".to_string(),
                line: 1,
                end_line: 1,
                signature: Some("export interface CommandContext".to_string()),
                extends: None,
                implements: None,
                has_body: None,
                members: None,
                type_parameters: None,
            }],
        }];
        let markdown = generate_docs_markdown(
            docs,
            Some(JsDocsMarkdownOptions {
                group_by: Some("file".to_string()),
                github_url: None,
                link_style: Some("clean".to_string()),
                base_path: Some("/api-ox".to_string()),
                path_strategy: None,
                render_style: None,
                ..Default::default()
            }),
        );
        let index = markdown.get("index.md").unwrap();

        assert!(index.contains("href=\"/api-ox/context\""));
        assert!(index.contains("href=\"/api-ox/context#commandcontext\""));
    }

    #[test]
    fn generate_docs_markdown_render_style_markdown_omits_html() {
        let docs = vec![JsDocsMarkdownModule {
            description: None,
            file: "/repo/src/context.ts".to_string(),
            source_path: None,
            examples: None,
            tags: None,
            entries: vec![JsDocsMarkdownEntry {
                name: "CommandContext".to_string(),
                kind: "interface".to_string(),
                description: "Runtime context.".to_string(),
                params: None,
                returns: None,
                examples: None,
                tags: None,
                private: false,
                file: "/repo/src/context.ts".to_string(),
                line: 1,
                end_line: 1,
                signature: Some("export interface CommandContext".to_string()),
                extends: None,
                implements: None,
                has_body: None,
                members: None,
                type_parameters: None,
            }],
        }];
        let markdown = generate_docs_markdown(
            docs,
            Some(JsDocsMarkdownOptions {
                group_by: Some("file".to_string()),
                github_url: None,
                link_style: None,
                base_path: None,
                path_strategy: None,
                render_style: Some("markdown".to_string()),
                ..Default::default()
            }),
        );
        let page = markdown.get("context.md").unwrap();

        assert!(!page.contains("<details"));
        assert!(!page.contains("class=\"ox-api"));
        assert!(page.contains("### CommandContext"));
        assert!(page.contains("```ts"));
    }

    #[test]
    fn generate_docs_markdown_accepts_display_format_options() {
        let docs = vec![JsDocsMarkdownModule {
            description: None,
            file: "default".to_string(),
            source_path: None,
            examples: None,
            tags: None,
            entries: vec![JsDocsMarkdownEntry {
                name: "make".to_string(),
                kind: "function".to_string(),
                description: "Make a thing.".to_string(),
                params: Some(vec![JsDocParam {
                    name: "value".to_string(),
                    r#type: "string".to_string(),
                    description: "Input value.".to_string(),
                    optional: Some(false),
                    r#default: None,
                }]),
                returns: None,
                examples: None,
                tags: None,
                private: false,
                file: "/repo/src/make.ts".to_string(),
                line: 1,
                end_line: 1,
                signature: Some("export function make(value: string): void".to_string()),
                extends: None,
                implements: None,
                has_body: None,
                members: None,
                type_parameters: None,
            }],
        }];

        let markdown = generate_docs_markdown(
            docs,
            Some(JsDocsMarkdownOptions {
                group_by: Some("file".to_string()),
                render_style: Some("markdown".to_string()),
                parameters_format: Some("table".to_string()),
                ..Default::default()
            }),
        );
        let page = markdown.get("default.md").unwrap();

        assert!(page.contains("| Name | Type | Description |"));
        assert!(page.contains("| `value` | `string` | Input value. |"));
    }

    #[test]
    fn generate_docs_markdown_type_declaration_format_table_renders_html() {
        let docs = vec![JsDocsMarkdownModule {
            description: None,
            file: "default".to_string(),
            source_path: None,
            examples: None,
            tags: None,
            entries: vec![
                JsDocsMarkdownEntry {
                    name: "resolveArgs".to_string(),
                    kind: "function".to_string(),
                    description: "Resolve.".to_string(),
                    params: None,
                    returns: Some(JsDocReturn {
                        r#type: "object".to_string(),
                        description: "Resolved args.".to_string(),
                        members: Some(vec![JsDocMember {
                            name: "values".to_string(),
                            kind: "property".to_string(),
                            description: "Resolved values.".to_string(),
                            signature: None,
                            r#type: Some("ArgValues<A>".to_string()),
                            r#default: None,
                            params: None,
                            type_parameters: None,
                            returns: None,
                            members: None,
                            optional: Some(false),
                            readonly: Some(false),
                            r#static: Some(false),
                            private: Some(false),
                            tags: None,
                            implementation_of: None,
                            line: 1,
                            end_line: 1,
                        }]),
                    }),
                    examples: None,
                    tags: None,
                    private: false,
                    file: "/repo/src/resolver.ts".to_string(),
                    line: 1,
                    end_line: 1,
                    signature: Some("export function resolveArgs(): object".to_string()),
                    extends: None,
                    implements: None,
                    has_body: None,
                    members: None,
                    type_parameters: None,
                },
                JsDocsMarkdownEntry {
                    name: "ArgValues".to_string(),
                    kind: "type".to_string(),
                    description: String::new(),
                    params: None,
                    returns: None,
                    examples: None,
                    tags: None,
                    private: false,
                    file: "/repo/src/types.ts".to_string(),
                    line: 1,
                    end_line: 1,
                    signature: Some("export type ArgValues = unknown".to_string()),
                    extends: None,
                    implements: None,
                    has_body: None,
                    members: None,
                    type_parameters: None,
                },
            ],
        }];

        let markdown = generate_docs_markdown(
            docs,
            Some(JsDocsMarkdownOptions {
                group_by: Some("file".to_string()),
                path_strategy: Some("typedoc".to_string()),
                render_style: Some("html".to_string()),
                type_declaration_format: Some("table".to_string()),
                ..Default::default()
            }),
        );
        let page = markdown.get("default/functions/resolveArgs.md").unwrap();

        assert!(page.contains("ox-api-entry__type-declaration-table"));
        assert!(page.contains("<td><code>values</code></td>"));
        assert!(page.contains("Resolved values."));
        assert!(!page.contains("ox-api-entry__return-members"));
    }

    #[test]
    fn generate_docs_markdown_property_members_format_table_renders_html() {
        let docs = vec![JsDocsMarkdownModule {
            description: None,
            file: "default".to_string(),
            source_path: None,
            examples: None,
            tags: None,
            entries: vec![JsDocsMarkdownEntry {
                name: "Options".to_string(),
                kind: "interface".to_string(),
                description: "Request options.".to_string(),
                params: None,
                returns: None,
                examples: None,
                tags: None,
                private: false,
                file: "/repo/src/options.ts".to_string(),
                line: 1,
                end_line: 8,
                signature: Some("export interface Options".to_string()),
                extends: None,
                implements: None,
                has_body: None,
                members: Some(vec![JsDocMember {
                    name: "http".to_string(),
                    kind: "property".to_string(),
                    description: "HTTP options.".to_string(),
                    signature: None,
                    r#type: Some("{ timeout?: number }".to_string()),
                    r#default: None,
                    params: None,
                    type_parameters: None,
                    returns: None,
                    members: Some(vec![JsDocMember {
                        name: "timeout".to_string(),
                        kind: "property".to_string(),
                        description: "Request timeout.".to_string(),
                        signature: None,
                        r#type: Some("number".to_string()),
                        r#default: None,
                        params: None,
                        type_parameters: None,
                        returns: None,
                        members: None,
                        optional: Some(true),
                        readonly: Some(false),
                        r#static: Some(false),
                        private: Some(false),
                        tags: None,
                        implementation_of: None,
                        line: 4,
                        end_line: 4,
                    }]),
                    optional: Some(false),
                    readonly: Some(false),
                    r#static: Some(false),
                    private: Some(false),
                    tags: None,
                    implementation_of: None,
                    line: 3,
                    end_line: 6,
                }]),
                type_parameters: None,
            }],
        }];

        let markdown = generate_docs_markdown(
            docs,
            Some(JsDocsMarkdownOptions {
                group_by: Some("file".to_string()),
                path_strategy: Some("typedoc".to_string()),
                render_style: Some("html".to_string()),
                interface_properties_format: Some("table".to_string()),
                property_members_format: Some("table".to_string()),
                ..Default::default()
            }),
        );
        let page = markdown.get("default/interfaces/Options.md").unwrap();

        assert!(page.contains("ox-api-entry__property-members-table"));
        assert!(page
            .contains("<td><code>timeout</code><span class=\"ox-api-badge\">optional</span></td>"));
        assert!(page.contains("Request timeout."));
    }

    #[test]
    fn generate_docs_markdown_resolves_jsdoc_inline_links() {
        let docs = vec![
            JsDocsMarkdownModule {
                description: None,
                file: "/repo/src/command.ts".to_string(),
                source_path: None,
                examples: None,
                tags: None,
                entries: vec![JsDocsMarkdownEntry {
                    name: "Command".to_string(),
                    kind: "interface".to_string(),
                    description: "Runtime command.".to_string(),
                    params: None,
                    returns: None,
                    examples: None,
                    tags: None,
                    private: false,
                    file: "/repo/src/command.ts".to_string(),
                    line: 1,
                    end_line: 10,
                    signature: Some("export interface Command".to_string()),
                    extends: None,
                    implements: None,
                    has_body: None,
                    members: Some(vec![JsDocMember {
                        name: "args".to_string(),
                        kind: "property".to_string(),
                        description: "All {@linkcode Command.args} names.".to_string(),
                        signature: None,
                        r#type: Some("Record<string, unknown>".to_string()),
                        r#default: None,
                        params: None,
                        type_parameters: None,
                        returns: None,
                        members: None,
                        optional: Some(false),
                        readonly: Some(false),
                        r#static: Some(false),
                        private: Some(false),
                        tags: None,
                        implementation_of: None,
                        line: 5,
                        end_line: 5,
                    }]),
                    type_parameters: None,
                }],
            },
            JsDocsMarkdownModule {
                description: None,
                file: "/repo/src/build.ts".to_string(),
                source_path: None,
                examples: None,
                tags: None,
                entries: vec![JsDocsMarkdownEntry {
                    name: "buildCommand".to_string(),
                    kind: "function".to_string(),
                    description: "Builds {@linkcode Command | command} metadata.".to_string(),
                    params: Some(vec![JsDocParam {
                        name: "entry".to_string(),
                        r#type: "Command".to_string(),
                        description: "A {@linkcode Command | entry command}".to_string(),
                        optional: Some(false),
                        r#default: None,
                    }]),
                    returns: Some(JsDocReturn {
                        r#type: "Command".to_string(),
                        description: "A {@link Command} result.".to_string(),
                        members: None,
                    }),
                    examples: None,
                    tags: Some(vec![JsDocsMarkdownTag {
                        tag: "see".to_string(),
                        value: "{@link https://github.com/unjs/std-env | std-env}".to_string(),
                    }]),
                    private: false,
                    file: "/repo/src/build.ts".to_string(),
                    line: 1,
                    end_line: 20,
                    signature: Some(
                        "export function buildCommand(entry: Command): Command".to_string(),
                    ),
                    extends: None,
                    implements: None,
                    has_body: None,
                    members: None,
                    type_parameters: None,
                }],
            },
        ];

        let markdown = generate_docs_markdown(docs, None);
        let build_page = markdown.get("build.md").unwrap();
        let command_page = markdown.get("command.md").unwrap();

        assert!(!build_page.contains("{@link"));
        assert!(!command_page.contains("{@link"));
        assert!(
            build_page.contains("<a href=\"./command.md#command\"><code>entry command</code></a>")
        );
        assert!(build_page.contains("<a href=\"./command.md#command\">Command</a>"));
        assert!(build_page.contains("<a href=\"https://github.com/unjs/std-env\">std-env</a>"));
        assert!(command_page.contains("<tr id=\"command-args\">"));
        assert!(command_page.contains("<a href=\"#command-args\"><code>Command.args</code></a>"));
    }

    #[test]
    fn generate_docs_markdown_accepts_typedoc_path_strategy() {
        let docs = vec![JsDocsMarkdownModule {
            description: None,
            file: "default".to_string(),
            source_path: None,
            examples: None,
            tags: None,
            entries: vec![
                JsDocsMarkdownEntry {
                    name: "Command".to_string(),
                    kind: "interface".to_string(),
                    description: "Runtime command.".to_string(),
                    params: None,
                    returns: None,
                    examples: None,
                    tags: None,
                    private: false,
                    file: "/repo/src/types.ts".to_string(),
                    line: 1,
                    end_line: 10,
                    signature: Some("export interface Command".to_string()),
                    extends: None,
                    implements: None,
                    has_body: None,
                    members: None,
                    type_parameters: None,
                },
                JsDocsMarkdownEntry {
                    name: "cli".to_string(),
                    kind: "function".to_string(),
                    description: "Runs {@link Command}.".to_string(),
                    params: None,
                    returns: None,
                    examples: None,
                    tags: None,
                    private: false,
                    file: "/repo/src/cli.ts".to_string(),
                    line: 1,
                    end_line: 10,
                    signature: Some("export function cli(): void".to_string()),
                    extends: None,
                    implements: None,
                    has_body: None,
                    members: None,
                    type_parameters: None,
                },
            ],
        }];

        let markdown = generate_docs_markdown(
            docs,
            Some(JsDocsMarkdownOptions {
                group_by: Some("file".to_string()),
                github_url: None,
                link_style: Some("clean".to_string()),
                base_path: Some("/api".to_string()),
                path_strategy: Some("typedoc".to_string()),
                render_style: None,
                ..Default::default()
            }),
        );
        let cli_page = markdown.get("default/functions/cli.md").unwrap();
        let root_index = markdown.get("index.md").unwrap();
        let module_index = markdown.get("default/index.md").unwrap();

        assert!(markdown.contains_key("default/index.md"));
        assert!(markdown.contains_key("default/interfaces/Command.md"));
        assert!(root_index.contains("[default](/api/default)"));
        assert!(!root_index.contains("[Default]"));
        assert!(module_index.starts_with("# default\n\n"));
        assert!(cli_page.contains("<a href=\"/api/default/interfaces/Command\">Command</a>"));
    }

    #[test]
    fn generate_docs_markdown_group_order_reorders_module_index() {
        fn entry(name: &str, kind: &str) -> JsDocsMarkdownEntry {
            JsDocsMarkdownEntry {
                name: name.to_string(),
                kind: kind.to_string(),
                description: String::new(),
                params: None,
                returns: None,
                examples: None,
                tags: None,
                private: false,
                file: format!("/repo/src/{name}.ts"),
                line: 1,
                end_line: 1,
                signature: Some(format!("export declare const {name}: unknown")),
                extends: None,
                implements: None,
                has_body: None,
                members: None,
                type_parameters: None,
            }
        }
        let docs = vec![JsDocsMarkdownModule {
            description: None,
            file: "default".to_string(),
            source_path: None,
            examples: None,
            tags: None,
            entries: vec![entry("alpha", "function"), entry("VERSION", "variable")],
        }];

        let markdown = generate_docs_markdown(
            docs,
            Some(JsDocsMarkdownOptions {
                path_strategy: Some("typedoc".to_string()),
                render_style: Some("markdown".to_string()),
                group_order: Some(vec!["Variables".to_string(), "Functions".to_string()]),
                ..Default::default()
            }),
        );
        let index = markdown.get("default/index.md").unwrap();

        assert!(index.find("## Variables").unwrap() < index.find("## Functions").unwrap());
    }

    fn docs_markdown_module() -> Vec<JsDocsMarkdownModule> {
        vec![JsDocsMarkdownModule {
            description: None,
            file: "default".to_string(),
            source_path: None,
            examples: None,
            tags: None,
            entries: vec![JsDocsMarkdownEntry {
                name: "cli".to_string(),
                kind: "function".to_string(),
                description: "Run.".to_string(),
                params: None,
                returns: None,
                examples: None,
                tags: None,
                private: false,
                file: "/repo/src/cli.ts".to_string(),
                line: 1,
                end_line: 1,
                signature: Some("export function cli(): void".to_string()),
                extends: None,
                implements: None,
                has_body: None,
                members: None,
                type_parameters: None,
            }],
        }]
    }

    #[test]
    fn generate_docs_markdown_render_stats_option_toggles_stats_summary() {
        fn options(render_stats: Option<bool>) -> JsDocsMarkdownOptions {
            JsDocsMarkdownOptions {
                path_strategy: Some("typedoc".to_string()),
                render_style: Some("markdown".to_string()),
                render_stats,
                ..Default::default()
            }
        }

        // Default (None -> true) keeps the stats summary.
        let with_stats = generate_docs_markdown(docs_markdown_module(), Some(options(None)));
        assert!(with_stats.get("default/index.md").unwrap().contains("symbols ·"));

        // Explicit false omits it.
        let without_stats =
            generate_docs_markdown(docs_markdown_module(), Some(options(Some(false))));
        assert!(!without_stats.get("default/index.md").unwrap().contains("symbols ·"));
    }

    #[test]
    fn generate_docs_markdown_render_generated_by_option_toggles_attribution() {
        fn options(render_generated_by: Option<bool>) -> JsDocsMarkdownOptions {
            JsDocsMarkdownOptions {
                path_strategy: Some("typedoc".to_string()),
                render_style: Some("markdown".to_string()),
                render_stats: Some(false),
                render_generated_by,
                ..Default::default()
            }
        }

        // Default (None -> true) keeps the generated-by attribution.
        let with_generated_by = generate_docs_markdown(docs_markdown_module(), Some(options(None)));
        assert!(with_generated_by.get("index.md").unwrap().contains("Generated by [Ox Content]"));

        // Explicit false omits it and leaves the Modules heading directly after
        // the H1 when stats are also disabled.
        let without_generated_by =
            generate_docs_markdown(docs_markdown_module(), Some(options(Some(false))));
        let root = without_generated_by.get("index.md").unwrap();
        assert!(!root.contains("Generated by [Ox Content]"));
        assert!(root.starts_with("# API Documentation\n\n## Modules\n\n"));
    }

    #[test]
    fn generate_docs_markdown_dedupes_cross_entrypoint_reexports() {
        // The same symbol re-exported from two entry points should yield a single
        // canonical page placed under its defining module via `sourcePath`.
        let entry = |name: &str| JsDocsMarkdownEntry {
            name: name.to_string(),
            kind: "function".to_string(),
            description: "Creates a command context.".to_string(),
            params: None,
            returns: None,
            examples: None,
            tags: None,
            private: false,
            file: "/repo/src/context.ts".to_string(),
            line: 1,
            end_line: 1,
            signature: Some("export function createCommandContext(): void".to_string()),
            extends: None,
            implements: None,
            has_body: None,
            members: None,
            type_parameters: None,
        };
        let docs = vec![
            JsDocsMarkdownModule {
                description: None,
                file: "context".to_string(),
                source_path: Some("/repo/src/context.ts".to_string()),
                examples: None,
                tags: None,
                entries: vec![entry("createCommandContext")],
            },
            JsDocsMarkdownModule {
                description: None,
                file: "default".to_string(),
                source_path: Some("/repo/src/index.ts".to_string()),
                examples: None,
                tags: None,
                entries: vec![entry("createCommandContext")],
            },
        ];

        let markdown = generate_docs_markdown(
            docs,
            Some(JsDocsMarkdownOptions {
                group_by: Some("file".to_string()),
                github_url: None,
                link_style: Some("markdown".to_string()),
                base_path: None,
                path_strategy: Some("typedoc".to_string()),
                render_style: None,
                ..Default::default()
            }),
        );

        assert!(markdown.contains_key("context/functions/createCommandContext.md"));
        assert!(!markdown.contains_key("default/functions/createCommandContext.md"));
        assert!(markdown.get("default/index.md").unwrap().contains("Re-exports"));
    }

    #[test]
    fn generate_docs_markdown_renders_type_parameters() {
        let docs = vec![JsDocsMarkdownModule {
            description: None,
            file: "default".to_string(),
            source_path: None,
            examples: None,
            tags: None,
            entries: vec![JsDocsMarkdownEntry {
                name: "make".to_string(),
                kind: "function".to_string(),
                description: "Make a thing.".to_string(),
                params: None,
                returns: None,
                examples: None,
                tags: None,
                private: false,
                file: "/repo/src/make.ts".to_string(),
                line: 1,
                end_line: 1,
                signature: Some("export function make<G>(): G".to_string()),
                extends: None,
                implements: None,
                has_body: None,
                members: None,
                type_parameters: Some(vec![JsTypeParam {
                    name: "G".to_string(),
                    constraint: Some("Base".to_string()),
                    r#default: Some("Default".to_string()),
                    description: "The thing type.".to_string(),
                }]),
            }],
        }];

        let markdown = generate_docs_markdown(
            docs,
            Some(JsDocsMarkdownOptions {
                group_by: Some("file".to_string()),
                github_url: None,
                link_style: Some("markdown".to_string()),
                base_path: None,
                path_strategy: Some("typedoc".to_string()),
                render_style: Some("markdown".to_string()),
                ..Default::default()
            }),
        );
        let page = markdown.get("default/functions/make.md").unwrap();

        assert!(page.contains("## Type Parameters"));
        assert!(!page.contains("**Type Parameters**"));
        assert!(page.contains("`G` *extends* `Base` = `Default`"));
        assert!(page.contains("The thing type."));
    }

    #[test]
    fn generate_docs_markdown_collapses_multiline_linked_type_parameter_defaults() {
        fn entry(name: &str, kind: &str, signature: &str) -> JsDocsMarkdownEntry {
            JsDocsMarkdownEntry {
                name: name.to_string(),
                kind: kind.to_string(),
                description: String::new(),
                params: None,
                returns: None,
                examples: None,
                tags: None,
                private: false,
                file: format!("/repo/src/{name}.ts"),
                line: 1,
                end_line: 1,
                signature: Some(signature.to_string()),
                extends: None,
                implements: None,
                has_body: None,
                members: None,
                type_parameters: None,
            }
        }

        let mut plugin = entry("plugin", "function", "export function plugin(): void");
        plugin.type_parameters = Some(vec![
            JsTypeParam {
                name: "Extension".to_string(),
                constraint: None,
                r#default: None,
                description: String::new(),
            },
            JsTypeParam {
                name: "ResolvedDepExtensions".to_string(),
                constraint: None,
                r#default: None,
                description: String::new(),
            },
            JsTypeParam {
                name: "PluginExt".to_string(),
                constraint: Some("PluginExtension<Extension, DefaultGunshiParams>".to_string()),
                r#default: Some(
                    "PluginExtension<\n    Extension,\n    ResolvedDepExtensions\n  >".to_string(),
                ),
                description: String::new(),
            },
        ]);

        let docs = vec![JsDocsMarkdownModule {
            description: None,
            file: "default".to_string(),
            source_path: None,
            examples: None,
            tags: None,
            entries: vec![
                plugin,
                entry("PluginExtension", "type", "export type PluginExtension = unknown"),
                entry("DefaultGunshiParams", "type", "export type DefaultGunshiParams = unknown"),
            ],
        }];

        let markdown = generate_docs_markdown(
            docs,
            Some(JsDocsMarkdownOptions {
                group_by: Some("file".to_string()),
                link_style: Some("markdown".to_string()),
                path_strategy: Some("typedoc".to_string()),
                render_style: Some("markdown".to_string()),
                parameters_format: Some("table".to_string()),
                ..Default::default()
            }),
        );
        let page = markdown.get("default/functions/plugin.md").unwrap();

        assert!(page.contains("| Name |\n| --- |"));
        assert!(!page.contains("| Name | Description |"));
        assert!(page.contains("| `PluginExt` *extends* [`PluginExtension`](../type-aliases/PluginExtension.md)\\<`Extension`, [`DefaultGunshiParams`](../type-aliases/DefaultGunshiParams.md)\\> = [`PluginExtension`](../type-aliases/PluginExtension.md)\\<`Extension`, `ResolvedDepExtensions`\\> |"));
        assert!(!page.contains("\\<\n"));
        assert!(!page.contains("ResolvedDepExtensions`\n"));
    }

    #[test]
    fn generate_docs_markdown_renders_return_members() {
        let docs = vec![JsDocsMarkdownModule {
            description: None,
            file: "default".to_string(),
            source_path: None,
            examples: None,
            tags: None,
            entries: vec![
                JsDocsMarkdownEntry {
                    name: "resolveArgs".to_string(),
                    kind: "function".to_string(),
                    description: "Resolve.".to_string(),
                    params: None,
                    returns: Some(JsDocReturn {
                        r#type: "object".to_string(),
                        description: "Resolved args.".to_string(),
                        members: Some(vec![JsDocMember {
                            name: "values".to_string(),
                            kind: "property".to_string(),
                            description: String::new(),
                            signature: None,
                            r#type: Some("ArgValues<A>".to_string()),
                            r#default: None,
                            params: None,
                            type_parameters: None,
                            returns: None,
                            members: None,
                            optional: Some(false),
                            readonly: Some(false),
                            r#static: Some(false),
                            private: Some(false),
                            tags: None,
                            implementation_of: None,
                            line: 1,
                            end_line: 1,
                        }]),
                    }),
                    examples: None,
                    tags: None,
                    private: false,
                    file: "/repo/src/resolver.ts".to_string(),
                    line: 1,
                    end_line: 1,
                    signature: Some("export function resolveArgs(): object".to_string()),
                    extends: None,
                    implements: None,
                    has_body: None,
                    members: None,
                    type_parameters: None,
                },
                JsDocsMarkdownEntry {
                    name: "ArgValues".to_string(),
                    kind: "type".to_string(),
                    description: String::new(),
                    params: None,
                    returns: None,
                    examples: None,
                    tags: None,
                    private: false,
                    file: "/repo/src/types.ts".to_string(),
                    line: 1,
                    end_line: 1,
                    signature: Some("export type ArgValues = unknown".to_string()),
                    extends: None,
                    implements: None,
                    has_body: None,
                    members: None,
                    type_parameters: None,
                },
            ],
        }];

        let markdown = generate_docs_markdown(
            docs,
            Some(JsDocsMarkdownOptions {
                path_strategy: Some("typedoc".to_string()),
                render_style: Some("markdown".to_string()),
                ..Default::default()
            }),
        );
        let page = markdown.get("default/functions/resolveArgs.md").unwrap();

        assert!(page.contains("## Returns\n\n`object` — Resolved args."));
        assert!(page.contains("### values\n\n```ts\nvalues: ArgValues<A>;\n```"));
    }

    #[test]
    fn generate_docs_markdown_renders_type_alias_function_metadata() {
        let docs = vec![JsDocsMarkdownModule {
            description: None,
            file: "default".to_string(),
            source_path: None,
            examples: None,
            tags: None,
            entries: vec![
                JsDocsMarkdownEntry {
                    name: "CommandRunner".to_string(),
                    kind: "type".to_string(),
                    description: "Run a command.".to_string(),
                    params: Some(vec![JsDocParam {
                        name: "ctx".to_string(),
                        r#type: "Readonly<CommandContext<G>>".to_string(),
                        description: String::new(),
                        optional: Some(false),
                        r#default: None,
                    }]),
                    returns: Some(JsDocReturn {
                        r#type: "Awaitable<string | void>".to_string(),
                        description: "CLI output.".to_string(),
                        members: None,
                    }),
                    examples: None,
                    tags: None,
                    private: false,
                    file: "/repo/src/types.ts".to_string(),
                    line: 1,
                    end_line: 1,
                    signature: Some(
                        "export type CommandRunner<G> = (ctx: Readonly<CommandContext<G>>) => Awaitable<string | void>"
                            .to_string(),
                    ),
                    extends: None,
                    implements: None,
                    has_body: None,
                    members: None,
                    type_parameters: None,
                },
                JsDocsMarkdownEntry {
                    name: "CommandContext".to_string(),
                    kind: "type".to_string(),
                    description: String::new(),
                    params: Some(vec![]),
                    returns: None,
                    examples: None,
                    tags: None,
                    private: false,
                    file: "/repo/src/context.ts".to_string(),
                    line: 1,
                    end_line: 1,
                    signature: Some("export type CommandContext = unknown".to_string()),
                    extends: None,
                    implements: None,
                    has_body: None,
                    members: None,
                    type_parameters: None,
                },
            ],
        }];

        let markdown = generate_docs_markdown(
            docs,
            Some(JsDocsMarkdownOptions {
                group_by: Some("file".to_string()),
                path_strategy: Some("typedoc".to_string()),
                render_style: Some("markdown".to_string()),
                parameters_format: Some("table".to_string()),
                ..Default::default()
            }),
        );
        let page = markdown.get("default/type-aliases/CommandRunner.md").unwrap();

        assert!(page.contains("## Parameters"));
        assert!(page.contains("Readonly"));
        assert!(page.contains("CommandContext"));
        assert!(page.contains("## Returns"));
        assert!(!page.contains("| `ctx` | `unknown` |"));
        assert!(page.contains("`Awaitable<string | void>`"));
        assert!(page.contains("CLI output."));
        assert!(!page.contains("`unknown`"));
    }

    #[test]
    fn generate_docs_markdown_does_not_escape_return_union_pipe_inside_inline_code() {
        let docs = vec![JsDocsMarkdownModule {
            description: None,
            file: "default".to_string(),
            source_path: None,
            examples: None,
            tags: None,
            entries: vec![JsDocsMarkdownEntry {
                name: "cli".to_string(),
                kind: "function".to_string(),
                description: "Run the command.".to_string(),
                params: Some(vec![JsDocParam {
                    name: "entry".to_string(),
                    r#type: "Command<G> | CommandRunner<G>".to_string(),
                    description: "Command entry.".to_string(),
                    optional: Some(false),
                    r#default: None,
                }]),
                returns: Some(JsDocReturn {
                    r#type: "Promise<string | undefined>".to_string(),
                    description: "A rendered usage or undefined.".to_string(),
                    members: None,
                }),
                examples: None,
                tags: None,
                private: false,
                file: "/repo/src/cli.ts".to_string(),
                line: 1,
                end_line: 5,
                signature: Some(
                    "export function cli(entry: Command<G> | CommandRunner<G>): Promise<string | undefined>"
                        .to_string(),
                ),
                extends: None,
                implements: None,
                has_body: None,
                members: None,
                type_parameters: None,
            }],
        }];

        let markdown = generate_docs_markdown(
            docs,
            Some(JsDocsMarkdownOptions {
                group_by: Some("file".to_string()),
                path_strategy: Some("typedoc".to_string()),
                render_style: Some("markdown".to_string()),
                parameters_format: Some("table".to_string()),
                ..Default::default()
            }),
        );
        let page = markdown.get("default/functions/cli.md").unwrap();

        assert!(page.contains("| `entry` | `Command<G> \\| CommandRunner<G>` | Command entry. |"));
        assert!(page.contains(
            "## Returns\n\n`Promise<string | undefined>` — A rendered usage or undefined."
        ));
        assert!(!page.contains("`Promise<string \\| undefined>`"));
    }

    #[test]
    fn generate_docs_markdown_renders_type_alias_return_without_description() {
        let docs = vec![JsDocsMarkdownModule {
            description: None,
            file: "default".to_string(),
            source_path: None,
            examples: None,
            tags: None,
            entries: vec![JsDocsMarkdownEntry {
                name: "OnPluginExtension".to_string(),
                kind: "type".to_string(),
                description: "Plugin extension hook.".to_string(),
                params: Some(vec![
                    JsDocParam {
                        name: "ctx".to_string(),
                        r#type: "Readonly<CommandContext<G>>".to_string(),
                        description: "The command context.".to_string(),
                        optional: Some(false),
                        r#default: None,
                    },
                    JsDocParam {
                        name: "cmd".to_string(),
                        r#type: "Readonly<Command<G>>".to_string(),
                        description: "The command.".to_string(),
                        optional: Some(false),
                        r#default: None,
                    },
                ]),
                returns: Some(JsDocReturn {
                    r#type: "Awaitable<void>".to_string(),
                    description: String::new(),
                    members: None,
                }),
                examples: None,
                tags: None,
                private: false,
                file: "/repo/src/plugin.ts".to_string(),
                line: 1,
                end_line: 5,
                signature: Some(
                    "export type OnPluginExtension<G> = (ctx: Readonly<CommandContext<G>>, cmd: Readonly<Command<G>>) => Awaitable<void>"
                        .to_string(),
                ),
                extends: None,
                implements: None,
                has_body: None,
                members: None,
                type_parameters: None,
            }],
        }];

        let markdown = generate_docs_markdown(
            docs,
            Some(JsDocsMarkdownOptions {
                group_by: Some("file".to_string()),
                path_strategy: Some("typedoc".to_string()),
                render_style: Some("markdown".to_string()),
                parameters_format: Some("table".to_string()),
                ..Default::default()
            }),
        );
        let page = markdown.get("default/type-aliases/OnPluginExtension.md").unwrap();

        assert!(page.contains("## Parameters"));
        assert!(page.contains("The command context."));
        assert!(page.contains("The command."));
        assert!(page.contains("## Returns\n\n`Awaitable<void>`"));
        assert!(!page.contains("`unknown`"));
    }

    #[test]
    fn generate_docs_markdown_renders_index_signature_members() {
        let docs = vec![JsDocsMarkdownModule {
            description: None,
            file: "default".to_string(),
            source_path: None,
            examples: None,
            tags: None,
            entries: vec![
                JsDocsMarkdownEntry {
                    name: "ArgSchema".to_string(),
                    kind: "interface".to_string(),
                    description: "Value type.".to_string(),
                    params: None,
                    returns: None,
                    examples: None,
                    tags: None,
                    private: false,
                    file: "/repo/src/args.ts".to_string(),
                    line: 1,
                    end_line: 1,
                    signature: Some("export interface ArgSchema".to_string()),
                    extends: None,
                    implements: None,
                    has_body: None,
                    members: None,
                    type_parameters: None,
                },
                JsDocsMarkdownEntry {
                    name: "Args".to_string(),
                    kind: "interface".to_string(),
                    description: "Arguments.".to_string(),
                    params: None,
                    returns: None,
                    examples: None,
                    tags: None,
                    private: false,
                    file: "/repo/src/args.ts".to_string(),
                    line: 1,
                    end_line: 5,
                    signature: Some("export interface Args".to_string()),
                    extends: None,
                    implements: None,
                    has_body: None,
                    members: Some(vec![JsDocMember {
                        name: "[option: string]".to_string(),
                        kind: "indexSignature".to_string(),
                        description: "Argument schema by option name.".to_string(),
                        signature: Some("readonly [option: string]: ArgSchema".to_string()),
                        r#type: Some("ArgSchema".to_string()),
                        r#default: None,
                        params: Some(vec![JsDocParam {
                            name: "option".to_string(),
                            r#type: "string".to_string(),
                            description: String::new(),
                            optional: None,
                            r#default: None,
                        }]),
                        type_parameters: None,
                        returns: None,
                        members: None,
                        optional: Some(false),
                        readonly: Some(true),
                        r#static: Some(false),
                        private: Some(false),
                        tags: None,
                        implementation_of: None,
                        line: 4,
                        end_line: 4,
                    }]),
                    type_parameters: None,
                },
            ],
        }];

        let markdown = generate_docs_markdown(
            docs,
            Some(JsDocsMarkdownOptions {
                path_strategy: Some("typedoc".to_string()),
                render_style: Some("markdown".to_string()),
                ..Default::default()
            }),
        );
        let page = markdown.get("default/interfaces/Args.md").unwrap();

        assert!(page.contains("## Indexable\n\n"));
        assert!(page.contains("```ts\nreadonly [option: string]: ArgSchema\n```"));
        assert!(page.contains("Argument schema by option name."));
    }

    #[test]
    fn generate_docs_markdown_renders_module_description_in_typedoc_index() {
        let docs = vec![JsDocsMarkdownModule {
            description: Some("The entry for gunshi context.".to_string()),
            file: "context".to_string(),
            source_path: None,
            examples: Some(vec!["```ts\ncreateCommandContext()\n```".to_string()]),
            tags: None,
            entries: vec![JsDocsMarkdownEntry {
                name: "createCommandContext".to_string(),
                kind: "function".to_string(),
                description: "Creates a command context.".to_string(),
                params: None,
                returns: None,
                examples: None,
                tags: None,
                private: false,
                file: "/repo/src/context.ts".to_string(),
                line: 1,
                end_line: 10,
                signature: Some("export function createCommandContext(): void".to_string()),
                extends: None,
                implements: None,
                has_body: None,
                members: None,
                type_parameters: None,
            }],
        }];

        let markdown = generate_docs_markdown(
            docs,
            Some(JsDocsMarkdownOptions {
                group_by: Some("file".to_string()),
                github_url: None,
                link_style: Some("markdown".to_string()),
                base_path: None,
                path_strategy: Some("typedoc".to_string()),
                render_style: Some("markdown".to_string()),
                ..Default::default()
            }),
        );

        let index = markdown.get("index.md").unwrap();
        assert!(index.contains("The entry for gunshi context."));
        assert!(!index.contains("Creates a command context."));
        let module_index = markdown.get("context/index.md").unwrap();
        assert!(module_index.contains("The entry for gunshi context."));
        assert!(module_index.contains("## Example\n\n```ts\ncreateCommandContext()\n```"));
    }

    #[test]
    fn generate_docs_nav_metadata_from_docs_returns_typedoc_tree() {
        let docs = vec![JsDocsMarkdownModule {
            description: None,
            file: "default".to_string(),
            source_path: None,
            examples: None,
            tags: None,
            entries: vec![
                JsDocsMarkdownEntry {
                    name: "cli".to_string(),
                    kind: "function".to_string(),
                    description: String::new(),
                    params: None,
                    returns: None,
                    examples: None,
                    tags: None,
                    private: false,
                    file: "/repo/src/cli.ts".to_string(),
                    line: 1,
                    end_line: 1,
                    signature: None,
                    extends: None,
                    implements: None,
                    has_body: None,
                    members: None,
                    type_parameters: None,
                },
                JsDocsMarkdownEntry {
                    name: "Mode".to_string(),
                    kind: "enum".to_string(),
                    description: String::new(),
                    params: None,
                    returns: None,
                    examples: None,
                    tags: None,
                    private: false,
                    file: "/repo/src/mode.ts".to_string(),
                    line: 1,
                    end_line: 1,
                    signature: None,
                    extends: None,
                    implements: None,
                    has_body: None,
                    members: None,
                    type_parameters: None,
                },
            ],
        }];

        let nav = generate_docs_nav_metadata_from_docs_napi(
            docs,
            Some(JsDocsNavOptions {
                base_path: Some("/api".to_string()),
                path_strategy: Some("typedoc".to_string()),
                group_order: None,
                sort: None,
                sort_entry_points: None,
                kind_sort_order: None,
            }),
        );

        assert_eq!(nav[0].title, "default");
        assert_eq!(nav[0].path, "/api/default");
        let children = nav[0].children.as_ref().unwrap();
        assert_eq!(children[0].title, "Functions");
        assert_eq!(children[0].children.as_ref().unwrap()[0].path, "/api/default/functions/cli");
        assert_eq!(children[1].title, "Enumerations");
        assert_eq!(
            children[1].children.as_ref().unwrap()[0].path,
            "/api/default/enumerations/Mode"
        );
    }

    #[test]
    fn generate_docs_nav_metadata_from_docs_defaults_to_flat() {
        let docs = vec![JsDocsMarkdownModule {
            description: None,
            file: "/repo/src/context.ts".to_string(),
            source_path: None,
            examples: None,
            tags: None,
            entries: vec![],
        }];

        let nav = generate_docs_nav_metadata_from_docs_napi(docs, None);

        assert_eq!(nav.len(), 1);
        assert_eq!(nav[0].path, "/api/context");
        assert!(nav[0].children.is_none());
    }

    #[test]
    fn write_generated_docs_writes_typedoc_nested_files() {
        use super::{write_generated_docs, JsDocsOutputOptions};

        let unique =
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let out_dir = std::env::temp_dir()
            .join(format!("ox-content-napi-typedoc-write-{}-{unique}", std::process::id()));

        let extracted = vec![JsDocsMarkdownModule {
            description: None,
            file: "default".to_string(),
            source_path: None,
            examples: None,
            tags: None,
            entries: vec![
                JsDocsMarkdownEntry {
                    name: "cli".to_string(),
                    kind: "function".to_string(),
                    description: "Runs the CLI.".to_string(),
                    params: None,
                    returns: None,
                    examples: None,
                    tags: None,
                    private: false,
                    file: "/repo/src/cli.ts".to_string(),
                    line: 1,
                    end_line: 1,
                    signature: Some("export function cli(): void".to_string()),
                    extends: None,
                    implements: None,
                    has_body: None,
                    members: None,
                    type_parameters: None,
                },
                JsDocsMarkdownEntry {
                    name: "version".to_string(),
                    kind: "variable".to_string(),
                    description: "Package version.".to_string(),
                    params: None,
                    returns: None,
                    examples: None,
                    tags: None,
                    private: false,
                    file: "/repo/src/version.ts".to_string(),
                    line: 2,
                    end_line: 2,
                    signature: Some("export const version = '1.0.0'".to_string()),
                    extends: None,
                    implements: None,
                    has_body: None,
                    members: None,
                    type_parameters: None,
                },
            ],
        }];

        let markdown = generate_docs_markdown(
            extracted.clone(),
            Some(JsDocsMarkdownOptions {
                group_by: Some("file".to_string()),
                github_url: None,
                link_style: Some("clean".to_string()),
                base_path: Some("/api".to_string()),
                path_strategy: Some("typedoc".to_string()),
                render_style: None,
                ..Default::default()
            }),
        );

        write_generated_docs(
            markdown,
            out_dir.to_string_lossy().to_string(),
            Some(extracted),
            Some(JsDocsOutputOptions {
                generate_nav: Some(true),
                group_by: Some("file".to_string()),
                generated_at: Some("2026-01-01T00:00:00.000Z".to_string()),
                base_path: Some("/api".to_string()),
                path_strategy: Some("typedoc".to_string()),
                group_order: Some(vec!["Variables".to_string(), "Functions".to_string()]),
                sort: None,
                sort_entry_points: None,
                kind_sort_order: None,
            }),
        )
        .unwrap();

        assert!(out_dir.join("default/index.md").exists());
        assert!(out_dir.join("default/functions/cli.md").exists());
        assert!(out_dir.join("default/variables/version.md").exists());

        let nav = fs::read_to_string(out_dir.join("nav.ts")).unwrap();
        assert!(nav.contains(r#""title": "default""#));
        assert!(nav.contains("\"/api/default/functions/cli\""));
        assert!(nav.contains("\"/api/default/variables/version\""));
        assert!(
            nav.find(r#""title": "Variables""#).unwrap()
                < nav.find(r#""title": "Functions""#).unwrap()
        );

        fs::remove_dir_all(&out_dir).unwrap();
    }

    #[test]
    fn extract_docs_from_entry_points_preserves_explicit_module_name() {
        let unique =
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let root = std::env::temp_dir()
            .join(format!("ox-content-napi-module-name-{}-{unique}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("src/index.ts"),
            r"
/**
 * gunshi cli entry point.
 *
 * @example
 * ```ts
 * cli()
 * ```
 *
 * @experimental This module is experimental.
 *
 * @module default
 */
/** Runs the CLI. */
export function cli(): void {}
",
        )
        .unwrap();

        let modules = extract_docs_from_entry_points_napi(
            vec![JsEntryPointSpec { path: "src/index.ts".to_string(), name: None }],
            Some(JsEntryPointDocsOptions {
                root: Some(root.to_string_lossy().into_owned()),
                ..Default::default()
            }),
        )
        .unwrap();

        assert_eq!(modules[0].name, "default");
        assert_eq!(modules[0].file, "default");
        assert_eq!(modules[0].description, "gunshi cli entry point.");
        assert_eq!(modules[0].examples, vec!["```ts\ncli()\n```".to_string()]);
        assert_eq!(modules[0].tags.len(), 1);
        assert_eq!(modules[0].tags[0].tag, "experimental");
        assert_eq!(modules[0].tags[0].value, "This module is experimental.");

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn extract_docs_from_entry_points_accepts_external_docs_options() {
        let unique =
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let root = std::env::temp_dir()
            .join(format!("ox-content-napi-external-docs-{}-{unique}", std::process::id()));
        let package_root = root.join("node_modules/external-pkg");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("src")).unwrap();
        fs::create_dir_all(package_root.join("lib")).unwrap();
        fs::write(root.join("src/index.ts"), "export { ExternalThing } from 'external-pkg';\n")
            .unwrap();
        fs::write(
            package_root.join("package.json"),
            r#"{
  "name": "external-pkg",
  "type": "module",
  "exports": {
    ".": {
      "types": "./lib/index.d.ts",
      "default": "./lib/index.js"
    }
  }
}"#,
        )
        .unwrap();
        fs::write(
            package_root.join("lib/index.d.ts"),
            r"
/** External thing. */
export interface ExternalThing {
  value: string;
}
",
        )
        .unwrap();

        let modules = extract_docs_from_entry_points_napi(
            vec![JsEntryPointSpec {
                path: "src/index.ts".to_string(),
                name: Some("default".to_string()),
            }],
            Some(JsEntryPointDocsOptions {
                root: Some(root.to_string_lossy().into_owned()),
                external_docs: Some(true),
                ..Default::default()
            }),
        )
        .unwrap();

        assert_eq!(modules[0].entries[0].name, "ExternalThing");
        assert_eq!(modules[0].entries[0].description, "External thing.");
        assert_eq!(modules[0].exports[0].source.kind, "external");
        assert!(modules[0].exports[0]
            .source
            .module
            .as_deref()
            .is_some_and(|module| { module.ends_with("external-pkg/lib/index.d.ts") }));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn extract_docs_from_entry_points_emits_undocumented_public_const_and_diagnostics() {
        let unique =
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let root = std::env::temp_dir()
            .join(format!("ox-content-napi-local-export-docs-{}-{unique}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("src/index.ts"),
            r"
export { ANONYMOUS_COMMAND_NAME } from './constants';
export type { ExtractArgs } from './types';
",
        )
        .unwrap();
        fs::write(
            root.join("src/constants.ts"),
            r#"
export const ANONYMOUS_COMMAND_NAME = "(anonymous)";
"#,
        )
        .unwrap();
        fs::write(
            root.join("src/types.ts"),
            r"
/**
 * Type helper to extract args.
 *
 * @internal
 */
export type ExtractArgs<G> = G extends { args: infer A } ? A : never;
",
        )
        .unwrap();

        let modules = extract_docs_from_entry_points_napi(
            vec![JsEntryPointSpec {
                path: "src/index.ts".to_string(),
                name: Some("default".to_string()),
            }],
            Some(JsEntryPointDocsOptions {
                root: Some(root.to_string_lossy().into_owned()),
                ..Default::default()
            }),
        )
        .unwrap();

        assert_eq!(modules[0].entries.len(), 1);
        assert_eq!(modules[0].entries[0].name, "ANONYMOUS_COMMAND_NAME");
        assert_eq!(modules[0].entries[0].kind, "variable");
        assert!(modules[0].entries[0].description.is_empty());
        assert_eq!(modules[0].diagnostics.len(), 1);
        assert_eq!(modules[0].diagnostics[0].code, "filteredByVisibility");
        assert_eq!(modules[0].diagnostics[0].export_name, "ExtractArgs");
        assert_eq!(modules[0].diagnostics[0].source.original_name, "ExtractArgs");

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn prepare_source_returns_object_shaped_frontmatter_and_origin() {
        let result = super::prepare_source(
            "---\ntitle: Guide\nmeta:\n  draft: false\n---\n# Body".to_string(),
            None,
        );

        assert_eq!(result.content, "# Body");
        assert_eq!(result.frontmatter.get("title"), Some(&json!("Guide")));
        assert_eq!(result.frontmatter.get("meta"), Some(&json!({"draft": false})));
        assert_eq!(result.source_offset.line, 6);
        assert_eq!(result.source_offset.column, 1);
    }

    #[test]
    fn javascript_wrapper_and_declarations_cover_expected_exports() {
        let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let index_js = fs::read_to_string(manifest_dir.join("index.js")).unwrap();
        let declarations = fs::read_to_string(manifest_dir.join("index.d.ts")).unwrap();
        let expected_exports = [
            "buildSearchIndex",
            "buildSearchIndexFromDirectory",
            "buildSsgNavItems",
            "buildSsgThemeNavItems",
            "buildExportGraph",
            "checkI18n",
            "checkI18nProject",
            "collectDocsSourceFiles",
            "collectSearchMarkdownFiles",
            "collectSsgMarkdownFiles",
            "externalizeSsgAssets",
            "extractDocsFromDirectories",
            "extractDocsFromEntryPoints",
            "extractFileDocEntries",
            "extractFileDocs",
            "extractSearchContent",
            "extractSsgTitle",
            "extractTranslationKeys",
            "formatSsgTitle",
            "generateDocsDataJson",
            "generateDocsMarkdown",
            "generateDocsNavCode",
            "generateDocsNavMetadata",
            "generateDocsNavMetadataFromDocs",
            "generateI18nModule",
            "generateOgImageSvg",
            "generateSearchModule",
            "generateSearchModuleFromOptions",
            "generateSsgBareHtml",
            "generateSsgHtml",
            "getGitLastUpdated",
            "getSearchDocumentScopes",
            "getSsgHref",
            "getSsgOutputPath",
            "getSsgPageLocale",
            "getSsgUrlPath",
            "lintMarkdown",
            "lintMarkdownDocuments",
            "loadDictionaries",
            "loadDictionariesFlat",
            "matchesSearchScopes",
            "mergeHighlightedCodeBlocks",
            "normalizeVitePressFrontmatter",
            "parse",
            "parseAndRender",
            "parseAndRenderAsync",
            "parseMdastRaw",
            "parseScopedSearchQuery",
            "parseTransferRaw",
            "prepareSource",
            "prepareSourceRaw",
            "render",
            "resolveSsgNavigationGroups",
            "resolveSsgRoutePaths",
            "searchIndex",
            "transform",
            "transformAsync",
            "transformMdastRaw",
            "transformMermaid",
            "validateMf2",
            "version",
            "writeGeneratedDocs",
            "writeSearchIndex",
        ];

        for export_name in expected_exports {
            assert!(
                index_js
                    .contains(&format!("module.exports.{export_name} = binding.{export_name};")),
                "index.js is missing {export_name}"
            );
            assert!(
                declarations.contains(&format!("export declare function {export_name}(")),
                "index.d.ts is missing {export_name}"
            );
        }
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
    fn writes_search_index_through_napi() {
        let root =
            std::env::temp_dir().join(format!("ox-content-napi-search-out-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);

        super::write_search_index(r#"{"doc_count":0}"#.to_string(), root.to_string_lossy().into())
            .unwrap();

        assert_eq!(
            fs::read_to_string(root.join("search-index.json")).unwrap(),
            r#"{"doc_count":0}"#
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn check_i18n_project_collects_source_and_markdown_keys() {
        let unique =
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let root = std::env::temp_dir()
            .join(format!("ox-content-napi-i18n-{}-{unique}", std::process::id()));
        let dict_root = root.join("content/i18n");
        let src_dir = root.join("src");
        let content_dir = root.join("content");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(dict_root.join("en")).unwrap();
        fs::create_dir_all(&src_dir).unwrap();

        fs::write(
            dict_root.join("en/common.json"),
            r#"{"fromSrc":"From source","fromMd":"From markdown"}"#,
        )
        .unwrap();
        fs::write(src_dir.join("app.ts"), "const label = t('common.fromSrc');").unwrap();
        fs::write(content_dir.join("guide.md"), "{{t('common.fromMd')}}").unwrap();

        let result = super::check_i18n_project(
            dict_root.to_string_lossy().into_owned(),
            vec![
                src_dir.to_string_lossy().into_owned(),
                content_dir.to_string_lossy().into_owned(),
            ],
            vec!["t".to_string(), "$t".to_string()],
            "en".to_string(),
        );
        let messages: Vec<&str> = result.diagnostics.iter().map(|d| d.message.as_str()).collect();

        assert_eq!(result.error_count, 0, "diagnostics: {messages:?}");
        assert_eq!(result.warning_count, 0, "diagnostics: {messages:?}");
        assert!(result.diagnostics.is_empty());

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn generates_search_module_from_typed_options() {
        let module = super::generate_search_module_from_options(
            super::JsSearchRuntimeOptions {
                enabled: true,
                limit: 7,
                prefix: false,
                placeholder: "Find".to_string(),
                hotkey: "k".to_string(),
            },
            "/docs/search-index.json".to_string(),
        );

        assert!(module.contains(
            r#"const searchOptions = {"enabled":true,"limit":7,"prefix":false,"placeholder":"Find","hotkey":"k"};"#
        ));
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
