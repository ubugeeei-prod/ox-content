//! Documentation extraction from source code using OXC parser.

use ox_jsdoc_binary::decoder::nodes::comment_ast::LazyJsdocTag;
use ox_jsdoc_binary::decoder::LazySourceFile;
use ox_jsdoc_binary::parser::{parse_to_bytes, ParseOptions};
use oxc_allocator::Allocator;
use oxc_ast::ast::{
    BindingPatternKind, Class, Comment, Declaration, ExportDefaultDeclarationKind, Expression,
    Function, Statement, TSSignature, TSType, TSTypeName,
};
use oxc_ast::visit::walk;
use oxc_ast::Visit;
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
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
    /// Return description from JSDoc @returns tag.
    pub return_description: Option<String>,
    /// Code examples from JSDoc @example tags.
    pub examples: Vec<String>,
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
}

#[derive(Debug, Clone)]
struct ParsedParamTag {
    name: String,
    type_annotation: Option<String>,
    optional: bool,
    default_value: Option<String>,
    description: Option<String>,
}

#[derive(Debug, Clone)]
struct ParsedReturnTag {
    type_annotation: Option<String>,
    description: Option<String>,
}

#[derive(Debug, Clone)]
struct ParsedJsdoc {
    raw: String,
    doc: String,
    tags: Vec<DocTag>,
    param_tags: Vec<ParsedParamTag>,
    return_tag: Option<ParsedReturnTag>,
    examples: Vec<String>,
}

#[derive(Debug, Clone)]
struct ParsedJsdocBody {
    doc: String,
    tags: Vec<DocTag>,
    param_tags: Vec<ParsedParamTag>,
    return_tag: Option<ParsedReturnTag>,
    examples: Vec<String>,
}

impl ParsedJsdoc {
    fn doc_option(&self) -> Option<String> {
        (!self.doc.is_empty()).then(|| self.doc.clone())
    }

    fn return_description(&self) -> Option<String> {
        self.return_tag.as_ref().and_then(|tag| tag.description.clone())
    }
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

        let mut visitor = DocVisitor::new(
            source,
            file_path,
            self.include_private,
            ret.program.comments.iter().copied().collect(),
        );
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

/// AST visitor for extracting documentation.
struct DocVisitor<'a> {
    source: &'a str,
    file_path: &'a str,
    include_private: bool,
    comments: Vec<Comment>,
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
        comments: Vec<Comment>,
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
            comments,
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

    fn extract_jsdoc(&self, attached_to: u32) -> Option<ParsedJsdoc> {
        let comment =
            self.comments.iter().rev().find(|comment| {
                comment.attached_to == attached_to && comment.is_jsdoc(self.source)
            })?;

        let full_raw = comment.span.source_text(self.source);
        let mut raw = comment.content_span().source_text(self.source).to_string();
        if raw.starts_with('*') {
            raw.remove(0);
        }
        let raw = raw.trim_matches('\n').to_string();

        let body = Self::parse_jsdoc(full_raw)?;

        Some(ParsedJsdoc {
            raw,
            doc: body.doc,
            tags: body.tags,
            param_tags: body.param_tags,
            return_tag: body.return_tag,
            examples: body.examples,
        })
    }

    /// Parse JSDoc comment into description and tags with ox_jsdoc_binary.
    fn parse_jsdoc(comment: &str) -> Option<ParsedJsdocBody> {
        let result = parse_to_bytes(
            comment,
            ParseOptions { compat_mode: true, fence_aware: true, ..ParseOptions::default() },
        );
        let source_file = LazySourceFile::new(&result.binary_bytes).ok()?;
        let root = source_file.asts().next().flatten()?;

        let doc = root.description().unwrap_or_default().trim().to_string();
        let mut tags = Vec::new();
        let mut param_tags = Vec::new();
        let mut return_tag = None;
        let mut examples = Vec::new();

        for tag in root.tags() {
            let tag_name = tag.tag().value().to_string();
            let value = tag.raw_body().unwrap_or_default().trim().to_string();

            if let Some(param_tag) = Self::parse_param_tag(tag) {
                param_tags.push(param_tag);
            }
            if return_tag.is_none() {
                return_tag = Self::parse_return_tag(tag);
            }
            if tag_name == "example" {
                let example = tag
                    .description()
                    .or_else(|| tag.raw_body())
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(str::to_string);
                if let Some(example) = example {
                    examples.push(example);
                }
            }

            tags.push(DocTag { tag: tag_name, value });
        }

        Some(ParsedJsdocBody { doc, tags, param_tags, return_tag, examples })
    }

