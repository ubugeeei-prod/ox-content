use oxc_ast::ast::{Class, Function};

use super::{DocItem, DocItemKind, DocVisitor};

impl<'a> DocVisitor<'a> {
    /// Create a DocItem from a function.
    pub(super) fn create_function_item(
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
    pub(super) fn create_class_item(
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
}
