//! Documentation extraction from source code using OXC parser.

mod jsdoc;
mod jsdoc_tags;
mod model;

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    BindingPattern, Class, Comment, Declaration, ExportDefaultDeclarationKind, Expression,
    Function, Statement, TSEnumDeclaration, TSEnumMember, TSIndexSignature, TSIndexSignatureName,
    TSInterfaceDeclaration, TSSignature, TSType, TSTypeAliasDeclaration, TSTypeAnnotation,
    TSTypeLiteral, TSTypeName, VariableDeclaration,
};
use oxc_ast_visit::{walk, Visit};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use rustc_hash::FxHashMap;
use std::path::Path;

#[allow(unused_imports)]
use crate::profile_span;
use crate::string_builder::{join2, join3, join5, StringBuilder};

use self::jsdoc::{build_jsdoc_cache, parse_jsdoc_payload, ParsedJsdoc, MODULE_MARKER_TAGS};
pub use self::model::{
    DocItem, DocItemKind, DocTag, ExtractError, ExtractResult, ParamDoc, TypeParamDoc,
};
use self::model::{FunctionTypeMetadata, ParsedParamTag};

/// Documentation extractor.
pub struct DocExtractor {
    /// Include private items.
    include_private: bool,
    /// Include internal items.
    include_internal: bool,
    /// Include declarations without JSDoc. Used for public entry point exports.
    include_undocumented_declarations: bool,
    /// Capture the verbatim JSDoc comment text into [`DocItem::jsdoc`].
    ///
    /// Only the raw `extract_file_docs` NAPI path reads it; the normalize-bound
    /// paths (directory + entry-point extraction) discard it, so they opt out
    /// to skip a per-comment allocation and a per-declaration clone.
    capture_jsdoc_raw: bool,
}

impl DocExtractor {
    /// Creates a new documentation extractor.
    #[must_use]
    pub fn new() -> Self {
        Self {
            include_private: false,
            include_internal: false,
            include_undocumented_declarations: false,
            capture_jsdoc_raw: true,
        }
    }

    /// Creates a new extractor that includes private items.
    #[must_use]
    pub fn with_private(include_private: bool) -> Self {
        Self {
            include_private,
            include_internal: false,
            include_undocumented_declarations: false,
            capture_jsdoc_raw: true,
        }
    }

    /// Creates a new extractor with explicit visibility options.
    #[must_use]
    pub fn with_visibility(include_private: bool, include_internal: bool) -> Self {
        Self {
            include_private,
            include_internal,
            include_undocumented_declarations: false,
            capture_jsdoc_raw: true,
        }
    }

    /// Drops the verbatim JSDoc text from extracted items.
    ///
    /// For callers that immediately normalize (which discards `DocItem::jsdoc`),
    /// this avoids allocating and cloning the raw comment text per declaration.
    #[must_use]
    pub fn without_raw_jsdoc(mut self) -> Self {
        self.capture_jsdoc_raw = false;
        self
    }

    /// Creates a new extractor for public entry point exports.
    ///
    /// This path always normalizes, so it never captures the raw JSDoc text.
    #[must_use]
    pub(crate) fn for_entrypoint_exports(include_private: bool, include_internal: bool) -> Self {
        Self {
            include_private,
            include_internal,
            include_undocumented_declarations: true,
            capture_jsdoc_raw: false,
        }
    }

    /// Extracts documentation from a source file.
    pub fn extract_file(&self, path: &Path) -> ExtractResult<Vec<DocItem>> {
        let mut allocator = Allocator::default();
        self.extract_file_with(&mut allocator, path)
    }

