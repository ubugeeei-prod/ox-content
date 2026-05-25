//! Documentation extraction from source code using OXC parser.

use ox_jsdoc::decoder::nodes::comment_ast::{LazyJsdocTag, LazyJsdocTagBody};
use ox_jsdoc::parser::{
    parse_batch_to_bytes as parse_jsdoc_batch_to_bytes, BatchItem as JsdocBatchItem,
    ParseOptions as JsdocParseOptions,
};
use oxc_allocator::Allocator;
use oxc_ast::ast::{
    BindingPattern, Class, Comment, Declaration, ExportDefaultDeclarationKind, Expression,
    Function, Statement, TSSignature, TSType, TSTypeAnnotation, TSTypeName,
};
use oxc_ast_visit::{walk, Visit};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use rustc_hash::{FxBuildHasher, FxHashMap, FxHashSet};
use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;

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
    /// Parameters (for functions/methods).
    pub params: Vec<ParamDoc>,
    /// Return type (for functions/methods).
    pub return_type: Option<String>,
    /// Child items (for classes, modules, etc.).
    pub children: Vec<DocItem>,
    /// JSDoc tags.
    pub tags: Vec<DocTag>,
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
}

/// Documentation extractor.
pub struct DocExtractor {
    /// Include private items.
    include_private: bool,
}

impl DocExtractor {
    /// Creates a new documentation extractor.
    #[must_use]
    pub fn new() -> Self {
        Self { include_private: false }
    }

