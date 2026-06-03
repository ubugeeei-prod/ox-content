//! Documentation extraction from source code using OXC parser.

use ox_jsdoc::decoder::nodes::comment_ast::{LazyJsdocTag, LazyJsdocTagBody};
use ox_jsdoc::parser::{
    parse_batch_to_bytes as parse_jsdoc_batch_to_bytes, BatchItem as JsdocBatchItem,
    ParseOptions as JsdocParseOptions,
};
use oxc_allocator::Allocator;
use oxc_ast::ast::{
    BindingPattern, Class, Comment, Declaration, ExportDefaultDeclarationKind, Expression,
    Function, Statement, TSEnumDeclaration, TSEnumMember, TSInterfaceDeclaration, TSSignature,
    TSType, TSTypeAliasDeclaration, TSTypeAnnotation, TSTypeName, VariableDeclaration,
};
use oxc_ast_visit::{walk, Visit};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use rustc_hash::{FxBuildHasher, FxHashMap, FxHashSet};
use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;

use crate::string_builder::{join2, join3, join5, StringBuilder};

/// Result type for extraction operations.
pub type ExtractResult<T> = Result<T, ExtractError>;

/// Errors during documentation extraction.
#[derive(Debug, Error)]
pub enum ExtractError {
    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Parse error.
    #[error("Parse error: {0}")]
    Parse(String),

    /// Unsupported file type.
    #[error("Unsupported file type: {0}")]
    UnsupportedFile(String),
}

/// Documentation item extracted from source code.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocItem {
    /// Item name.
    pub name: String,
    /// Item kind (function, class, interface, etc.).
    pub kind: DocItemKind,
    /// Documentation comment (JSDoc).
    pub doc: Option<String>,
    /// Source file path.
    pub source_path: String,
    /// Line number in source.
    pub line: u32,
    /// End line number in source.
    pub end_line: u32,
    /// Column number in source.
    pub column: u32,
    /// Raw JSDoc comment content without the outer delimiters.
    pub jsdoc: Option<String>,
    /// Whether the item is exported.
    pub exported: bool,
    /// Type signature (if applicable).
    pub signature: Option<String>,
    /// Whether a function/method declaration carries an implementation body.
    /// `false` for overload signatures and ambient (`declare` / `.d.ts`)
    /// declarations, and for non-callable items. Used to hide the implementation
    /// signature when grouping overloads on TypeDoc symbol pages.
    #[serde(default)]
    pub has_body: bool,
    /// Whether the item is optional.
    #[serde(default)]
    pub optional: bool,
    /// Whether the item is readonly.
    #[serde(default)]
    pub readonly: bool,
    /// Whether the item is static.
    #[serde(default)]
    pub r#static: bool,
    /// Parameters (for functions/methods).
    pub params: Vec<ParamDoc>,
    /// Return type (for functions/methods).
    pub return_type: Option<String>,
    /// Child items (for classes, modules, etc.).
    pub children: Vec<DocItem>,
    /// JSDoc tags.
    pub tags: Vec<DocTag>,
    /// Declaration type parameters (`<T extends C = D>`), in declaration order.
    #[serde(default)]
    pub type_parameters: Vec<TypeParamDoc>,
}

/// Parameter documentation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamDoc {
    /// Parameter name.
    pub name: String,
    /// Parameter type.
    pub type_annotation: Option<String>,
    /// Whether the parameter is optional.
    pub optional: bool,
    /// Default value (if any).
    pub default_value: Option<String>,
    /// Description from JSDoc @param tag.
    pub description: Option<String>,
}

/// Type parameter documentation (`<T extends C = D>`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeParamDoc {
    /// Type parameter name (e.g. `T`).
    pub name: String,
    /// Constraint after `extends`, when present.
    pub constraint: Option<String>,
    /// Default type after `=`, when present.
    pub default: Option<String>,
    /// Description merged from a `@typeParam` / `@template` tag (TSDoc).
    pub description: String,
}

/// JSDoc tag.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocTag {
    /// Tag name (e.g., "param", "returns", "example").
    pub tag: String,
    /// Tag value.
    pub value: String,
    /// JSDoc type annotation, when the tag has a `{type}` part.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub type_annotation: Option<String>,
    /// JSDoc name, when the tag has a name part.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Whether the named part was marked optional.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub optional: Option<bool>,
    /// Default value from `[name=value]` syntax.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_value: Option<String>,
    /// Structured tag description parsed by `ox_jsdoc`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl DocTag {
    fn new(tag: String, value: String) -> Self {
        Self {
            tag,
            value,
            type_annotation: None,
            name: None,
            optional: None,
            default_value: None,
            description: None,
        }
    }
}

#[derive(Debug, Clone)]
struct ParsedParamTag {
    name: String,
    type_annotation: Option<String>,
    optional: bool,
    default_value: Option<String>,
    description: Option<String>,
}

/// Kind of documentation item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DocItemKind {
    /// Module or namespace.
    Module,
    /// Function.
    Function,
    /// Class.
    Class,
    /// Interface (TypeScript).
    Interface,
    /// Type alias.
    Type,
    /// Enum.
    Enum,
    /// Variable or constant.
    Variable,
    /// Class method.
    Method,
    /// Class property.
    Property,
    /// Constructor.
    Constructor,
    /// Getter.
    Getter,
    /// Setter.
    Setter,
    /// Enum member.
    #[serde(rename = "enumMember")]
    EnumMember,
}

/// Documentation extractor.
pub struct DocExtractor {
    /// Include private items.
    include_private: bool,
    /// Include internal items.
    include_internal: bool,
    /// Include declarations without JSDoc. Used for public entry point exports.
    include_undocumented_declarations: bool,
}

impl DocExtractor {
    /// Creates a new documentation extractor.
    #[must_use]
    pub fn new() -> Self {
        Self {
            include_private: false,
            include_internal: false,
            include_undocumented_declarations: false,
        }
    }

    /// Creates a new extractor that includes private items.
    #[must_use]
    pub fn with_private(include_private: bool) -> Self {
        Self { include_private, include_internal: false, include_undocumented_declarations: false }
    }

    /// Creates a new extractor with explicit visibility options.
    #[must_use]
    pub fn with_visibility(include_private: bool, include_internal: bool) -> Self {
        Self { include_private, include_internal, include_undocumented_declarations: false }
    }

    /// Creates a new extractor for public entry point exports.
    #[must_use]
    pub(crate) fn for_entrypoint_exports(include_private: bool, include_internal: bool) -> Self {
        Self { include_private, include_internal, include_undocumented_declarations: true }
    }

    /// Extracts documentation from a source file.
    pub fn extract_file(&self, path: &Path) -> ExtractResult<Vec<DocItem>> {
        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        match extension {
            "ts" | "tsx" | "js" | "jsx" | "mts" | "mjs" | "cts" | "cjs" => self.extract_js_ts(path),
            _ => Err(ExtractError::UnsupportedFile(extension.to_string())),
        }
    }

