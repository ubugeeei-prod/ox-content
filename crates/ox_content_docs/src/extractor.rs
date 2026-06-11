//! Documentation extraction from source code using OXC parser.

mod context;
mod driver;
mod jsdoc;
mod jsdoc_tags;
mod model;
mod params;
mod returns;
mod tags;
mod types;

use oxc_ast::ast::{
    BindingPattern, Class, Declaration, ExportDefaultDeclarationKind, Expression, Function,
    Statement, TSEnumDeclaration, TSEnumMember, TSIndexSignature, TSIndexSignatureName,
    TSInterfaceDeclaration, TSSignature, TSType, TSTypeAliasDeclaration, TSTypeAnnotation,
    VariableDeclaration,
};
use oxc_ast_visit::{walk, Visit};
use oxc_span::GetSpan;
#[cfg(test)]
use oxc_span::SourceType;
use rustc_hash::FxHashMap;

#[allow(unused_imports)]
use crate::profile_span;
use crate::string_builder::{join5, StringBuilder};

use self::jsdoc::ParsedJsdoc;
use self::model::FunctionTypeMetadata;
pub use self::model::{
    DocItem, DocItemKind, DocTag, ExtractError, ExtractResult, ParamDoc, TypeParamDoc,
};

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
mod tests;
