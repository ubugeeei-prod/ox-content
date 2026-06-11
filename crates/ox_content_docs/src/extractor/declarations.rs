use oxc_ast::ast::{
    TSEnumDeclaration, TSEnumMember, TSInterfaceDeclaration, TSTypeAliasDeclaration,
};
use oxc_span::GetSpan;

use super::{DocItem, DocItemKind, DocVisitor};

impl<'a> DocVisitor<'a> {
    pub(super) fn visit_type_alias_declaration(
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

    pub(super) fn visit_interface_declaration(
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

    pub(super) fn visit_enum_declaration(
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