    /// Extracts documentation from source code string.
    pub fn extract_source(
        &self,
        source: &str,
        file_path: &str,
        source_type: SourceType,
    ) -> ExtractResult<Vec<DocItem>> {
        let allocator = Allocator::default();
        let ret = Parser::new(&allocator, source, source_type).parse();

        if !ret.errors.is_empty() {
            let error_msg = ret
                .errors
                .iter()
                .map(std::string::ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ");
            return Err(ExtractError::Parse(error_msg));
        }

        let comments: Vec<Comment> = ret.program.comments.iter().copied().collect();
        let jsdoc_cache = build_jsdoc_cache(source, &comments);

        let mut visitor = DocVisitor::new(
            source,
            file_path,
            self.include_private,
            self.include_internal,
            self.include_undocumented_declarations,
            jsdoc_cache,
        );
        let first_stmt_start = ret.program.body.first().map(|statement| statement.span().start);
        if let Some(module_item) = visitor.extract_module_entry(&comments, first_stmt_start) {
            visitor.items.push(module_item);
        }
        visitor.visit_program(&ret.program);

        Ok(visitor.items)
    }

    /// Extracts documentation from a JavaScript/TypeScript file.
    fn extract_js_ts(&self, path: &Path) -> ExtractResult<Vec<DocItem>> {
        let content = std::fs::read_to_string(path)?;
        let file_path = path.to_string_lossy().to_string();
        let source_type = SourceType::from_path(path).unwrap_or_default();

        self.extract_source(&content, &file_path, source_type)
    }
}

impl Default for DocExtractor {
    fn default() -> Self {
        Self::new()
    }
}

/// Pre-parsed JSDoc data for one comment: `(raw, description, tags)`.
type ParsedJsdoc = (String, String, Vec<DocTag>);

/// JSDoc/TSDoc tags that mark a leading comment as a module/file comment.
const MODULE_MARKER_TAGS: [&str; 3] = ["module", "packageDocumentation", "fileoverview"];

/// Extract the JSDoc content body from a comment with a single allocation.
fn extract_raw_jsdoc(comment: &Comment, source: &str) -> String {
    let content = comment.content_span().source_text(source);
    let trimmed = content.strip_prefix('*').unwrap_or(content);
    trimmed.trim_matches('\n').to_string()
}

/// Parse a single JSDoc comment into `(raw, description, tags)` directly from its
/// own span.
///
/// Unlike [`build_jsdoc_cache`], this does not key by `attached_to`, so it stays
/// correct for comments that share an `attached_to` target — e.g. two leading
/// file comments (`/** … @module */` followed by `/** @author … */`) that the
/// parser both attaches to the first statement. Keying by `attached_to` would let
/// the second comment overwrite the first in the cache.
fn parse_jsdoc_payload(source: &str, comment: &Comment) -> ParsedJsdoc {
    let raw = extract_raw_jsdoc(comment, source);
    let items = [JsdocBatchItem {
        source_text: comment.span.source_text(source),
        base_offset: comment.span.start,
    }];
    let options = JsdocParseOptions { preserve_whitespace: true, ..JsdocParseOptions::default() };
    let result = parse_jsdoc_batch_to_bytes(&items, options);

    if result.diagnostics.is_empty() {
        if let Ok(source_file) =
            ox_jsdoc::decoder::source_file::LazySourceFile::new(&result.binary_bytes)
        {
            if let Some(Some(root)) = source_file.asts().next() {
                let doc = root
                    .description_text(false)
                    .map_or_else(String::new, |description| description.trim().to_string());
                let tags = root.tags().map(DocVisitor::convert_jsdoc_tag).collect();
                return (raw, doc, tags);
            }
        }
    }

    let (doc, tags) = DocVisitor::parse_jsdoc_fallback(&raw);
    (raw, doc, tags)
}

/// Pre-parses every JSDoc comment in the program with a single batch call.
///
/// Oxc exposes comments separately from declarations, and the visitor resolves
/// documentation by `attached_to` while walking the AST. Parsing each comment
/// on demand repeats decoder setup and makes the visitor pay parse cost at
/// every declaration. This cache batches decoder input once, stores the parsed
/// `(raw, description, tags)` payload by `attached_to`, and lets the visitor do
/// a cheap hash lookup in the hot declaration path. Comments that fail the
/// batch parser still fall back individually so diagnostics do not poison the
/// whole file.
fn build_jsdoc_cache(source: &str, comments: &[Comment]) -> FxHashMap<u32, ParsedJsdoc> {
    let jsdoc_comments: Vec<&Comment> =
        comments.iter().filter(|comment| comment.is_jsdoc()).collect();
    if jsdoc_comments.is_empty() {
        return FxHashMap::default();
    }

    let items: Vec<JsdocBatchItem<'_>> = jsdoc_comments
        .iter()
        .map(|comment| JsdocBatchItem {
            source_text: comment.span.source_text(source),
            base_offset: comment.span.start,
        })
        .collect();

    let options = JsdocParseOptions { preserve_whitespace: true, ..JsdocParseOptions::default() };
    let result = parse_jsdoc_batch_to_bytes(&items, options);

    let failed: FxHashSet<u32> =
        result.diagnostics.iter().map(|diagnostic| diagnostic.root_index).collect();

    let mut cache: FxHashMap<u32, ParsedJsdoc> =
        FxHashMap::with_capacity_and_hasher(jsdoc_comments.len(), FxBuildHasher);

    if let Ok(source_file) =
        ox_jsdoc::decoder::source_file::LazySourceFile::new(&result.binary_bytes)
    {
        for (index, (comment, root)) in jsdoc_comments.iter().zip(source_file.asts()).enumerate() {
            let raw = extract_raw_jsdoc(comment, source);
            let (doc, tags) = match root {
                Some(root) if !failed.contains(&(index as u32)) => {
                    let doc = root
                        .description_text(false)
                        .map_or_else(String::new, |description| description.trim().to_string());
                    let tags = root.tags().map(DocVisitor::convert_jsdoc_tag).collect();
                    (doc, tags)
                }
                _ => DocVisitor::parse_jsdoc_fallback(&raw),
            };
            cache.insert(comment.attached_to, (raw, doc, tags));
        }
    } else {
        for comment in &jsdoc_comments {
            let raw = extract_raw_jsdoc(comment, source);
            let (doc, tags) = DocVisitor::parse_jsdoc_fallback(&raw);
            cache.insert(comment.attached_to, (raw, doc, tags));
        }
    }

    cache
}

/// AST visitor for extracting documentation.
struct DocVisitor<'a> {
    source: &'a str,
    file_path: &'a str,
    include_private: bool,
    include_internal: bool,
    include_undocumented_declarations: bool,
    jsdoc_cache: FxHashMap<u32, ParsedJsdoc>,
    line_starts: Vec<usize>,
    items: Vec<DocItem>,
    /// Track default export
    has_default_export: bool,
}

impl<'a> DocVisitor<'a> {
    fn new(
        source: &'a str,
        file_path: &'a str,
        include_private: bool,
        include_internal: bool,
        include_undocumented_declarations: bool,
        jsdoc_cache: FxHashMap<u32, ParsedJsdoc>,
    ) -> Self {
        let mut line_starts = Vec::new();
        line_starts.push(0);
        line_starts.extend(
            source
                .bytes()
                .enumerate()
                .filter_map(|(index, byte)| (byte == b'\n').then_some(index + 1)),
        );

        Self {
            source,
            file_path,
            include_private,
            include_internal,
            include_undocumented_declarations,
            jsdoc_cache,
            line_starts,
            items: Vec::new(),
            has_default_export: false,
        }
    }

    fn slice(&self, start: u32, end: u32) -> String {
        self.source[start as usize..end as usize].to_string()
    }

    fn line_number(&self, position: u32) -> u32 {
        let position = position as usize;
        self.line_starts.partition_point(|&start| start <= position) as u32
    }

    fn column_number(&self, position: u32) -> u32 {
        let position = position as usize;
        let line_index = self.line_starts.partition_point(|&start| start <= position);
        let line_start = self.line_starts[line_index.saturating_sub(1)];
        (position.saturating_sub(line_start)) as u32
    }

    fn span_lines(&self, start: u32, end: u32) -> (u32, u32) {
        let start_line = self.line_number(start);
        let end_position = end.saturating_sub(1).max(start);
        let end_line = self.line_number(end_position);
        (start_line, end_line)
    }

    fn extract_jsdoc(&self, attached_to: u32) -> Option<(String, String, Vec<DocTag>)> {
        self.jsdoc_cache.get(&attached_to).cloned()
    }

    fn extract_declaration_docs(
        &self,
        attached_to: u32,
    ) -> Option<(Option<String>, Option<String>, Vec<DocTag>)> {
        if let Some((jsdoc, doc, tags)) = self.extract_jsdoc(attached_to) {
            if self.should_skip_by_visibility(&tags) {
                return None;
            }
            return Some((Some(jsdoc), (!doc.is_empty()).then_some(doc), tags));
        }

        self.include_undocumented_declarations.then(|| (None, None, Vec::new()))
    }

    fn extract_module_entry(
        &self,
        comments: &[Comment],
        first_stmt_start: Option<u32>,
    ) -> Option<DocItem> {
        let comment = comments.iter().find(|comment| comment.is_jsdoc())?;

        // Only the leading file comment (before the first statement) can be the
        // module comment; a JSDoc that follows code documents that declaration.
        if let Some(stmt_start) = first_stmt_start {
            if comment.span.start > stmt_start {
                return None;
            }
        }

        // Parse the candidate from its own span rather than the `attached_to`
        // cache, which collides when two leading comments share a target.
        let (raw, doc, tags) = parse_jsdoc_payload(self.source, comment);

        // Treat the leading comment as the module description when it either
        // carries a module marker tag (`@module` / `@packageDocumentation` /
        // `@fileoverview`, matching TypeDoc) or is detached from the following
        // code by a blank line. Otherwise it belongs to the first declaration.
        let has_module_marker =
            tags.iter().any(|tag| MODULE_MARKER_TAGS.contains(&tag.tag.as_str()));
        if !has_module_marker
            && !self.is_detached_leading_comment(comment, comments, first_stmt_start)
        {
            return None;
        }

        let (module_name, module_description) =
            Self::parse_module_tag(&tags).unwrap_or((None, None));
        let name = module_name.unwrap_or_else(|| self.file_stem_module_name());
        let (line, end_line) = self.span_lines(comment.span.start, comment.span.end);

        Some(DocItem {
            name,
            kind: DocItemKind::Module,
            doc: if doc.is_empty() { module_description } else { Some(doc) },
            source_path: self.file_path.to_string(),
            line,
            end_line,
            column: self.column_number(comment.span.start),
            jsdoc: Some(raw),
            exported: true,
            signature: None,
            has_body: false,
            optional: false,
            readonly: false,
            r#static: false,
            params: Vec::new(),
            return_type: None,
            children: Vec::new(),
            tags,
            type_parameters: Vec::new(),
        })
    }

    /// Returns true when the first leading JSDoc comment is detached from the
    /// following code by a blank line (or is the file's only content). Such a
    /// comment is a file-level/module comment rather than the doc of the first
    /// declaration, matching TypeDoc's leading-comment handling.
    fn is_detached_leading_comment(
        &self,
        comment: &Comment,
        comments: &[Comment],
        first_stmt_start: Option<u32>,
    ) -> bool {
        let after = comment.span.end;
        // The next syntactic element after this comment: the nearest of the
        // first statement and any later comment.
        let next_comment =
            comments.iter().map(|other| other.span.start).filter(|&start| start > after).min();
        let next_pos = match (next_comment, first_stmt_start) {
            (Some(comment_start), Some(stmt_start)) => Some(comment_start.min(stmt_start)),
            (Some(position), None) | (None, Some(position)) => Some(position),
            (None, None) => None,
        };
        // Nothing follows the comment: a comment-only file is a module comment.
        let Some(next_pos) = next_pos else {
            return true;
        };
        if next_pos <= after {
            return false;
        }
        // A blank line means two or more newlines between the comment and the
        // next element.
        self.source[after as usize..next_pos as usize].bytes().filter(|&byte| byte == b'\n').count()
            >= 2
    }