    /// Creates a new extractor that includes private items.
    #[must_use]
    pub fn with_private(include_private: bool) -> Self {
        Self { include_private }
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

        let mut visitor = DocVisitor::new(source, file_path, self.include_private, jsdoc_cache);
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

/// Extract the JSDoc content body from a comment with a single allocation.
fn extract_raw_jsdoc(comment: &Comment, source: &str) -> String {
    let content = comment.content_span().source_text(source);
    let trimmed = content.strip_prefix('*').unwrap_or(content);
    trimmed.trim_matches('\n').to_string()
}

/// Pre-parse every JSDoc comment in the program with a single batch call so the
/// AST visitor can resolve documentation by `attached_to` via a cheap lookup.
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
        jsdoc_cache: FxHashMap<u32, ParsedJsdoc>,
    ) -> Self {
        let mut line_starts = vec![0];
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
                    raw_body.strip_prefix(&format!("{{{type_annotation}}}")).map(str::trim_start)
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
                parts.push(format!("{{{raw_type}}}"));
            }
        }

        if let Some(name) = tag.name() {
            let name = name.raw().trim();
            if !name.is_empty() {
                let name = if tag.optional() {
                    tag.default_value().map_or_else(
                        || format!("[{name}]"),
                        |default_value| format!("[{name}={default_value}]"),
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
                parts.push(format!("- {description}"));
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
                current_tag = Some((tag_name, vec![tag_value]));
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

    fn find_param_tag(tags: &[DocTag], name: &str) -> Option<ParsedParamTag> {
        tags.iter()
            .filter(|tag| matches!(tag.tag.as_str(), "param" | "arg" | "argument"))
            .filter_map(Self::parse_param_tag)
            .find(|tag| {
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
            TSType::TSTypeReference(ref_type) => Self::format_ts_type_name(&ref_type.type_name),
            TSType::TSArrayType(arr) => format!("{}[]", self.format_ts_type(&arr.element_type)),
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
                format!("({}) => {}", params.join(", "), ret)
            }
            TSType::TSTypeLiteral(_) => "{ ... }".to_string(),
            TSType::TSTupleType(tuple) => {
                let types: Vec<String> = tuple
                    .element_types
                    .iter()
                    .map(|t| self.format_ts_type(t.to_ts_type()))
                    .collect();
                format!("[{}]", types.join(", "))
            }
            TSType::TSLiteralType(lit) => match &lit.literal {
                oxc_ast::ast::TSLiteral::StringLiteral(s) => format!("\"{}\"", s.value),
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
                format!("{}.{}", Self::format_ts_type_name(&qn.left), qn.right.name)
            }
            TSTypeName::ThisExpression(_) => "this".to_string(),
        }
    }

    fn extract_params_from_formals(
        &self,
        params: &oxc_ast::ast::FormalParameters<'a>,
        tags: &[DocTag],
    ) -> Vec<ParamDoc> {
        let mut docs = params
            .items
            .iter()
            .map(|param| {
                let name = Self::binding_pattern_name(&param.pattern);
                let tag = Self::find_param_tag(tags, &name);
                let default_value = self
                    .binding_pattern_default_value(&param.pattern)
                    .or_else(|| {
                        param
                            .initializer
                            .as_ref()
                            .map(|init| self.slice(init.span().start, init.span().end))
                    })
                    .or_else(|| tag.as_ref().and_then(|tag| tag.default_value.clone()));

                let type_annotation = param
                    .type_annotation
                    .as_ref()
                    .map(|t| self.format_ts_type(&t.type_annotation))
                    .or_else(|| tag.as_ref().and_then(|tag| tag.type_annotation.clone()));

                let optional = param.optional
                    || default_value.is_some()
                    || tag.as_ref().is_some_and(|tag| tag.optional);
                let description = tag.and_then(|tag| tag.description);

                ParamDoc { name, type_annotation, optional, default_value, description }
            })
            .collect::<Vec<_>>();

        if let Some(rest) = params.rest.as_ref() {
            let name = Self::binding_pattern_name(&rest.rest.argument);
            let tag = Self::find_param_tag(tags, &name);
            let type_annotation = rest
                .type_annotation
                .as_ref()
                .map(|t| self.format_ts_type(&t.type_annotation))
                .or_else(|| tag.as_ref().and_then(|tag| tag.type_annotation.clone()));

            docs.push(ParamDoc {
                name,
                type_annotation,
                optional: tag.as_ref().is_some_and(|tag| tag.optional),
                default_value: tag.as_ref().and_then(|tag| tag.default_value.clone()),
                description: tag.and_then(|tag| tag.description),
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
        let (jsdoc, doc, tags) = self.extract_jsdoc(attached_to)?;
        if !self.include_private && Self::has_private_tag(&tags) {
            return None;
        }
        let (line, end_line) = self.span_lines(attached_to, func.span.end);

        Some(DocItem {
            name,
            kind: DocItemKind::Function,
            doc: if doc.is_empty() { None } else { Some(doc) },
            source_path: self.file_path.to_string(),
            line,
            end_line,
            column: self.column_number(attached_to),
            jsdoc: Some(jsdoc),
            exported,
            signature: Some(self.format_function_signature(
                func,
                func.id.as_ref()?.name.as_str(),
                exported,
            )),
            params: self.extract_params(func, &tags),
            return_type: self.extract_return_type(func, &tags),
            children: Vec::new(),
            tags,
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
        let (jsdoc, doc, tags) = self.extract_jsdoc(attached_to)?;
        if !self.include_private && Self::has_private_tag(&tags) {
            return None;
        }
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

                    let Some((method_jsdoc, method_doc, method_tags)) =
                        self.extract_jsdoc(method.span.start)
                    else {
                        continue;
                    };
                    if !self.include_private && Self::has_private_tag(&method_tags) {
                        continue;
                    }
                    let (method_line, method_end_line) =
                        self.span_lines(method.span.start, method.span.end);

                    children.push(DocItem {
                        name: method_name,
                        kind,
                        doc: if method_doc.is_empty() { None } else { Some(method_doc) },
                        source_path: self.file_path.to_string(),
                        line: method_line,
                        end_line: method_end_line,
                        column: self.column_number(method.span.start),
                        jsdoc: Some(method_jsdoc),
                        exported: false,
                        signature: Some(self.format_assigned_function_signature(
                            "",
                            method.value.r#async,
                            method.value.type_parameters.as_ref(),
                            &method.value.params,
                            method.value.return_type.as_ref(),
                        )),
                        params: self.extract_params(&method.value, &method_tags),
                        return_type: self.extract_return_type(&method.value, &method_tags),
                        children: Vec::new(),
                        tags: method_tags,
                    });
                }
                oxc_ast::ast::ClassElement::PropertyDefinition(prop) => {
                    let prop_name = match &prop.key {
                        oxc_ast::ast::PropertyKey::StaticIdentifier(id) => id.name.to_string(),
                        _ => continue,
                    };

                    let Some((prop_jsdoc, prop_doc, prop_tags)) =
                        self.extract_jsdoc(prop.span.start)
                    else {
                        continue;
                    };
                    if !self.include_private && Self::has_private_tag(&prop_tags) {
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
                        doc: if prop_doc.is_empty() { None } else { Some(prop_doc) },
                        source_path: self.file_path.to_string(),
                        line: prop_line,
                        end_line: prop_end_line,
                        column: self.column_number(prop.span.start),
                        jsdoc: Some(prop_jsdoc),
                        exported: false,
                        signature: type_annotation,
                        params: Vec::new(),
                        return_type: None,
                        children: Vec::new(),
                        tags: prop_tags,
                    });
                }
                _ => {}
            }
        }

        Some(DocItem {
            name: name.to_string(),
            kind: DocItemKind::Class,
            doc: if doc.is_empty() { None } else { Some(doc) },
            source_path: self.file_path.to_string(),
            line,
            end_line,
            column: self.column_number(attached_to),
            jsdoc: Some(jsdoc),
            exported,
            signature: Some(self.format_class_signature(class, name, exported)),
            params: Vec::new(),
            return_type: None,
            children,
            tags,
        })
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
                let Some((jsdoc, doc, tags)) = self.extract_jsdoc(attached_to) else {
                    return;
                };
                if !self.include_private && Self::has_private_tag(&tags) {
                    return;
                }
                let (line, end_line) = self.span_lines(attached_to, var_decl.span.end);

                for declarator in &var_decl.declarations {
                    if let BindingPattern::BindingIdentifier(id) = &declarator.id {
                        let name = id.name.to_string();

                        let Some(initializer) = &declarator.init else {
                            continue;
                        };

                        match initializer {
                            Expression::ArrowFunctionExpression(arrow) => {
                                self.items.push(DocItem {
                                    name: name.clone(),
                                    kind: DocItemKind::Function,
                                    doc: if doc.is_empty() { None } else { Some(doc.clone()) },
                                    source_path: self.file_path.to_string(),
                                    line,
                                    end_line,
                                    column: self.column_number(attached_to),
                                    jsdoc: Some(jsdoc.clone()),
                                    exported,
                                    signature: Some(self.format_assigned_function_signature(
                                        &name,
                                        arrow.r#async,
                                        arrow.type_parameters.as_ref(),
                                        &arrow.params,
                                        arrow.return_type.as_ref(),
                                    )),
                                    params: self.extract_params_from_formals(&arrow.params, &tags),
                                    return_type: self.extract_return_type_from_annotation(
                                        arrow.return_type.as_ref(),
                                        &tags,
                                    ),
                                    children: Vec::new(),
                                    tags: tags.clone(),
                                });
                            }
                            Expression::FunctionExpression(func_expr) => {
                                self.items.push(DocItem {
                                    name: name.clone(),
                                    kind: DocItemKind::Function,
                                    doc: if doc.is_empty() { None } else { Some(doc.clone()) },
                                    source_path: self.file_path.to_string(),
                                    line,
                                    end_line,
                                    column: self.column_number(attached_to),
                                    jsdoc: Some(jsdoc.clone()),
                                    exported,
                                    signature: Some(self.format_assigned_function_signature(
                                        &name,
                                        func_expr.r#async,
                                        func_expr.type_parameters.as_ref(),
                                        &func_expr.params,
                                        func_expr.return_type.as_ref(),
                                    )),
                                    params: self.extract_params(func_expr, &tags),
                                    return_type: self.extract_return_type(func_expr, &tags),
                                    children: Vec::new(),
                                    tags: tags.clone(),
                                });
                            }
                            other => {
                                self.items.push(DocItem {
                                    name: name.clone(),
                                    kind: DocItemKind::Variable,
                                    doc: if doc.is_empty() { None } else { Some(doc.clone()) },
                                    source_path: self.file_path.to_string(),
                                    line,
                                    end_line,
                                    column: self.column_number(attached_to),
                                    jsdoc: Some(jsdoc.clone()),
                                    exported,
                                    signature: Some(self.format_variable_signature(
                                        &name,
                                        exported,
                                        var_decl.kind.as_str(),
                                        declarator.type_annotation.as_deref(),
                                        Some(other),
                                    )),
                                    params: Vec::new(),
                                    return_type: None,
                                    children: Vec::new(),
                                    tags: tags.clone(),
                                });
                            }
                        }
                    }
                }
            }
            Declaration::TSTypeAliasDeclaration(type_alias) => {
                let Some((jsdoc, doc, tags)) = self.extract_jsdoc(attached_to) else {
                    return;
                };
                if !self.include_private && Self::has_private_tag(&tags) {
                    return;
                }
                let (line, end_line) = self.span_lines(attached_to, type_alias.span.end);

                self.items.push(DocItem {
                    name: type_alias.id.name.to_string(),
                    kind: DocItemKind::Type,
                    doc: if doc.is_empty() { None } else { Some(doc) },
                    source_path: self.file_path.to_string(),
                    line,
                    end_line,
                    column: self.column_number(attached_to),
                    jsdoc: Some(jsdoc),
                    exported,
                    signature: Some(self.format_type_alias_signature(type_alias, exported)),
                    params: Vec::new(),
                    return_type: None,
                    children: Vec::new(),
                    tags,
                });
            }
            Declaration::TSInterfaceDeclaration(interface) => {
                let Some((jsdoc, doc, tags)) = self.extract_jsdoc(attached_to) else {
                    return;
                };
                if !self.include_private && Self::has_private_tag(&tags) {
                    return;
                }
                let (line, end_line) = self.span_lines(attached_to, interface.span.end);

                let mut children = Vec::new();

                // Extract interface members
                for sig in &interface.body.body {
                    match sig {
                        TSSignature::TSPropertySignature(prop) => {
                            let prop_name = match &prop.key {
                                oxc_ast::ast::PropertyKey::StaticIdentifier(id) => {
                                    id.name.to_string()
                                }
                                _ => continue,
                            };

                            let Some((prop_jsdoc, prop_doc, prop_tags)) =
                                self.extract_jsdoc(prop.span.start)
                            else {
                                continue;
                            };
                            if !self.include_private && Self::has_private_tag(&prop_tags) {
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
                                doc: if prop_doc.is_empty() { None } else { Some(prop_doc) },
                                source_path: self.file_path.to_string(),
                                line: prop_line,
                                end_line: prop_end_line,
                                column: self.column_number(prop.span.start),
                                jsdoc: Some(prop_jsdoc),
                                exported: false,
                                signature: type_annotation,
                                params: Vec::new(),
                                return_type: None,
                                children: Vec::new(),
                                tags: prop_tags,
                            });
                        }
                        TSSignature::TSMethodSignature(method) => {
                            let method_name = match &method.key {
                                oxc_ast::ast::PropertyKey::StaticIdentifier(id) => {
                                    id.name.to_string()
                                }
                                _ => continue,
                            };

                            let Some((method_jsdoc, method_doc, method_tags)) =
                                self.extract_jsdoc(method.span.start)
                            else {
                                continue;
                            };
                            if !self.include_private && Self::has_private_tag(&method_tags) {
                                continue;
                            }
                            let (method_line, method_end_line) =
                                self.span_lines(method.span.start, method.span.end);

                            children.push(DocItem {
                                name: method_name.clone(),
                                kind: DocItemKind::Method,
                                doc: if method_doc.is_empty() { None } else { Some(method_doc) },
                                source_path: self.file_path.to_string(),
                                line: method_line,
                                end_line: method_end_line,
                                column: self.column_number(method.span.start),
                                jsdoc: Some(method_jsdoc),
                                exported: false,
                                signature: Some(self.format_assigned_function_signature(
                                    &method_name,
                                    false,
                                    method.type_parameters.as_ref(),
                                    &method.params,
                                    method.return_type.as_ref(),
                                )),
                                params: self
                                    .extract_params_from_formals(&method.params, &method_tags),
                                return_type: self.extract_return_type_from_annotation(
                                    method.return_type.as_ref(),
                                    &method_tags,
                                ),
                                children: Vec::new(),
                                tags: method_tags,
                            });
                        }
                        _ => {}
                    }
                }

                self.items.push(DocItem {
                    name: interface.id.name.to_string(),
                    kind: DocItemKind::Interface,
                    doc: if doc.is_empty() { None } else { Some(doc) },
                    source_path: self.file_path.to_string(),
                    line,
                    end_line,
                    column: self.column_number(attached_to),
                    jsdoc: Some(jsdoc),
                    exported,
                    signature: Some(self.format_interface_signature(interface, exported)),
                    params: Vec::new(),
                    return_type: None,
                    children,
                    tags,
                });
            }
            Declaration::TSEnumDeclaration(enum_decl) => {
                let Some((jsdoc, doc, tags)) = self.extract_jsdoc(attached_to) else {
                    return;
                };
                if !self.include_private && Self::has_private_tag(&tags) {
                    return;
                }
                let (line, end_line) = self.span_lines(attached_to, enum_decl.span.end);

                let children: Vec<DocItem> = enum_decl
                    .body
                    .members
                    .iter()
                    .map(|member| {
                        let member_name = match &member.id {
                            oxc_ast::ast::TSEnumMemberName::Identifier(id) => id.name.to_string(),
                            oxc_ast::ast::TSEnumMemberName::String(s) => s.value.to_string(),
                            oxc_ast::ast::TSEnumMemberName::ComputedString(s) => {
                                s.value.to_string()
                            }
                            oxc_ast::ast::TSEnumMemberName::ComputedTemplateString(template) => {
                                self.slice(template.span.start, template.span.end)
                            }
                        };
                        let (member_line, member_end_line) =
                            self.span_lines(member.span.start, member.span.end);
                        DocItem {
                            name: member_name,
                            kind: DocItemKind::Property,
                            doc: None,
                            source_path: self.file_path.to_string(),
                            line: member_line,
                            end_line: member_end_line,
                            column: self.column_number(member.span.start),
                            jsdoc: None,
                            exported: false,
                            signature: None,
                            params: Vec::new(),
                            return_type: None,
                            children: Vec::new(),
                            tags: Vec::new(),
                        }
                    })
                    .collect();

                self.items.push(DocItem {
                    name: enum_decl.id.name.to_string(),
                    kind: DocItemKind::Enum,
                    doc: if doc.is_empty() { None } else { Some(doc) },
                    source_path: self.file_path.to_string(),
                    line,
                    end_line,
                    column: self.column_number(attached_to),
                    jsdoc: Some(jsdoc),
                    exported,
                    signature: None,
                    params: Vec::new(),
                    return_type: None,
                    children,
                    tags,
                });
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