    /// Like [`extract_file`](Self::extract_file), but reuses a caller-owned
    /// arena allocator.
    ///
    /// Batch callers (directory walks, multi-file extraction) create one
    /// [`Allocator`] and pass it to every file. The arena is rewound at the
    /// start of each parse, so a single allocation is reused instead of
    /// allocating and freeing a fresh multi-MB arena per file. The public
    /// [`extract_file`](Self::extract_file) wrapper just hands in a fresh one.
    pub(crate) fn extract_file_with(
        &self,
        allocator: &mut Allocator,
        path: &Path,
    ) -> ExtractResult<Vec<DocItem>> {
        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        match extension {
            "ts" | "tsx" | "js" | "jsx" | "mts" | "mjs" | "cts" | "cjs" => {
                self.extract_js_ts(allocator, path)
            }
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
        let mut allocator = Allocator::default();
        self.extract_source_with(&mut allocator, source, file_path, source_type)
    }

    /// Like [`extract_source`](Self::extract_source), but reuses a caller-owned
    /// arena allocator. See [`extract_file_with`](Self::extract_file_with).
    pub(crate) fn extract_source_with(
        &self,
        allocator: &mut Allocator,
        source: &str,
        file_path: &str,
        source_type: SourceType,
    ) -> ExtractResult<Vec<DocItem>> {
        profile_span!("docs::extract_source");
        // Rewind the arena so a reused allocator starts clean for this file.
        // Extraction returns owned `DocItem`s, so by the time the next file
        // resets, nothing borrows the previous file's arena.
        allocator.reset();
        let ret = {
            profile_span!("docs::oxc_parse");
            Parser::new(&*allocator, source, source_type).parse()
        };

        if !ret.errors.is_empty() {
            let error_msg = ret
                .errors
                .iter()
                .map(std::string::ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ");
            return Err(ExtractError::Parse(error_msg));
        }

        Ok(self.extract_items_from_program(source, file_path, &ret.program))
    }

    /// Extract doc items from an already-parsed program (build the JSDoc cache,
    /// then walk the AST).
    ///
    /// Shared by [`extract_source_with`](Self::extract_source_with) and the
    /// export-graph walk: the graph parses every reachable module to collect its
    /// exports, so it extracts docs from that same AST instead of parsing the
    /// file a second time.
    pub(crate) fn extract_items_from_program(
        &self,
        source: &str,
        file_path: &str,
        program: &oxc_ast::ast::Program<'_>,
    ) -> Vec<DocItem> {
        let comments: Vec<Comment> = program.comments.iter().copied().collect();
        let jsdoc_cache = build_jsdoc_cache(source, &comments, self.capture_jsdoc_raw);

        let mut visitor = DocVisitor::new(
            source,
            file_path,
            self.include_private,
            self.include_internal,
            self.include_undocumented_declarations,
            jsdoc_cache,
        );
        let first_stmt_start = program.body.first().map(|statement| statement.span().start);
        {
            profile_span!("docs::visit_ast");
            if let Some(module_item) = visitor.extract_module_entry(&comments, first_stmt_start) {
                visitor.items.push(module_item);
            }
            visitor.visit_program(program);
        }

        visitor.items
    }

    /// Extracts documentation from a JavaScript/TypeScript file, reusing the
    /// caller-owned arena allocator.
    fn extract_js_ts(&self, allocator: &mut Allocator, path: &Path) -> ExtractResult<Vec<DocItem>> {
        let content = std::fs::read_to_string(path)?;
        let file_path = path.to_string_lossy().to_string();
        let source_type = SourceType::from_path(path).unwrap_or_default();

        self.extract_source_with(allocator, &content, &file_path, source_type)
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
    include_internal: bool,
    include_undocumented_declarations: bool,
    jsdoc_cache: FxHashMap<u32, ParsedJsdoc>,
    line_starts: Vec<usize>,
    items: Vec<DocItem>,
    type_alias_function_metadata: FxHashMap<String, FunctionTypeMetadata>,
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
            type_alias_function_metadata: FxHashMap::default(),
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
        if let Some((jsdoc, doc, tags)) = self.jsdoc_cache.get(&attached_to) {
            // Borrow from the cache and apply the visibility filter first, so a
            // private/internal declaration is dropped without cloning the whole
            // (raw, description, tags) payload only to throw it away.
            if self.should_skip_by_visibility(tags) {
                return None;
            }
            return Some((
                Some(jsdoc.clone()),
                (!doc.is_empty()).then(|| doc.clone()),
                tags.clone(),
            ));
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
            extends: Vec::new(),
            implements: Vec::new(),
            has_body: false,
            optional: false,
            readonly: false,
            r#static: false,
            params: Vec::new(),
            return_type: None,
            return_members: Vec::new(),
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

    fn format_type_formal_parameters(&self, params: &oxc_ast::ast::FormalParameters<'a>) -> String {
        let mut items = Vec::with_capacity(params.items.len() + usize::from(params.rest.is_some()));

        for param in &params.items {
            let name = Self::binding_pattern_name(&param.pattern);
            let mut item = StringBuilder::with_capacity(name.len() + 16);
            item.push_str(&name);
            if param.optional {
                item.push_char('?');
            }
            if let Some(type_annotation) = param.type_annotation.as_ref() {
                let formatted = self.format_ts_type(&type_annotation.type_annotation);
                item.push_str(": ");
                item.push_str(&formatted);
            }
            items.push(item.into_string());
        }

        if let Some(rest) = params.rest.as_ref() {
            let name = Self::binding_pattern_name(&rest.rest.argument);
            let mut item = StringBuilder::with_capacity(name.len() + 19);
            item.push_str("...");
            item.push_str(&name);
            if let Some(type_annotation) = rest.type_annotation.as_ref() {
                let formatted = self.format_ts_type(&type_annotation.type_annotation);
                item.push_str(": ");
                item.push_str(&formatted);
            }
            items.push(item.into_string());
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

    fn format_class_signature(
        &self,
        class: &Class,
        name: &str,
        exported: bool,
        extends: &[String],
        implements: &[String],
    ) -> String {
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

        if !extends.is_empty() {
            sig.push_str(" extends ");
            Self::push_joined(&mut sig, extends);
        }

        if !implements.is_empty() {
            sig.push_str(" implements ");
            Self::push_joined(&mut sig, implements);
        }

        sig
    }

    fn extract_class_extends(&self, class: &Class<'a>) -> Vec<String> {
        let Some(super_class) = &class.super_class else {
            return Vec::new();
        };
        let mut value = self.slice(super_class.span().start, super_class.span().end);
        if let Some(type_params) = &class.super_type_arguments {
            value.push_str(&self.format_type_parameter_declaration(Some(type_params)));
        }
        Vec::from([value])
    }

    fn extract_class_implements(&self, class: &Class<'a>) -> Vec<String> {
        class
            .implements
            .iter()
            .map(|item| {
                let mut value = Self::format_ts_type_name(&item.expression);
                if let Some(type_params) = &item.type_arguments {
                    value.push_str(&self.format_type_parameter_declaration(Some(type_params)));
                }
                value
            })
            .collect()
    }

    fn format_interface_signature(
        &self,
        interface: &oxc_ast::ast::TSInterfaceDeclaration<'a>,
        exported: bool,
        extends: &[String],
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

        if !extends.is_empty() {
            sig.push_str(" extends ");
            Self::push_joined(&mut sig, extends);
        }

        sig
    }

    fn push_joined(out: &mut String, items: &[String]) {
        for (index, item) in items.iter().enumerate() {
            if index > 0 {
                out.push_str(", ");
            }
            out.push_str(item);
        }
    }

    fn extract_interface_extends(
        &self,
        interface: &oxc_ast::ast::TSInterfaceDeclaration<'a>,
    ) -> Vec<String> {
        interface
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
            .collect()
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
        sig.push_str(&self.format_ts_type(&type_alias.type_annotation));
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
    fn find_parsed_param_tag_index(parsed: &[ParsedParamTag], name: &str) -> Option<usize> {
        parsed.iter().position(|tag| {
            let tag_name = tag.name.trim_start_matches("...");
            tag_name == name || tag_name.split('.').next() == Some(name)
        })
    }

    fn find_exact_parsed_param_tag<'t>(
        parsed: &'t [ParsedParamTag],
        name: &str,
    ) -> Option<&'t ParsedParamTag> {
        parsed.iter().find(|tag| tag.name.trim_start_matches("...").trim_end_matches('?') == name)
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

    fn binding_pattern_identifier_name(pattern: &BindingPattern<'a>) -> Option<String> {
        match pattern {
            BindingPattern::BindingIdentifier(id) => Some(id.name.to_string()),
            BindingPattern::AssignmentPattern(assign) => {
                Self::binding_pattern_identifier_name(&assign.left)
            }
            BindingPattern::ObjectPattern(_) | BindingPattern::ArrayPattern(_) => None,
        }
    }

    fn binding_pattern_is_destructured(pattern: &BindingPattern<'a>) -> bool {
        match pattern {
            BindingPattern::AssignmentPattern(assign) => {
                Self::binding_pattern_is_destructured(&assign.left)
            }
            BindingPattern::ObjectPattern(_) | BindingPattern::ArrayPattern(_) => true,
            BindingPattern::BindingIdentifier(_) => false,
        }
    }

    fn top_level_param_tag_name(tag: &ParsedParamTag) -> Option<&str> {
        let raw_name = tag.name.trim();
        if raw_name.starts_with("...") {
            return None;
        }
        let name = raw_name.trim_end_matches('?');
        (!name.is_empty() && !name.contains('.')).then_some(name)
    }

    fn find_destructured_param_tag_index(
        parsed: &[ParsedParamTag],
        used_indices: &[usize],
        reserved_names: &[String],
    ) -> Option<usize> {
        parsed.iter().enumerate().find_map(|(index, tag)| {
            if used_indices.contains(&index) {
                return None;
            }
            let name = Self::top_level_param_tag_name(tag)?;
            if reserved_names.iter().any(|reserved| reserved == name) {
                return None;
            }
            Some(index)
        })
    }

    fn binding_pattern_default_value(&self, pattern: &BindingPattern<'a>) -> Option<String> {
        match pattern {
            BindingPattern::AssignmentPattern(assign) => {
                Some(self.slice(assign.right.span().start, assign.right.span().end))
            }
            _ => None,
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
            TSType::TSUnknownKeyword(_) => "unknown".to_string(),
            TSType::TSTypeReference(ref_type) => {
                self.format_type_span(ref_type.span().start, ref_type.span().end)
            }
            TSType::TSArrayType(arr) => join2(&self.format_ts_type(&arr.element_type), "[]"),
            TSType::TSTypeOperatorType(op) => {
                let inner = self.format_ts_type(&op.type_annotation);
                match op.operator {
                    oxc_ast::ast::TSTypeOperatorOperator::Keyof => join2("keyof ", &inner),
                    oxc_ast::ast::TSTypeOperatorOperator::Unique => join2("unique ", &inner),
                    oxc_ast::ast::TSTypeOperatorOperator::Readonly => join2("readonly ", &inner),
                }
            }
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
                let params = self.format_type_formal_parameters(&func.params);
                let ret = self.format_ts_type(&func.return_type.type_annotation);
                let mut out = StringBuilder::with_capacity(params.len() + ret.len() + 6);
                out.push_char('(');
                out.push_str(&params);
                out.push_str(") => ");
                out.push_str(&ret);
                out.into_string()
            }
            TSType::TSTypeLiteral(type_literal) => self.format_type_literal(type_literal),
            TSType::TSParenthesizedType(paren) => {
                join3("(", &self.format_ts_type(&paren.type_annotation), ")")
            }
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
            _ => self.format_type_from_span(ts_type),
        }
    }

    fn format_type_from_span(&self, ts_type: &TSType) -> String {
        self.format_type_span(ts_type.span().start, ts_type.span().end)
    }

    fn format_span(&self, start: u32, end: u32) -> String {
        self.slice(start, end).split_whitespace().collect::<Vec<_>>().join(" ").trim().to_string()
    }

    fn format_type_span(&self, start: u32, end: u32) -> String {
        Self::collapse_type_annotation_text(&self.slice(start, end))
    }

    fn collapse_type_annotation_text(text: &str) -> String {
        let text = text.trim();
        if text.is_empty() {
            return String::new();
        }

        let mut out = String::with_capacity(text.len());
        let mut pending_space = false;
        for ch in text.chars() {
            if ch.is_whitespace() {
                pending_space = !out.is_empty();
                continue;
            }

            if pending_space {
                if !matches!(out.chars().next_back(), Some('<')) && ch != '>' {
                    out.push(' ');
                }
                pending_space = false;
            }
            out.push(ch);
        }
        out
    }

    fn property_key_name(key: &oxc_ast::ast::PropertyKey<'a>) -> Option<String> {
        match key {
            oxc_ast::ast::PropertyKey::StaticIdentifier(id) => Some(id.name.to_string()),
            _ => None,
        }
    }

    fn format_type_literal(&self, type_literal: &TSTypeLiteral<'a>) -> String {
        let members = type_literal
            .members
            .iter()
            .map(|member| self.format_type_literal_member(member))
            .filter(|member| !member.is_empty())
            .collect::<Vec<_>>();

        if members.is_empty() {
            "{}".to_string()
        } else {
            join3("{ ", &members.join("; "), " }")
        }
    }

    fn format_type_literal_member(&self, member: &TSSignature<'a>) -> String {
        match member {
            TSSignature::TSPropertySignature(prop) => {
                let Some(name) = Self::property_key_name(&prop.key) else {
                    return self.format_span(prop.span.start, prop.span.end);
                };
                let type_annotation = prop.type_annotation.as_ref().map_or_else(
                    || "unknown".to_string(),
                    |t| self.format_ts_type(&t.type_annotation),
                );
                let mut out = StringBuilder::with_capacity(name.len() + type_annotation.len() + 16);
                if prop.readonly {
                    out.push_str("readonly ");
                }
                out.push_str(&name);
                if prop.optional {
                    out.push_char('?');
                }
                out.push_str(": ");
                out.push_str(&type_annotation);
                out.into_string()
            }
            TSSignature::TSMethodSignature(method) => {
                let Some(name) = Self::property_key_name(&method.key) else {
                    return self.format_span(method.span.start, method.span.end);
                };
                let params = self.format_type_formal_parameters(&method.params);
                let return_type = method.return_type.as_ref().map_or_else(
                    || "unknown".to_string(),
                    |t| self.format_ts_type(&t.type_annotation),
                );
                let type_parameters =
                    self.format_type_parameter_declaration(method.type_parameters.as_ref());
                let mut out = StringBuilder::with_capacity(
                    name.len() + type_parameters.len() + params.len() + return_type.len() + 8,
                );
                out.push_str(&name);
                if method.optional {
                    out.push_char('?');
                }
                out.push_str(&type_parameters);
                out.push_char('(');
                out.push_str(&params);
                out.push_str("): ");
                out.push_str(&return_type);
                out.into_string()
            }
            TSSignature::TSIndexSignature(index_signature) => {
                let Some(parameter) = index_signature.parameters.first() else {
                    return self.format_span(index_signature.span.start, index_signature.span.end);
                };
                let (name, _, _) = self.format_index_signature_name(parameter);
                let value_type =
                    self.format_ts_type(&index_signature.type_annotation.type_annotation);
                Self::format_index_signature(index_signature, &name, &value_type)
            }
            _ => self.format_span(member.span().start, member.span().end),
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
        let mut reserved_param_names = params
            .items
            .iter()
            .filter_map(|param| Self::binding_pattern_identifier_name(&param.pattern))
            .collect::<Vec<_>>();
        if let Some(rest) = params.rest.as_ref() {
            if let Some(name) = Self::binding_pattern_identifier_name(&rest.rest.argument) {
                reserved_param_names.push(name);
            }
        }
        let mut used_param_tag_indices = Vec::new();

        let mut docs = Vec::with_capacity(params.items.len() + usize::from(params.rest.is_some()));

        for param in &params.items {
            let fallback_name = Self::binding_pattern_name(&param.pattern);
            let tag_index = if Self::binding_pattern_is_destructured(&param.pattern) {
                Self::find_destructured_param_tag_index(
                    &parsed_param_tags,
                    &used_param_tag_indices,
                    &reserved_param_names,
                )
            } else {
                Self::find_parsed_param_tag_index(&parsed_param_tags, &fallback_name)
            };
            let tag = tag_index.map(|index| &parsed_param_tags[index]);
            if let Some(index) = tag_index {
                used_param_tag_indices.push(index);
            }
            let name =
                tag.and_then(Self::top_level_param_tag_name).unwrap_or(&fallback_name).to_string();
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

            let optional =
                param.optional || default_value.is_some() || tag.is_some_and(|tag| tag.optional);
            let description = tag.and_then(|tag| tag.description.clone());

            docs.push(ParamDoc {
                name: name.clone(),
                type_annotation,
                optional,
                default_value,
                description,
            });

            if let Some(type_literal) = param
                .type_annotation
                .as_ref()
                .and_then(|t| Self::type_literal_from_ts_type(&t.type_annotation))
            {
                docs.extend(self.extract_type_literal_param_members(
                    &name,
                    type_literal,
                    &parsed_param_tags,
                ));
            }
        }

        if let Some(rest) = params.rest.as_ref() {
            let name = Self::binding_pattern_name(&rest.rest.argument);
            let tag_index = Self::find_parsed_param_tag_index(&parsed_param_tags, &name);
            let tag = tag_index.map(|index| &parsed_param_tags[index]);
            let type_annotation = rest
                .type_annotation
                .as_ref()
                .map(|t| self.format_ts_type(&t.type_annotation))
                .or_else(|| tag.and_then(|tag| tag.type_annotation.clone()));

            docs.push(ParamDoc {
                name: name.clone(),
                type_annotation,
                optional: tag.is_some_and(|tag| tag.optional),
                default_value: tag.and_then(|tag| tag.default_value.clone()),
                description: tag.and_then(|tag| tag.description.clone()),
            });

            if let Some(type_literal) = rest
                .type_annotation
                .as_ref()
                .and_then(|t| Self::type_literal_from_ts_type(&t.type_annotation))
            {
                docs.extend(self.extract_type_literal_param_members(
                    &name,
                    type_literal,
                    &parsed_param_tags,
                ));
            }
        }

        docs
    }

    fn type_literal_from_ts_type<'t>(ts_type: &'t TSType<'a>) -> Option<&'t TSTypeLiteral<'a>> {
        match ts_type {
            TSType::TSTypeLiteral(type_literal) => Some(type_literal),
            TSType::TSParenthesizedType(paren) => {
                Self::type_literal_from_ts_type(&paren.type_annotation)
            }
            _ => None,
        }
    }

    fn extract_type_literal_param_members(
        &self,
        parent_name: &str,
        type_literal: &TSTypeLiteral<'a>,
        parsed_param_tags: &[ParsedParamTag],
    ) -> Vec<ParamDoc> {
        let mut params = Vec::new();
        for member in &type_literal.members {
            match member {
                TSSignature::TSPropertySignature(prop) => {
                    let Some(property_name) = Self::property_key_name(&prop.key) else {
                        continue;
                    };
                    let lookup_name = join3(parent_name, ".", &property_name);
                    let tag = Self::find_exact_parsed_param_tag(parsed_param_tags, &lookup_name);
                    let optional = prop.optional || tag.is_some_and(|tag| tag.optional);
                    let name =
                        if optional { join2(&lookup_name, "?") } else { lookup_name.clone() };
                    let type_annotation = prop
                        .type_annotation
                        .as_ref()
                        .map(|t| self.format_ts_type(&t.type_annotation))
                        .or_else(|| tag.and_then(|tag| tag.type_annotation.clone()));

                    params.push(ParamDoc {
                        name,
                        type_annotation,
                        optional,
                        default_value: tag.and_then(|tag| tag.default_value.clone()),
                        description: tag.and_then(|tag| tag.description.clone()),
                    });
                }
                TSSignature::TSMethodSignature(method) => {
                    let Some(method_name) = Self::property_key_name(&method.key) else {
                        continue;
                    };
                    let lookup_name = join3(parent_name, ".", &method_name);
                    let tag = Self::find_exact_parsed_param_tag(parsed_param_tags, &lookup_name);
                    let optional = method.optional || tag.is_some_and(|tag| tag.optional);
                    let name =
                        if optional { join2(&lookup_name, "?") } else { lookup_name.clone() };
                    let params_text = self.format_type_formal_parameters(&method.params);
                    let return_type = method.return_type.as_ref().map_or_else(
                        || "unknown".to_string(),
                        |t| self.format_ts_type(&t.type_annotation),
                    );
                    let mut type_annotation =
                        StringBuilder::with_capacity(params_text.len() + return_type.len() + 6);
                    type_annotation.push_char('(');
                    type_annotation.push_str(&params_text);
                    type_annotation.push_str(") => ");
                    type_annotation.push_str(&return_type);

                    params.push(ParamDoc {
                        name,
                        type_annotation: Some(type_annotation.into_string()),
                        optional,
                        default_value: tag.and_then(|tag| tag.default_value.clone()),
                        description: tag.and_then(|tag| tag.description.clone()),
                    });
                }
                _ => {}
            }
        }
        params
    }

    /// Extract parameters from a function.
    fn extract_params(&self, func: &Function, tags: &[DocTag]) -> Vec<ParamDoc> {
        self.extract_params_from_formals(&func.params, tags)
    }

    fn extract_return_type_from_annotation(
        &self,
        return_type: Option<&oxc_ast::ast::TSTypeAnnotation<'a>>,
        tags: &[DocTag],
    ) -> Option<String> {
        return_type.map(|r| self.format_ts_type(&r.type_annotation)).or_else(|| {
            tags.iter().find(|tag| tag.tag == "returns" || tag.tag == "return").and_then(|tag| {
                let (type_annotation, description) = Self::parse_return_tag(tag);
                type_annotation.or(description)
            })
        })
    }

    fn extract_return_from_annotation(
        &self,
        return_type: Option<&oxc_ast::ast::TSTypeAnnotation<'a>>,
        tags: &[DocTag],
    ) -> (Option<String>, Vec<DocItem>) {
        if let Some(return_type) = return_type {
            if let TSType::TSTypeLiteral(type_literal) = &return_type.type_annotation {
                let members = self.extract_ts_signature_members(&type_literal.members);
                let type_annotation = if members.is_empty() {
                    self.format_ts_type(&return_type.type_annotation)
                } else {
                    "object".to_string()
                };
                return (Some(type_annotation), members);
            }
            return (Some(self.format_ts_type(&return_type.type_annotation)), Vec::new());
        }

        (self.extract_return_type_from_annotation(None, tags), Vec::new())
    }

    fn extract_return(&self, func: &Function, tags: &[DocTag]) -> (Option<String>, Vec<DocItem>) {
        self.extract_return_from_annotation(func.return_type.as_ref().map(AsRef::as_ref), tags)
    }

    fn extract_function_type_metadata(
        &self,
        ts_type: &TSType<'a>,
        tags: &[DocTag],
    ) -> Option<FunctionTypeMetadata> {
        match ts_type {
            TSType::TSFunctionType(func) => {
                let (return_type, return_members) =
                    self.extract_return_from_annotation(Some(&func.return_type), tags);
                Some(FunctionTypeMetadata {
                    params: self.extract_params_from_formals(&func.params, tags),
                    return_type,
                    return_members,
                    type_parameters: self.extract_type_parameters(func.type_parameters.as_ref()),
                })
            }
            TSType::TSParenthesizedType(paren) => {
                self.extract_function_type_metadata(&paren.type_annotation, tags)
            }
            TSType::TSTypeReference(ref_type) => {
                let name = Self::type_reference_identifier_name(ref_type)?;
                self.type_alias_function_metadata
                    .get(&name)
                    .map(FunctionTypeMetadata::as_reference_metadata)
            }
            TSType::TSIntersectionType(inter) => inter
                .types
                .iter()
                .find_map(|ts_type| self.extract_function_type_metadata(ts_type, tags)),
            _ => None,
        }
    }

    fn type_reference_identifier_name(
        ref_type: &oxc_ast::ast::TSTypeReference<'a>,
    ) -> Option<String> {
        match &ref_type.type_name {
            TSTypeName::IdentifierReference(id) => Some(id.name.to_string()),
            TSTypeName::QualifiedName(_) | TSTypeName::ThisExpression(_) => None,
        }
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
        let (return_type, return_members) = self.extract_return(func, &tags);

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
            extends: Vec::new(),
            implements: Vec::new(),
            has_body: func.body.is_some(),
            optional: false,
            readonly: false,
            r#static: false,
            params: self.extract_params(func, &tags),
            return_type,
            return_members,
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
                    let (return_type, return_members) =
                        self.extract_return(&method.value, &method_tags);

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
                        extends: Vec::new(),
                        implements: Vec::new(),
                        has_body: false,
                        optional: method.optional,
                        readonly: false,
                        r#static: method.r#static,
                        params: self.extract_params(&method.value, &method_tags),
                        return_type,
                        return_members,
                        children: Vec::new(),
                        tags: method_tags,
                        type_parameters: self
                            .extract_type_parameters(method.value.type_parameters.as_ref()),
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

                    let mut params = Vec::new();
                    let mut return_type = None;
                    let mut return_members = Vec::new();
                    let mut type_parameters = Vec::new();
                    let mut property_members = Vec::new();
                    let type_annotation = prop.type_annotation.as_ref().map(|t| {
                        let ts_type = &t.type_annotation;
                        if let Some(metadata) =
                            self.extract_function_type_metadata(ts_type, &prop_tags)
                        {
                            params = metadata.params;
                            return_type = metadata.return_type;
                            return_members = metadata.return_members;
                            type_parameters = metadata.type_parameters;
                        }
                        property_members = self.extract_type_alias_members_from_type(ts_type);

                        self.format_ts_type(ts_type)
                    });

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
                        extends: Vec::new(),
                        implements: Vec::new(),
                        has_body: false,
                        optional: prop.optional,
                        readonly: prop.readonly,
                        r#static: prop.r#static,
                        params,
                        return_type,
                        return_members,
                        children: property_members,
                        tags: prop_tags,
                        type_parameters,
                    });
                }
                oxc_ast::ast::ClassElement::TSIndexSignature(index_signature) => {
                    if let Some(item) = self.create_index_signature_item(index_signature) {
                        children.push(item);
                    }
                }
                _ => {}
            }
        }

        let extends = self.extract_class_extends(class);
        let implements = self.extract_class_implements(class);
        let signature = self.format_class_signature(class, name, exported, &extends, &implements);

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
            signature: Some(signature),
            extends,
            implements,
            has_body: false,
            optional: false,
            readonly: false,
            r#static: false,
            params: Vec::new(),
            return_type: None,
            return_members: Vec::new(),
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

                    let mut params = Vec::new();
                    let mut return_type = None;
                    let mut return_members = Vec::new();
                    let mut type_parameters = Vec::new();
                    let mut property_members = Vec::new();
                    let type_annotation = prop.type_annotation.as_ref().map(|t| {
                        let ts_type = &t.type_annotation;
                        if let Some(metadata) =
                            self.extract_function_type_metadata(ts_type, &prop_tags)
                        {
                            params = metadata.params;
                            return_type = metadata.return_type;
                            return_members = metadata.return_members;
                            type_parameters = metadata.type_parameters;
                        }
                        property_members = self.extract_type_alias_members_from_type(ts_type);

                        self.format_ts_type(ts_type)
                    });

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
                        extends: Vec::new(),
                        implements: Vec::new(),
                        has_body: false,
                        optional: prop.optional,
                        readonly: prop.readonly,
                        r#static: false,
                        params,
                        return_type,
                        return_members,
                        children: property_members,
                        tags: prop_tags,
                        type_parameters,
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
                    let (return_type, return_members) = self.extract_return_from_annotation(
                        method.return_type.as_deref(),
                        &method_tags,
                    );

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
                        extends: Vec::new(),
                        implements: Vec::new(),
                        has_body: false,
                        optional: method.optional,
                        readonly: false,
                        r#static: false,
                        params: self.extract_params_from_formals(&method.params, &method_tags),
                        return_type,
                        return_members,
                        children: Vec::new(),
                        tags: method_tags,
                        type_parameters: self
                            .extract_type_parameters(method.type_parameters.as_ref()),
                    });
                }
                TSSignature::TSIndexSignature(index_signature) => {
                    if let Some(item) = self.create_index_signature_item(index_signature) {
                        children.push(item);
                    }
                }
                _ => {}
            }
        }

        children
    }

    fn extract_type_alias_members_from_type(&self, ts_type: &TSType<'a>) -> Vec<DocItem> {
        match ts_type {
            TSType::TSTypeLiteral(type_literal) => {
                self.extract_ts_signature_members(&type_literal.members)
            }
            TSType::TSParenthesizedType(paren) => {
                self.extract_type_alias_members_from_type(&paren.type_annotation)
            }
            TSType::TSIntersectionType(intersection) => intersection
                .types
                .iter()
                .flat_map(|ts_type| self.extract_type_alias_members_from_type(ts_type))
                .collect(),
            _ => Vec::new(),
        }
    }

    fn create_index_signature_item(
        &self,
        index_signature: &TSIndexSignature<'a>,
    ) -> Option<DocItem> {
        let parameter = index_signature.parameters.first()?;
        let (name, param_name, param_type) = self.format_index_signature_name(parameter);
        let value_type = self.format_ts_type(&index_signature.type_annotation.type_annotation);
        let signature = Self::format_index_signature(index_signature, &name, &value_type);

        let (jsdoc, doc, tags) = self
            .extract_jsdoc(index_signature.span.start)
            .map_or((None, None, Vec::new()), |(jsdoc, doc, tags)| {
                (Some(jsdoc), (!doc.is_empty()).then_some(doc), tags)
            });
        if self.should_skip_by_visibility(&tags) {
            return None;
        }
        let (line, end_line) =
            self.span_lines(index_signature.span.start, index_signature.span.end);
        let params = Vec::from([ParamDoc {
            name: param_name,
            type_annotation: Some(param_type),
            optional: false,
            default_value: None,
            description: None,
        }]);

        Some(DocItem {
            name,
            kind: DocItemKind::IndexSignature,
            doc,
            source_path: self.file_path.to_string(),
            line,
            end_line,
            column: self.column_number(index_signature.span.start),
            jsdoc,
            exported: false,
            signature: Some(signature),
            extends: Vec::new(),
            implements: Vec::new(),
            has_body: false,
            optional: false,
            readonly: index_signature.readonly,
            r#static: index_signature.r#static,
            params,
            return_type: Some(value_type),
            return_members: Vec::new(),
            children: Vec::new(),
            tags,
            type_parameters: Vec::new(),
        })
    }

    fn format_index_signature_name(
        &self,
        parameter: &TSIndexSignatureName<'a>,
    ) -> (String, String, String) {
        let param_name = parameter.name.to_string();
        let param_type = self.format_ts_type(&parameter.type_annotation.type_annotation);
        let name = join5("[", &param_name, ": ", &param_type, "]");
        (name, param_name, param_type)
    }

    fn format_index_signature(
        index_signature: &TSIndexSignature<'a>,
        name: &str,
        value_type: &str,
    ) -> String {
        let mut signature = StringBuilder::new();
        if index_signature.readonly {
            signature.push_str("readonly ");
        }
        signature.push_str(name);
        signature.push_str(": ");
        signature.push_str(value_type);
        signature.into_string()
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
                Expression::ArrowFunctionExpression(arrow) => {
                    let (return_type, return_members) =
                        self.extract_return_from_annotation(arrow.return_type.as_deref(), &tags);
                    DocItem {
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
                        extends: Vec::new(),
                        implements: Vec::new(),
                        has_body: false,
                        optional: false,
                        readonly: false,
                        r#static: false,
                        params: self.extract_params_from_formals(&arrow.params, &tags),
                        return_type,
                        return_members,
                        children: Vec::new(),
                        tags: tags.clone(),
                        type_parameters: self
                            .extract_type_parameters(arrow.type_parameters.as_ref()),
                    }
                }
                Expression::FunctionExpression(func_expr) => {
                    let (return_type, return_members) = self.extract_return(func_expr, &tags);
                    DocItem {
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
                        extends: Vec::new(),
                        implements: Vec::new(),
                        has_body: false,
                        optional: false,
                        readonly: false,
                        r#static: false,
                        params: self.extract_params(func_expr, &tags),
                        return_type,
                        return_members,
                        children: Vec::new(),
                        tags: tags.clone(),
                        type_parameters: self
                            .extract_type_parameters(func_expr.type_parameters.as_ref()),
                    }
                }
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
                    extends: Vec::new(),
                    implements: Vec::new(),
                    has_body: false,
                    optional: false,
                    readonly: false,
                    r#static: false,
                    params: Vec::new(),
                    return_type: None,
                    return_members: Vec::new(),
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
        let name = type_alias.id.name.to_string();
        if let Some(metadata) =
            self.extract_function_type_metadata(&type_alias.type_annotation, &[])
        {
            self.type_alias_function_metadata
                .insert(name.clone(), metadata.as_reference_metadata());
        }

        let Some((jsdoc, doc, tags)) = self.extract_declaration_docs(attached_to) else {
            return;
        };
        let (line, end_line) = self.span_lines(attached_to, type_alias.span.end);
        let children = self.extract_type_alias_members_from_type(&type_alias.type_annotation);
        let (params, return_type, return_members, function_type_parameters) =
            if let Some(metadata) =
                self.extract_function_type_metadata(&type_alias.type_annotation, &tags)
            {
                (
                    metadata.params,
                    metadata.return_type,
                    metadata.return_members,
                    metadata.type_parameters,
                )
            } else {
                (Vec::new(), None, Vec::new(), Vec::new())
            };

        self.items.push(DocItem {
            name,
            kind: DocItemKind::Type,
            doc,
            source_path: self.file_path.to_string(),
            line,
            end_line,
            column: self.column_number(attached_to),
            jsdoc,
            exported,
            signature: Some(self.format_type_alias_signature(type_alias, exported)),
            extends: Vec::new(),
            implements: Vec::new(),
            has_body: false,
            optional: false,
            readonly: false,
            r#static: false,
            params,
            return_type,
            return_members,
            children,
            tags,
            type_parameters: {
                let mut type_parameters =
                    self.extract_type_parameters(type_alias.type_parameters.as_ref());
                type_parameters.extend(function_type_parameters);
                type_parameters
            },
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
        let extends = self.extract_interface_extends(interface);
        let signature = self.format_interface_signature(interface, exported, &extends);

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
            signature: Some(signature),
            extends,
            implements: Vec::new(),
            has_body: false,
            optional: false,
            readonly: false,
            r#static: false,
            params: Vec::new(),
            return_type: None,
            return_members: Vec::new(),
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
            extends: Vec::new(),
            implements: Vec::new(),
            has_body: false,
            optional: false,
            readonly: false,
            r#static: false,
            params: Vec::new(),
            return_type: None,
            return_members: Vec::new(),
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
            extends: Vec::new(),
            implements: Vec::new(),
            has_body: false,
            optional: false,
            readonly: false,
            r#static: false,
            params: Vec::new(),
            return_type: None,
            return_members: Vec::new(),
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
    fn function_object_literal_parameter_preserves_type_and_members() {
        let source = r"
/**
 * Define a plugin.
 *
 * @param options - Plugin options.
 * @param options.id - Plugin id.
 * @param options.name - Plugin display name.
 * @param options.setup - Setup hook.
 */
export function plugin<Id, Deps, PluginExt, MergedExtensions>(options: {
    id: Id;
    name?: string;
    dependencies?: Deps;
    setup?: (
        ctx: Readonly<
            PluginContext<MergedExtensions>
        >
    ) => Awaitable<void>;
    extension: PluginExt;
    onExtension?: OnPluginExtension<MergedExtensions>;
}): PluginWithExtension<PluginExt>;
";

        let extractor = DocExtractor::new();
        let items = extractor.extract_source(source, "plugin.ts", SourceType::ts()).unwrap();
        let plugin = items.iter().find(|item| item.name == "plugin").unwrap();

        assert_eq!(plugin.params.len(), 7);
        assert_eq!(plugin.params[0].name, "options");
        let parent_type = plugin.params[0].type_annotation.as_deref().unwrap();
        assert_ne!(parent_type, "{ ... }");
        assert!(parent_type.contains("id: Id"));
        assert!(parent_type.contains("name?: string"));
        assert!(parent_type.contains(
            "setup?: (ctx: Readonly<PluginContext<MergedExtensions>>) => Awaitable<void>"
        ));
        assert_eq!(plugin.params[0].description.as_deref(), Some("Plugin options."));

        let names = plugin.params.iter().map(|param| param.name.as_str()).collect::<Vec<_>>();
        assert_eq!(
            names,
            [
                "options",
                "options.id",
                "options.name?",
                "options.dependencies?",
                "options.setup?",
                "options.extension",
                "options.onExtension?",
            ]
        );
        assert_eq!(plugin.params[1].type_annotation.as_deref(), Some("Id"));
        assert_eq!(plugin.params[1].description.as_deref(), Some("Plugin id."));
        assert_eq!(plugin.params[2].description.as_deref(), Some("Plugin display name."));
        assert_eq!(
            plugin.params[4].type_annotation.as_deref(),
            Some("(ctx: Readonly<PluginContext<MergedExtensions>>) => Awaitable<void>")
        );
        assert_eq!(plugin.params[4].description.as_deref(), Some("Setup hook."));
        assert!(plugin.params[2].optional);
        assert!(plugin.params[3].optional);
        assert!(plugin.params[4].optional);
        assert!(plugin.params[6].optional);
    }

    #[test]
    fn destructured_parameter_uses_jsdoc_name_and_extracted_type() {
        let source = r"
/**
 * Resolve command line arguments.
 *
 * @param args - Argument schema.
 * @param tokens - Parsed tokens.
 * @param resolveArgs - Resolve options.
 */
export declare function resolveArgs<A extends Args>(
    args: A,
    tokens: ArgToken[],
    { shortGrouping, skipPositional, toKebab }?: ResolveArgs
): void;
";

        let extractor = DocExtractor::new();
        let items = extractor.extract_source(source, "resolver.ts", SourceType::ts()).unwrap();
        let resolve = items.iter().find(|item| item.name == "resolveArgs").unwrap();

        assert_eq!(
            resolve.params.iter().map(|param| param.name.as_str()).collect::<Vec<_>>(),
            ["args", "tokens", "resolveArgs"]
        );
        assert_eq!(resolve.params[2].type_annotation.as_deref(), Some("ResolveArgs"));
        assert!(resolve.params[2].optional);
        assert_eq!(resolve.params[2].description.as_deref(), Some("Resolve options."));
    }

    #[test]
    fn destructured_parameter_without_jsdoc_name_keeps_param_fallback() {
        let source = r"
/**
 * Run a command.
 */
export declare function run({ cwd }: RunOptions): void;
";

        let extractor = DocExtractor::new();
        let items = extractor.extract_source(source, "runner.ts", SourceType::ts()).unwrap();
        let run = items.iter().find(|item| item.name == "run").unwrap();

        assert_eq!(run.params.len(), 1);
        assert_eq!(run.params[0].name, "param");
        assert_eq!(run.params[0].type_annotation.as_deref(), Some("RunOptions"));
        assert_eq!(run.params[0].description, None);
    }

    #[test]
    fn destructured_parameter_keeps_nested_param_tags_on_members() {
        let source = r"
/**
 * Run a command.
 *
 * @param options - Runtime options.
 * @param options.cwd - Working directory.
 */
export declare function run({ cwd }: { cwd: string }): void;
";

        let extractor = DocExtractor::new();
        let items = extractor.extract_source(source, "runner.ts", SourceType::ts()).unwrap();
        let run = items.iter().find(|item| item.name == "run").unwrap();

        assert_eq!(
            run.params.iter().map(|param| param.name.as_str()).collect::<Vec<_>>(),
            ["options", "options.cwd"]
        );
        assert_eq!(run.params[0].type_annotation.as_deref(), Some("{ cwd: string }"));
        assert_eq!(run.params[0].description.as_deref(), Some("Runtime options."));
        assert_eq!(run.params[1].type_annotation.as_deref(), Some("string"));
        assert_eq!(run.params[1].description.as_deref(), Some("Working directory."));
    }

    #[test]
    fn function_return_type_literal_emits_return_members() {
        let source = r"
/**
 * Resolve arguments.
 * @returns Resolved args.
 */
export function resolveArgs<A extends Args>(): {
    values: ArgValues<A>;
    positionals: string[];
    error: AggregateError | undefined;
} {
    return {} as any;
}
";

        let extractor = DocExtractor::new();
        let items = extractor.extract_source(source, "resolver.ts", SourceType::ts()).unwrap();

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].return_type.as_deref(), Some("object"));
        assert_eq!(
            items[0].return_members.iter().map(|member| member.name.as_str()).collect::<Vec<_>>(),
            ["values", "positionals", "error"]
        );
        assert_eq!(items[0].return_members[0].signature.as_deref(), Some("ArgValues<A>"));
        assert_eq!(items[0].return_members[1].signature.as_deref(), Some("string[]"));
        assert_eq!(
            items[0].return_members[2].signature.as_deref(),
            Some("AggregateError | undefined")
        );
    }

    #[test]
    fn index_signatures_are_extracted_from_members_and_return_literals() {
        let source = r"
/**
 * Value type.
 */
export interface ArgSchema {}

/**
 * Arguments.
 */
export interface Args {
    /** Argument schema by option name. */
    readonly [option: string]: ArgSchema;
}

/**
 * Numeric arguments.
 */
export type NumericArgs = {
    [index: number]: ArgSchema;
};

/**
 * Argument store.
 */
export class Store {
    [key: string]: ArgSchema;
}

/**
 * Makes arguments.
 */
export function makeArgs(): {
    [key: string]: ArgSchema;
} {
    return {} as any;
}
";

        let extractor = DocExtractor::new();
        let items = extractor.extract_source(source, "args.ts", SourceType::ts()).unwrap();
        let args = items.iter().find(|item| item.name == "Args").unwrap();
        let numeric_args = items.iter().find(|item| item.name == "NumericArgs").unwrap();
        let store = items.iter().find(|item| item.name == "Store").unwrap();
        let make_args = items.iter().find(|item| item.name == "makeArgs").unwrap();

        let args_member = &args.children[0];
        assert_eq!(args_member.kind, DocItemKind::IndexSignature);
        assert_eq!(args_member.name, "[option: string]");
        assert_eq!(args_member.signature.as_deref(), Some("readonly [option: string]: ArgSchema"));
        assert_eq!(args_member.return_type.as_deref(), Some("ArgSchema"));
        assert_eq!(args_member.params[0].name, "option");
        assert_eq!(args_member.params[0].type_annotation.as_deref(), Some("string"));
        assert!(args_member.readonly);

        let numeric_member = &numeric_args.children[0];
        assert_eq!(numeric_member.kind, DocItemKind::IndexSignature);
        assert_eq!(numeric_member.name, "[index: number]");
        assert_eq!(numeric_member.signature.as_deref(), Some("[index: number]: ArgSchema"));

        let store_member = &store.children[0];
        assert_eq!(store_member.kind, DocItemKind::IndexSignature);
        assert_eq!(store_member.signature.as_deref(), Some("[key: string]: ArgSchema"));

        let return_member = &make_args.return_members[0];
        assert_eq!(return_member.kind, DocItemKind::IndexSignature);
        assert_eq!(return_member.signature.as_deref(), Some("[key: string]: ArgSchema"));
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
    fn extracts_interface_extends_and_class_implements() {
        let source = r"
/**
 * Base runtime adapter.
 */
export interface BaseAdapter {}

/**
 * Runtime adapter.
 */
export interface TranslationAdapter extends BaseAdapter {
    /**
     * Gets a locale resource.
     * @param locale - Locale name.
     * @returns The locale resource.
     */
    getResource(locale: string): Record<string, string> | undefined;
}

/**
 * Default runtime adapter.
 */
export class DefaultTranslation implements TranslationAdapter {
    /**
     * Gets a locale resource.
     * @param locale - Locale name.
     * @returns The locale resource.
     */
    getResource(locale: string): Record<string, string> | undefined {
        return undefined;
    }
}
";

        let extractor = DocExtractor::new();
        let items = extractor.extract_source(source, "adapter.ts", SourceType::ts()).unwrap();
        let adapter = items.iter().find(|item| item.name == "TranslationAdapter").unwrap();
        let implementation = items.iter().find(|item| item.name == "DefaultTranslation").unwrap();

        assert_eq!(adapter.kind, DocItemKind::Interface);
        assert_eq!(adapter.extends, vec!["BaseAdapter"]);
        assert_eq!(implementation.kind, DocItemKind::Class);
        assert_eq!(implementation.implements, vec!["TranslationAdapter"]);
        assert_eq!(
            implementation.signature.as_deref(),
            Some("export class DefaultTranslation implements TranslationAdapter")
        );
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
    fn interface_property_type_literal_emits_property_members() {
        let source = r"
/**
 * Request options.
 */
export interface RequestOptions {
    /** HTTP options. */
    http: {
        /** Request timeout. */
        timeout?: number;
        /** Request headers. */
        headers: Record<string, string>;
    };
}
";

        let extractor = DocExtractor::new();
        let items = extractor.extract_source(source, "request.ts", SourceType::ts()).unwrap();
        let http = &items[0].children[0];

        assert_eq!(http.name, "http");
        assert_eq!(http.kind, DocItemKind::Property);
        assert_eq!(http.children.len(), 2);
        assert_eq!(http.children[0].name, "timeout");
        assert_eq!(http.children[0].doc.as_deref(), Some("Request timeout."));
        assert_eq!(http.children[0].signature.as_deref(), Some("number"));
        assert!(http.children[0].optional);
        assert_eq!(http.children[1].name, "headers");
        assert_eq!(http.children[1].signature.as_deref(), Some("Record<string, string>"));
    }

    #[test]
    fn type_alias_signature_omits_nested_property_jsdoc_comments() {
        let source = r"
/**
 * A combinator produced by combinator factory functions.
 */
export type Combinator<T> = {
    /**
     * The parse function that converts a string to the desired type.
     *
     * @param value - The input string value.
     * @returns The parsed value of type T.
     */
    parse: (value: string) => T;
};
";

        let extractor = DocExtractor::new();
        let items = extractor.extract_source(source, "combinators.ts", SourceType::ts()).unwrap();
        let combinator = items.iter().find(|item| item.name == "Combinator").unwrap();
        let signature = combinator.signature.as_deref().unwrap();

        assert!(!signature.contains("/**"));
        assert!(!signature.contains("@param"));
        assert!(!signature.contains("@returns"));
        assert_eq!(signature, "export type Combinator<T> = { parse: (value: string) => T }");

        let parse = &combinator.children[0];
        assert_eq!(
            parse.doc.as_deref(),
            Some("The parse function that converts a string to the desired type.")
        );
        assert_eq!(parse.signature.as_deref(), Some("(value: string) => T"));
        assert_eq!(parse.params[0].name, "value");
        assert_eq!(parse.params[0].description.as_deref(), Some("The input string value."));
        assert_eq!(parse.return_type.as_deref(), Some("T"));
        let returns_tag = parse.tags.iter().find(|tag| tag.tag == "returns").unwrap();
        assert_eq!(returns_tag.description.as_deref(), Some("The parsed value of type T."));
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
        let signature = items[0].signature.as_deref().unwrap();
        let member = &items[0].children[0];

        assert!(!signature.contains("/**"));
        assert!(!signature.contains("@param"));
        assert_eq!(signature, "export type CommandOptions = { run(ctx: Context): void }");
        assert_eq!(member.name, "run");
        assert_eq!(member.kind, DocItemKind::Method);
        assert_eq!(member.signature.as_deref(), Some("run(ctx: Context): void"));
        assert_eq!(member.params.len(), 1);
        assert_eq!(member.params[0].description.as_deref(), Some("Runtime context"));
    }

    #[test]
    fn type_alias_intersection_extracts_object_literal_members() {
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
        assert_eq!(items[0].children.len(), 1);
        let signature = items[0].signature.as_deref().unwrap();
        assert!(!signature.contains("/**"));
        assert!(signature.contains("BaseOptions & { name: string }"));
        assert_eq!(items[0].children[0].name, "name");
        assert_eq!(items[0].children[0].signature.as_deref(), Some("string"));
        assert!(items[0].signature.as_deref().unwrap().contains("BaseOptions &"));
    }

    #[test]
    fn type_alias_intersection_resolves_callable_alias_and_members() {
        let source = r"
/**
 * Plugin function.
 */
export type PluginFunction<G> = (ctx: Readonly<PluginContext<G>>) => Awaitable<void>;

/**
 * Plugin.
 * @param ctx - Plugin context.
 * @returns Plugin setup result.
 */
export type Plugin<E> = PluginFunction & {
    id: string;
    name?: string;
};
";

        let extractor = DocExtractor::new();
        let items = extractor.extract_source(source, "plugin.ts", SourceType::ts()).unwrap();
        let plugin = items.iter().find(|item| item.name == "Plugin").unwrap();

        assert_eq!(plugin.params.len(), 1);
        assert_eq!(plugin.params[0].name, "ctx");
        assert_eq!(plugin.params[0].type_annotation.as_deref(), Some("Readonly<PluginContext<G>>"));
        assert_eq!(plugin.params[0].description, None);
        assert_eq!(plugin.return_type.as_deref(), Some("Awaitable<void>"));
        assert_eq!(
            plugin.children.iter().map(|child| child.name.as_str()).collect::<Vec<_>>(),
            ["id", "name"]
        );
        assert_eq!(plugin.children[0].signature.as_deref(), Some("string"));
        assert_eq!(plugin.children[1].signature.as_deref(), Some("string"));
        assert!(plugin.children[1].optional);
    }

    #[test]
    fn function_valued_interface_property_extracts_params_and_returns() {
        let source = r"
/**
 * Options for parsing.
 */
export interface ArgSchema {
    /**
     * Parse a value.
     */
    parse?: (value: string) => any;
}
";

        let extractor = DocExtractor::new();
        let items = extractor.extract_source(source, "options.ts", SourceType::ts()).unwrap();
        let schema = items.iter().find(|item| item.name == "ArgSchema").unwrap();
        let parse = schema.children.iter().find(|member| member.name == "parse").unwrap();

        assert_eq!(parse.kind, DocItemKind::Property);
        assert_eq!(parse.signature.as_deref(), Some("(value: string) => any"));
        assert_eq!(parse.params.len(), 1);
        assert_eq!(parse.params[0].name, "value");
        assert_eq!(parse.params[0].type_annotation.as_deref(), Some("string"));
        assert_eq!(parse.return_type.as_deref(), Some("any"));
    }

    #[test]
    fn function_valued_class_property_extracts_params_and_returns() {
        let source = r"
/**
 * Argument parser.
 */
export class ArgParser {
    /**
     * Parse a value.
     */
    parse: (value: string) => any;
}
";

        let extractor = DocExtractor::new();
        let items = extractor.extract_source(source, "parser.ts", SourceType::ts()).unwrap();
        let parser = items.iter().find(|item| item.name == "ArgParser").unwrap();
        let parse = parser.children.iter().find(|member| member.name == "parse").unwrap();

        assert_eq!(parse.kind, DocItemKind::Property);
        assert_eq!(parse.signature.as_deref(), Some("(value: string) => any"));
        assert_eq!(parse.params[0].name, "value");
        assert_eq!(parse.params[0].type_annotation.as_deref(), Some("string"));
        assert_eq!(parse.return_type.as_deref(), Some("any"));
    }

    #[test]
    fn readonly_type_and_parenthesized_union_preserve_types() {
        let source = r"
/**
 * Command arguments.
 */
export interface ArgSchema {
    /**
     * Parse a value.
     */
    choices?: string[] | readonly string[];
}

/**
 * Example rendering hooks.
 */
export interface SubCommandable {
    examples?: string | ((...args: any[]) => any);
}
";

        let extractor = DocExtractor::new();
        let items = extractor.extract_source(source, "schemas.ts", SourceType::ts()).unwrap();
        let arg_schema = items.iter().find(|item| item.name == "ArgSchema").unwrap();
        let sub = items.iter().find(|item| item.name == "SubCommandable").unwrap();

        let choices = arg_schema.children.iter().find(|member| member.name == "choices").unwrap();
        assert_eq!(choices.signature.as_deref(), Some("string[] | readonly string[]"));

        let examples = sub.children.iter().find(|member| member.name == "examples").unwrap();
        assert_eq!(examples.signature.as_deref(), Some("string | ((...args: any[]) => any)"));
    }

    #[test]
    fn type_alias_function_extracts_params_and_returns() {
        let source = r"
/**
 * Run a command.
 */
export type CommandRunner<G> = (ctx: Readonly<CommandContext<G>>) => Awaitable<string | void>;
";

        let extractor = DocExtractor::new();
        let items = extractor.extract_source(source, "runner.ts", SourceType::ts()).unwrap();
        let alias = items.iter().find(|item| item.name == "CommandRunner").unwrap();

        assert_eq!(alias.params.len(), 1);
        assert_eq!(alias.params[0].name, "ctx");
        assert_eq!(alias.params[0].type_annotation.as_deref(), Some("Readonly<CommandContext<G>>"));
        assert_eq!(alias.return_type.as_deref(), Some("Awaitable<string | void>"));
    }

    #[test]
    fn type_alias_function_with_multiple_parameters_extracts_all_params_and_return() {
        let source = r"
/**
 * Extend a command.
 */
export type PluginExtension<T, G> = (ctx: CommandContextCore<G>, cmd: Command<G>) => Awaitable<T>;
";

        let extractor = DocExtractor::new();
        let items = extractor.extract_source(source, "plugin.ts", SourceType::ts()).unwrap();
        let alias = items.iter().find(|item| item.name == "PluginExtension").unwrap();

        assert_eq!(alias.params.len(), 2);
        assert_eq!(alias.params[0].name, "ctx");
        assert_eq!(alias.params[0].type_annotation.as_deref(), Some("CommandContextCore<G>"));
        assert_eq!(alias.params[1].name, "cmd");
        assert_eq!(alias.params[1].type_annotation.as_deref(), Some("Command<G>"));
        assert_eq!(alias.return_type.as_deref(), Some("Awaitable<T>"));
    }

    #[test]
    fn type_alias_function_with_jsdoc_params_but_no_returns_tag_extracts_return() {
        let source = r"
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
";

        let extractor = DocExtractor::new();
        let items = extractor.extract_source(source, "plugin.ts", SourceType::ts()).unwrap();
        let alias = items.iter().find(|item| item.name == "OnPluginExtension").unwrap();

        assert_eq!(alias.params.len(), 2);
        assert_eq!(alias.params[0].name, "ctx");
        assert_eq!(alias.params[0].type_annotation.as_deref(), Some("Readonly<CommandContext<G>>"));
        assert_eq!(alias.params[0].description.as_deref(), Some("The command context."));
        assert_eq!(alias.params[1].name, "cmd");
        assert_eq!(alias.params[1].type_annotation.as_deref(), Some("Readonly<Command<G>>"));
        assert_eq!(alias.params[1].description.as_deref(), Some("The command."));
        assert_eq!(alias.return_type.as_deref(), Some("Awaitable<void>"));
    }

    #[test]
    fn type_alias_function_with_function_param_and_return_extracts_nested_function_types() {
        let source = r"
/**
 * Decorate a runner.
 */
export type CommandDecorator<G> = (
    baseRunner: (ctx: Readonly<CommandContext<G>>) => Awaitable<string | void>
) => (ctx: Readonly<CommandContext<G>>) => Awaitable<string | void>;
";

        let extractor = DocExtractor::new();
        let items = extractor.extract_source(source, "decorator.ts", SourceType::ts()).unwrap();
        let alias = items.iter().find(|item| item.name == "CommandDecorator").unwrap();

        assert_eq!(alias.params.len(), 1);
        assert_eq!(alias.params[0].name, "baseRunner");
        assert_eq!(
            alias.params[0].type_annotation.as_deref(),
            Some("(ctx: Readonly<CommandContext<G>>) => Awaitable<string | void>"),
        );
        assert_eq!(
            alias.return_type.as_deref(),
            Some("(ctx: Readonly<CommandContext<G>>) => Awaitable<string | void>"),
        );
    }

    #[test]
    fn type_alias_function_without_jsdoc_tags_still_extracts_metadata() {
        let source = r"
/**
 * Plugin function.
 */
export type PluginFunction<G> = (ctx: Readonly<PluginContext<G>>) => Awaitable<void>;
";

        let extractor = DocExtractor::new();
        let items = extractor.extract_source(source, "plugin.ts", SourceType::ts()).unwrap();
        let alias = items.iter().find(|item| item.name == "PluginFunction").unwrap();

        assert_eq!(alias.params.len(), 1);
        assert_eq!(alias.params[0].name, "ctx");
        assert_eq!(alias.params[0].type_annotation.as_deref(), Some("Readonly<PluginContext<G>>"));
        assert_eq!(alias.return_type.as_deref(), Some("Awaitable<void>"));
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
    fn test_extract_member_type_parameters() {
        let source = r"
/** Plugin context. */
export interface PluginContext<G> {
  /**
   * Decorate the command.
   * @typeParam L - Extension context.
   */
  decorateCommand<L extends Record<string, unknown> = DefaultExtensions>(
    decorator: (value: L) => void
  ): void;

  /**
   * Setup hook.
   * @typeParam T - Hook value.
   */
  setup?: <T extends BaseHook = DefaultHook>(value: T) => Result;
}
";
        let extractor = DocExtractor::new();
        let items = extractor.extract_source(source, "src/context.ts", SourceType::ts()).unwrap();
        let interface = items.iter().find(|item| item.name == "PluginContext").unwrap();

        let method = interface.children.iter().find(|item| item.name == "decorateCommand").unwrap();
        assert_eq!(method.type_parameters.len(), 1);
        assert_eq!(method.type_parameters[0].name, "L");
        assert_eq!(
            method.type_parameters[0].constraint.as_deref(),
            Some("Record<string, unknown>")
        );
        assert_eq!(method.type_parameters[0].default.as_deref(), Some("DefaultExtensions"));

        let property = interface.children.iter().find(|item| item.name == "setup").unwrap();
        assert_eq!(property.type_parameters.len(), 1);
        assert_eq!(property.type_parameters[0].name, "T");
        assert_eq!(property.type_parameters[0].constraint.as_deref(), Some("BaseHook"));
        assert_eq!(property.type_parameters[0].default.as_deref(), Some("DefaultHook"));
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