    fn parse_module_tag(tags: &[DocTag]) -> Option<(Option<String>, Option<String>)> {
        let tag = tags.iter().find(|tag| tag.tag == "module")?;
        let value = tag.value.trim();
        if value.is_empty() {
            return Some((None, tag.description.clone()));
        }

        let split_at = value
            .char_indices()
            .find_map(|(index, ch)| ch.is_whitespace().then_some(index))
            .unwrap_or(value.len());
        let name = value[..split_at].trim();
        let rest = value[split_at..].trim();
        let description = Self::clean_tag_description(rest).or_else(|| tag.description.clone());

        Some(((!name.is_empty()).then(|| name.to_string()), description))
    }

    fn file_stem_module_name(&self) -> String {
        Path::new(self.file_path)
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("module")
            .to_string()
    }

    fn convert_jsdoc_tag(tag: LazyJsdocTag<'_>) -> DocTag {
        let tag_name = tag.tag().value().to_string();
        let value = Self::format_jsdoc_tag_value(&tag_name, &tag);
        let type_annotation = tag
            .raw_type()
            .map(|raw_type| raw_type.raw().trim().to_string())
            .filter(|value| !value.is_empty());
        let name =
            tag.name().map(|name| name.raw().trim().to_string()).filter(|value| !value.is_empty());
        let optional = name.as_ref().map(|_| tag.optional());
        let default_value = tag.default_value().map(str::trim).filter(|value| !value.is_empty());
        let description =
            Self::format_structured_tag_description(&tag_name, &tag, type_annotation.as_deref());

        DocTag {
            tag: tag_name,
            value,
            type_annotation,
            name,
            optional,
            default_value: default_value.map(str::to_string),
            description,
        }
    }

    fn format_structured_tag_description(
        tag_name: &str,
        tag: &LazyJsdocTag<'_>,
        type_annotation: Option<&str>,
    ) -> Option<String> {
        if matches!(tag_name, "returns" | "return") {
            let raw_body = tag.raw_body().map(str::trim).filter(|value| !value.is_empty())?;
            let without_type = type_annotation
                .and_then(|type_annotation| {
                    let type_prefix = join3("{", type_annotation, "}");
                    raw_body.strip_prefix(&type_prefix).map(str::trim_start)
                })
                .unwrap_or(raw_body);
            return Self::clean_tag_description(without_type);
        }

        tag.description().and_then(Self::clean_tag_description)
    }

    fn format_jsdoc_tag_value(tag_name: &str, tag: &LazyJsdocTag<'_>) -> String {
        if !matches!(tag_name, "param" | "arg" | "argument") {
            if let Some(raw_body) = tag.raw_body().map(str::trim).filter(|value| !value.is_empty())
            {
                return raw_body.to_string();
            }
        }

        let mut parts = Vec::new();

        if let Some(raw_type) = tag.raw_type() {
            let raw_type = raw_type.raw().trim();
            if !raw_type.is_empty() {
                parts.push(join3("{", raw_type, "}"));
            }
        }

        if let Some(name) = tag.name() {
            let name = name.raw().trim();
            if !name.is_empty() {
                let name = if tag.optional() {
                    tag.default_value().map_or_else(
                        || join3("[", name, "]"),
                        |default_value| join5("[", name, "=", default_value, "]"),
                    )
                } else {
                    name.to_string()
                };
                parts.push(name);
            }
        }

        if let Some(description) =
            tag.description().map(str::trim).filter(|value| !value.is_empty())
        {
            if parts.is_empty() {
                parts.push(description.to_string());
            } else {
                parts.push(join2("- ", description));
            }
        }

        if !parts.is_empty() {
            return parts.join(" ");
        }

        if let Some(raw_body) = tag.raw_body().map(str::trim).filter(|value| !value.is_empty()) {
            return raw_body.to_string();
        }

        tag.body().map_or_else(String::new, |body| match body {
            LazyJsdocTagBody::Generic(body) => body.description().unwrap_or_default().to_string(),
            LazyJsdocTagBody::Raw(body) => body.raw().to_string(),
            LazyJsdocTagBody::Borrows(_) => String::new(),
        })
    }

    /// Fallback parser used when the external JSDoc parser cannot produce a root.
    fn parse_jsdoc_fallback(comment: &str) -> (String, Vec<DocTag>) {
        let mut description_lines = Vec::new();
        let mut tags = Vec::new();
        let mut current_tag: Option<(String, Vec<String>)> = None;

        let lines: Vec<String> = comment
            .lines()
            .map(|line| {
                let trimmed = line.trim_start();
                let trimmed = trimmed.strip_prefix('*').unwrap_or(trimmed);
                trimmed.strip_prefix(' ').unwrap_or(trimmed).trim_end().to_string()
            })
            .collect();

        for line in lines {
            let trimmed = line.trim_start();
            if let Some(without_at) = trimmed.strip_prefix('@') {
                // Save previous tag if any
                if let Some((tag, value_lines)) = current_tag.take() {
                    tags.push(DocTag::new(tag, value_lines.join("\n").trim().to_string()));
                }

                let split_at = without_at
                    .char_indices()
                    .find_map(|(index, ch)| ch.is_whitespace().then_some(index))
                    .unwrap_or(without_at.len());
                let tag_name = without_at[..split_at].to_string();
                let tag_value = without_at[split_at..].trim_start().to_string();
                current_tag = Some((tag_name, Vec::from([tag_value])));
            } else if let Some((_, ref mut value_lines)) = current_tag {
                value_lines.push(line);
            } else {
                description_lines.push(line);
            }
        }

        // Save last tag if any
        if let Some((tag, value_lines)) = current_tag {
            tags.push(DocTag::new(tag, value_lines.join("\n").trim().to_string()));
        }

        (description_lines.join("\n").trim().to_string(), tags)
    }

    fn format_type_parameter_declaration<T>(
        &self,
        type_params: Option<&oxc_allocator::Box<'a, T>>,
    ) -> String
    where
        T: oxc_span::GetSpan,
    {
        type_params
            .map(|type_params| self.slice(type_params.span().start, type_params.span().end))
            .unwrap_or_default()
    }

    /// Extracts structured type parameters (`name`, `extends` constraint, `=`
    /// default) from a declaration's type-parameter list. Descriptions are filled
    /// later from `@typeParam` tags during normalization.
    fn extract_type_parameters(
        &self,
        type_params: Option<&oxc_allocator::Box<'a, oxc_ast::ast::TSTypeParameterDeclaration<'a>>>,
    ) -> Vec<TypeParamDoc> {
        let Some(type_params) = type_params else {
            return Vec::new();
        };
        type_params
            .params
            .iter()
            .map(|param| TypeParamDoc {
                name: param.name.name.to_string(),
                constraint: param
                    .constraint
                    .as_ref()
                    .map(|constraint| self.slice(constraint.span().start, constraint.span().end)),
                default: param
                    .default
                    .as_ref()
                    .map(|default| self.slice(default.span().start, default.span().end)),
                description: String::new(),
            })
            .collect()
    }

    fn format_formal_parameters(&self, params: &oxc_ast::ast::FormalParameters<'a>) -> String {
        let mut items = params
            .items
            .iter()
            .map(|param| self.slice(param.span.start, param.span.end))
            .collect::<Vec<_>>();

        if let Some(rest) = &params.rest {
            items.push(self.slice(rest.span.start, rest.span.end));
        }

        items.join(", ")
    }

    fn format_function_signature(&self, func: &Function<'a>, name: &str, exported: bool) -> String {
        let mut sig = String::new();

        if exported {
            sig.push_str("export ");
        }
        if func.declare {
            sig.push_str("declare ");
        }
        if func.r#async {
            sig.push_str("async ");
        }
        sig.push_str("function ");
        if func.generator {
            sig.push('*');
        }
        sig.push_str(name);
        sig.push_str(&self.format_type_parameter_declaration(func.type_parameters.as_ref()));
        sig.push('(');
        sig.push_str(&self.format_formal_parameters(&func.params));
        sig.push(')');

        if let Some(return_type) = func.return_type.as_ref() {
            sig.push_str(": ");
            sig.push_str(&self.slice(
                return_type.type_annotation.span().start,
                return_type.type_annotation.span().end,
            ));
        }