    fn parse_param_tag(tag: LazyJsdocTag<'_>) -> Option<ParsedParamTag> {
        if tag.tag().value() != "param" {
            return None;
        }

        Some(ParsedParamTag {
            name: tag.name()?.raw().trim_start_matches("...").to_string(),
            type_annotation: tag.raw_type().map(|raw_type| raw_type.raw().to_string()),
            optional: tag.optional(),
            default_value: tag.default_value().map(str::to_string),
            description: tag.description().map(str::to_string),
        })
    }

    fn parse_return_tag(tag: LazyJsdocTag<'_>) -> Option<ParsedReturnTag> {
        if tag.tag().value() != "returns" && tag.tag().value() != "return" {
            return None;
        }

        Some(ParsedReturnTag {
            type_annotation: tag.raw_type().map(|raw_type| raw_type.raw().to_string()),
            description: tag.description().map(str::to_string),
        })
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
            items.push(format!(
                "...{}",
                self.slice(rest.argument.span().start, rest.argument.span().end)
            ));
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
            if let Some(type_params) = &class.super_type_parameters {
                sig.push_str(&self.format_type_parameter_declaration(Some(type_params)));
            }
        }

        if let Some(implements) = &class.implements {
            let implements = implements
                .iter()
                .map(|item| {
                    let mut value =
                        self.slice(item.expression.span().start, item.expression.span().end);
                    if let Some(type_params) = &item.type_parameters {
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

        if let Some(extends) = &interface.extends {
            let extends = extends
                .iter()
                .map(|item| {
                    let mut value =
                        self.slice(item.expression.span().start, item.expression.span().end);
                    if let Some(type_params) = &item.type_parameters {
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

    fn has_private_tag(tags: &[DocTag]) -> bool {
        tags.iter().any(|tag| tag.tag == "private")
    }

    /// Format a binding pattern.
    fn format_binding_pattern(&self, pattern: &oxc_ast::ast::BindingPattern) -> String {
        match &pattern.kind {
            BindingPatternKind::BindingIdentifier(id) => {
                let mut s = id.name.to_string();
                if pattern.optional {
                    s.push('?');
                }
                if let Some(type_ann) = &pattern.type_annotation {
                    s.push_str(": ");
                    s.push_str(&self.format_ts_type(&type_ann.type_annotation));
                }
                s
            }
            BindingPatternKind::ObjectPattern(_) => "{...}".to_string(),
            BindingPatternKind::ArrayPattern(_) => "[...]".to_string(),
            BindingPatternKind::AssignmentPattern(assign) => {
                self.format_binding_pattern(&assign.left)
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
                    .map(|p| self.format_binding_pattern(&p.pattern))
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
        }
    }

    fn extract_params_from_formals(
        &self,
        params: &oxc_ast::ast::FormalParameters<'a>,
        parsed_tags: &[ParsedParamTag],
    ) -> Vec<ParamDoc> {
        params
            .items
            .iter()
            .map(|param| {
                let name = Self::binding_pattern_name(&param.pattern).unwrap_or_else(|| {
                    parsed_tags
                        .iter()
                        .find(|tag| {
                            !params.items.iter().any(|item| {
                                Self::binding_pattern_name(&item.pattern).as_deref()
                                    == Some(tag.name.as_str())
                            })
                        })
                        .map_or_else(|| "param".to_string(), |tag| tag.name.clone())
                });

                let type_annotation = param
                    .pattern
                    .type_annotation
                    .as_ref()
                    .map(|t| self.format_ts_type(&t.type_annotation))
                    .or_else(|| self.assignment_pattern_type_annotation(&param.pattern));

                let default_value = self.assignment_pattern_default_value(&param.pattern);
                let tag = parsed_tags.iter().find(|tag| tag.name == name);

                ParamDoc {
                    name,
                    type_annotation: tag
                        .and_then(|tag| tag.type_annotation.clone())
                        .or(type_annotation),
                    optional: param.pattern.optional
                        || default_value.is_some()
                        || tag.is_some_and(|tag| tag.optional),
                    default_value: tag.and_then(|tag| tag.default_value.clone()).or(default_value),
                    description: tag.and_then(|tag| tag.description.clone()),
                }
            })
            .collect()
    }

    fn binding_pattern_name(pattern: &oxc_ast::ast::BindingPattern<'a>) -> Option<String> {
        match &pattern.kind {
            BindingPatternKind::BindingIdentifier(id) => Some(id.name.to_string()),
            BindingPatternKind::AssignmentPattern(assign) => {
                Self::binding_pattern_name(&assign.left)
            }
            _ => None,
        }
    }

    fn assignment_pattern_default_value(
        &self,
        pattern: &oxc_ast::ast::BindingPattern<'a>,
    ) -> Option<String> {
        match &pattern.kind {
            BindingPatternKind::AssignmentPattern(assign) => {
                Some(self.slice(assign.right.span().start, assign.right.span().end))
            }
            _ => None,
        }
    }

    fn assignment_pattern_type_annotation(
        &self,
        pattern: &oxc_ast::ast::BindingPattern<'a>,
    ) -> Option<String> {
        match &pattern.kind {
            BindingPatternKind::AssignmentPattern(assign) => assign
                .left
                .type_annotation
                .as_ref()
                .map(|t| self.format_ts_type(&t.type_annotation)),
            _ => None,
        }
    }

    /// Extract parameters from a function.
    fn extract_params(&self, func: &Function, param_tags: &[ParsedParamTag]) -> Vec<ParamDoc> {
        self.extract_params_from_formals(&func.params, param_tags)
    }

    fn extract_return_type_from_annotation(
        &self,
        return_type: Option<&oxc_allocator::Box<'a, oxc_ast::ast::TSTypeAnnotation<'a>>>,
        return_tag: Option<&ParsedReturnTag>,
    ) -> Option<String> {
        return_type
            .map(|r| self.format_ts_type(&r.type_annotation))
            .or_else(|| return_tag.and_then(|tag| tag.type_annotation.clone()))
    }

    /// Extract return type from tags.
    fn extract_return_type(
        &self,
        func: &Function,
        return_tag: Option<&ParsedReturnTag>,
    ) -> Option<String> {
        self.extract_return_type_from_annotation(func.return_type.as_ref(), return_tag)
    }

    /// Create a DocItem from a function.
    fn create_function_item(
        &self,
        func: &Function,
        exported: bool,
        attached_to: u32,
    ) -> Option<DocItem> {
        let name = func.id.as_ref()?.name.to_string();
        let jsdoc = self.extract_jsdoc(attached_to)?;
        if !self.include_private && Self::has_private_tag(&jsdoc.tags) {
            return None;
        }
        let (line, end_line) = self.span_lines(attached_to, func.span.end);
        let params = self.extract_params(func, &jsdoc.param_tags);
        let return_type = self.extract_return_type(func, jsdoc.return_tag.as_ref());
        let return_description = jsdoc.return_description();

        Some(DocItem {
            name,
            kind: DocItemKind::Function,
            doc: jsdoc.doc_option(),
            source_path: self.file_path.to_string(),
            line,
            end_line,
            column: self.column_number(attached_to),
            jsdoc: Some(jsdoc.raw),
            exported,
            signature: Some(self.format_function_signature(
                func,
                func.id.as_ref()?.name.as_str(),
                exported,
            )),
            params,
            return_type,
            return_description,
            examples: jsdoc.examples,
            children: Vec::new(),
            tags: jsdoc.tags,
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
        let jsdoc = self.extract_jsdoc(attached_to)?;
        if !self.include_private && Self::has_private_tag(&jsdoc.tags) {
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

                    let Some(method_jsdoc) = self.extract_jsdoc(method.span.start) else {
                        continue;
                    };
                    if !self.include_private && Self::has_private_tag(&method_jsdoc.tags) {
                        continue;
                    }
                    let (method_line, method_end_line) =
                        self.span_lines(method.span.start, method.span.end);
                    let params = self.extract_params(&method.value, &method_jsdoc.param_tags);
                    let return_type =
                        self.extract_return_type(&method.value, method_jsdoc.return_tag.as_ref());
                    let return_description = method_jsdoc.return_description();

                    children.push(DocItem {
                        name: method_name,
                        kind,
                        doc: method_jsdoc.doc_option(),
                        source_path: self.file_path.to_string(),
                        line: method_line,
                        end_line: method_end_line,
                        column: self.column_number(method.span.start),
                        jsdoc: Some(method_jsdoc.raw),
                        exported: false,
                        signature: Some(self.format_assigned_function_signature(
                            "",
                            method.value.r#async,
                            method.value.type_parameters.as_ref(),
                            &method.value.params,
                            method.value.return_type.as_ref(),
                        )),
                        params,
                        return_type,
                        return_description,
                        examples: method_jsdoc.examples,
                        children: Vec::new(),
                        tags: method_jsdoc.tags,
                    });
                }
                oxc_ast::ast::ClassElement::PropertyDefinition(prop) => {
                    let prop_name = match &prop.key {
                        oxc_ast::ast::PropertyKey::StaticIdentifier(id) => id.name.to_string(),
                        _ => continue,
                    };

                    let Some(prop_jsdoc) = self.extract_jsdoc(prop.span.start) else {
                        continue;
                    };
                    if !self.include_private && Self::has_private_tag(&prop_jsdoc.tags) {
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
                        doc: prop_jsdoc.doc_option(),
                        source_path: self.file_path.to_string(),
                        line: prop_line,
                        end_line: prop_end_line,
                        column: self.column_number(prop.span.start),
                        jsdoc: Some(prop_jsdoc.raw),
                        exported: false,
                        signature: type_annotation,
                        params: Vec::new(),
                        return_type: None,
                        return_description: None,
                        examples: prop_jsdoc.examples,
                        children: Vec::new(),
                        tags: prop_jsdoc.tags,
                    });
                }
                _ => {}
            }
        }

        Some(DocItem {
            name: name.to_string(),
            kind: DocItemKind::Class,
            doc: jsdoc.doc_option(),
            source_path: self.file_path.to_string(),
            line,
            end_line,
            column: self.column_number(attached_to),
            jsdoc: Some(jsdoc.raw),
            exported,
            signature: Some(self.format_class_signature(class, name, exported)),
            params: Vec::new(),
            return_type: None,
            return_description: None,
            examples: jsdoc.examples,
            children,
            tags: jsdoc.tags,
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
                let Some(jsdoc) = self.extract_jsdoc(attached_to) else {
                    return;
                };
                if !self.include_private && Self::has_private_tag(&jsdoc.tags) {
                    return;
                }
                let (line, end_line) = self.span_lines(attached_to, var_decl.span.end);

                for declarator in &var_decl.declarations {
                    if let BindingPatternKind::BindingIdentifier(id) = &declarator.id.kind {
                        let name = id.name.to_string();

                        let Some(initializer) = &declarator.init else {
                            continue;
                        };

                        match initializer {
                            Expression::ArrowFunctionExpression(arrow) => {
                                let params = self
                                    .extract_params_from_formals(&arrow.params, &jsdoc.param_tags);
                                let return_type = self.extract_return_type_from_annotation(
                                    arrow.return_type.as_ref(),
                                    jsdoc.return_tag.as_ref(),
                                );
                                self.items.push(DocItem {
                                    name: name.clone(),
                                    kind: DocItemKind::Function,
                                    doc: jsdoc.doc_option(),
                                    source_path: self.file_path.to_string(),
                                    line,
                                    end_line,
                                    column: self.column_number(attached_to),
                                    jsdoc: Some(jsdoc.raw.clone()),
                                    exported,
                                    signature: Some(self.format_assigned_function_signature(
                                        &name,
                                        arrow.r#async,
                                        arrow.type_parameters.as_ref(),
                                        &arrow.params,
                                        arrow.return_type.as_ref(),
                                    )),
                                    params,
                                    return_type,
                                    return_description: jsdoc.return_description(),
                                    examples: jsdoc.examples.clone(),
                                    children: Vec::new(),
                                    tags: jsdoc.tags.clone(),
                                });
                            }
                            Expression::FunctionExpression(func_expr) => {
                                let params = self.extract_params(func_expr, &jsdoc.param_tags);
                                let return_type =
                                    self.extract_return_type(func_expr, jsdoc.return_tag.as_ref());
                                self.items.push(DocItem {
                                    name: name.clone(),
                                    kind: DocItemKind::Function,
                                    doc: jsdoc.doc_option(),
                                    source_path: self.file_path.to_string(),
                                    line,
                                    end_line,
                                    column: self.column_number(attached_to),
                                    jsdoc: Some(jsdoc.raw.clone()),
                                    exported,
                                    signature: Some(self.format_assigned_function_signature(
                                        &name,
                                        func_expr.r#async,
                                        func_expr.type_parameters.as_ref(),
                                        &func_expr.params,
                                        func_expr.return_type.as_ref(),
                                    )),
                                    params,
                                    return_type,
                                    return_description: jsdoc.return_description(),
                                    examples: jsdoc.examples.clone(),
                                    children: Vec::new(),
                                    tags: jsdoc.tags.clone(),
                                });
                            }
                            _ => {}
                        }
                    }
                }
            }
            Declaration::TSTypeAliasDeclaration(type_alias) => {
                let Some(jsdoc) = self.extract_jsdoc(attached_to) else {
                    return;
                };
                if !self.include_private && Self::has_private_tag(&jsdoc.tags) {
                    return;
                }
                let (line, end_line) = self.span_lines(attached_to, type_alias.span.end);

                self.items.push(DocItem {
                    name: type_alias.id.name.to_string(),
                    kind: DocItemKind::Type,
                    doc: jsdoc.doc_option(),
                    source_path: self.file_path.to_string(),
                    line,
                    end_line,
                    column: self.column_number(attached_to),
                    jsdoc: Some(jsdoc.raw),
                    exported,
                    signature: Some(self.format_type_alias_signature(type_alias, exported)),
                    params: Vec::new(),
                    return_type: None,
                    return_description: None,
                    examples: jsdoc.examples,
                    children: Vec::new(),
                    tags: jsdoc.tags,
                });
            }
            Declaration::TSInterfaceDeclaration(interface) => {
                let Some(jsdoc) = self.extract_jsdoc(attached_to) else {
                    return;
                };
                if !self.include_private && Self::has_private_tag(&jsdoc.tags) {
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

                            let Some(prop_jsdoc) = self.extract_jsdoc(prop.span.start) else {
                                continue;
                            };
                            if !self.include_private && Self::has_private_tag(&prop_jsdoc.tags) {
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
                                doc: prop_jsdoc.doc_option(),
                                source_path: self.file_path.to_string(),
                                line: prop_line,
                                end_line: prop_end_line,
                                column: self.column_number(prop.span.start),
                                jsdoc: Some(prop_jsdoc.raw),
                                exported: false,
                                signature: type_annotation,
                                params: Vec::new(),
                                return_type: None,
                                return_description: None,
                                examples: prop_jsdoc.examples,
                                children: Vec::new(),
                                tags: prop_jsdoc.tags,
                            });
                        }
                        TSSignature::TSMethodSignature(method) => {
                            let method_name = match &method.key {
                                oxc_ast::ast::PropertyKey::StaticIdentifier(id) => {
                                    id.name.to_string()
                                }
                                _ => continue,
                            };

                            let Some(method_jsdoc) = self.extract_jsdoc(method.span.start) else {
                                continue;
                            };
                            if !self.include_private && Self::has_private_tag(&method_jsdoc.tags) {
                                continue;
                            }
                            let (method_line, method_end_line) =
                                self.span_lines(method.span.start, method.span.end);
                            let params = self.extract_params_from_formals(
                                &method.params,
                                &method_jsdoc.param_tags,
                            );
                            let return_type = self.extract_return_type_from_annotation(
                                method.return_type.as_ref(),
                                method_jsdoc.return_tag.as_ref(),
                            );
                            let return_description = method_jsdoc.return_description();

                            children.push(DocItem {
                                name: method_name.clone(),
                                kind: DocItemKind::Method,
                                doc: method_jsdoc.doc_option(),
                                source_path: self.file_path.to_string(),
                                line: method_line,
                                end_line: method_end_line,
                                column: self.column_number(method.span.start),
                                jsdoc: Some(method_jsdoc.raw),
                                exported: false,
                                signature: Some(self.format_assigned_function_signature(
                                    &method_name,
                                    false,
                                    method.type_parameters.as_ref(),
                                    &method.params,
                                    method.return_type.as_ref(),
                                )),
                                params,
                                return_type,
                                return_description,
                                examples: method_jsdoc.examples,
                                children: Vec::new(),
                                tags: method_jsdoc.tags,
                            });
                        }
                        _ => {}
                    }
                }

                self.items.push(DocItem {
                    name: interface.id.name.to_string(),
                    kind: DocItemKind::Interface,
                    doc: jsdoc.doc_option(),
                    source_path: self.file_path.to_string(),
                    line,
                    end_line,
                    column: self.column_number(attached_to),
                    jsdoc: Some(jsdoc.raw),
                    exported,
                    signature: Some(self.format_interface_signature(interface, exported)),
                    params: Vec::new(),
                    return_type: None,
                    return_description: None,
                    examples: jsdoc.examples,
                    children,
                    tags: jsdoc.tags,
                });
            }
            Declaration::TSEnumDeclaration(enum_decl) => {
                let Some(jsdoc) = self.extract_jsdoc(attached_to) else {
                    return;
                };
                if !self.include_private && Self::has_private_tag(&jsdoc.tags) {
                    return;
                }
                let (line, end_line) = self.span_lines(attached_to, enum_decl.span.end);

                let children: Vec<DocItem> = enum_decl
                    .members
                    .iter()
                    .map(|member| {
                        let member_name = match &member.id {
                            oxc_ast::ast::TSEnumMemberName::Identifier(id) => id.name.to_string(),
                            oxc_ast::ast::TSEnumMemberName::String(s) => s.value.to_string(),
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
                            return_description: None,
                            examples: Vec::new(),
                            children: Vec::new(),
                            tags: Vec::new(),
                        }
                    })
                    .collect();

                self.items.push(DocItem {
                    name: enum_decl.id.name.to_string(),
                    kind: DocItemKind::Enum,
                    doc: jsdoc.doc_option(),
                    source_path: self.file_path.to_string(),
                    line,
                    end_line,
                    column: self.column_number(attached_to),
                    jsdoc: Some(jsdoc.raw),
                    exported,
                    signature: None,
                    params: Vec::new(),
                    return_type: None,
                    return_description: None,
                    examples: jsdoc.examples,
                    children,
                    tags: jsdoc.tags,
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
    fn test_extract_structured_jsdoc_metadata() {
        let source = r#"
/**
 * Formats a value.
 *
 * @param {number} value - Input value.
 * @param {"short" | "long"} [mode="short"] - Format mode.
 * @returns {string} Rendered label.
 * @example
 * ```ts
 * const snippet = "@param not-a-real-tag";
 * formatValue(1, "short");
 * ```
 * @since 2.4.0
 */
export function formatValue(value: number, mode: "short" | "long" = "short"): string {
    return mode === "short" ? String(value) : `value: ${value}`;
}
"#;

        let extractor = DocExtractor::new();
        let items = extractor.extract_source(source, "test.ts", SourceType::ts()).unwrap();
        let item = &items[0];

        assert_eq!(item.doc.as_deref(), Some("Formats a value."));
        assert_eq!(item.return_type.as_deref(), Some("string"));
        assert_eq!(item.return_description.as_deref(), Some("Rendered label."));
        assert_eq!(item.examples.len(), 1);
        assert!(item.examples[0].contains("@param not-a-real-tag"));
        assert!(item.tags.iter().any(|tag| tag.tag == "since" && tag.value == "2.4.0"));

        assert_eq!(item.params.len(), 2);
        assert_eq!(item.params[0].name, "value");
        assert_eq!(item.params[0].type_annotation.as_deref(), Some("number"));
        assert_eq!(item.params[0].description.as_deref(), Some("Input value."));
        assert_eq!(item.params[1].name, "mode");
        assert_eq!(item.params[1].type_annotation.as_deref(), Some(r#""short" | "long""#));
        assert_eq!(item.params[1].default_value.as_deref(), Some(r#""short""#));
        assert_eq!(item.params[1].description.as_deref(), Some("Format mode."));
        assert!(item.params[1].optional);
    }
}
