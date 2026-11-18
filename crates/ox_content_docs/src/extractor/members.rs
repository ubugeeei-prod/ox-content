use oxc_ast::ast::{
    FormalParameters, TSIndexSignature, TSIndexSignatureName, TSSignature, TSType,
    TSTypeAnnotation, TSTypeParameterDeclaration,
};
use oxc_span::Span;

use crate::string_builder::{StringBuilder, join5};

use super::{DocItem, DocItemKind, DocVisitor, ParamDoc};

impl<'a> DocVisitor<'a> {
    pub(super) fn extract_ts_signature_members(
        &self,
        signatures: &[TSSignature<'a>],
    ) -> Vec<DocItem> {
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

                    let kind = match method.kind {
                        oxc_ast::ast::TSMethodSignatureKind::Method => DocItemKind::Method,
                        oxc_ast::ast::TSMethodSignatureKind::Get => DocItemKind::Getter,
                        oxc_ast::ast::TSMethodSignatureKind::Set => DocItemKind::Setter,
                    };
                    if let Some(mut item) = self.create_callable_member_item(
                        method_name,
                        kind,
                        method.span,
                        method.type_parameters.as_ref(),
                        &method.params,
                        method.return_type.as_ref(),
                    ) {
                        item.optional = method.optional;
                        children.push(item);
                    }
                }
                TSSignature::TSIndexSignature(index_signature) => {
                    if let Some(item) = self.create_index_signature_item(index_signature) {
                        children.push(item);
                    }
                }
                TSSignature::TSCallSignatureDeclaration(call) => {
                    children.extend(self.create_callable_member_item(
                        String::new(),
                        DocItemKind::CallSignature,
                        call.span,
                        call.type_parameters.as_ref(),
                        &call.params,
                        call.return_type.as_ref(),
                    ));
                }
                TSSignature::TSConstructSignatureDeclaration(construct) => {
                    children.extend(self.create_callable_member_item(
                        String::new(),
                        DocItemKind::ConstructSignature,
                        construct.span,
                        construct.type_parameters.as_ref(),
                        &construct.params,
                        construct.return_type.as_ref(),
                    ));
                }
            }
        }

        children
    }

    pub(super) fn extract_type_alias_members_from_type(
        &self,
        ts_type: &TSType<'a>,
    ) -> Vec<DocItem> {
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

    pub(super) fn create_index_signature_item(
        &self,
        index_signature: &TSIndexSignature<'a>,
    ) -> Option<DocItem> {
        let (name, param_name, param_type) =
            self.format_index_signature_name(&index_signature.parameter);
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

    /// Builds a member for a method signature, a call signature
    /// (`(a: string): string`) or a construct signature (`new (a: string): object`).
    ///
    /// Call and construct signatures have no name, so they get an empty `name`.
    /// The renderers label them, and a stable sort keeps overloads in source order.
    fn create_callable_member_item(
        &self,
        name: String,
        kind: DocItemKind,
        span: Span,
        type_parameters: Option<&oxc_allocator::Box<'a, TSTypeParameterDeclaration<'a>>>,
        params: &FormalParameters<'a>,
        return_type: Option<&oxc_allocator::Box<'a, TSTypeAnnotation<'a>>>,
    ) -> Option<DocItem> {
        let (jsdoc, doc, tags) = self
            .extract_jsdoc(span.start)
            .map_or((None, None, Vec::new()), |(jsdoc, doc, tags)| {
                (Some(jsdoc), (!doc.is_empty()).then_some(doc), tags)
            });
        if self.should_skip_by_visibility(&tags) {
            return None;
        }
        let (line, end_line) = self.span_lines(span.start, span.end);
        let (return_type_text, return_members) =
            self.extract_return_from_annotation(return_type.map(|t| &**t), &tags);
        let signature_name = match kind {
            DocItemKind::CallSignature => "",
            DocItemKind::ConstructSignature => "new ",
            _ => &name,
        };
        let signature = self.format_assigned_function_signature(
            signature_name,
            false,
            type_parameters,
            params,
            return_type,
        );

        Some(DocItem {
            name,
            kind,
            doc,
            source_path: self.file_path.to_string(),
            line,
            end_line,
            column: self.column_number(span.start),
            jsdoc,
            exported: false,
            signature: Some(signature),
            extends: Vec::new(),
            implements: Vec::new(),
            has_body: false,
            optional: false,
            readonly: false,
            r#static: false,
            params: self.extract_params_from_formals(params, &tags),
            return_type: return_type_text,
            return_members,
            children: Vec::new(),
            tags,
            type_parameters: self.extract_type_parameters(type_parameters),
        })
    }

    pub(super) fn format_index_signature_name(
        &self,
        parameter: &TSIndexSignatureName<'a>,
    ) -> (String, String, String) {
        let param_name = parameter.name.to_string();
        let param_type = self.format_ts_type(&parameter.type_annotation.type_annotation);
        let name = join5("[", &param_name, ": ", &param_type, "]");
        (name, param_name, param_type)
    }

    pub(super) fn format_index_signature(
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