        sig
    }

    fn format_assigned_function_signature(
        &self,
        name: &str,
        r#async: bool,
        type_parameters: Option<
            &oxc_allocator::Box<'a, oxc_ast::ast::TSTypeParameterDeclaration<'a>>,
        >,
        params: &oxc_ast::ast::FormalParameters<'a>,
        return_type: Option<&oxc_allocator::Box<'a, oxc_ast::ast::TSTypeAnnotation<'a>>>,
    ) -> String {
        let mut sig = String::new();
        if r#async {
            sig.push_str("async ");
        }
        sig.push_str(name);
        sig.push_str(&self.format_type_parameter_declaration(type_parameters));
        sig.push('(');
        sig.push_str(&self.format_formal_parameters(params));
        sig.push(')');

        if let Some(return_type) = return_type {
            sig.push_str(": ");
            sig.push_str(&self.slice(
                return_type.type_annotation.span().start,
                return_type.type_annotation.span().end,
            ));
        }

        sig
    }

    fn format_class_signature(&self, class: &Class, name: &str, exported: bool) -> String {
        let mut sig = String::new();
        if exported {
            sig.push_str("export ");
        }
        if class.r#abstract {
            sig.push_str("abstract ");
        }
        if class.declare {
            sig.push_str("declare ");
        }
        sig.push_str("class ");
        sig.push_str(name);
        sig.push_str(&self.format_type_parameter_declaration(class.type_parameters.as_ref()));

        if let Some(super_class) = &class.super_class {
            sig.push_str(" extends ");
            sig.push_str(&self.slice(super_class.span().start, super_class.span().end));
            if let Some(type_params) = &class.super_type_arguments {
                sig.push_str(&self.format_type_parameter_declaration(Some(type_params)));
            }
        }

        let implements = class
            .implements
            .iter()
            .map(|item| {
                let mut value = Self::format_ts_type_name(&item.expression);
                if let Some(type_params) = &item.type_arguments {
                    value.push_str(&self.format_type_parameter_declaration(Some(type_params)));
                }
                value
            })
            .collect::<Vec<_>>()
            .join(", ");

        if !implements.is_empty() {
            sig.push_str(" implements ");
            sig.push_str(&implements);
        }

        sig
    }

    fn format_interface_signature(
        &self,
        interface: &oxc_ast::ast::TSInterfaceDeclaration<'a>,
        exported: bool,
    ) -> String {
        let mut sig = String::new();
        if exported {
            sig.push_str("export ");
        }
        if interface.declare {
            sig.push_str("declare ");
        }
        sig.push_str("interface ");
        sig.push_str(interface.id.name.as_str());
        sig.push_str(&self.format_type_parameter_declaration(interface.type_parameters.as_ref()));

        let extends = interface
            .extends
            .iter()
            .map(|item| {
                let mut value =
                    self.slice(item.expression.span().start, item.expression.span().end);
                if let Some(type_params) = &item.type_arguments {
                    value.push_str(&self.format_type_parameter_declaration(Some(type_params)));
                }
                value
            })
            .collect::<Vec<_>>()
            .join(", ");

        if !extends.is_empty() {
            sig.push_str(" extends ");
            sig.push_str(&extends);
        }

        sig
    }

    fn format_type_alias_signature(
        &self,
        type_alias: &oxc_ast::ast::TSTypeAliasDeclaration<'a>,
        exported: bool,
    ) -> String {
        let mut sig = String::new();
        if exported {
            sig.push_str("export ");
        }
        if type_alias.declare {
            sig.push_str("declare ");
        }
        sig.push_str("type ");
        sig.push_str(type_alias.id.name.as_str());
        sig.push_str(&self.format_type_parameter_declaration(type_alias.type_parameters.as_ref()));
        sig.push_str(" = ");
        sig.push_str(
            &self.slice(
                type_alias.type_annotation.span().start,
                type_alias.type_annotation.span().end,
            ),
        );
        sig
    }

    fn format_variable_signature(
        &self,
        name: &str,
        exported: bool,
        decl_kind: &str,
        type_annotation: Option<&TSTypeAnnotation<'a>>,
        initializer: Option<&Expression<'a>>,
    ) -> String {
        let mut sig = String::new();
        if exported {
            sig.push_str("export ");
        }
        sig.push_str(decl_kind);
        sig.push(' ');
        sig.push_str(name);

        if let Some(type_annotation) = type_annotation {
            sig.push_str(": ");
            sig.push_str(&self.format_ts_type(&type_annotation.type_annotation));
        } else if let Some(initializer) = initializer {
            sig.push_str(" = ");
            sig.push_str(&self.slice(initializer.span().start, initializer.span().end));
        }

        sig
    }

    fn has_private_tag(tags: &[DocTag]) -> bool {
        tags.iter().any(|tag| tag.tag == "private")
    }

    fn has_internal_tag(tags: &[DocTag]) -> bool {
        tags.iter().any(|tag| tag.tag == "internal")
    }

    fn should_skip_by_visibility(&self, tags: &[DocTag]) -> bool {
        (!self.include_private && Self::has_private_tag(tags))
            || (!self.include_internal && Self::has_internal_tag(tags))
    }

    fn split_leading_jsdoc_type(value: &str) -> (Option<String>, &str) {
        let value = value.trim_start();
        let Some(rest) = value.strip_prefix('{') else {
            return (None, value);
        };

        let mut depth = 1_u32;
        for (index, ch) in rest.char_indices() {
            match ch {
                '{' => depth += 1,
                '}' => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        let type_annotation = rest[..index].trim();
                        let remaining = rest[index + ch.len_utf8()..].trim_start();
                        return (
                            (!type_annotation.is_empty()).then(|| type_annotation.to_string()),
                            remaining,
                        );
                    }
                }
                _ => {}
            }
        }

        (None, value)
    }

    fn clean_tag_description(value: &str) -> Option<String> {
        let value = value.trim();
        let value = value.strip_prefix('-').map_or(value, str::trim_start).trim();
        (!value.is_empty()).then(|| value.to_string())
    }

    fn split_name_and_description(value: &str) -> (&str, &str) {
        let value = value.trim_start();
        if let Some(rest) = value.strip_prefix('[') {
            if let Some(close_index) = rest.find(']') {
                let close_index = close_index + 2;
                return (&value[..close_index], value[close_index..].trim_start());
            }
        }

        value
            .char_indices()
            .find_map(|(index, ch)| {
                ch.is_whitespace()
                    .then_some((&value[..index], value[index + ch.len_utf8()..].trim_start()))
            })
            .unwrap_or((value, ""))
    }

    fn parse_param_tag_value(value: &str) -> Option<ParsedParamTag> {
        let (type_annotation, rest) = Self::split_leading_jsdoc_type(value);
        let (name, description) = Self::split_name_and_description(rest);
        let mut name = name.trim().to_string();
        if name.is_empty() {
            return None;
        }

        let mut optional = false;
        let mut default_value = None;
        if name.starts_with('[') && name.ends_with(']') {
            optional = true;
            let inner = name[1..name.len() - 1].to_string();
            if let Some((inner_name, inner_default)) = inner.split_once('=') {
                name = inner_name.trim().to_string();
                let inner_default = inner_default.trim();
                if !inner_default.is_empty() {
                    default_value = Some(inner_default.to_string());
                }
            } else {
                name = inner.trim().to_string();
            }
        }

        (!name.is_empty()).then(|| ParsedParamTag {
            name,
            type_annotation,
            optional,
            default_value,
            description: Self::clean_tag_description(description),
        })
    }

    fn parse_param_tag(tag: &DocTag) -> Option<ParsedParamTag> {
        if tag.name.is_none()
            && tag.type_annotation.is_none()
            && tag.default_value.is_none()
            && tag.description.is_none()
        {
            return Self::parse_param_tag_value(&tag.value);
        }

        let name = tag.name.as_ref()?.trim().to_string();
        (!name.is_empty()).then(|| ParsedParamTag {
            name,
            type_annotation: tag.type_annotation.clone(),
            optional: tag.optional.unwrap_or(false),
            default_value: tag.default_value.clone(),
            description: tag.description.clone(),
        })
    }

    /// Find the first pre-parsed `@param` tag matching `name`, using the same
    /// predicate as before (strip a leading `...`, then exact-name or
    /// dotted-prefix match). Operating on already-parsed tags avoids
    /// re-parsing every `@param` for each formal parameter.
    fn find_parsed_param_tag<'t>(
        parsed: &'t [ParsedParamTag],
        name: &str,
    ) -> Option<&'t ParsedParamTag> {
        parsed.iter().find(|tag| {
            let tag_name = tag.name.trim_start_matches("...");
            tag_name == name || tag_name.split('.').next() == Some(name)
        })
    }

    fn parse_return_tag(tag: &DocTag) -> (Option<String>, Option<String>) {
        if tag.type_annotation.is_some() || tag.description.is_some() {
            return (tag.type_annotation.clone(), tag.description.clone());
        }

        let (type_annotation, rest) = Self::split_leading_jsdoc_type(&tag.value);
        (type_annotation, Self::clean_tag_description(rest))
    }

    fn binding_pattern_name(pattern: &BindingPattern<'a>) -> String {
        match pattern {
            BindingPattern::BindingIdentifier(id) => id.name.to_string(),
            BindingPattern::AssignmentPattern(assign) => Self::binding_pattern_name(&assign.left),
            BindingPattern::ObjectPattern(_) => "param".to_string(),
            BindingPattern::ArrayPattern(_) => "param".to_string(),
        }
    }

    fn binding_pattern_default_value(&self, pattern: &BindingPattern<'a>) -> Option<String> {
        match pattern {
            BindingPattern::AssignmentPattern(assign) => {
                Some(self.slice(assign.right.span().start, assign.right.span().end))
            }
            _ => None,
        }
    }

    /// Format a binding pattern.
    fn format_binding_pattern(
        &self,
        pattern: &BindingPattern<'a>,
        optional: bool,
        type_annotation: Option<&TSTypeAnnotation<'a>>,
    ) -> String {
        match pattern {
            BindingPattern::BindingIdentifier(id) => {
                let mut s = id.name.to_string();
                if optional {
                    s.push('?');
                }
                if let Some(type_ann) = type_annotation {
                    s.push_str(": ");
                    s.push_str(&self.format_ts_type(&type_ann.type_annotation));
                }
                s
            }
            BindingPattern::ObjectPattern(_) => "{...}".to_string(),
            BindingPattern::ArrayPattern(_) => "[...]".to_string(),
            BindingPattern::AssignmentPattern(assign) => {
                self.format_binding_pattern(&assign.left, optional, type_annotation)
            }
        }
    }

    /// Format a TypeScript type.
    fn format_ts_type(&self, ts_type: &TSType) -> String {
        match ts_type {
            TSType::TSAnyKeyword(_) => "any".to_string(),
            TSType::TSBooleanKeyword(_) => "boolean".to_string(),
            TSType::TSNumberKeyword(_) => "number".to_string(),
            TSType::TSStringKeyword(_) => "string".to_string(),
            TSType::TSVoidKeyword(_) => "void".to_string(),
            TSType::TSNullKeyword(_) => "null".to_string(),
            TSType::TSUndefinedKeyword(_) => "undefined".to_string(),
            TSType::TSNeverKeyword(_) => "never".to_string(),
            TSType::TSBigIntKeyword(_) => "bigint".to_string(),
            TSType::TSSymbolKeyword(_) => "symbol".to_string(),
            TSType::TSObjectKeyword(_) => "object".to_string(),
            TSType::TSTypeReference(ref_type) => {
                self.slice(ref_type.span().start, ref_type.span().end)
            }
            TSType::TSArrayType(arr) => join2(&self.format_ts_type(&arr.element_type), "[]"),
            TSType::TSUnionType(union) => {
                let types: Vec<String> =
                    union.types.iter().map(|t| self.format_ts_type(t)).collect();
                types.join(" | ")
            }
            TSType::TSIntersectionType(inter) => {
                let types: Vec<String> =
                    inter.types.iter().map(|t| self.format_ts_type(t)).collect();
                types.join(" & ")
            }
            TSType::TSFunctionType(func) => {
                let params: Vec<String> = func
                    .params
                    .items
                    .iter()
                    .map(|p| {
                        self.format_binding_pattern(
                            &p.pattern,
                            p.optional,
                            p.type_annotation.as_deref(),
                        )
                    })
                    .collect();
                let ret = self.format_ts_type(&func.return_type.type_annotation);
                let params = params.join(", ");
                let mut out = StringBuilder::with_capacity(params.len() + ret.len() + 6);
                out.push_char('(');
                out.push_str(&params);
                out.push_str(") => ");
                out.push_str(&ret);
                out.into_string()
            }
            TSType::TSTypeLiteral(_) => "{ ... }".to_string(),
            TSType::TSTupleType(tuple) => {
                let types: Vec<String> = tuple
                    .element_types
                    .iter()
                    .map(|t| self.format_ts_type(t.to_ts_type()))
                    .collect();
                join3("[", &types.join(", "), "]")
            }
            TSType::TSLiteralType(lit) => match &lit.literal {
                oxc_ast::ast::TSLiteral::StringLiteral(s) => join3("\"", s.value.as_str(), "\""),
                oxc_ast::ast::TSLiteral::NumericLiteral(n) => n
                    .raw
                    .as_ref()
                    .map_or_else(|| n.value.to_string(), std::string::ToString::to_string),
                oxc_ast::ast::TSLiteral::BooleanLiteral(b) => b.value.to_string(),
                _ => "literal".to_string(),
            },
            _ => "unknown".to_string(),
        }
    }

    /// Format a TypeScript type name.
    fn format_ts_type_name(name: &TSTypeName) -> String {
        match name {
            TSTypeName::IdentifierReference(id) => id.name.to_string(),
            TSTypeName::QualifiedName(qn) => {
                join3(&Self::format_ts_type_name(&qn.left), ".", qn.right.name.as_str())
            }
            TSTypeName::ThisExpression(_) => "this".to_string(),
        }
    }

    fn extract_params_from_formals(
        &self,
        params: &oxc_ast::ast::FormalParameters<'a>,
        tags: &[DocTag],
    ) -> Vec<ParamDoc> {
        // Parse the `@param`/`@arg`/`@argument` tags once, in source order,
        // instead of re-filtering and re-parsing them for every formal
        // parameter (the old `find_param_tag` did O(P*T) parses).
        let parsed_param_tags: Vec<ParsedParamTag> = tags
            .iter()
            .filter(|tag| matches!(tag.tag.as_str(), "param" | "arg" | "argument"))
            .filter_map(Self::parse_param_tag)
            .collect();

        let mut docs = params
            .items
            .iter()
            .map(|param| {
                let name = Self::binding_pattern_name(&param.pattern);
                let tag = Self::find_parsed_param_tag(&parsed_param_tags, &name);
                let default_value = self
                    .binding_pattern_default_value(&param.pattern)
                    .or_else(|| {
                        param
                            .initializer
                            .as_ref()
                            .map(|init| self.slice(init.span().start, init.span().end))
                    })
                    .or_else(|| tag.and_then(|tag| tag.default_value.clone()));

                let type_annotation = param
                    .type_annotation
                    .as_ref()
                    .map(|t| self.format_ts_type(&t.type_annotation))
                    .or_else(|| tag.and_then(|tag| tag.type_annotation.clone()));

                let optional = param.optional
                    || default_value.is_some()
                    || tag.is_some_and(|tag| tag.optional);
                let description = tag.and_then(|tag| tag.description.clone());

                ParamDoc { name, type_annotation, optional, default_value, description }
            })
            .collect::<Vec<_>>();

        if let Some(rest) = params.rest.as_ref() {
            let name = Self::binding_pattern_name(&rest.rest.argument);
            let tag = Self::find_parsed_param_tag(&parsed_param_tags, &name);
            let type_annotation = rest
                .type_annotation
                .as_ref()
                .map(|t| self.format_ts_type(&t.type_annotation))
                .or_else(|| tag.and_then(|tag| tag.type_annotation.clone()));

            docs.push(ParamDoc {
                name,
                type_annotation,
                optional: tag.is_some_and(|tag| tag.optional),
                default_value: tag.and_then(|tag| tag.default_value.clone()),
                description: tag.and_then(|tag| tag.description.clone()),
            });
        }

        docs
    }

    /// Extract parameters from a function.
    fn extract_params(&self, func: &Function, tags: &[DocTag]) -> Vec<ParamDoc> {
        self.extract_params_from_formals(&func.params, tags)
    }

    fn extract_return_type_from_annotation(
        &self,
        return_type: Option<&oxc_allocator::Box<'a, oxc_ast::ast::TSTypeAnnotation<'a>>>,
        tags: &[DocTag],
    ) -> Option<String> {
        return_type.map(|r| self.format_ts_type(&r.type_annotation)).or_else(|| {
            tags.iter().find(|tag| tag.tag == "returns" || tag.tag == "return").and_then(|tag| {
                let (type_annotation, description) = Self::parse_return_tag(tag);
                type_annotation.or(description)
            })
        })
    }

    /// Extract return type from tags.
    fn extract_return_type(&self, func: &Function, tags: &[DocTag]) -> Option<String> {
        self.extract_return_type_from_annotation(func.return_type.as_ref(), tags)
    }

    /// Create a DocItem from a function.
    fn create_function_item(
        &self,
        func: &Function,
        exported: bool,
        attached_to: u32,
    ) -> Option<DocItem> {
        let name = func.id.as_ref()?.name.to_string();
        let (jsdoc, doc, tags) = self.extract_declaration_docs(attached_to)?;
        let (line, end_line) = self.span_lines(attached_to, func.span.end);

        Some(DocItem {
            name,
            kind: DocItemKind::Function,
            doc,
            source_path: self.file_path.to_string(),
            line,
            end_line,
            column: self.column_number(attached_to),
            jsdoc,
            exported,
            signature: Some(self.format_function_signature(
                func,
                func.id.as_ref()?.name.as_str(),
                exported,
            )),
            has_body: func.body.is_some(),
            optional: false,
            readonly: false,
            r#static: false,
            params: self.extract_params(func, &tags),
            return_type: self.extract_return_type(func, &tags),
            children: Vec::new(),
            tags,
            type_parameters: self.extract_type_parameters(func.type_parameters.as_ref()),
        })
    }

    /// Create a DocItem from a class.
    fn create_class_item(
        &self,
        class: &Class,
        name: &str,
        exported: bool,
        attached_to: u32,
    ) -> Option<DocItem> {
        let (jsdoc, doc, tags) = self.extract_declaration_docs(attached_to)?;
        let (line, end_line) = self.span_lines(attached_to, class.span.end);

        let mut children = Vec::new();

        // Extract class members
        for element in &class.body.body {
            match element {
                oxc_ast::ast::ClassElement::MethodDefinition(method) => {
                    let method_name = match &method.key {
                        oxc_ast::ast::PropertyKey::StaticIdentifier(id) => id.name.to_string(),
                        _ => continue,
                    };

                    let kind = match method.kind {
                        oxc_ast::ast::MethodDefinitionKind::Constructor => DocItemKind::Constructor,
                        oxc_ast::ast::MethodDefinitionKind::Get => DocItemKind::Getter,
                        oxc_ast::ast::MethodDefinitionKind::Set => DocItemKind::Setter,
                        oxc_ast::ast::MethodDefinitionKind::Method => DocItemKind::Method,
                    };

                    let (method_jsdoc, method_doc, method_tags) = self
                        .extract_jsdoc(method.span.start)
                        .map_or((None, None, Vec::new()), |(jsdoc, doc, tags)| {
                            (Some(jsdoc), (!doc.is_empty()).then_some(doc), tags)
                        });
                    if self.should_skip_by_visibility(&method_tags) {
                        continue;
                    }
                    let (method_line, method_end_line) =
                        self.span_lines(method.span.start, method.span.end);

                    children.push(DocItem {
                        name: method_name.clone(),
                        kind,
                        doc: method_doc,
                        source_path: self.file_path.to_string(),
                        line: method_line,
                        end_line: method_end_line,
                        column: self.column_number(method.span.start),
                        jsdoc: method_jsdoc,
                        exported: false,
                        signature: Some(self.format_assigned_function_signature(
                            if kind == DocItemKind::Constructor {
                                "constructor"
                            } else {
                                &method_name
                            },
                            method.value.r#async,
                            method.value.type_parameters.as_ref(),
                            &method.value.params,
                            method.value.return_type.as_ref(),
                        )),
                        has_body: false,
                        optional: method.optional,
                        readonly: false,
                        r#static: method.r#static,
                        params: self.extract_params(&method.value, &method_tags),
                        return_type: self.extract_return_type(&method.value, &method_tags),
                        children: Vec::new(),
                        tags: method_tags,
                        type_parameters: Vec::new(),
                    });
                }
                oxc_ast::ast::ClassElement::PropertyDefinition(prop) => {
                    let prop_name = match &prop.key {
                        oxc_ast::ast::PropertyKey::StaticIdentifier(id) => id.name.to_string(),
                        _ => continue,
                    };

                    let (prop_jsdoc, prop_doc, prop_tags) = self
                        .extract_jsdoc(prop.span.start)
                        .map_or((None, None, Vec::new()), |(jsdoc, doc, tags)| {
                            (Some(jsdoc), (!doc.is_empty()).then_some(doc), tags)
                        });
                    if self.should_skip_by_visibility(&prop_tags) {
                        continue;
                    }
                    let (prop_line, prop_end_line) =
                        self.span_lines(prop.span.start, prop.span.end);

                    let type_annotation = prop
                        .type_annotation
                        .as_ref()
                        .map(|t| self.format_ts_type(&t.type_annotation));

                    children.push(DocItem {
                        name: prop_name,
                        kind: DocItemKind::Property,
                        doc: prop_doc,
                        source_path: self.file_path.to_string(),
                        line: prop_line,
                        end_line: prop_end_line,
                        column: self.column_number(prop.span.start),
                        jsdoc: prop_jsdoc,
                        exported: false,
                        signature: type_annotation,
                        has_body: false,
                        optional: prop.optional,
                        readonly: prop.readonly,
                        r#static: prop.r#static,
                        params: Vec::new(),
                        return_type: None,
                        children: Vec::new(),
                        tags: prop_tags,
                        type_parameters: Vec::new(),
                    });
                }
                _ => {}
            }
        }

        Some(DocItem {
            name: name.to_string(),
            kind: DocItemKind::Class,
            doc,
            source_path: self.file_path.to_string(),
            line,
            end_line,
            column: self.column_number(attached_to),
            jsdoc,
            exported,
            signature: Some(self.format_class_signature(class, name, exported)),
            has_body: false,
            optional: false,
            readonly: false,
            r#static: false,
            params: Vec::new(),
            return_type: None,
            children,
            tags,
            type_parameters: self.extract_type_parameters(class.type_parameters.as_ref()),
        })
    }

    fn extract_ts_signature_members(&self, signatures: &[TSSignature<'a>]) -> Vec<DocItem> {
        let mut children = Vec::new();

        for sig in signatures {
            match sig {
                TSSignature::TSPropertySignature(prop) => {
                    let prop_name = match &prop.key {
                        oxc_ast::ast::PropertyKey::StaticIdentifier(id) => id.name.to_string(),
                        _ => continue,
                    };

                    let (prop_jsdoc, prop_doc, prop_tags) = self
                        .extract_jsdoc(prop.span.start)
                        .map_or((None, None, Vec::new()), |(jsdoc, doc, tags)| {
                            (Some(jsdoc), (!doc.is_empty()).then_some(doc), tags)
                        });
                    if self.should_skip_by_visibility(&prop_tags) {
                        continue;
                    }
                    let (prop_line, prop_end_line) =
                        self.span_lines(prop.span.start, prop.span.end);

                    let type_annotation = prop
                        .type_annotation
                        .as_ref()
                        .map(|t| self.format_ts_type(&t.type_annotation));

                    children.push(DocItem {
                        name: prop_name,
                        kind: DocItemKind::Property,
                        doc: prop_doc,
                        source_path: self.file_path.to_string(),
                        line: prop_line,
                        end_line: prop_end_line,
                        column: self.column_number(prop.span.start),
                        jsdoc: prop_jsdoc,
                        exported: false,
                        signature: type_annotation,
                        has_body: false,
                        optional: prop.optional,
                        readonly: prop.readonly,
                        r#static: false,
                        params: Vec::new(),
                        return_type: None,
                        children: Vec::new(),
                        tags: prop_tags,
                        type_parameters: Vec::new(),
                    });
                }
                TSSignature::TSMethodSignature(method) => {
                    let method_name = match &method.key {
                        oxc_ast::ast::PropertyKey::StaticIdentifier(id) => id.name.to_string(),
                        _ => continue,
                    };

                    let (method_jsdoc, method_doc, method_tags) = self
                        .extract_jsdoc(method.span.start)
                        .map_or((None, None, Vec::new()), |(jsdoc, doc, tags)| {
                            (Some(jsdoc), (!doc.is_empty()).then_some(doc), tags)
                        });
                    if self.should_skip_by_visibility(&method_tags) {
                        continue;
                    }
                    let (method_line, method_end_line) =
                        self.span_lines(method.span.start, method.span.end);

                    let kind = match method.kind {
                        oxc_ast::ast::TSMethodSignatureKind::Method => DocItemKind::Method,
                        oxc_ast::ast::TSMethodSignatureKind::Get => DocItemKind::Getter,
                        oxc_ast::ast::TSMethodSignatureKind::Set => DocItemKind::Setter,
                    };

                    children.push(DocItem {
                        name: method_name.clone(),
                        kind,
                        doc: method_doc,
                        source_path: self.file_path.to_string(),
                        line: method_line,
                        end_line: method_end_line,
                        column: self.column_number(method.span.start),
                        jsdoc: method_jsdoc,
                        exported: false,
                        signature: Some(self.format_assigned_function_signature(
                            &method_name,
                            false,
                            method.type_parameters.as_ref(),
                            &method.params,
                            method.return_type.as_ref(),
                        )),
                        has_body: false,
                        optional: method.optional,
                        readonly: false,
                        r#static: false,
                        params: self.extract_params_from_formals(&method.params, &method_tags),
                        return_type: self.extract_return_type_from_annotation(
                            method.return_type.as_ref(),
                            &method_tags,
                        ),
                        children: Vec::new(),
                        tags: method_tags,
                        type_parameters: Vec::new(),
                    });
                }
                _ => {}
            }
        }

        children
    }
}

