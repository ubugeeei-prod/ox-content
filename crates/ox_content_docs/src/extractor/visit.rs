use oxc_ast::ast::{
    BindingPattern, Declaration, ExportDefaultDeclarationKind, Expression, Statement,
    VariableDeclaration,
};
use oxc_ast_visit::{walk, Visit};
use oxc_span::GetSpan;

use super::{DocItem, DocItemKind, DocVisitor};

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
}
