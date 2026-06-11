use oxc_ast::ast::{TSSignature, TSType, TSTypeLiteral, TSTypeName};
use oxc_span::GetSpan;

use crate::string_builder::{join2, join3, StringBuilder};

use super::DocVisitor;

impl<'a> DocVisitor<'a> {
    /// Format a TypeScript type.
    pub(super) fn format_ts_type(&self, ts_type: &TSType) -> String {
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

    pub(super) fn property_key_name(key: &oxc_ast::ast::PropertyKey<'a>) -> Option<String> {
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
    pub(super) fn format_ts_type_name(name: &TSTypeName) -> String {
        match name {
            TSTypeName::IdentifierReference(id) => id.name.to_string(),
            TSTypeName::QualifiedName(qn) => {
                join3(&Self::format_ts_type_name(&qn.left), ".", qn.right.name.as_str())
            }
            TSTypeName::ThisExpression(_) => "this".to_string(),
        }
    }
}