impl<'a> Visit<'a> for DocVisitor<'a> {
    fn visit_statement(&mut self, stmt: &Statement<'a>) {
        match stmt {
            Statement::ExportNamedDeclaration(export) => {
                if let Some(ref decl) = export.declaration {
                    self.visit_declaration_as_exported(decl, export.span.start);
                }
            }
            Statement::ExportDefaultDeclaration(export) => {
                self.has_default_export = true;
                match &export.declaration {
                    ExportDefaultDeclarationKind::FunctionDeclaration(func) => {
                        if let Some(item) = self.create_function_item(func, true, export.span.start)
                        {
                            self.items.push(item);
                        }
                    }
                    ExportDefaultDeclarationKind::ClassDeclaration(class) => {
                        let name = class
                            .id
                            .as_ref()
                            .map_or_else(|| "default".to_string(), |id| id.name.to_string());
                        if let Some(item) =
                            self.create_class_item(class, &name, true, export.span.start)
                        {
                            self.items.push(item);
                        }
                    }
                    _ => {}
                }
            }
            _ => {
                walk::walk_statement(self, stmt);
            }
        }
    }

    fn visit_declaration(&mut self, decl: &Declaration<'a>) {
        // Only visit non-exported declarations
        self.visit_declaration_internal(decl, false, decl.span().start);
    }
}

