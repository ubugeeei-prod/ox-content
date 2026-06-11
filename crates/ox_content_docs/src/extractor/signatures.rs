use oxc_ast::ast::{Class, Expression, Function, TSTypeAnnotation};
use oxc_span::GetSpan;

use super::DocVisitor;

impl<'a> DocVisitor<'a> {
    pub(super) fn format_function_signature(
        &self,
        func: &Function<'a>,
        name: &str,
        exported: bool,
    ) -> String {
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

    pub(super) fn format_assigned_function_signature(
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

    pub(super) fn format_class_signature(
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

    pub(super) fn extract_class_extends(&self, class: &Class<'a>) -> Vec<String> {
        let Some(super_class) = &class.super_class else {
            return Vec::new();
        };
        let mut value = self.slice(super_class.span().start, super_class.span().end);
        if let Some(type_params) = &class.super_type_arguments {
            value.push_str(&self.format_type_parameter_declaration(Some(type_params)));
        }
        Vec::from([value])
    }

    pub(super) fn extract_class_implements(&self, class: &Class<'a>) -> Vec<String> {
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

    pub(super) fn format_interface_signature(
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

    pub(super) fn extract_interface_extends(
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

    pub(super) fn format_type_alias_signature(
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

    pub(super) fn format_variable_signature(
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
}
