use oxc_ast::ast::{Function, TSSignature, TSType, TSTypeLiteral};
use oxc_span::GetSpan;

use crate::string_builder::{join2, join3, StringBuilder};

use super::model::{DocTag, ParamDoc, ParsedParamTag};
use super::DocVisitor;

impl<'a> DocVisitor<'a> {
    pub(super) fn extract_params_from_formals(
        &self,
        params: &oxc_ast::ast::FormalParameters<'a>,
        tags: &[DocTag],
    ) -> Vec<ParamDoc> {
        // Parse the `@param`/`@arg`/`@argument` tags once, in source order,
        // instead of re-filtering and re-parsing them for every formal
        // parameter (the old `find_param_tag` did O(P*T) parses).
        let parsed_param_tags: Vec<ParsedParamTag> = tags
            .iter()
            .filter(|tag| matches!(tag.tag.as_str(), "param" | "arg" | "argument"))
            .filter_map(Self::parse_param_tag)
            .collect();
        let mut reserved_param_names = params
            .items
            .iter()
            .filter_map(|param| Self::binding_pattern_identifier_name(&param.pattern))
            .collect::<Vec<_>>();
        if let Some(rest) = params.rest.as_ref() {
            if let Some(name) = Self::binding_pattern_identifier_name(&rest.rest.argument) {
                reserved_param_names.push(name);
            }
        }
        let mut used_param_tag_indices = Vec::new();

        let mut docs = Vec::with_capacity(params.items.len() + usize::from(params.rest.is_some()));

        for param in &params.items {
            let fallback_name = Self::binding_pattern_name(&param.pattern);
            let tag_index = if Self::binding_pattern_is_destructured(&param.pattern) {
                Self::find_destructured_param_tag_index(
                    &parsed_param_tags,
                    &used_param_tag_indices,
                    &reserved_param_names,
                )
            } else {
                Self::find_parsed_param_tag_index(&parsed_param_tags, &fallback_name)
            };
            let tag = tag_index.map(|index| &parsed_param_tags[index]);
            if let Some(index) = tag_index {
                used_param_tag_indices.push(index);
            }
            let name =
                tag.and_then(Self::top_level_param_tag_name).unwrap_or(&fallback_name).to_string();
            let default_value = self
                .binding_pattern_default_value(&param.pattern)
                .or_else(|| {
                    param
                        .initializer
                        .as_ref()
                        .map(|init| self.slice(init.span().start, init.span().end))
                })
                .or_else(|| tag.and_then(|tag| tag.default_value.clone()));

            let type_annotation = param
                .type_annotation
                .as_ref()
                .map(|t| self.format_ts_type(&t.type_annotation))
                .or_else(|| tag.and_then(|tag| tag.type_annotation.clone()));

            let optional =
                param.optional || default_value.is_some() || tag.is_some_and(|tag| tag.optional);
            let description = tag.and_then(|tag| tag.description.clone());

            docs.push(ParamDoc {
                name: name.clone(),
                type_annotation,
                optional,
                default_value,
                description,
            });

            if let Some(type_literal) = param
                .type_annotation
                .as_ref()
                .and_then(|t| Self::type_literal_from_ts_type(&t.type_annotation))
            {
                docs.extend(self.extract_type_literal_param_members(
                    &name,
                    type_literal,
                    &parsed_param_tags,
                ));
            }
        }

        if let Some(rest) = params.rest.as_ref() {
            let name = Self::binding_pattern_name(&rest.rest.argument);
            let tag_index = Self::find_parsed_param_tag_index(&parsed_param_tags, &name);
            let tag = tag_index.map(|index| &parsed_param_tags[index]);
            let type_annotation = rest
                .type_annotation
                .as_ref()
                .map(|t| self.format_ts_type(&t.type_annotation))
                .or_else(|| tag.and_then(|tag| tag.type_annotation.clone()));

            docs.push(ParamDoc {
                name: name.clone(),
                type_annotation,
                optional: tag.is_some_and(|tag| tag.optional),
                default_value: tag.and_then(|tag| tag.default_value.clone()),
                description: tag.and_then(|tag| tag.description.clone()),
            });

            if let Some(type_literal) = rest
                .type_annotation
                .as_ref()
                .and_then(|t| Self::type_literal_from_ts_type(&t.type_annotation))
            {
                docs.extend(self.extract_type_literal_param_members(
                    &name,
                    type_literal,
                    &parsed_param_tags,
                ));
            }
        }

        docs
    }

    fn type_literal_from_ts_type<'t>(ts_type: &'t TSType<'a>) -> Option<&'t TSTypeLiteral<'a>> {
        match ts_type {
            TSType::TSTypeLiteral(type_literal) => Some(type_literal),
            TSType::TSParenthesizedType(paren) => {
                Self::type_literal_from_ts_type(&paren.type_annotation)
            }
            _ => None,
        }
    }

    fn extract_type_literal_param_members(
        &self,
        parent_name: &str,
        type_literal: &TSTypeLiteral<'a>,
        parsed_param_tags: &[ParsedParamTag],
    ) -> Vec<ParamDoc> {
        let mut params = Vec::new();
        for member in &type_literal.members {
            match member {
                TSSignature::TSPropertySignature(prop) => {
                    let Some(property_name) = Self::property_key_name(&prop.key) else {
                        continue;
                    };
                    let lookup_name = join3(parent_name, ".", &property_name);
                    let tag = Self::find_exact_parsed_param_tag(parsed_param_tags, &lookup_name);
                    let optional = prop.optional || tag.is_some_and(|tag| tag.optional);
                    let name =
                        if optional { join2(&lookup_name, "?") } else { lookup_name.clone() };
                    let type_annotation = prop
                        .type_annotation
                        .as_ref()
                        .map(|t| self.format_ts_type(&t.type_annotation))
                        .or_else(|| tag.and_then(|tag| tag.type_annotation.clone()));

                    params.push(ParamDoc {
                        name,
                        type_annotation,
                        optional,
                        default_value: tag.and_then(|tag| tag.default_value.clone()),
                        description: tag.and_then(|tag| tag.description.clone()),
                    });
                }
                TSSignature::TSMethodSignature(method) => {
                    let Some(method_name) = Self::property_key_name(&method.key) else {
                        continue;
                    };
                    let lookup_name = join3(parent_name, ".", &method_name);
                    let tag = Self::find_exact_parsed_param_tag(parsed_param_tags, &lookup_name);
                    let optional = method.optional || tag.is_some_and(|tag| tag.optional);
                    let name =
                        if optional { join2(&lookup_name, "?") } else { lookup_name.clone() };
                    let params_text = self.format_type_formal_parameters(&method.params);
                    let return_type = method.return_type.as_ref().map_or_else(
                        || "unknown".to_string(),
                        |t| self.format_ts_type(&t.type_annotation),
                    );
                    let mut type_annotation =
                        StringBuilder::with_capacity(params_text.len() + return_type.len() + 6);
                    type_annotation.push_char('(');
                    type_annotation.push_str(&params_text);
                    type_annotation.push_str(") => ");
                    type_annotation.push_str(&return_type);

                    params.push(ParamDoc {
                        name,
                        type_annotation: Some(type_annotation.into_string()),
                        optional,
                        default_value: tag.and_then(|tag| tag.default_value.clone()),
                        description: tag.and_then(|tag| tag.description.clone()),
                    });
                }
                _ => {}
            }
        }
        params
    }

    /// Extract parameters from a function.
    pub(super) fn extract_params(&self, func: &Function, tags: &[DocTag]) -> Vec<ParamDoc> {
        self.extract_params_from_formals(&func.params, tags)
    }
}
