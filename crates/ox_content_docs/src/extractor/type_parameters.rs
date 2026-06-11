use oxc_span::GetSpan;

use crate::string_builder::StringBuilder;

use super::{DocVisitor, TypeParamDoc};

impl<'a> DocVisitor<'a> {
    pub(super) fn format_type_parameter_declaration<T>(
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
    pub(super) fn extract_type_parameters(
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

    pub(super) fn format_formal_parameters(
        &self,
        params: &oxc_ast::ast::FormalParameters<'a>,
    ) -> String {
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

    pub(super) fn format_type_formal_parameters(
        &self,
        params: &oxc_ast::ast::FormalParameters<'a>,
    ) -> String {
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
}