impl<'a> DocVisitor<'a> {
    fn visit_declaration_as_exported(&mut self, decl: &Declaration<'a>, attached_to: u32) {
        self.visit_declaration_internal(decl, true, attached_to);
    }

    fn visit_declaration_internal(
        &mut self,
        decl: &Declaration<'a>,
        exported: bool,
        attached_to: u32,
    ) {
        match decl {
            Declaration::FunctionDeclaration(func) => {
                if let Some(item) = self.create_function_item(func, exported, attached_to) {
                    self.items.push(item);
                }
            }
            Declaration::ClassDeclaration(class) => {
                if let Some(id) = &class.id {
                    let name = id.name.to_string();
                    if let Some(item) = self.create_class_item(class, &name, exported, attached_to)
                    {
                        self.items.push(item);
                    }
                }
            }
            Declaration::VariableDeclaration(var_decl) => {
                self.visit_variable_declaration(var_decl, exported, attached_to);
            }
            Declaration::TSTypeAliasDeclaration(type_alias) => {
                self.visit_type_alias_declaration(type_alias, exported, attached_to);
            }
            Declaration::TSInterfaceDeclaration(interface) => {
                self.visit_interface_declaration(interface, exported, attached_to);
            }
            Declaration::TSEnumDeclaration(enum_decl) => {
                self.visit_enum_declaration(enum_decl, exported, attached_to);
            }
            _ => {}
        }
    }

