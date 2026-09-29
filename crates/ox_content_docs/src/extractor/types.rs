use oxc_ast::ast::{TSSignature, TSTupleElement, TSType, TSTypeLiteral, TSTypeName};
use oxc_ast_visit::Visit;
use oxc_span::GetSpan;

use crate::string_builder::{StringBuilder, join2, join3};

use super::DocVisitor;

/// Rebuilds a type from its source, with each type literal in it rebuilt from
/// the source of its members, joined by `; `. Members that the source separates
/// by line breaks alone keep a separator once whitespace is collapsed, and the
/// comments between members stay out of the type text.
struct TypeLiteralsInSource<'v, 'a> {
    visitor: &'v DocVisitor<'a>,
    out: String,
    cursor: u32,
}

impl<'v, 'a> TypeLiteralsInSource<'v, 'a> {
    fn new(visitor: &'v DocVisitor<'a>, start: u32) -> Self {
        Self { visitor, out: String::new(), cursor: start }
    }

    /// The rebuilt source, up to `end`.
    fn finish(mut self, end: u32) -> String {
        self.out.push_str(&self.visitor.slice(self.cursor, end));
        self.out
    }
}

impl<'a> Visit<'a> for TypeLiteralsInSource<'_, 'a> {
    fn visit_ts_type_literal(&mut self, it: &TSTypeLiteral<'a>) {
        self.out.push_str(&self.visitor.slice(self.cursor, it.span.start));
        let members = it
            .members
            .iter()
            .map(|member| {
                let span = member.span();
                let mut literals = TypeLiteralsInSource::new(self.visitor, span.start);
                literals.visit_ts_signature(member);
                let member = literals.finish(span.end);
                member.trim().trim_end_matches([';', ',']).trim_end().to_string()
            })
            .filter(|member| !member.is_empty())
            .collect::<Vec<_>>();
        if members.is_empty() {
            self.out.push_str("{}");
        } else {
            self.out.push_str(&join3("{ ", &members.join("; "), " }"));
        }
        self.cursor = it.span.end;
    }
}

impl<'a> DocVisitor<'a> {
    /// Format a TypeScript type.
    pub(super) fn format_ts_type(&self, ts_type: &TSType) -> String {
        // Flow types parse through a TypeScript rewrite, so the AST drops
        // Flow-only notation (`?T`, `{| |}`, `$Keys<T>` stays). Show the
        // original source instead; type literals are still formatted
        // structurally so member JSDoc comments stay out of the type text.
        if self.flow && !matches!(ts_type, TSType::TSTypeLiteral(_)) {
            let span = ts_type.span();
            return self.format_type_span(span.start, span.end);
        }
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
            TSType::TSTypeReference(_) => self.format_type_with_literals(ts_type),
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
                    .map(|element| self.format_ts_tuple_element(element))
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
            _ => self.format_type_with_literals(ts_type),
        }
    }

    fn format_ts_tuple_element(&self, element: &TSTupleElement<'a>) -> String {
        match element {
            TSTupleElement::TSOptionalType(optional) => {
                join2(&self.format_ts_type(&optional.type_annotation), "?")
            }
            TSTupleElement::TSRestType(rest) => {
                join2("...", &self.format_ts_type(&rest.type_annotation))
            }
            _ => element.as_ts_type().map_or_else(
                || self.format_type_span(element.span().start, element.span().end),
                |ts_type| self.format_ts_type(ts_type),
            ),
        }
    }

    /// Formats a type from its source, with the type literals in it rebuilt from
    /// the source of their members (see `TypeLiteralsInSource`). Without a type
    /// literal, it is the source as `format_type_span()` formats it.
    fn format_type_with_literals(&self, ts_type: &TSType<'a>) -> String {
        let span = ts_type.span();
        let mut literals = TypeLiteralsInSource::new(self, span.start);
        literals.visit_ts_type(ts_type);
        if literals.cursor == span.start {
            return self.format_type_span(span.start, span.end);
        }
        Self::collapse_type_annotation_text(&literals.finish(span.end))
    }

    /// Formats the constraint or the default of a type parameter. Flow keeps the
    /// source as written, since Flow-only notation does not survive the
    /// TypeScript rewrite (see `format_ts_type()`).
    pub(super) fn format_type_parameter_type(&self, ts_type: &TSType<'a>) -> String {
        if self.flow {
            let span = ts_type.span();
            return self.slice(span.start, span.end);
        }
        self.format_type_with_literals(ts_type)
    }

    /// The Flow variance sigil (`+`/`-`) blanked in front of a member.
    fn flow_variance(&self, member_start: u32) -> Option<char> {
        if !self.flow {
            return None;
        }
        let start = member_start as usize;
        let at = |index: usize| self.source.as_bytes().get(index).copied();
        [at(start), start.checked_sub(1).and_then(at)]
            .into_iter()
            .flatten()
            .find(|byte| matches!(byte, b'+' | b'-'))
            .map(char::from)
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

        let exact = self.flow
            && self.source.as_bytes().get(type_literal.span.start as usize + 1) == Some(&b'|');
        let (open, close) = if exact { ("{| ", " |}") } else { ("{ ", " }") };
        if members.is_empty() {
            if exact { "{||}".to_string() } else { "{}".to_string() }
        } else {
            join3(open, &members.join("; "), close)
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
                if let Some(variance) = self.flow_variance(prop.span.start) {
                    out.push_char(variance);
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
                let (name, _, _) = self.format_index_signature_name(&index_signature.parameter);
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
