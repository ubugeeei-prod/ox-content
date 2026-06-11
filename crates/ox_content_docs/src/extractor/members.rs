use oxc_ast::ast::{TSIndexSignature, TSIndexSignatureName, TSSignature, TSType};

use crate::string_builder::{join5, StringBuilder};

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