    fn visit_variable_declaration(
        &mut self,
        var_decl: &VariableDeclaration<'a>,
        exported: bool,
        attached_to: u32,
    ) {
        let Some((jsdoc, doc, tags)) = self.extract_declaration_docs(attached_to) else {
            return;
        };
        let (line, end_line) = self.span_lines(attached_to, var_decl.span.end);

        for declarator in &var_decl.declarations {
            let BindingPattern::BindingIdentifier(id) = &declarator.id else {
                continue;
            };
            let Some(initializer) = &declarator.init else {
                continue;
            };

            let name = id.name.to_string();
            let item = match initializer {
                Expression::ArrowFunctionExpression(arrow) => DocItem {
                    name: name.clone(),
                    kind: DocItemKind::Function,
                    doc: doc.clone(),
                    source_path: self.file_path.to_string(),
                    line,
                    end_line,
                    column: self.column_number(attached_to),
                    jsdoc: jsdoc.clone(),
                    exported,
                    signature: Some(self.format_assigned_function_signature(
                        &name,
                        arrow.r#async,
                        arrow.type_parameters.as_ref(),
                        &arrow.params,
                        arrow.return_type.as_ref(),
                    )),
                    has_body: false,
                    optional: false,
                    readonly: false,
                    r#static: false,
                    params: self.extract_params_from_formals(&arrow.params, &tags),
                    return_type: self
                        .extract_return_type_from_annotation(arrow.return_type.as_ref(), &tags),
                    children: Vec::new(),
                    tags: tags.clone(),
                    type_parameters: self.extract_type_parameters(arrow.type_parameters.as_ref()),
                },
                Expression::FunctionExpression(func_expr) => DocItem {
                    name: name.clone(),
                    kind: DocItemKind::Function,
                    doc: doc.clone(),
                    source_path: self.file_path.to_string(),
                    line,
                    end_line,
                    column: self.column_number(attached_to),
                    jsdoc: jsdoc.clone(),
                    exported,
                    signature: Some(self.format_assigned_function_signature(
                        &name,
                        func_expr.r#async,
                        func_expr.type_parameters.as_ref(),
                        &func_expr.params,
                        func_expr.return_type.as_ref(),
                    )),
                    has_body: false,
                    optional: false,
                    readonly: false,
                    r#static: false,
                    params: self.extract_params(func_expr, &tags),
                    return_type: self.extract_return_type(func_expr, &tags),
                    children: Vec::new(),
                    tags: tags.clone(),
                    type_parameters: self
                        .extract_type_parameters(func_expr.type_parameters.as_ref()),
                },
                other => DocItem {
                    name: name.clone(),
                    kind: DocItemKind::Variable,
                    doc: doc.clone(),
                    source_path: self.file_path.to_string(),
                    line,
                    end_line,
                    column: self.column_number(attached_to),
                    jsdoc: jsdoc.clone(),
                    exported,
                    signature: Some(self.format_variable_signature(
                        &name,
                        exported,
                        var_decl.kind.as_str(),
                        declarator.type_annotation.as_deref(),
                        Some(other),
                    )),
                    has_body: false,
                    optional: false,
                    readonly: false,
                    r#static: false,
                    params: Vec::new(),
                    return_type: None,
                    children: Vec::new(),
                    tags: tags.clone(),
                    type_parameters: Vec::new(),
                },
            };
            self.items.push(item);
        }
    }

    fn visit_type_alias_declaration(
        &mut self,
        type_alias: &TSTypeAliasDeclaration<'a>,
        exported: bool,
        attached_to: u32,
    ) {
        let Some((jsdoc, doc, tags)) = self.extract_declaration_docs(attached_to) else {
            return;
        };
        let (line, end_line) = self.span_lines(attached_to, type_alias.span.end);
        let children = match &type_alias.type_annotation {
            TSType::TSTypeLiteral(type_literal) => {
                self.extract_ts_signature_members(&type_literal.members)
            }
            _ => Vec::new(),
        };

        self.items.push(DocItem {
            name: type_alias.id.name.to_string(),
            kind: DocItemKind::Type,
            doc,
            source_path: self.file_path.to_string(),
            line,
            end_line,
            column: self.column_number(attached_to),
            jsdoc,
            exported,
            signature: Some(self.format_type_alias_signature(type_alias, exported)),
            has_body: false,
            optional: false,
            readonly: false,
            r#static: false,
            params: Vec::new(),
            return_type: None,
            children,
            tags,
            type_parameters: self.extract_type_parameters(type_alias.type_parameters.as_ref()),
        });
    }

    fn visit_interface_declaration(
        &mut self,
        interface: &TSInterfaceDeclaration<'a>,
        exported: bool,
        attached_to: u32,
    ) {
        let Some((jsdoc, doc, tags)) = self.extract_declaration_docs(attached_to) else {
            return;
        };
        let (line, end_line) = self.span_lines(attached_to, interface.span.end);
        let children = self.extract_ts_signature_members(&interface.body.body);

        self.items.push(DocItem {
            name: interface.id.name.to_string(),
            kind: DocItemKind::Interface,
            doc,
            source_path: self.file_path.to_string(),
            line,
            end_line,
            column: self.column_number(attached_to),
            jsdoc,
            exported,
            signature: Some(self.format_interface_signature(interface, exported)),
            has_body: false,
            optional: false,
            readonly: false,
            r#static: false,
            params: Vec::new(),
            return_type: None,
            children,
            tags,
            type_parameters: self.extract_type_parameters(interface.type_parameters.as_ref()),
        });
    }

    fn visit_enum_declaration(
        &mut self,
        enum_decl: &TSEnumDeclaration<'a>,
        exported: bool,
        attached_to: u32,
    ) {
        let Some((jsdoc, doc, tags)) = self.extract_declaration_docs(attached_to) else {
            return;
        };
        let (line, end_line) = self.span_lines(attached_to, enum_decl.span.end);
        let children = enum_decl
            .body
            .members
            .iter()
            .filter_map(|member| self.create_enum_member_item(member))
            .collect();

        self.items.push(DocItem {
            name: enum_decl.id.name.to_string(),
            kind: DocItemKind::Enum,
            doc,
            source_path: self.file_path.to_string(),
            line,
            end_line,
            column: self.column_number(attached_to),
            jsdoc,
            exported,
            signature: None,
            has_body: false,
            optional: false,
            readonly: false,
            r#static: false,
            params: Vec::new(),
            return_type: None,
            children,
            tags,
            type_parameters: Vec::new(),
        });
    }

    fn create_enum_member_item(&self, member: &TSEnumMember<'a>) -> Option<DocItem> {
        let member_name = match &member.id {
            oxc_ast::ast::TSEnumMemberName::Identifier(id) => id.name.to_string(),
            oxc_ast::ast::TSEnumMemberName::String(s) => s.value.to_string(),
            oxc_ast::ast::TSEnumMemberName::ComputedString(s) => s.value.to_string(),
            oxc_ast::ast::TSEnumMemberName::ComputedTemplateString(template) => {
                self.slice(template.span.start, template.span.end)
            }
        };
        let (member_jsdoc, member_doc, member_tags) = self
            .extract_jsdoc(member.span.start)
            .map_or((None, None, Vec::new()), |(jsdoc, doc, tags)| {
                (Some(jsdoc), (!doc.is_empty()).then_some(doc), tags)
            });
        if self.should_skip_by_visibility(&member_tags) {
            return None;
        }
        let (line, end_line) = self.span_lines(member.span.start, member.span.end);

        Some(DocItem {
            name: member_name,
            kind: DocItemKind::EnumMember,
            doc: member_doc,
            source_path: self.file_path.to_string(),
            line,
            end_line,
            column: self.column_number(member.span.start),
            jsdoc: member_jsdoc,
            exported: false,
            signature: member
                .initializer
                .as_ref()
                .map(|initializer| self.slice(initializer.span().start, initializer.span().end)),
            has_body: false,
            optional: false,
            readonly: false,
            r#static: false,
            params: Vec::new(),
            return_type: None,
            children: Vec::new(),
            tags: member_tags,
            type_parameters: Vec::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_overloaded_function_marks_only_implementation_has_body() {
        let source = r"
/**
 * Define a plugin with extension.
 */
export function plugin<E>(options: WithExt): WithExtResult;
/**
 * Define a plugin without extension.
 */
export function plugin(options: WithoutExt): WithoutExtResult;
/**
 * Define a plugin.
 */
export function plugin(options: any = {}): any {
    return options;
}
";

        let extractor = DocExtractor::new();
        let items = extractor.extract_source(source, "plugin.ts", SourceType::ts()).unwrap();
        let plugins = items.iter().filter(|item| item.name == "plugin").collect::<Vec<_>>();

        assert_eq!(plugins.len(), 3);
        // Overload signatures carry no body; only the implementation does.
        assert!(!plugins[0].has_body);
        assert!(!plugins[1].has_body);
        assert!(plugins[2].has_body);
    }

    #[test]
    fn test_extract_function() {
        let source = r"
/**
 * Adds two numbers together.
 * @param a - The first number
 * @param b - The second number
 * @returns The sum of a and b
 */
export function add(a: number, b: number): number {
    return a + b;
}
";

        let extractor = DocExtractor::new();
        let items = extractor.extract_source(source, "test.ts", SourceType::ts()).unwrap();

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].name, "add");
        assert_eq!(items[0].kind, DocItemKind::Function);
        assert!(items[0].exported);
        assert!(items[0].doc.as_ref().unwrap().contains("Adds two numbers"));
        assert_eq!(items[0].params.len(), 2);
    }

    #[test]
    fn test_extract_interface() {
        let source = r"
/**
 * User interface.
 */
export interface User {
    /** User's name */
    name: string;
    /** User's age */
    age: number;
}
";

        let extractor = DocExtractor::new();
        let items = extractor.extract_source(source, "test.ts", SourceType::ts()).unwrap();

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].name, "User");
        assert_eq!(items[0].kind, DocItemKind::Interface);
        assert_eq!(items[0].children.len(), 2);
    }

    #[test]
    fn type_alias_object_literal_emits_property_children() {
        let source = r"
/**
 * Command options.
 */
export type CommandOptions = {
    name: string;
    aliases?: string[];
};
";

        let extractor = DocExtractor::new();
        let items = extractor.extract_source(source, "options.ts", SourceType::ts()).unwrap();

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].name, "CommandOptions");
        assert_eq!(items[0].children.len(), 2);
        assert_eq!(items[0].children[0].name, "name");
        assert_eq!(items[0].children[0].kind, DocItemKind::Property);
        assert_eq!(items[0].children[0].signature.as_deref(), Some("string"));
        assert!(!items[0].children[0].optional);
        assert_eq!(items[0].children[1].name, "aliases");
        assert_eq!(items[0].children[1].signature.as_deref(), Some("string[]"));
        assert!(items[0].children[1].optional);
    }

    #[test]
    fn type_alias_object_literal_with_method_signature() {
        let source = r"
/**
 * Command options.
 */
export type CommandOptions = {
    /**
     * Runs the command.
     * @param ctx - Runtime context
     */
    run(ctx: Context): void;
};
";

        let extractor = DocExtractor::new();
        let items = extractor.extract_source(source, "options.ts", SourceType::ts()).unwrap();
        let member = &items[0].children[0];

        assert_eq!(member.name, "run");
        assert_eq!(member.kind, DocItemKind::Method);
        assert_eq!(member.signature.as_deref(), Some("run(ctx: Context): void"));
        assert_eq!(member.params.len(), 1);
        assert_eq!(member.params[0].description.as_deref(), Some("Runtime context"));
    }

    #[test]
    fn type_alias_intersection_falls_back_to_signature_only() {
        let source = r"
/**
 * Command options.
 */
export type CommandOptions = BaseOptions & {
    /** Command name. */
    name: string;
};
";

        let extractor = DocExtractor::new();
        let items = extractor.extract_source(source, "options.ts", SourceType::ts()).unwrap();

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].name, "CommandOptions");
        assert!(items[0].children.is_empty());
        assert!(items[0].signature.as_deref().unwrap().contains("BaseOptions &"));
    }

    #[test]
    fn test_extract_jsdoc_types_from_javascript() {
        let source = r"
/**
 * Creates a user-facing label.
 *
 * @param {string} value - The label source
 * @param {number} [maxLength=20] - Maximum length before truncation
 * @returns {string} Formatted label
 */
export function label(value, maxLength = 20) {
    return value.slice(0, maxLength);
}
";

        let extractor = DocExtractor::new();
        let items = extractor.extract_source(source, "test.js", SourceType::mjs()).unwrap();

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].name, "label");
        assert_eq!(items[0].doc.as_deref(), Some("Creates a user-facing label."));
        assert_eq!(items[0].return_type.as_deref(), Some("string"));
        assert_eq!(items[0].params.len(), 2);
        assert_eq!(items[0].params[0].type_annotation.as_deref(), Some("string"));
        assert_eq!(items[0].params[0].description.as_deref(), Some("The label source"));
        assert_eq!(items[0].params[1].type_annotation.as_deref(), Some("number"));
        assert!(items[0].params[1].optional);
        assert_eq!(items[0].params[1].default_value.as_deref(), Some("20"));
        assert_eq!(
            items[0].params[1].description.as_deref(),
            Some("Maximum length before truncation")
        );

        let value_tag = items[0]
            .tags
            .iter()
            .find(|tag| tag.tag == "param" && tag.name.as_deref() == Some("value"))
            .unwrap();
        assert_eq!(value_tag.type_annotation.as_deref(), Some("string"));
        assert_eq!(value_tag.description.as_deref(), Some("The label source"));

        let max_length_tag = items[0]
            .tags
            .iter()
            .find(|tag| tag.tag == "param" && tag.name.as_deref() == Some("maxLength"))
            .unwrap();
        assert_eq!(max_length_tag.type_annotation.as_deref(), Some("number"));
        assert_eq!(max_length_tag.optional, Some(true));
        assert_eq!(max_length_tag.default_value.as_deref(), Some("20"));

        let returns_tag = items[0].tags.iter().find(|tag| tag.tag == "returns").unwrap();
        assert_eq!(returns_tag.type_annotation.as_deref(), Some("string"));
        assert_eq!(returns_tag.description.as_deref(), Some("Formatted label"));
    }

    #[test]
    fn test_extract_plain_top_level_variable() {
        let source = r"
/** Default placeholder when a command has no explicit name. */
export const ANONYMOUS_COMMAND_NAME = '__anonymous__';

/** Default retry count. */
export let retries: number = 3;

/** Creates labels. */
export const label = (value: string): string => value;
";

        let extractor = DocExtractor::new();
        let items = extractor.extract_source(source, "constants.ts", SourceType::ts()).unwrap();

        assert_eq!(items.len(), 3);
        assert_eq!(items[0].name, "ANONYMOUS_COMMAND_NAME");
        assert_eq!(items[0].kind, DocItemKind::Variable);
        assert_eq!(
            items[0].signature.as_deref(),
            Some("export const ANONYMOUS_COMMAND_NAME = '__anonymous__'")
        );
        assert_eq!(items[1].name, "retries");
        assert_eq!(items[1].kind, DocItemKind::Variable);
        assert_eq!(items[1].signature.as_deref(), Some("export let retries: number"));
        assert_eq!(items[2].name, "label");
        assert_eq!(items[2].kind, DocItemKind::Function);
    }

    #[test]
    fn test_undocumented_top_level_variable_is_skipped_by_default() {
        let source = "export const ANONYMOUS_COMMAND_NAME = '(anonymous)';";

        let items =
            DocExtractor::new().extract_source(source, "constants.ts", SourceType::ts()).unwrap();

        assert!(items.is_empty());
    }

    #[test]
    fn test_extract_file_level_module_jsdoc() {
        let source = r"
/**
 * @module default
 *
 * Main entry point for the framework.
 */
export { cli } from './core';

/** Runs the CLI. */
export function cli(): void {}
";

        let extractor = DocExtractor::new();
        let items = extractor.extract_source(source, "src/index.ts", SourceType::ts()).unwrap();

        assert_eq!(items.len(), 2);
        assert_eq!(items[0].name, "default");
        assert_eq!(items[0].kind, DocItemKind::Module);
        assert_eq!(items[0].doc.as_deref(), Some("Main entry point for the framework."));
        assert!(items[0].tags.iter().any(|tag| tag.tag == "module"));
        assert_eq!(items[1].name, "cli");
    }

    #[test]
    fn test_extract_function_type_parameters() {
        let source = r"
/** Make a thing. */
export function make<G extends Base = Default, V>(value: V): G {
  return value as unknown as G;
}
";
        let extractor = DocExtractor::new();
        let items = extractor.extract_source(source, "src/make.ts", SourceType::ts()).unwrap();
        let func = items.iter().find(|item| item.name == "make").unwrap();

        assert_eq!(func.type_parameters.len(), 2);
        assert_eq!(func.type_parameters[0].name, "G");
        assert_eq!(func.type_parameters[0].constraint.as_deref(), Some("Base"));
        assert_eq!(func.type_parameters[0].default.as_deref(), Some("Default"));
        assert_eq!(func.type_parameters[1].name, "V");
        assert_eq!(func.type_parameters[1].constraint, None);
        assert_eq!(func.type_parameters[1].default, None);
    }

    #[test]
    fn test_module_description_survives_trailing_author_comment() {
        // Regression: `@module` block immediately followed by a second leading
        // block comment (`@author`/`@license`). Both comments attach to the same
        // first statement, so an `attached_to`-keyed lookup would surface the
        // second comment and drop the `@module` description.
        let source = r"
/**
 * Module summary.
 * @module
 */

/**
 * @author kazupon
 * @license MIT
 */
export const z = 1;
";

        let extractor = DocExtractor::new();
        let items = extractor.extract_source(source, "src/context.ts", SourceType::ts()).unwrap();
        let module = items.iter().find(|item| item.kind == DocItemKind::Module).unwrap();

        assert_eq!(module.name, "context");
        assert_eq!(module.doc.as_deref(), Some("Module summary."));
        assert!(module.tags.iter().any(|tag| tag.tag == "module"));
    }

    #[test]
    fn test_module_description_from_detached_comment_without_module_tag() {
        // Gap 1: a leading file comment without `@module`, separated from the code
        // by a blank line, should still be used as the module description
        // (matching TypeDoc). The file stem becomes the module name.
        let source = r"
/**
 * The entry point for AI agent detection utility.
 *
 * @author kazupon
 * @license MIT
 */

import { agentInfo } from 'std-env';

/** A profile. */
export function getAgentProfile(): void {}
";

        let extractor = DocExtractor::new();
        let items = extractor.extract_source(source, "src/agent.ts", SourceType::ts()).unwrap();
        let module = items.iter().find(|item| item.kind == DocItemKind::Module).unwrap();

        assert_eq!(module.name, "agent");
        assert_eq!(module.doc.as_deref(), Some("The entry point for AI agent detection utility."));
        // The real declaration is still extracted with its own doc.
        let func = items.iter().find(|item| item.name == "getAgentProfile").unwrap();
        assert_eq!(func.doc.as_deref(), Some("A profile."));
    }

    #[test]
    fn test_leading_comment_attached_to_declaration_is_not_a_module() {
        // A doc comment that directly precedes the first declaration (no blank
        // line, no module marker) documents that declaration, not the module.
        let source = r"
/** Documents foo. */
export function foo(): void {}
";

        let extractor = DocExtractor::new();
        let items = extractor.extract_source(source, "src/foo.ts", SourceType::ts()).unwrap();

        assert!(items.iter().all(|item| item.kind != DocItemKind::Module));
        let func = items.iter().find(|item| item.name == "foo").unwrap();
        assert_eq!(func.doc.as_deref(), Some("Documents foo."));
    }

    #[test]
    fn test_module_jsdoc_name_falls_back_to_file_stem() {
        let source = r"
/**
 * @module
 */
export { value } from './value';
";

        let extractor = DocExtractor::new();
        let items = extractor.extract_source(source, "src/runtime.ts", SourceType::ts()).unwrap();

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].name, "runtime");
        assert_eq!(items[0].kind, DocItemKind::Module);
    }

    #[test]
    fn test_internal_items_are_excluded_by_default() {
        let source = r"
/** Public command. */
export function publicCommand(): void {}

/**
 * Internal helper.
 * @internal
 */
export function internalHelper(): void {}
";

        let public_only =
            DocExtractor::new().extract_source(source, "visibility.ts", SourceType::ts()).unwrap();
        assert_eq!(public_only.len(), 1);
        assert_eq!(public_only[0].name, "publicCommand");

        let with_internal = DocExtractor::with_visibility(false, true)
            .extract_source(source, "visibility.ts", SourceType::ts())
            .unwrap();
        assert_eq!(with_internal.len(), 2);
        assert_eq!(with_internal[1].name, "internalHelper");
    }
}
